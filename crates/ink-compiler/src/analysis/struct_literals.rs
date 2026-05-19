use std::collections::BTreeSet;

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        ConstantDeclaration, Expression, Object, Story, StructLiteralField, TypeName,
        VariableAssignment,
    },
    source::SourceSpan,
};

use super::{
    context::{EnumTypeIndex, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    enums::{build_enum_type_index, type_name_contains_enum},
    expression_types::infer_expression_type,
    interface_values::{
        build_module_implementation_index, infer_expected_interface_expression_type,
        ModuleImplementationIndex,
    },
    interfaces::{build_interface_member_index, InterfaceMemberIndex},
    modules::{build_module_import_index, ModuleImportIndex},
    structs::{build_struct_type_index, resolve_struct_symbol},
    target_symbols::build_target_symbol_index,
    variables::build_variable_scope_index,
};

pub(super) fn struct_literal_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let struct_types = build_struct_type_index(story);
    let enum_types = build_enum_type_index(story);
    let variable_scopes = build_variable_scope_index(story);
    let target_symbols = build_target_symbol_index(story);
    let module_implementations = build_module_implementation_index(story);
    let module_imports = build_module_import_index(story);
    let interface_members = build_interface_member_index(story);
    let mut checker = StructLiteralChecker::new(
        &struct_types,
        &enum_types,
        &variable_scopes,
        &target_symbols,
        &module_implementations,
        &module_imports,
        &interface_members,
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
        match (expected_type, expression) {
            (TypeName::Struct(struct_name), Expression::StructLiteral(fields)) => {
                self.check_struct_literal(struct_name, fields, span, context);
            }
            (TypeName::QualifiedStruct(struct_name), Expression::StructLiteral(fields)) => {
                self.check_struct_literal(struct_name.as_str(), fields, span, context);
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

    fn check_non_literal_expression(
        &mut self,
        expression: &Expression,
        expected_type: &TypeName,
        context_name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if let Some(result) = infer_expected_interface_expression_type(
            expression,
            expected_type,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.module_implementations,
            self.module_imports,
            self.interface_members,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            match result {
                Ok(actual_type) if &actual_type != expected_type => {
                    self.diagnostics.push(type_mismatch_diagnostic(
                        context_name,
                        expected_type,
                        &actual_type,
                        span,
                    ));
                }
                Ok(_) => {}
                Err(error) => self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check value for '{}': {}",
                        context_name,
                        error.message()
                    ),
                )),
            }
            return;
        }

        match infer_expression_type(
            expression,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            Ok(actual_type) if &actual_type != expected_type => {
                self.diagnostics.push(type_mismatch_diagnostic(
                    context_name,
                    expected_type,
                    &actual_type,
                    span,
                ));
            }
            Ok(_) => {}
            Err(error)
                if expected_type.primitive_type().is_some()
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
            Err(_) => {}
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
}

impl ParsedVisitor for StructLiteralChecker<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        match object {
            Object::ConstantDeclaration(declaration) => self.check_constant(declaration, context),
            Object::VariableAssignment(assignment) => self.check_assignment(assignment, context),
            _ => {}
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
    fn accepts_full_struct_literal_initializers() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             name: string\n\
             }\n\
             CONST default_player: Player = { hp: 5, name: \"Lin\" }\n\
             VAR player: Player = { hp: 10, name: \"Ada\" }\n\
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
             VAR item: Item = { hp: 1 }\n\
             == main ==\n\
             -> DONE\n\
             === module items ===\n\
             STRUCT Item {\n\
             label: string\n\
             }\n\
             VAR item: Item = { label: \"sword\" }\n\
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
             VAR item: items::Item = { hp: 1 }\n\
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
             VAR route: Route = { next: left }\n\
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
             VAR result: Result = { score: {route}::score(1) }\n\
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
             VAR route: Route = { next: left }\n\
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
             VAR route: Route = {}\n\
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
             VAR item: items::Item = { hp: \"full\" }\n\
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
             VAR player: Player = { hp: 10 }\n\
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
             VAR route: Route = { visits: 1 }\n\
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
             VAR player: Player = { hp: 10, mp: 5 }\n\
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
             VAR player: Player = { hp: 10, hp: 11 }\n\
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
             VAR player: Player = { hp: \"full\" }\n\
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
             VAR player: Player = { stats: { hp: 10 }, name: \"Ada\" }\n\
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
             VAR player: Player = {}\n\
             ~ player = { hp: \"full\" }\n\
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
    fn does_not_infer_struct_type_from_field_names() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             { { hp: \"dynamic\" } }\n\
             -> DONE",
        );

        assert_eq!(struct_literal_diagnostics(&story), []);
    }
}
