use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        ConstantDeclaration, DefaultValue, Object, Story, TypeName, VariableAssignment,
    },
};

use super::{
    context::{EnumTypeIndex, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    enums::{build_enum_type_index, type_name_is_enum},
    expression_types::infer_expression_type,
    interface_values::{
        build_module_implementation_index, infer_expected_interface_expression_type,
        ModuleImplementationIndex,
    },
    modules::{build_module_import_index, ModuleImportIndex},
    structs::build_struct_type_index,
    target_symbols::build_target_symbol_index,
    variables::build_variable_scope_index,
};

pub(super) fn variable_initializer_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let struct_types = build_struct_type_index(story);
    let enum_types = build_enum_type_index(story);
    let variable_scopes = build_variable_scope_index(story);
    let target_symbols = build_target_symbol_index(story);
    let module_implementations = build_module_implementation_index(story);
    let module_imports = build_module_import_index(story);
    let mut checker = VariableInitializerChecker::new(
        &variable_scopes,
        &struct_types,
        &enum_types,
        &target_symbols,
        &module_implementations,
        &module_imports,
    );
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct VariableInitializerChecker<'a> {
    variable_scopes: &'a VariableScopeIndex,
    struct_types: &'a StructTypeIndex,
    enum_types: &'a EnumTypeIndex,
    target_symbols: &'a TargetSymbolIndex,
    module_implementations: &'a ModuleImplementationIndex,
    module_imports: &'a ModuleImportIndex,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> VariableInitializerChecker<'a> {
    fn new(
        variable_scopes: &'a VariableScopeIndex,
        struct_types: &'a StructTypeIndex,
        enum_types: &'a EnumTypeIndex,
        target_symbols: &'a TargetSymbolIndex,
        module_implementations: &'a ModuleImplementationIndex,
        module_imports: &'a ModuleImportIndex,
    ) -> Self {
        Self {
            variable_scopes,
            struct_types,
            enum_types,
            target_symbols,
            module_implementations,
            module_imports,
            diagnostics: Vec::new(),
        }
    }

    fn check_assignment(&mut self, assignment: &VariableAssignment, context: &VisitContext) {
        if !assignment.is_global() && !assignment.is_temporary() {
            return;
        }

        let Some(declared_type) = assignment.declared_type() else {
            return;
        };

        let Some(expression) = assignment.expression() else {
            if default_initializer_metadata(assignment).is_none() {
                self.diagnostics.push(Diagnostic::error(
                    assignment.span().clone(),
                    format!(
                        "Variable '{}' of type {} cannot be default-initialized",
                        assignment.name(),
                        declared_type.display_name()
                    ),
                ));
            }
            return;
        };

        if let Some(result) = infer_expected_interface_expression_type(
            expression,
            declared_type,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.module_implementations,
            self.module_imports,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            match result {
                Ok(actual_type) if &actual_type != declared_type => {
                    self.diagnostics.push(type_mismatch_diagnostic(
                        assignment,
                        declared_type,
                        &actual_type,
                    ));
                }
                Ok(_) => {}
                Err(error) => self.diagnostics.push(Diagnostic::error(
                    assignment.span().clone(),
                    format!(
                        "Cannot type-check initializer for variable '{}': {}",
                        assignment.name(),
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
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            Ok(actual_type) if &actual_type != declared_type => {
                self.diagnostics.push(type_mismatch_diagnostic(
                    assignment,
                    declared_type,
                    &actual_type,
                ));
            }
            Ok(_) => {}
            Err(error)
                if declared_type.primitive_type().is_some()
                    || type_name_is_enum(
                        declared_type,
                        self.enum_types,
                        context.current_module.as_deref(),
                    ) =>
            {
                self.diagnostics.push(Diagnostic::error(
                    assignment.span().clone(),
                    format!(
                        "Cannot type-check initializer for variable '{}': {}",
                        assignment.name(),
                        error.message()
                    ),
                ));
            }
            Err(_) => {}
        }
    }

    fn check_constant(&mut self, declaration: &ConstantDeclaration, context: &VisitContext) {
        if let Some(result) = infer_expected_interface_expression_type(
            declaration.expression(),
            declaration.declared_type(),
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.module_implementations,
            self.module_imports,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            match result {
                Ok(actual_type) if &actual_type != declaration.declared_type() => {
                    self.diagnostics.push(Diagnostic::error(
                        declaration.span().clone(),
                        format!(
                            "Initializer for constant '{}' has type {} but declared type is {}",
                            declaration.name(),
                            actual_type.display_name(),
                            declaration.declared_type().display_name()
                        ),
                    ));
                }
                Ok(_) => {}
                Err(error) => self.diagnostics.push(Diagnostic::error(
                    declaration.span().clone(),
                    format!(
                        "Cannot type-check initializer for constant '{}': {}",
                        declaration.name(),
                        error.message()
                    ),
                )),
            }
            return;
        }

        match infer_expression_type(
            declaration.expression(),
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            Ok(actual_type) if &actual_type != declaration.declared_type() => {
                self.diagnostics.push(Diagnostic::error(
                    declaration.span().clone(),
                    format!(
                        "Initializer for constant '{}' has type {} but declared type is {}",
                        declaration.name(),
                        actual_type.display_name(),
                        declaration.declared_type().display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error)
                if declaration.declared_type().primitive_type().is_some()
                    || type_name_is_enum(
                        declaration.declared_type(),
                        self.enum_types,
                        context.current_module.as_deref(),
                    ) =>
            {
                self.diagnostics.push(Diagnostic::error(
                    declaration.span().clone(),
                    format!(
                        "Cannot type-check initializer for constant '{}': {}",
                        declaration.name(),
                        error.message()
                    ),
                ));
            }
            Err(_) => {}
        }
    }
}

impl ParsedVisitor for VariableInitializerChecker<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        match object {
            Object::ConstantDeclaration(declaration) => self.check_constant(declaration, context),
            Object::VariableAssignment(assignment) => self.check_assignment(assignment, context),
            _ => {}
        }
    }
}

fn type_mismatch_diagnostic(
    assignment: &VariableAssignment,
    expected_type: &TypeName,
    actual_type: &TypeName,
) -> Diagnostic {
    Diagnostic::error(
        assignment.span().clone(),
        format!(
            "Initializer for variable '{}' has type {} but declared type is {}",
            assignment.name(),
            actual_type.display_name(),
            expected_type.display_name()
        ),
    )
}

fn default_initializer_metadata(assignment: &VariableAssignment) -> Option<DefaultValue> {
    if assignment.expression().is_some() {
        return None;
    }
    assignment.declared_type()?.default_value()
}

#[cfg(test)]
mod tests {
    use crate::{
        analysis::test_support::assert_single_diagnostic,
        diagnostic::DiagnosticSeverity,
        parsed::{Object, TypeName},
    };

    use super::{super::test_support::parse_story, *};

    #[test]
    fn accepts_valid_primitive_initializers() {
        let story = parse_story(
            "CONST max_score: int = 10\n\
             VAR score: int = max_score\n\
             VAR ratio: float = 1.5\n\
             VAR ready: bool = true\n\
             VAR label: string = \"start\"\n\
             == knot ==\n\
             ~ temp next: int = score + 1\n\
             -> DONE",
        );

        assert_eq!(variable_initializer_diagnostics(&story), []);
    }

    #[test]
    fn accepts_imported_qualified_constant_initializers() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT MAX_SCORE\n\
             VAR score: int = items::MAX_SCORE\n\
             == main ==\n\
             -> DONE\n\
             === module items ===\n\
             CONST MAX_SCORE: int = 10\n\
             == helper ==\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_default_initializers_for_imported_qualified_struct_types() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT Item\n\
             VAR item: items::Item\n\
             == main ==\n\
             -> DONE\n\
             === module items ===\n\
             STRUCT Item {\n\
             hp: int\n\
             }\n\
             == helper ==\n\
             -> DONE",
        );

        assert_eq!(variable_initializer_diagnostics(&story), []);
    }

    #[test]
    fn reports_wrong_type_for_imported_qualified_constant_initializers() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT MAX_SCORE\n\
             VAR score: string = items::MAX_SCORE\n\
             == main ==\n\
             -> DONE\n\
             === module items ===\n\
             CONST MAX_SCORE: int = 10\n\
             == helper ==\n\
             -> DONE",
        );

        let diagnostics = variable_initializer_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Initializer for variable 'score' has type int but declared type is string",
        );
    }

    #[test]
    fn reports_invalid_constant_initializer_type() {
        let story = parse_story("CONST score: int = \"high\"\n-> DONE");

        let diagnostics = variable_initializer_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Initializer for constant 'score' has type string but declared type is int",
        );
    }

    #[test]
    fn reports_invalid_primitive_initializer_type() {
        let story = parse_story("VAR score: int = \"high\"\n-> DONE");

        let diagnostics = variable_initializer_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Initializer for variable 'score' has type string but declared type is int",
        );
    }

    #[test]
    fn rejects_implicit_numeric_conversion_in_initializers() {
        let story = parse_story("VAR ratio: float = 1\n-> DONE");

        let diagnostics = variable_initializer_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Initializer for variable 'ratio' has type int but declared type is float",
        );
    }

    #[test]
    fn rejects_operator_errors_in_primitive_initializers() {
        let story = parse_story("VAR valid: bool = 1 && true\n-> DONE");

        let diagnostics = variable_initializer_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot type-check initializer for variable 'valid': Operator '&&' is not defined for types int and bool",
        );
    }

    #[test]
    fn accepts_divert_target_initializers_and_rejects_missing_defaults() {
        let story = parse_story(
            "CONST fallback: -> = -> knot\n\
             VAR next: -> = fallback\n\
             VAR route: ->[] = [-> knot, next]\n\
             -> DONE\n\
             == knot ==\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);

        let missing = parse_story("VAR next: ->\n-> DONE");
        let diagnostics = variable_initializer_diagnostics(&missing);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Variable 'next' of type -> cannot be default-initialized",
        );
    }

    #[test]
    fn accepts_interface_module_literal_initializers() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module left implements IItem ===\n\
             CONST default_route: interface<IItem> = left\n\
             VAR route: interface<IItem> = left\n\
             == main ==\n\
             -> END\n\
             == target ==\n\
             -> END",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_missing_import_for_interface_module_literal_initializers() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             VAR route: interface<IItem> = left\n\
             == main ==\n\
             -> END\n\
             === module left implements IItem ===\n\
             == target ==\n\
             -> END",
        );

        let diagnostics = variable_initializer_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot type-check initializer for variable 'route': Module literal 'left' requires a bare import in module 'game': FROM left",
        );
    }

    #[test]
    fn rejects_interface_default_initializers() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             VAR route: interface<IItem>\n\
             == main ==\n\
             -> END",
        );

        let diagnostics = variable_initializer_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Variable 'route' of type interface<IItem> cannot be default-initialized",
        );
    }

    #[test]
    fn accepts_interface_array_default_initializers() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             VAR routes: interface<IItem>[]\n\
             == main ==\n\
             -> END",
        );

        assert_eq!(variable_initializer_diagnostics(&story), []);
    }

    #[test]
    fn rejects_non_implementing_modules_as_interface_initializers() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             VAR route: interface<IItem> = left\n\
             == main ==\n\
             -> END\n\
             === module left ===\n\
             == target ==\n\
             -> END",
        );

        let diagnostics = variable_initializer_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot type-check initializer for variable 'route': Module 'left' does not implement interface 'IItem'",
        );
    }

    #[test]
    fn rejects_plain_strings_as_interface_initializers() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             VAR route: interface<IItem> = \"left\"\n\
             == main ==\n\
             -> END",
        );

        let diagnostics = variable_initializer_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Initializer for variable 'route' has type string but declared type is interface<IItem>",
        );
    }

    #[test]
    fn accepts_enum_initializers_and_defaults() {
        let story = parse_story(
            "ENUM State { Idle Busy }\n\
             CONST DEFAULT_STATE: State = State.Busy\n\
             VAR state: State = State.Idle\n\
             VAR default_state: State\n\
             -> DONE",
        );

        assert_eq!(variable_initializer_diagnostics(&story), []);
    }

    #[test]
    fn reports_unknown_enum_member_initializers() {
        let story = parse_story(
            "ENUM State { Idle Busy }\n\
             VAR state: State = State.Missing\n\
             -> DONE",
        );

        let diagnostics = variable_initializer_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot type-check initializer for variable 'state': Unknown member 'Missing' in enum 'State'",
        );
    }

    #[test]
    fn rejects_string_initializers_for_enum_types() {
        let story = parse_story(
            "ENUM State { Idle Busy }\n\
             VAR state: State = \"State.Idle\"\n\
             -> DONE",
        );

        let diagnostics = variable_initializer_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Initializer for variable 'state' has type string but declared type is State",
        );
    }

    #[test]
    fn rejects_non_target_initializer_for_divert_target_type() {
        let story = parse_story("VAR next: -> = 1\n-> DONE");

        let diagnostics = variable_initializer_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Initializer for variable 'next' has type int but declared type is ->",
        );
    }

    #[test]
    fn records_default_metadata_for_omitted_typed_temp_initializers() {
        let story = parse_story(
            "== knot ==\n\
             ~ temp hp: int\n\
             -> DONE",
        );
        let assignment = story.flows()[0]
            .weave()
            .content()
            .iter()
            .find_map(|object| match object {
                Object::VariableAssignment(assignment) if assignment.name() == "hp" => {
                    Some(assignment)
                }
                _ => None,
            })
            .expect("typed temp should parse as a variable assignment");

        assert_eq!(variable_initializer_diagnostics(&story), []);
        assert_eq!(
            default_initializer_metadata(assignment),
            Some(DefaultValue::Int(0))
        );
        assert_eq!(assignment.declared_type(), Some(&TypeName::int()));
    }
}
