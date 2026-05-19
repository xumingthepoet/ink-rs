use std::collections::HashSet;

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        Expression, Object, Story, TypeName, VariableAssignment,
    },
};

use super::{
    context::{EnumTypeIndex, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    enums::build_enum_type_index,
    expression_types::infer_expression_type,
    interfaces::{build_interface_member_index, InterfaceMemberIndex},
    structs::build_struct_type_index,
    target_symbols::build_target_symbol_index,
    variables::build_variable_scope_index,
};

pub(super) fn index_access_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let struct_types = build_struct_type_index(story);
    let enum_types = build_enum_type_index(story);
    let variable_scopes = build_variable_scope_index(story);
    let target_symbols = build_target_symbol_index(story);
    let interface_members = build_interface_member_index(story);
    let mut checker = IndexAccessChecker::new(
        &struct_types,
        &enum_types,
        &variable_scopes,
        &target_symbols,
        &interface_members,
    );
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct IndexAccessChecker<'a> {
    struct_types: &'a StructTypeIndex,
    enum_types: &'a EnumTypeIndex,
    variable_scopes: &'a VariableScopeIndex,
    target_symbols: &'a TargetSymbolIndex,
    interface_members: &'a InterfaceMemberIndex,
    diagnostics: Vec<Diagnostic>,
    covered_index_access_ids: HashSet<usize>,
}

impl<'a> IndexAccessChecker<'a> {
    fn new(
        struct_types: &'a StructTypeIndex,
        enum_types: &'a EnumTypeIndex,
        variable_scopes: &'a VariableScopeIndex,
        target_symbols: &'a TargetSymbolIndex,
        interface_members: &'a InterfaceMemberIndex,
    ) -> Self {
        Self {
            struct_types,
            enum_types,
            variable_scopes,
            target_symbols,
            interface_members,
            diagnostics: Vec::new(),
            covered_index_access_ids: HashSet::new(),
        }
    }

    fn mark_assignment_index_accesses(
        &mut self,
        assignment: &VariableAssignment,
        context: &VisitContext,
    ) {
        let Some(expression) = assignment.expression() else {
            return;
        };

        if assignment_expected_type(assignment, context, self.variable_scopes).is_some() {
            mark_index_accesses(expression, &mut self.covered_index_access_ids);
        }
    }

    fn check_index_access(&mut self, expression: &Expression, context: &VisitContext) {
        let expression_id = expression as *const Expression as usize;
        if self.covered_index_access_ids.contains(&expression_id) {
            return;
        }
        mark_index_accesses(expression, &mut self.covered_index_access_ids);

        if let Err(error) = infer_expression_type(
            expression,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            if is_unknown_variable_error(error.message()) {
                return;
            }
            self.diagnostics.push(Diagnostic::error(
                crate::source::SourceSpan::new(None, 1, 1),
                error.message().to_string(),
            ));
        }
    }
}

impl ParsedVisitor for IndexAccessChecker<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        if let Object::VariableAssignment(assignment) = object {
            self.mark_assignment_index_accesses(assignment, context);
        }
    }

    fn visit_expression(&mut self, expression: &Expression, context: &VisitContext) {
        if matches!(expression, Expression::IndexAccess { .. }) {
            self.check_index_access(expression, context);
        }
    }
}

fn assignment_expected_type(
    assignment: &VariableAssignment,
    context: &VisitContext,
    variable_scopes: &VariableScopeIndex,
) -> Option<TypeName> {
    if assignment.is_global() || assignment.is_temporary() {
        return assignment.declared_type().cloned();
    }

    assignment
        .target()
        .variable_name()
        .and_then(|name| {
            variable_scopes.visible_variable_declared_type(
                name,
                context.current_module.as_deref(),
                context.current_flow_path.as_deref(),
            )
        })
        .and_then(|declared_type| declared_type.cloned())
}

fn mark_index_accesses(expression: &Expression, index_access_ids: &mut HashSet<usize>) {
    match expression {
        Expression::IndexAccess { base, index } => {
            index_access_ids.insert(expression as *const Expression as usize);
            mark_index_accesses(base, index_access_ids);
            mark_index_accesses(index, index_access_ids);
        }
        Expression::FieldAccess { base, .. } => {
            mark_index_accesses(base, index_access_ids);
        }
        Expression::FunctionCall { args, .. }
        | Expression::QualifiedFunctionCall { args, .. }
        | Expression::ArrayLiteral(args)
        | Expression::MultipleCondition(args) => {
            for arg in args {
                mark_index_accesses(arg, index_access_ids);
            }
        }
        Expression::DynamicInterfaceAccess { target, .. } => {
            mark_index_accesses(target, index_access_ids);
        }
        Expression::DynamicInterfaceFunctionCall { target, args, .. } => {
            mark_index_accesses(target, index_access_ids);
            for arg in args {
                mark_index_accesses(arg, index_access_ids);
            }
        }
        Expression::StructLiteral(fields) => {
            for field in fields {
                mark_index_accesses(field.expression(), index_access_ids);
            }
        }
        Expression::DictLiteral(entries) => {
            for entry in entries {
                mark_index_accesses(entry.value(), index_access_ids);
            }
        }
        Expression::Binary { left, right, .. } => {
            mark_index_accesses(left, index_access_ids);
            mark_index_accesses(right, index_access_ids);
        }
        Expression::Unary { expression, .. } => {
            mark_index_accesses(expression, index_access_ids);
        }
        Expression::String(_)
        | Expression::StringContent(_)
        | Expression::NumberInt(_)
        | Expression::NumberFloat(_)
        | Expression::NumberBool(_)
        | Expression::DivertTarget(_)
        | Expression::VariableReference(_)
        | Expression::QualifiedReference(_)
        | Expression::EmptyCompositeLiteral => {}
    }
}

fn is_unknown_variable_error(message: &str) -> bool {
    message.starts_with("Unknown variable '")
}

#[cfg(test)]
mod tests {
    use crate::{analysis::test_support::assert_single_diagnostic, diagnostic::DiagnosticSeverity};

    use super::{super::test_support::parse_story, *};

    #[test]
    fn resolves_array_index_type_in_typed_initializer_context() {
        let story = parse_story(
            "VAR items: int[] = [1]\n\
             VAR first: int = items[0]\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn resolves_nested_array_index_type() {
        let story = parse_story(
            "VAR matrix: int[][] = [[1]]\n\
             VAR first: int = matrix[0][0]\n\
             -> DONE",
        );

        assert_eq!(index_access_diagnostics(&story), []);
    }

    #[test]
    fn resolves_dict_index_type_in_typed_initializer_context() {
        let story = parse_story(
            "VAR scores: Dict<string, int> = {\"ada\": 10}\n\
             VAR first: int = scores[\"ada\"]\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn resolves_nested_dict_and_array_index_chains() {
        let story = parse_story(
            "VAR table: Dict<int, Dict<string, int[]>> = {1: {\"scores\": [10]}}\n\
             VAR first: int = table[1][\"scores\"][0]\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn resolves_index_reads_from_dynamic_interface_function_results() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == function scores() => int[] ==\n\
             === module game ===\n\
             FROM left\n\
             VAR route: interface<IItem> = left\n\
             VAR first: int = {route}::scores()[0]\n\
             == main ==\n\
             -> END\n\
             === module left implements IItem ===\n\
             VAR source_scores: int[] = [10]\n\
             == function scores() => int[] ==\n\
             ~ return source_scores",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_non_int_index_expression() {
        let story = parse_story(
            "VAR items: int[] = [1]\n\
             {items[true]}\n\
             -> DONE",
        );

        let diagnostics = index_access_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Index expression has type bool but expected int",
        );
    }

    #[test]
    fn reports_string_array_index_expression() {
        let story = parse_story(
            "VAR items: int[] = [1]\n\
             {items[\"0\"]}\n\
             -> DONE",
        );

        let diagnostics = index_access_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Index expression has type string but expected int",
        );
    }

    #[test]
    fn reports_wrong_dict_key_type() {
        let story = parse_story(
            "VAR scores: Dict<string, int> = {\"ada\": 10}\n\
             {scores[1]}\n\
             -> DONE",
        );

        let diagnostics = index_access_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Index expression has type int but expected string",
        );
    }

    #[test]
    fn accepts_int_dict_keys() {
        let story = parse_story(
            "VAR scores: Dict<int, string> = {1: \"ada\"}\n\
             VAR first: string = scores[1]\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_indexing_non_array_values() {
        let story = parse_story(
            "VAR score: int = 1\n\
             {score[0]}\n\
             -> DONE",
        );

        let diagnostics = index_access_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot index non-array/non-Dict type int",
        );
    }

    #[test]
    fn does_not_allow_string_indexing() {
        let story = parse_story(
            "VAR label: string = \"abc\"\n\
             {label[0]}\n\
             -> DONE",
        );

        let diagnostics = index_access_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot index non-array/non-Dict type string",
        );
    }
}
