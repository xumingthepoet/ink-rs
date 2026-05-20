use std::collections::BTreeSet;

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        ConstantDeclaration, DictLiteralEntry, Divert, DivertTarget, Expression,
        InterfaceMemberKind, InterfaceMemberSignature, Object, Story, StructLiteralField,
        TunnelOnwards, TypeName, VariableAssignment,
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
    expected_expressions::{expression_context_span, ExpectedExpressionSet},
    indexes::AnalysisIndexes,
    interface_values::{
        check_expression_type_with_expected, ExpectedTypeCheckError, ExpectedTypeInference,
        ModuleImplementationIndex,
    },
    interfaces::InterfaceMemberIndex,
    modules::ModuleImportIndex,
    structs::resolve_struct_symbol,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StructLiteralMode {
    ArraysOnly,
    Full,
}

#[cfg(test)]
use super::modules::ModuleAnalysis;

#[cfg(test)]
pub(super) fn array_literal_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let module_analysis = ModuleAnalysis::build(story);
    let indexes = AnalysisIndexes::build(story, &module_analysis);
    array_literal_diagnostics_with_indexes(story, &indexes)
}

pub(super) fn array_literal_diagnostics_with_indexes(
    story: &Story,
    indexes: &AnalysisIndexes<'_>,
) -> Vec<Diagnostic> {
    let mut checker = ArrayLiteralChecker::new(
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

struct ArrayLiteralChecker<'a> {
    struct_types: &'a StructTypeIndex,
    enum_types: &'a EnumTypeIndex,
    variable_scopes: &'a VariableScopeIndex,
    target_symbols: &'a TargetSymbolIndex,
    module_implementations: &'a ModuleImplementationIndex,
    module_imports: &'a ModuleImportIndex,
    interface_members: &'a InterfaceMemberIndex,
    diagnostics: Vec<Diagnostic>,
    expected_expressions: ExpectedExpressionSet,
}

impl<'a> ArrayLiteralChecker<'a> {
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
            expected_expressions: ExpectedExpressionSet::default(),
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

        if let Some(expected_type) = expected_type {
            self.check_expression_for_arrays(
                expression,
                &expected_type,
                assignment.name(),
                assignment.span(),
                context,
            );
        }
    }

    fn check_constant(&mut self, declaration: &ConstantDeclaration, context: &VisitContext) {
        self.check_expression_for_arrays(
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

    fn check_resolved_expected_arguments(
        &mut self,
        expected_arguments: &ResolvedExpectedArguments,
        arguments: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let _ = expected_arguments.target_name();
        for (argument, parameter) in arguments.iter().zip(expected_arguments.arguments()) {
            let Some(expected_type) = parameter.declared_type() else {
                continue;
            };
            self.check_expression_for_arrays(
                argument,
                expected_type,
                parameter.name(),
                span,
                context,
            );
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
            self.check_expression_for_arrays(argument, expected_type, &context_name, span, context);
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
    fn check_expression_for_arrays(
        &mut self,
        expression: &Expression,
        expected_type: &TypeName,
        context_name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        self.mark_expected_expression(expression);
        match (expected_type, expression) {
            (TypeName::Array(element_type), Expression::ArrayLiteral(elements)) => {
                self.check_array_literal(element_type, elements, context_name, span, context);
            }
            (TypeName::Array(_), _) => {
                self.check_non_array_expression(
                    expression,
                    expected_type,
                    context_name,
                    span,
                    context,
                );
            }
            (TypeName::Dict { value_type, .. }, Expression::DictLiteral(entries)) => {
                self.check_dict_literal_values(value_type, entries, context_name, span, context);
            }
            (TypeName::Struct(struct_name), Expression::StructLiteral { fields, .. }) => {
                self.check_struct_literal(
                    struct_name,
                    fields,
                    span,
                    context,
                    StructLiteralMode::ArraysOnly,
                );
            }
            (_, Expression::ArrayLiteral(_)) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Value for '{}' is an array literal but expected {}",
                        context_name,
                        expected_type.display_name()
                    ),
                ));
            }
            _ => {}
        }
    }

    fn check_array_literal(
        &mut self,
        element_type: &TypeName,
        elements: &[Expression],
        context_name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        for (index, element) in elements.iter().enumerate() {
            self.mark_expected_expression(element);
            let element_context = format!("{context_name}[{index}]");
            self.check_array_element(element, element_type, &element_context, span, context);
        }
    }

    fn check_array_element(
        &mut self,
        element: &Expression,
        element_type: &TypeName,
        context_name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        match (element_type, element) {
            (TypeName::Array(nested_element_type), Expression::ArrayLiteral(elements)) => {
                self.check_array_literal(
                    nested_element_type,
                    elements,
                    context_name,
                    span,
                    context,
                );
            }
            (TypeName::Array(_), _) => {
                self.check_non_array_expression(element, element_type, context_name, span, context);
            }
            (TypeName::Struct(struct_name), Expression::StructLiteral { fields, .. }) => {
                self.check_struct_literal(
                    struct_name,
                    fields,
                    span,
                    context,
                    StructLiteralMode::Full,
                );
            }
            (TypeName::QualifiedStruct(struct_name), Expression::StructLiteral { fields, .. }) => {
                self.check_struct_literal(
                    struct_name.as_str(),
                    fields,
                    span,
                    context,
                    StructLiteralMode::Full,
                );
            }
            (TypeName::Dict { value_type, .. }, Expression::DictLiteral(entries)) => {
                self.check_dict_literal_values(value_type, entries, context_name, span, context);
            }
            (TypeName::Struct(_), _)
            | (TypeName::QualifiedStruct(_), _)
            | (TypeName::Interface { .. }, _)
            | (TypeName::Dict { .. }, _)
            | (TypeName::Primitive(_), _) => {
                self.check_exact_expression_type(
                    element,
                    element_type,
                    context_name,
                    span,
                    context,
                );
            }
            (TypeName::Void, _) => {}
        }
    }

    fn check_non_array_expression(
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
            Err(ExpectedTypeCheckError::Inference(error)) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check value for '{}': {}",
                        context_name,
                        error.message()
                    ),
                ))
            }
        }
    }

    fn check_exact_expression_type(
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
        mode: StructLiteralMode,
    ) {
        let Some(symbol) = resolve_struct_symbol(
            self.struct_types,
            struct_name,
            context.current_module.as_deref(),
        ) else {
            if mode == StructLiteralMode::Full {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!("Unknown struct type '{struct_name}' for struct literal"),
                ));
            }
            return;
        };

        let mut provided_fields = BTreeSet::new();
        for field in fields {
            self.mark_expected_expression(field.expression());
            if !provided_fields.insert(field.name().to_string()) {
                if mode == StructLiteralMode::Full {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Duplicate field '{}' in struct literal for '{}'",
                            field.name(),
                            struct_name
                        ),
                    ));
                }
                continue;
            }

            let Some(field_type) = symbol.fields().get(field.name()) else {
                if mode == StructLiteralMode::Full {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Unknown field '{}' in struct literal for '{}'",
                            field.name(),
                            struct_name
                        ),
                    ));
                }
                continue;
            };

            let field_context = format!("{struct_name}.{}", field.name());
            match (mode, field_type) {
                (StructLiteralMode::ArraysOnly, TypeName::Array(_))
                | (StructLiteralMode::Full, _) => {
                    self.check_array_element(
                        field.expression(),
                        field_type,
                        &field_context,
                        span,
                        context,
                    );
                }
                (StructLiteralMode::ArraysOnly, TypeName::Struct(nested_struct_name)) => {
                    if let Expression::StructLiteral {
                        fields: nested_fields,
                        ..
                    } = field.expression()
                    {
                        self.check_struct_literal(
                            nested_struct_name,
                            nested_fields,
                            span,
                            context,
                            StructLiteralMode::ArraysOnly,
                        );
                    }
                }
                (StructLiteralMode::ArraysOnly, TypeName::QualifiedStruct(nested_struct_name)) => {
                    if let Expression::StructLiteral {
                        fields: nested_fields,
                        ..
                    } = field.expression()
                    {
                        self.check_struct_literal(
                            nested_struct_name.as_str(),
                            nested_fields,
                            span,
                            context,
                            StructLiteralMode::ArraysOnly,
                        );
                    }
                }
                (StructLiteralMode::ArraysOnly, TypeName::Dict { value_type, .. }) => {
                    if let Expression::DictLiteral(entries) = field.expression() {
                        self.check_dict_literal_values(
                            value_type,
                            entries,
                            &field_context,
                            span,
                            context,
                        );
                    }
                }
                (
                    StructLiteralMode::ArraysOnly,
                    TypeName::Primitive(_) | TypeName::Interface { .. } | TypeName::Void,
                ) => {}
            }
        }
    }

    fn check_dict_literal_values(
        &mut self,
        value_type: &TypeName,
        entries: &[DictLiteralEntry],
        context_name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        for entry in entries {
            self.mark_expected_expression(entry.value());
            let value_context = format!("{context_name}[]");
            self.check_array_element(entry.value(), value_type, &value_context, span, context);
        }
    }

    fn mark_expected_expression(&mut self, expression: &Expression) {
        self.expected_expressions.mark(expression);
    }
}

impl ParsedVisitor for ArrayLiteralChecker<'_> {
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
        let span = expression_context_span(context);
        match expression {
            Expression::FunctionCall { name, args } => {
                self.check_function_call_arguments(name, args, &span, context)
            }
            Expression::QualifiedFunctionCall { name, args } => {
                self.check_function_call_arguments(name.as_str(), args, &span, context)
            }
            Expression::DynamicInterfaceFunctionCall {
                target,
                member,
                args,
            } => self
                .check_dynamic_interface_function_arguments(target, member, args, &span, context),
            _ => {}
        }

        if matches!(expression, Expression::ArrayLiteral(_))
            && !self.expected_expressions.contains(expression)
        {
            self.diagnostics.push(Diagnostic::error(
                span,
                "Array literal requires an expected array type",
            ));
        }
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
    fn accepts_primitive_array_literals() {
        let story = parse_story(
            "CONST default_scores: int[] = [1, 2, 3]\n\
             VAR scores: int[] = [1, 2, 3]\n\
             -> DONE",
        );

        assert_eq!(array_literal_diagnostics(&story), []);
    }

    #[test]
    fn accepts_empty_array_literals_with_expected_type() {
        let story = parse_story(
            "VAR scores: int[] = []\n\
             -> DONE",
        );

        assert_eq!(array_literal_diagnostics(&story), []);
    }

    #[test]
    fn accepts_interface_module_literals_in_array_literals() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             VAR routes: interface<IItem>[] = [left]\n\
             == main ==\n\
             -> END\n\
             === module left implements IItem ===\n\
             == target ==\n\
             -> END",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_dynamic_interface_function_calls_in_array_literals() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == function score(amount: int) => int ==\n\
             === module game ===\n\
             FROM left\n\
             VAR route: interface<IItem> = left\n\
             VAR scores: int[] = [{route}::score(1), 2]\n\
             == main ==\n\
             -> END\n\
             === module left implements IItem ===\n\
             == function score(amount: int) => int ==\n\
             ~ return amount",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn rejects_non_implementing_modules_in_interface_array_literals() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             VAR routes: interface<IItem>[] = [left]\n\
             == main ==\n\
             -> END\n\
             === module left ===\n\
             == target ==\n\
             -> END",
        );

        let diagnostics = array_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot type-check value for 'routes[0]': Module 'left' does not implement interface 'IItem'",
        );
    }

    #[test]
    fn accepts_nested_array_literals() {
        let story = parse_story(
            "VAR matrix: int[][] = [[1, 2], []]\n\
             -> DONE",
        );

        assert_eq!(array_literal_diagnostics(&story), []);
    }

    #[test]
    fn accepts_array_literals_in_typed_divert_arguments() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             -> start([1, 2, 3])\n\
             == start(ids: int[]) ==\n\
             -> END",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_array_literals_in_typed_function_arguments() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             ~ temp values: int[] = identity([1, 2, 3])\n\
             {values[0]}\n\
             -> END\n\
             == function identity(values: int[]) => int[] ==\n\
             ~ return values",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_struct_array_literals_in_typed_divert_arguments() {
        let story = parse_story(
            "=== module game ===\n\
             STRUCT Player {\n\
             hp: int\n\
             }\n\
             == main ==\n\
             -> start([%Player{ hp: 10 }])\n\
             == start(players: Player[]) ==\n\
             -> END",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_array_literals_in_dynamic_interface_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target(ids: int[]) ==\n\
             == function score(ids: int[]) => int ==\n\
             === module game ===\n\
             FROM left\n\
             VAR route: interface<IItem> = left\n\
             == main ==\n\
             ~ temp value: int = {route}::score([1, 2])\n\
             -> {{route}::target}([1, 2])\n\
             === module left implements IItem ===\n\
             == target(ids: int[]) ==\n\
             -> END\n\
             == function score(ids: int[]) => int ==\n\
             ~ return ids[0]",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_array_literal_argument_element_type_mismatch() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             -> start([1, \"two\"])\n\
             == start(ids: int[]) ==\n\
             -> END",
        );

        let diagnostics = array_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Value for 'ids[1]' has type string but expected int",
        );
    }

    #[test]
    fn reports_function_call_array_argument_errors_at_containing_object_span() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             ~ temp value: int = collect([\"bad\"])\n\
             -> END\n\
             == function collect(values: int[]) => int ==\n\
             ~ return values[0]",
        );

        let diagnostics = array_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Value for 'values[0]' has type string but expected int",
        );
        assert_eq!(diagnostics[0].line, 3);
        assert_eq!(diagnostics[0].column, 1);
    }

    #[test]
    fn accepts_struct_array_literals() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR party: Player[] = [%Player{ hp: 10 }, %Player{}]\n\
             -> DONE",
        );

        assert_eq!(array_literal_diagnostics(&story), []);
    }

    #[test]
    fn accepts_array_literal_fields_inside_struct_literals() {
        let story = parse_story(
            "STRUCT Player {\n\
             scores: int[]\n\
             }\n\
             VAR player: Player = %Player{ scores: [1, 2] }\n\
             -> DONE",
        );

        assert_eq!(array_literal_diagnostics(&story), []);
    }

    #[test]
    fn reports_mixed_array_element_type() {
        let story = parse_story(
            "VAR scores: int[] = [1, \"two\"]\n\
             -> DONE",
        );

        let diagnostics = array_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Value for 'scores[1]' has type string but expected int",
        );
    }

    #[test]
    fn reports_array_literal_without_expected_type() {
        let story = parse_story(
            "{[]}\n\
             -> DONE",
        );

        let diagnostics = array_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Array literal requires an expected array type",
        );
    }
}
