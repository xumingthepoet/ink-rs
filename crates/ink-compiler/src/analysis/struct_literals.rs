use std::collections::{BTreeSet, HashSet};

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        ConstantDeclaration, Divert, DivertTarget, Expression, InterfaceMemberKind,
        InterfaceMemberSignature, Object, Story, StructLiteralField, TunnelOnwards, TypeName,
        VariableAssignment,
    },
    source::SourceSpan,
};

use super::{
    argument_resolution::{
        resolve_dynamic_interface_signature, resolve_function_call_expected_arguments,
        resolve_static_target_expected_arguments, DynamicInterfaceSignatureInputs,
        FunctionCallArgumentResolution, ResolvedExpectedArguments,
    },
    context::{EnumTypeIndex, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    enums::type_name_contains_enum,
    expression_types::infer_expression_type,
    indexes::AnalysisIndexes,
    interface_values::{
        check_expression_type_with_expected, ExpectedTypeCheckError, ExpectedTypeInference,
        ModuleImplementationIndex,
    },
    interfaces::InterfaceMemberIndex,
    modules::ModuleImportIndex,
    structs::resolve_struct_symbol,
};

#[cfg(test)]
use super::modules::ModuleAnalysis;

#[cfg(test)]
pub(super) fn struct_literal_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let module_analysis = ModuleAnalysis::build(story);
    let indexes = AnalysisIndexes::build(story, &module_analysis);
    struct_literal_diagnostics_with_indexes(story, &indexes)
}

pub(super) fn struct_literal_diagnostics_with_indexes(
    story: &Story,
    indexes: &AnalysisIndexes<'_>,
) -> Vec<Diagnostic> {
    let mut checker = StructLiteralChecker::new(
        &indexes.struct_types,
        &indexes.enum_types,
        &indexes.variable_scopes,
        &indexes.target_symbols,
        &indexes.module_implementations,
        indexes.module_imports,
        &indexes.interface_members,
    );
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct StructLiteralChecker<'a> {
    struct_types: &'a StructTypeIndex,
    enum_types: &'a EnumTypeIndex,
    variable_scopes: &'a VariableScopeIndex,
    target_symbols: &'a TargetSymbolIndex,
    module_implementations: &'a ModuleImplementationIndex,
    module_imports: &'a ModuleImportIndex,
    interface_members: &'a InterfaceMemberIndex,
    diagnostics: Vec<Diagnostic>,
    expected_expression_ids: HashSet<usize>,
}

impl<'a> StructLiteralChecker<'a> {
    fn new(
        struct_types: &'a StructTypeIndex,
        enum_types: &'a EnumTypeIndex,
        variable_scopes: &'a VariableScopeIndex,
        target_symbols: &'a TargetSymbolIndex,
        module_implementations: &'a ModuleImplementationIndex,
        module_imports: &'a ModuleImportIndex,
        interface_members: &'a InterfaceMemberIndex,
    ) -> Self {
        Self {
            struct_types,
            enum_types,
            variable_scopes,
            target_symbols,
            module_implementations,
            module_imports,
            interface_members,
            diagnostics: Vec::new(),
            expected_expression_ids: HashSet::new(),
        }
    }

    fn expected_type_inference(&self) -> ExpectedTypeInference<'_> {
        ExpectedTypeInference {
            variable_scopes: self.variable_scopes,
            struct_types: self.struct_types,
            enum_types: self.enum_types,
            target_symbols: self.target_symbols,
            module_implementations: self.module_implementations,
            module_imports: self.module_imports,
            interface_members: self.interface_members,
        }
    }

    fn dynamic_interface_signature_inputs(&self) -> DynamicInterfaceSignatureInputs<'_> {
        DynamicInterfaceSignatureInputs {
            variable_scopes: self.variable_scopes,
            struct_types: self.struct_types,
            enum_types: self.enum_types,
            target_symbols: self.target_symbols,
            interface_members: self.interface_members,
        }
    }

    fn check_assignment(&mut self, assignment: &VariableAssignment, context: &VisitContext) {
        let Some(expression) = assignment.expression() else {
            return;
        };

        let expected_type = if assignment.is_global() || assignment.is_temporary() {
            assignment.declared_type().cloned()
        } else {
            assignment
                .target()
                .variable_name()
                .and_then(|name| self.visible_declared_type(name, context))
        };

        let Some(expected_type) = expected_type else {
            return;
        };

        self.check_expression_against_type(
            expression,
            &expected_type,
            assignment.name(),
            assignment.span(),
            context,
        );
    }

    fn check_constant(&mut self, declaration: &ConstantDeclaration, context: &VisitContext) {
        self.check_expression_against_type(
            declaration.expression(),
            declaration.declared_type(),
            declaration.name(),
            declaration.span(),
            context,
        );
    }

    fn check_divert_arguments(&mut self, divert: &Divert, context: &VisitContext) {
        self.check_divert_target_arguments(
            divert.target(),
            divert.arguments(),
            divert.span(),
            context,
        );
    }

    fn check_tunnel_onwards_arguments(
        &mut self,
        tunnel_onwards: &TunnelOnwards,
        context: &VisitContext,
    ) {
        if let Some(target) = tunnel_onwards.override_target() {
            self.check_divert_target_arguments(
                target,
                tunnel_onwards.arguments(),
                tunnel_onwards.span(),
                context,
            );
        }
    }

    fn check_divert_target_arguments(
        &mut self,
        target: &DivertTarget,
        arguments: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        match target {
            DivertTarget::Path(_) | DivertTarget::QualifiedPath(_) => {
                if let Some(expected_arguments) = resolve_static_target_expected_arguments(
                    target,
                    context.current_module.as_deref(),
                    context.current_flow_path.as_deref(),
                    self.target_symbols,
                ) {
                    self.check_resolved_expected_arguments(
                        &expected_arguments,
                        arguments,
                        span,
                        context,
                    );
                }
            }
            DivertTarget::Dynamic(Expression::DynamicInterfaceAccess { target, member }) => {
                if let Ok(signature) = resolve_dynamic_interface_signature(
                    target,
                    member,
                    InterfaceMemberKind::Knot,
                    self.dynamic_interface_signature_inputs(),
                    context.current_module.as_deref(),
                    context.current_flow_path.as_deref(),
                ) {
                    self.check_interface_signature_arguments(
                        member, arguments, &signature, span, context,
                    );
                }
            }
            DivertTarget::Dynamic(_)
            | DivertTarget::Done
            | DivertTarget::End
            | DivertTarget::Empty => {}
        }
    }

    fn check_function_call_arguments(
        &mut self,
        name: &str,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let FunctionCallArgumentResolution::Function(expected_arguments) =
            resolve_function_call_expected_arguments(
                name,
                context.current_module.as_deref(),
                context.current_flow_path.as_deref(),
                self.target_symbols,
            )
        else {
            return;
        };

        self.check_resolved_expected_arguments(&expected_arguments, args, span, context);
    }

    fn check_resolved_expected_arguments(
        &mut self,
        expected_arguments: &ResolvedExpectedArguments,
        arguments: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        for (argument, parameter) in arguments.iter().zip(expected_arguments.arguments()) {
            let Some(expected_type) = parameter.declared_type() else {
                continue;
            };
            self.check_expression_against_type(
                argument,
                expected_type,
                parameter.name(),
                span,
                context,
            );
        }
    }

    fn check_dynamic_interface_function_arguments(
        &mut self,
        target: &Expression,
        member: &str,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if let Ok(signature) = resolve_dynamic_interface_signature(
            target,
            member,
            InterfaceMemberKind::Function,
            self.dynamic_interface_signature_inputs(),
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            self.check_interface_signature_arguments(member, args, &signature, span, context);
        }
    }
    fn check_interface_signature_arguments(
        &mut self,
        member: &str,
        arguments: &[Expression],
        signature: &InterfaceMemberSignature,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        for (argument, parameter) in arguments.iter().zip(signature.arguments()) {
            let Some(expected_type) = parameter.declared_type() else {
                continue;
            };
            let context_name = format!("{member}.{}", parameter.name());
            self.check_expression_against_type(
                argument,
                expected_type,
                &context_name,
                span,
                context,
            );
        }
    }

    fn visible_declared_type(&self, name: &str, context: &VisitContext) -> Option<TypeName> {
        self.variable_scopes
            .visible_variable_declared_type(
                name,
                context.current_module.as_deref(),
                context.current_flow_path.as_deref(),
            )
            .and_then(|declared_type| declared_type.cloned())
    }

    fn check_expression_against_type(
        &mut self,
        expression: &Expression,
        expected_type: &TypeName,
        context_name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        self.mark_expected_expression(expression);
        match (expected_type, expression) {
            (
                TypeName::Struct(_) | TypeName::QualifiedStruct(_),
                Expression::StructLiteral { type_name, fields },
            ) => {
                if self.check_struct_literal_type_matches(
                    type_name,
                    expected_type,
                    context_name,
                    span,
                    context,
                ) {
                    self.check_struct_literal_for_type(type_name, fields, span, context);
                }
            }
            (TypeName::Struct(_), _) | (TypeName::QualifiedStruct(_), _) => {
                self.check_non_literal_expression(
                    expression,
                    expected_type,
                    context_name,
                    span,
                    context,
                );
            }
            (TypeName::Primitive(_), _) | (TypeName::Interface { .. }, _) => {
                self.check_non_literal_expression(
                    expression,
                    expected_type,
                    context_name,
                    span,
                    context,
                );
            }
            (TypeName::Dict { .. }, _) => {}
            (TypeName::Array(_), _) => {
                if let Ok(actual_type) = infer_expression_type(
                    expression,
                    self.variable_scopes,
                    self.struct_types,
                    self.enum_types,
                    self.target_symbols,
                    self.interface_members,
                    context.current_module.as_deref(),
                    context.current_flow_path.as_deref(),
                ) {
                    if &actual_type != expected_type {
                        self.diagnostics.push(type_mismatch_diagnostic(
                            context_name,
                            expected_type,
                            &actual_type,
                            span,
                        ));
                    }
                }
            }
            (TypeName::Void, _) => {}
        }
    }

    fn check_struct_literal_for_type(
        &mut self,
        type_name: &TypeName,
        fields: &[StructLiteralField],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let Some(struct_name) = type_name.as_struct_name() else {
            return;
        };
        self.check_struct_literal(struct_name, fields, span, context);
    }

    fn check_struct_literal_type_matches(
        &mut self,
        actual_type: &TypeName,
        expected_type: &TypeName,
        context_name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) -> bool {
        if struct_type_key(actual_type, context.current_module.as_deref())
            == struct_type_key(expected_type, context.current_module.as_deref())
        {
            return true;
        }

        self.diagnostics.push(type_mismatch_diagnostic(
            context_name,
            expected_type,
            actual_type,
            span,
        ));
        false
    }

    fn check_non_literal_expression(
        &mut self,
        expression: &Expression,
        expected_type: &TypeName,
        context_name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        match check_expression_type_with_expected(
            expression,
            expected_type,
            self.expected_type_inference(),
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            Err(ExpectedTypeCheckError::Mismatch(actual_type)) => {
                self.diagnostics.push(type_mismatch_diagnostic(
                    context_name,
                    expected_type,
                    &actual_type,
                    span,
                ));
            }
            Ok(()) => {}
            Err(ExpectedTypeCheckError::Inference(error))
                if expected_type.primitive_type().is_some()
                    || expected_type.as_interface_name().is_some()
                    || type_name_contains_enum(
                        expected_type,
                        self.enum_types,
                        context.current_module.as_deref(),
                    ) =>
            {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check value for '{}': {}",
                        context_name,
                        error.message()
                    ),
                ));
            }
            Err(ExpectedTypeCheckError::Inference(_)) => {}
        }
    }

    fn check_struct_literal(
        &mut self,
        struct_name: &str,
        fields: &[StructLiteralField],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let Some(symbol) = resolve_struct_symbol(
            self.struct_types,
            struct_name,
            context.current_module.as_deref(),
        ) else {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("Unknown struct type '{struct_name}' for struct literal"),
            ));
            return;
        };

        let mut provided_fields = BTreeSet::new();
        for field in fields {
            if !provided_fields.insert(field.name().to_string()) {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Duplicate field '{}' in struct literal for '{}'",
                        field.name(),
                        struct_name
                    ),
                ));
                continue;
            }

            let Some(field_type) = symbol.fields().get(field.name()) else {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Unknown field '{}' in struct literal for '{}'",
                        field.name(),
                        struct_name
                    ),
                ));
                continue;
            };

            let field_context_name = format!("{struct_name}.{}", field.name());
            self.check_expression_against_type(
                field.expression(),
                field_type,
                &field_context_name,
                span,
                context,
            );
        }

        for (field_name, field_type) in symbol.fields() {
            if provided_fields.contains(field_name) {
                continue;
            }
            if field_type.default_value().is_none() {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Missing field '{}' in struct literal for '{}' cannot be default-initialized",
                        field_name, struct_name
                    ),
                ));
            }
        }
    }

    fn mark_expected_expression(&mut self, expression: &Expression) {
        self.expected_expression_ids
            .insert(expression as *const Expression as usize);
    }
}

impl ParsedVisitor for StructLiteralChecker<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        match object {
            Object::ConstantDeclaration(declaration) => self.check_constant(declaration, context),
            Object::Divert(divert) => self.check_divert_arguments(divert, context),
            Object::TunnelOnwards(tunnel_onwards) => {
                self.check_tunnel_onwards_arguments(tunnel_onwards, context);
            }
            Object::VariableAssignment(assignment) => self.check_assignment(assignment, context),
            _ => {}
        }
    }

    fn visit_expression(&mut self, expression: &Expression, context: &VisitContext) {
        match expression {
            Expression::FunctionCall { name, args } => self.check_function_call_arguments(
                name,
                args,
                &SourceSpan::new(None, 1, 1),
                context,
            ),
            Expression::QualifiedFunctionCall { name, args } => self.check_function_call_arguments(
                name.as_str(),
                args,
                &SourceSpan::new(None, 1, 1),
                context,
            ),
            Expression::DynamicInterfaceFunctionCall {
                target,
                member,
                args,
            } => self.check_dynamic_interface_function_arguments(
                target,
                member,
                args,
                &SourceSpan::new(None, 1, 1),
                context,
            ),
            Expression::StructLiteral { type_name, fields }
                if !self
                    .expected_expression_ids
                    .contains(&(expression as *const Expression as usize)) =>
            {
                self.check_struct_literal_for_type(
                    type_name,
                    fields,
                    &SourceSpan::new(None, 1, 1),
                    context,
                );
            }
            _ => {}
        }
    }
}

fn struct_type_key(type_name: &TypeName, current_module: Option<&str>) -> Option<String> {
    match type_name {
        TypeName::Struct(name) => Some(
            current_module
                .map(|module| format!("{module}::{name}"))
                .unwrap_or_else(|| name.to_string()),
        ),
        TypeName::QualifiedStruct(name) => Some(name.as_str().to_string()),
        _ => None,
    }
}

fn type_mismatch_diagnostic(
    context_name: &str,
    expected_type: &TypeName,
    actual_type: &TypeName,
    span: &SourceSpan,
) -> Diagnostic {
    Diagnostic::error(
        span.clone(),
        format!(
            "Value for '{}' has type {} but expected {}",
            context_name,
            actual_type.display_name(),
            expected_type.display_name()
        ),
    )
}

#[cfg(test)]
mod tests {
    use crate::{analysis::test_support::assert_single_diagnostic, diagnostic::DiagnosticSeverity};

    use super::{super::test_support::parse_story, *};

    #[test]
    fn accepts_full_struct_literal_initializers() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             name: string\n\
             }\n\
             CONST default_player: Player = %Player{ hp: 5, name: \"Lin\" }\n\
             VAR player: Player = %Player{ hp: 10, name: \"Ada\" }\n\
             -> DONE",
        );

        assert_eq!(struct_literal_diagnostics(&story), []);
    }

    #[test]
    fn resolves_struct_literals_against_current_module_structs() {
        let story = parse_story(
            "=== module game ===\n\
             STRUCT Item {\n\
             hp: int\n\
             }\n\
             VAR item: Item = %Item{ hp: 1 }\n\
             == main ==\n\
             -> DONE\n\
             === module items ===\n\
             STRUCT Item {\n\
             label: string\n\
             }\n\
             VAR item: Item = %Item{ label: \"sword\" }\n\
             == helper ==\n\
             -> DONE",
        );

        assert_eq!(struct_literal_diagnostics(&story), []);
    }

    #[test]
    fn accepts_imported_qualified_struct_literals() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT Item\n\
             VAR item: items::Item = %items::Item{ hp: 1 }\n\
             == main ==\n\
             -> DONE\n\
             === module items ===\n\
             STRUCT Item {\n\
             hp: int\n\
             }\n\
             == helper ==\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_interface_module_literals_in_struct_fields() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             STRUCT Route {\n\
             next: interface<IItem>\n\
             }\n\
             VAR route: Route = %Route{ next: left }\n\
             == main ==\n\
             -> END\n\
             === module left implements IItem ===\n\
             == target ==\n\
             -> END",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_dynamic_interface_function_calls_in_struct_literals() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == function score(amount: int) => int ==\n\
             === module game ===\n\
             FROM left\n\
             STRUCT Result {\n\
             score: int\n\
             }\n\
             VAR route: interface<IItem> = left\n\
             VAR result: Result = %Result{ score: {route}::score(1) }\n\
             == main ==\n\
             -> END\n\
             === module left implements IItem ===\n\
             == function score(amount: int) => int ==\n\
             ~ return amount",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn rejects_non_implementing_modules_in_interface_struct_fields() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             STRUCT Route {\n\
             next: interface<IItem>\n\
             }\n\
             VAR route: Route = %Route{ next: left }\n\
             == main ==\n\
             -> END\n\
             === module left ===\n\
             == target ==\n\
             -> END",
        );

        let diagnostics = struct_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot type-check value for 'Route.next': Module 'left' does not implement interface 'IItem'",
        );
    }

    #[test]
    fn rejects_missing_interface_struct_fields_without_defaults() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             STRUCT Route {\n\
             next: interface<IItem>\n\
             }\n\
             VAR route: Route = %Route{}\n\
             == main ==\n\
             -> END",
        );

        let diagnostics = struct_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Missing field 'next' in struct literal for 'Route' cannot be default-initialized",
        );
    }

    #[test]
    fn reports_wrong_type_for_imported_qualified_struct_literals() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT Item\n\
             VAR item: items::Item = %items::Item{ hp: \"full\" }\n\
             == main ==\n\
             -> DONE\n\
             === module items ===\n\
             STRUCT Item {\n\
             hp: int\n\
             }\n\
             == helper ==\n\
             -> DONE",
        );

        let diagnostics = struct_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Value for 'items::Item.hp' has type string but expected int",
        );
    }

    #[test]
    fn accepts_partial_struct_literal_with_defaulted_array_field() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             inventory: int[]\n\
             }\n\
             VAR player: Player = %Player{ hp: 10 }\n\
             -> DONE",
        );

        assert_eq!(struct_literal_diagnostics(&story), []);
    }

    #[test]
    fn rejects_missing_divert_target_struct_field() {
        let story = parse_story(
            "STRUCT Route {\n\
             next: ->\n\
             visits: int\n\
             }\n\
             VAR route: Route = %Route{ visits: 1 }\n\
             -> DONE",
        );

        let diagnostics = struct_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Missing field 'next' in struct literal for 'Route' cannot be default-initialized",
        );
    }

    #[test]
    fn reports_unknown_struct_literal_field() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR player: Player = %Player{ hp: 10, mp: 5 }\n\
             -> DONE",
        );

        let diagnostics = struct_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Unknown field 'mp' in struct literal for 'Player'",
        );
    }

    #[test]
    fn reports_duplicate_struct_literal_field() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR player: Player = %Player{ hp: 10, hp: 11 }\n\
             -> DONE",
        );

        let diagnostics = struct_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Duplicate field 'hp' in struct literal for 'Player'",
        );
    }

    #[test]
    fn reports_wrong_struct_literal_field_type() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR player: Player = %Player{ hp: \"full\" }\n\
             -> DONE",
        );

        let diagnostics = struct_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Value for 'Player.hp' has type string but expected int",
        );
    }

    #[test]
    fn checks_nested_struct_literals() {
        let story = parse_story(
            "STRUCT Stats {\n\
             hp: int\n\
             }\n\
             STRUCT Player {\n\
             stats: Stats\n\
             name: string\n\
             }\n\
             VAR player: Player = %Player{ stats: %Stats{ hp: 10 }, name: \"Ada\" }\n\
             -> DONE",
        );

        assert_eq!(struct_literal_diagnostics(&story), []);
    }

    #[test]
    fn checks_reassignment_struct_literals_against_variable_type() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR player: Player = %Player{}\n\
             ~ player = %Player{ hp: \"full\" }\n\
             -> DONE",
        );

        let diagnostics = struct_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Value for 'Player.hp' has type string but expected int",
        );
    }

    #[test]
    fn checks_standalone_struct_literals_against_explicit_type() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             { %Player{ hp: \"dynamic\" } }\n\
             -> DONE",
        );

        let diagnostics = struct_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Value for 'Player.hp' has type string but expected int",
        );
    }
}
