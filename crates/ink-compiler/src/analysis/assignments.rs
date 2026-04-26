use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        AssignmentTarget, IncDec, Object, Story, TypeName, VariableAssignment,
    },
};

use super::{
    context::{StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    expression_types::infer_expression_type,
    structs::{build_struct_type_index, resolve_struct_symbol},
    target_symbols::build_target_symbol_index,
    variables::build_variable_scope_index,
};

pub(super) fn variable_assignment_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let struct_types = build_struct_type_index(story);
    let variable_scopes = build_variable_scope_index(story);
    let target_symbols = build_target_symbol_index(story);
    let mut checker =
        VariableAssignmentChecker::new(&variable_scopes, &struct_types, &target_symbols);
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct VariableAssignmentChecker<'a> {
    variable_scopes: &'a VariableScopeIndex,
    struct_types: &'a StructTypeIndex,
    target_symbols: &'a TargetSymbolIndex,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> VariableAssignmentChecker<'a> {
    fn new(
        variable_scopes: &'a VariableScopeIndex,
        struct_types: &'a StructTypeIndex,
        target_symbols: &'a TargetSymbolIndex,
    ) -> Self {
        Self {
            variable_scopes,
            struct_types,
            target_symbols,
            diagnostics: Vec::new(),
        }
    }

    fn check_assignment(&mut self, assignment: &VariableAssignment, context: &VisitContext) {
        if assignment.is_global() || assignment.is_temporary() {
            return;
        }

        let Some(expression) = assignment.expression() else {
            return;
        };

        let Some(declared_type) =
            self.resolve_assignment_target_type(assignment.target(), context, assignment.span())
        else {
            return;
        };

        self.check_simple_assignment_type(
            assignment.name(),
            declared_type,
            expression,
            assignment.span(),
            context,
        );
    }

    fn check_inc_dec(&mut self, inc_dec: &IncDec, context: &VisitContext) {
        let Some(declared_type) =
            self.resolve_assignment_target_type(inc_dec.target(), context, inc_dec.span())
        else {
            return;
        };

        self.check_compound_assignment_type(
            inc_dec.name(),
            declared_type,
            inc_dec.expression(),
            inc_dec.is_increment(),
            inc_dec.span(),
            context,
        );
    }

    fn visible_declared_type(
        &mut self,
        name: &str,
        context: &VisitContext,
        span: &crate::source::SourceSpan,
    ) -> Option<TypeName> {
        match self.variable_scopes.visible_variable_declared_type(
            name,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            Some(Some(type_name)) => Some(type_name.clone()),
            Some(None) => None,
            None => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!("Cannot assign to undeclared variable '{name}'"),
                ));
                None
            }
        }
    }

    fn qualified_assignment_declared_type(
        &mut self,
        name: &str,
        span: &crate::source::SourceSpan,
    ) -> Option<TypeName> {
        match self
            .variable_scopes
            .qualified_global_variable_declared_type(name)
        {
            Some(Some(type_name)) => return Some(type_name.clone()),
            Some(None) => return None,
            None => {}
        }

        if self
            .variable_scopes
            .qualified_constant_declared_type(name)
            .is_some()
        {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("Cannot assign to constant '{name}'"),
            ));
            return None;
        }

        self.diagnostics.push(Diagnostic::error(
            span.clone(),
            format!("Cannot assign to undeclared variable '{name}'"),
        ));
        None
    }

    fn resolve_assignment_target_type(
        &mut self,
        target: &AssignmentTarget,
        context: &VisitContext,
        span: &crate::source::SourceSpan,
    ) -> Option<TypeName> {
        match target {
            AssignmentTarget::Variable(name) => self.visible_declared_type(name, context, span),
            AssignmentTarget::QualifiedVariable(name) => {
                self.qualified_assignment_declared_type(name.as_str(), span)
            }
            AssignmentTarget::FieldAccess { base, field } => {
                let base_type = self.resolve_assignment_target_type(base, context, span)?;
                let Some(struct_name) = base_type.as_struct_name() else {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Cannot access field '{field}' on non-struct assignment target type {}",
                            base_type.display_name()
                        ),
                    ));
                    return None;
                };

                let Some(symbol) = resolve_struct_symbol(
                    self.struct_types,
                    struct_name,
                    context.current_module.as_deref(),
                ) else {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!("Unknown struct type '{struct_name}' for assignment target"),
                    ));
                    return None;
                };

                symbol.fields().get(field).cloned().or_else(|| {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Unknown field '{field}' on assignment target struct '{struct_name}'"
                        ),
                    ));
                    None
                })
            }
            AssignmentTarget::IndexAccess { base, index } => {
                let base_type = self.resolve_assignment_target_type(base, context, span)?;
                let index_type = match infer_expression_type(
                    index,
                    self.variable_scopes,
                    self.struct_types,
                    self.target_symbols,
                    context.current_module.as_deref(),
                    context.current_flow_path.as_deref(),
                ) {
                    Ok(index_type) => index_type,
                    Err(error) => {
                        self.diagnostics.push(Diagnostic::error(
                            span.clone(),
                            format!(
                                "Cannot type-check index for assignment target '{}': {}",
                                target.display_name(),
                                error.message()
                            ),
                        ));
                        return None;
                    }
                };
                if index_type != TypeName::int() {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Index for assignment target '{}' has type {} but expected int",
                            target.display_name(),
                            index_type.display_name()
                        ),
                    ));
                    return None;
                }

                base_type.array_element_type().cloned().or_else(|| {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Cannot index non-array assignment target type {}",
                            base_type.display_name()
                        ),
                    ));
                    None
                })
            }
        }
    }

    fn check_simple_assignment_type(
        &mut self,
        name: &str,
        declared_type: TypeName,
        expression: &crate::parsed::Expression,
        span: &crate::source::SourceSpan,
        context: &VisitContext,
    ) {
        match infer_assignable_primitive_type(
            &declared_type,
            expression,
            self.variable_scopes,
            self.struct_types,
            self.target_symbols,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            Some(Ok(actual_type)) if actual_type != declared_type => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Assignment to variable '{name}' has type {} but declared type is {}",
                        actual_type.display_name(),
                        declared_type.display_name()
                    ),
                ));
            }
            Some(Ok(_)) | None => {}
            Some(Err(error)) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check assignment to variable '{name}': {}",
                    error.message()
                ),
            )),
        }
    }

    fn check_compound_assignment_type(
        &mut self,
        name: &str,
        declared_type: TypeName,
        expression: &crate::parsed::Expression,
        is_increment: bool,
        span: &crate::source::SourceSpan,
        context: &VisitContext,
    ) {
        match infer_assignable_primitive_type(
            &declared_type,
            expression,
            self.variable_scopes,
            self.struct_types,
            self.target_symbols,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            Some(Ok(value_type))
                if compound_assignment_is_valid(&declared_type, &value_type, is_increment) => {}
            Some(Ok(value_type)) => self.diagnostics.push(compound_assignment_diagnostic(
                name,
                &declared_type,
                &value_type,
                is_increment,
                span,
            )),
            Some(Err(error)) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check compound assignment to variable '{name}': {}",
                    error.message()
                ),
            )),
            None => {}
        }
    }
}

impl ParsedVisitor for VariableAssignmentChecker<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        match object {
            Object::VariableAssignment(assignment) => self.check_assignment(assignment, context),
            Object::IncDec(inc_dec) => self.check_inc_dec(inc_dec, context),
            _ => {}
        }
    }
}

fn infer_assignable_primitive_type(
    declared_type: &TypeName,
    expression: &crate::parsed::Expression,
    variable_scopes: &VariableScopeIndex,
    struct_types: &StructTypeIndex,
    target_symbols: &TargetSymbolIndex,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
) -> Option<Result<TypeName, super::expression_types::TypeInferenceError>> {
    match infer_expression_type(
        expression,
        variable_scopes,
        struct_types,
        target_symbols,
        current_module,
        current_flow_path,
    ) {
        Ok(actual_type) => Some(Ok(actual_type)),
        Err(error) if declared_type.primitive_type().is_some() => Some(Err(error)),
        Err(_) => None,
    }
}

fn compound_assignment_is_valid(
    declared_type: &TypeName,
    value_type: &TypeName,
    is_increment: bool,
) -> bool {
    if declared_type != value_type {
        return false;
    }

    if declared_type == &TypeName::int() || declared_type == &TypeName::float() {
        return true;
    }

    is_increment && declared_type == &TypeName::string()
}

fn compound_assignment_diagnostic(
    name: &str,
    declared_type: &TypeName,
    value_type: &TypeName,
    is_increment: bool,
    span: &crate::source::SourceSpan,
) -> Diagnostic {
    Diagnostic::error(
        span.clone(),
        format!(
            "Operator '{}' is not defined for variable '{}' of type {} and value type {}",
            if is_increment { "+=" } else { "-=" },
            name,
            declared_type.display_name(),
            value_type.display_name()
        ),
    )
}

#[cfg(test)]
mod tests {
    use crate::{analysis::test_support::assert_single_diagnostic, diagnostic::DiagnosticSeverity};

    use super::{super::test_support::parse_story, *};

    #[test]
    fn accepts_valid_simple_assignments() {
        let story = parse_story(
            "VAR score: int = 0\n\
             VAR label: string = \"a\"\n\
             == knot ==\n\
             ~ temp ready: bool = true\n\
             ~ score = score + 1\n\
             ~ ready = false\n\
             ~ label = \"b\"\n\
             -> DONE",
        );

        assert_eq!(variable_assignment_diagnostics(&story), []);
    }

    #[test]
    fn reports_invalid_simple_assignment_type() {
        let story = parse_story(
            "VAR score: int = 0\n\
             ~ score = 1.5\n\
             -> DONE",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Assignment to variable 'score' has type float but declared type is int",
        );
    }

    #[test]
    fn rejects_implicit_numeric_conversion_in_assignment() {
        let story = parse_story(
            "VAR ratio: float = 0.0\n\
             ~ ratio = 1\n\
             -> DONE",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Assignment to variable 'ratio' has type int but declared type is float",
        );
    }

    #[test]
    fn reports_assignment_to_undeclared_variable() {
        let story = parse_story("~ missing = 1\n-> DONE");

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot assign to undeclared variable 'missing'",
        );
    }

    #[test]
    fn accepts_valid_numeric_and_string_compound_assignments() {
        let story = parse_story(
            "VAR score: int = 0\n\
             VAR ratio: float = 0.0\n\
             VAR label: string = \"a\"\n\
             ~ score += 1\n\
             ~ score -= 1\n\
             ~ ratio += 1.0\n\
             ~ label += \"b\"\n\
             -> DONE",
        );

        assert_eq!(variable_assignment_diagnostics(&story), []);
    }

    #[test]
    fn rejects_invalid_compound_assignment_type() {
        let story = parse_story(
            "VAR score: int = 0\n\
             ~ score += 1.0\n\
             -> DONE",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Operator '+=' is not defined for variable 'score' of type int and value type float",
        );
    }

    #[test]
    fn rejects_string_subtract_assignment() {
        let story = parse_story(
            "VAR label: string = \"a\"\n\
             ~ label -= \"b\"\n\
             -> DONE",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Operator '-=' is not defined for variable 'label' of type string and value type string",
        );
    }

    #[test]
    fn accepts_valid_field_and_index_assignments() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             name: string\n\
             }\n\
             VAR player: Player = { hp: 10, name: \"Ada\" }\n\
             VAR scores: int[] = [1]\n\
             ~ player.hp = 11\n\
             ~ player.name = \"Grace\"\n\
             ~ scores[0] = 2\n\
             -> DONE",
        );

        assert_eq!(variable_assignment_diagnostics(&story), []);
    }

    #[test]
    fn reports_invalid_field_assignment_type() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR player: Player = { hp: 10 }\n\
             ~ player.hp = \"high\"\n\
             -> DONE",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Assignment to variable 'player.hp' has type string but declared type is int",
        );
    }

    #[test]
    fn reports_invalid_index_assignment_type() {
        let story = parse_story(
            "VAR scores: int[] = [1]\n\
             ~ scores[0] = \"two\"\n\
             -> DONE",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Assignment to variable 'scores[Number(0)]' has type string but declared type is int",
        );
    }

    #[test]
    fn accepts_nested_lvalue_assignment_and_compound_assignment() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR party: Player[] = [{ hp: 10 }]\n\
             ~ party[0].hp = 11\n\
             ~ party[0].hp += 1\n\
             -> DONE",
        );

        assert_eq!(variable_assignment_diagnostics(&story), []);
    }

    #[test]
    fn reports_invalid_nested_lvalue_assignment_type() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR party: Player[] = [{ hp: 10 }]\n\
             ~ party[0].hp = \"low\"\n\
             -> DONE",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Assignment to variable 'party[Number(0)].hp' has type string but declared type is int",
        );
    }

    #[test]
    fn rejects_invalid_field_compound_assignment_type() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR player: Player = { hp: 10 }\n\
             ~ player.hp += 1.0\n\
             -> DONE",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Operator '+=' is not defined for variable 'player.hp' of type int and value type float",
        );
    }

    #[test]
    fn module_assignment_lookup_uses_current_module_only() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             ~ value = 1\n\
             -> DONE\n\
             === module items ===\n\
             VAR value: int = 0\n\
             == helper ==\n\
             -> DONE",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot assign to undeclared variable 'value'",
        );
    }

    #[test]
    fn module_assignment_lookup_accepts_same_module_globals() {
        let story = parse_story(
            "=== module game ===\n\
             VAR value: int = 0\n\
             == main ==\n\
             ~ value = 1\n\
             -> DONE\n\
             === module items ===\n\
             VAR value: string = \"item\"\n\
             == helper ==\n\
             -> DONE",
        );

        assert_eq!(variable_assignment_diagnostics(&story), []);
    }

    #[test]
    fn accepts_qualified_imported_global_assignments() {
        let story = parse_story(
            "=== module game ===\n\
             IMPORT score FROM state\n\
             == main ==\n\
             ~ state::score = 1\n\
             ~ state::score += 2\n\
             ~ state::score -= 1\n\
             -> END\n\
             === module state ===\n\
             VAR score: int = 0\n\
             == helper ==\n\
             -> END",
        );

        assert_eq!(variable_assignment_diagnostics(&story), []);
    }

    #[test]
    fn rejects_assignment_to_qualified_constant() {
        let story = parse_story(
            "=== module game ===\n\
             IMPORT LIMIT FROM state\n\
             == main ==\n\
             ~ state::LIMIT = 1\n\
             -> END\n\
             === module state ===\n\
             CONST LIMIT: int = 0\n\
             == helper ==\n\
             -> END",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot assign to constant 'state::LIMIT'",
        );
    }

    #[test]
    fn rejects_invalid_qualified_global_assignment_type() {
        let story = parse_story(
            "=== module game ===\n\
             IMPORT score FROM state\n\
             == main ==\n\
             ~ state::score = \"high\"\n\
             -> END\n\
             === module state ===\n\
             VAR score: int = 0\n\
             == helper ==\n\
             -> END",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Assignment to variable 'state::score' has type string but declared type is int",
        );
    }

    #[test]
    fn rejects_invalid_qualified_global_compound_assignment_type() {
        let story = parse_story(
            "=== module game ===\n\
             IMPORT score FROM state\n\
             == main ==\n\
             ~ state::score += 1.5\n\
             -> END\n\
             === module state ===\n\
             VAR score: int = 0\n\
             == helper ==\n\
             -> END",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Operator '+=' is not defined for variable 'state::score' of type int and value type float",
        );
    }

    #[test]
    fn rejects_assignment_to_missing_qualified_global() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             ~ state::missing = 1\n\
             -> END\n\
             === module state ===\n\
             VAR score: int = 0\n\
             == helper ==\n\
             -> END",
        );

        let diagnostics = variable_assignment_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot assign to undeclared variable 'state::missing'",
        );
    }
}
