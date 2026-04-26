use std::collections::{BTreeSet, HashSet};

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
    context::{StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    expression_types::infer_expression_type,
    structs::build_struct_type_index,
    target_symbols::build_target_symbol_index,
    variables::build_variable_scope_index,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StructLiteralMode {
    ArraysOnly,
    Full,
}

pub(super) fn array_literal_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let struct_types = build_struct_type_index(story);
    let variable_scopes = build_variable_scope_index(story);
    let target_symbols = build_target_symbol_index(story);
    let mut checker = ArrayLiteralChecker::new(&struct_types, &variable_scopes, &target_symbols);
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct ArrayLiteralChecker<'a> {
    struct_types: &'a StructTypeIndex,
    variable_scopes: &'a VariableScopeIndex,
    target_symbols: &'a TargetSymbolIndex,
    diagnostics: Vec<Diagnostic>,
    expected_expression_ids: HashSet<usize>,
}

impl<'a> ArrayLiteralChecker<'a> {
    fn new(
        struct_types: &'a StructTypeIndex,
        variable_scopes: &'a VariableScopeIndex,
        target_symbols: &'a TargetSymbolIndex,
    ) -> Self {
        Self {
            struct_types,
            variable_scopes,
            target_symbols,
            diagnostics: Vec::new(),
            expected_expression_ids: HashSet::new(),
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

    fn visible_declared_type(&self, name: &str, context: &VisitContext) -> Option<TypeName> {
        self.variable_scopes
            .visible_variable_declared_type(name, context.current_flow_path.as_deref())
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
            (TypeName::Struct(struct_name), Expression::StructLiteral(fields)) => {
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
            (TypeName::Struct(struct_name), Expression::StructLiteral(fields)) => {
                self.check_struct_literal(
                    struct_name,
                    fields,
                    span,
                    context,
                    StructLiteralMode::Full,
                );
            }
            (TypeName::Struct(_), _) | (TypeName::Primitive(_), _) => {
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
        match infer_expression_type(
            expression,
            self.variable_scopes,
            self.struct_types,
            self.target_symbols,
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
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check value for '{}': {}",
                    context_name,
                    error.message()
                ),
            )),
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
        match infer_expression_type(
            expression,
            self.variable_scopes,
            self.struct_types,
            self.target_symbols,
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
            Err(error) if expected_type.primitive_type().is_some() => {
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
        mode: StructLiteralMode,
    ) {
        let Some(symbol) = self.struct_types.get(struct_name) else {
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
                    if let Expression::StructLiteral(nested_fields) = field.expression() {
                        self.check_struct_literal(
                            nested_struct_name,
                            nested_fields,
                            span,
                            context,
                            StructLiteralMode::ArraysOnly,
                        );
                    }
                }
                (StructLiteralMode::ArraysOnly, TypeName::Primitive(_) | TypeName::Void) => {}
            }
        }
    }

    fn mark_expected_expression(&mut self, expression: &Expression) {
        self.expected_expression_ids
            .insert(expression as *const Expression as usize);
    }
}

impl ParsedVisitor for ArrayLiteralChecker<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        match object {
            Object::ConstantDeclaration(declaration) => self.check_constant(declaration, context),
            Object::VariableAssignment(assignment) => self.check_assignment(assignment, context),
            _ => {}
        }
    }

    fn visit_expression(&mut self, expression: &Expression, _context: &VisitContext) {
        if matches!(expression, Expression::ArrayLiteral(_))
            && !self
                .expected_expression_ids
                .contains(&(expression as *const Expression as usize))
        {
            self.diagnostics.push(Diagnostic::error(
                SourceSpan::new(None, 1, 1),
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
    fn accepts_nested_array_literals() {
        let story = parse_story(
            "VAR matrix: int[][] = [[1, 2], []]\n\
             -> DONE",
        );

        assert_eq!(array_literal_diagnostics(&story), []);
    }

    #[test]
    fn accepts_struct_array_literals() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR party: Player[] = [{ hp: 10 }, {}]\n\
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
             VAR player: Player = { scores: [1, 2] }\n\
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
