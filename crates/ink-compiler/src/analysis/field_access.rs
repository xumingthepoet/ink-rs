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
    expression_types::infer_expression_type,
    indexes::AnalysisIndexes,
    interfaces::InterfaceMemberIndex,
};

#[cfg(test)]
use super::modules::ModuleAnalysis;

#[cfg(test)]
pub(super) fn field_access_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let module_analysis = ModuleAnalysis::build(story);
    let indexes = AnalysisIndexes::build(story, &module_analysis);
    field_access_diagnostics_with_indexes(story, &indexes)
}

pub(super) fn field_access_diagnostics_with_indexes(
    story: &Story,
    indexes: &AnalysisIndexes<'_>,
) -> Vec<Diagnostic> {
    let mut checker = FieldAccessChecker::new(
        &indexes.struct_types,
        &indexes.enum_types,
        &indexes.variable_scopes,
        &indexes.target_symbols,
        &indexes.interface_members,
    );
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct FieldAccessChecker<'a> {
    struct_types: &'a StructTypeIndex,
    enum_types: &'a EnumTypeIndex,
    variable_scopes: &'a VariableScopeIndex,
    target_symbols: &'a TargetSymbolIndex,
    interface_members: &'a InterfaceMemberIndex,
    diagnostics: Vec<Diagnostic>,
    covered_field_access_ids: HashSet<usize>,
}

impl<'a> FieldAccessChecker<'a> {
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
            covered_field_access_ids: HashSet::new(),
        }
    }

    fn mark_assignment_field_accesses(
        &mut self,
        assignment: &VariableAssignment,
        context: &VisitContext,
    ) {
        let Some(expression) = assignment.expression() else {
            return;
        };

        if assignment_expected_type(assignment, context, self.variable_scopes).is_some() {
            mark_field_accesses(expression, &mut self.covered_field_access_ids);
        }
    }

    fn check_field_access(&mut self, expression: &Expression, context: &VisitContext) {
        if self
            .covered_field_access_ids
            .contains(&(expression as *const Expression as usize))
        {
            return;
        }

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

impl ParsedVisitor for FieldAccessChecker<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        if let Object::VariableAssignment(assignment) = object {
            self.mark_assignment_field_accesses(assignment, context);
        }
    }

    fn visit_expression(&mut self, expression: &Expression, context: &VisitContext) {
        if matches!(expression, Expression::FieldAccess { .. }) {
            self.check_field_access(expression, context);
        }
    }
}

fn is_unknown_variable_error(message: &str) -> bool {
    message.starts_with("Unknown variable '")
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

fn mark_field_accesses(expression: &Expression, field_access_ids: &mut HashSet<usize>) {
    match expression {
        Expression::FieldAccess { base, .. } => {
            field_access_ids.insert(expression as *const Expression as usize);
            mark_field_accesses(base, field_access_ids);
        }
        Expression::IndexAccess { base, index } => {
            mark_field_accesses(base, field_access_ids);
            mark_field_accesses(index, field_access_ids);
        }
        Expression::FunctionCall { args, .. }
        | Expression::QualifiedFunctionCall { args, .. }
        | Expression::ArrayLiteral(args)
        | Expression::MultipleCondition(args) => {
            for arg in args {
                mark_field_accesses(arg, field_access_ids);
            }
        }
        Expression::DynamicInterfaceAccess { target, .. } => {
            mark_field_accesses(target, field_access_ids);
        }
        Expression::DynamicInterfaceFunctionCall { target, args, .. } => {
            mark_field_accesses(target, field_access_ids);
            for arg in args {
                mark_field_accesses(arg, field_access_ids);
            }
        }
        Expression::StructLiteral { fields, .. } => {
            for field in fields {
                mark_field_accesses(field.expression(), field_access_ids);
            }
        }
        Expression::DictLiteral(entries) => {
            for entry in entries {
                mark_field_accesses(entry.value(), field_access_ids);
            }
        }
        Expression::Binary { left, right, .. } => {
            mark_field_accesses(left, field_access_ids);
            mark_field_accesses(right, field_access_ids);
        }
        Expression::Unary { expression, .. } => {
            mark_field_accesses(expression, field_access_ids);
        }
        Expression::String(_)
        | Expression::StringContent(_)
        | Expression::NumberInt(_)
        | Expression::NumberFloat(_)
        | Expression::NumberBool(_)
        | Expression::DivertTarget(_)
        | Expression::VariableReference(_)
        | Expression::QualifiedReference(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use crate::{analysis::test_support::assert_single_diagnostic, diagnostic::DiagnosticSeverity};

    use super::{super::test_support::parse_story, *};

    #[test]
    fn resolves_field_read_type_in_typed_initializer_context() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR player: Player = %Player{ hp: 10 }\n\
             VAR hp: int = player.hp\n\
             -> DONE",
        );

        assert_eq!(field_access_diagnostics(&story), []);
    }

    #[test]
    fn resolves_nested_field_reads() {
        let story = parse_story(
            "STRUCT Stats {\n\
             hp: int\n\
             }\n\
             STRUCT Player {\n\
             stats: Stats\n\
             }\n\
             VAR player: Player = %Player{ stats: %Stats{ hp: 10 } }\n\
             VAR hp: int = player.stats.hp\n\
             -> DONE",
        );

        assert_eq!(field_access_diagnostics(&story), []);
    }

    #[test]
    fn resolves_field_reads_from_dynamic_interface_function_results() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == function player() => types::Player ==\n\
             === module game ===\n\
             FROM left\n\
             VAR route: interface<IItem> = left\n\
             VAR hp: int = {route}::player().hp\n\
             == main ==\n\
             -> END\n\
             === module types ===\n\
             STRUCT Player {\n\
             hp: int\n\
             }\n\
             == helper ==\n\
             -> END\n\
             === module left implements IItem ===\n\
             FROM types IMPORT Player\n\
             VAR source_player: types::Player = %types::Player{ hp: 10 }\n\
             == function player() => types::Player ==\n\
             ~ return source_player",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_unknown_field_reads() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR player: Player = %Player{ hp: 10 }\n\
             {player.mp}\n\
             -> DONE",
        );

        let diagnostics = field_access_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Unknown field 'mp' on struct 'Player'",
        );
    }

    #[test]
    fn reports_field_access_on_non_struct_values() {
        let story = parse_story(
            "VAR score: int = 0\n\
             {score.hp}\n\
             -> DONE",
        );

        let diagnostics = field_access_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cannot access field 'hp' on non-struct type int",
        );
    }

    #[test]
    fn dotted_divert_paths_are_not_struct_field_accesses() {
        let story = parse_story(
            "-> knot.stitch\n\
             == knot ==\n\
             = stitch\n\
             -> DONE",
        );

        assert_eq!(field_access_diagnostics(&story), []);
    }

    #[test]
    fn dotted_weave_paths_in_conditions_are_not_struct_field_accesses() {
        let story = parse_story(
            "-> knot\n\
             == knot ==\n\
             = stitch_one\n\
             - (gatherpoint) Some content.\n\
             -> knot.stitch_two\n\
             = stitch_two\n\
             * {knot.stitch_one.gatherpoint}: Found gatherpoint\n\
             -> DONE",
        );

        assert_eq!(field_access_diagnostics(&story), []);
    }
}
