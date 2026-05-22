use std::collections::HashMap;

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{ParsedVisitor, VisitContext},
        DivertTarget, Expression, Flow, Object,
    },
    source::SourceSpan,
};

use super::super::{
    context::{EnumTypeIndex, FlowContext, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    enums::is_enum_member_reference,
    expression_types::infer_expression_type,
    interface_values::{InterfaceModuleLiteralUses, ModuleImplementationIndex},
    interfaces::InterfaceMemberIndex,
    modules::ModuleImportIndex,
    span::object_span,
};
use super::context::scoped_context_key;
use super::diverts::static_divert_target_name;

pub(super) struct CallTargetChecker<'a> {
    pub(super) target_symbols: &'a TargetSymbolIndex,
    pub(super) variable_scopes: &'a VariableScopeIndex,
    pub(super) struct_types: &'a StructTypeIndex,
    pub(super) enum_types: &'a EnumTypeIndex,
    pub(super) interface_members: &'a InterfaceMemberIndex,
    pub(super) module_implementations: &'a ModuleImplementationIndex,
    pub(super) module_imports: &'a ModuleImportIndex,
    pub(super) interface_module_literal_uses: &'a InterfaceModuleLiteralUses,
    pub(super) diagnostics: Vec<Diagnostic>,
    pub(super) flow_contexts_by_path: HashMap<String, FlowContext>,
}

impl<'a> CallTargetChecker<'a> {
    pub(super) fn new(
        target_symbols: &'a TargetSymbolIndex,
        variable_scopes: &'a VariableScopeIndex,
        struct_types: &'a StructTypeIndex,
        enum_types: &'a EnumTypeIndex,
        interface_members: &'a InterfaceMemberIndex,
        module_implementations: &'a ModuleImplementationIndex,
        module_imports: &'a ModuleImportIndex,
        interface_module_literal_uses: &'a InterfaceModuleLiteralUses,
    ) -> Self {
        Self {
            target_symbols,
            variable_scopes,
            struct_types,
            enum_types,
            interface_members,
            module_implementations,
            module_imports,
            interface_module_literal_uses,
            diagnostics: Vec::new(),
            flow_contexts_by_path: HashMap::new(),
        }
    }

    pub(super) fn check_expression(
        &mut self,
        expression: &Expression,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        match expression {
            Expression::FunctionCall { name, args } => {
                self.check_function_call(name, args, span, context);
            }
            Expression::QualifiedFunctionCall { name, args } => {
                self.check_function_call(name.as_str(), args, span, context);
            }
            Expression::DynamicInterfaceAccess { target, member } => {
                let dynamic_access = Expression::DynamicInterfaceAccess {
                    target: target.clone(),
                    member: member.clone(),
                };
                if let Err(error) = infer_expression_type(
                    &dynamic_access,
                    self.variable_scopes,
                    self.struct_types,
                    self.enum_types,
                    self.target_symbols,
                    self.interface_members,
                    self.current_module(context),
                    self.current_flow_path(context),
                ) {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Cannot type-check dynamic interface access: {}",
                            error.message()
                        ),
                    ));
                }
                self.check_expression(target, span, context);
            }
            Expression::DynamicInterfaceFunctionCall {
                target,
                member,
                args,
            } => {
                self.check_dynamic_interface_function_call(target, member, args, span, context);
            }
            Expression::ArrayLiteral(elements) => {
                for element in elements {
                    self.check_expression(element, span, context);
                }
            }
            Expression::StructLiteral { fields, .. } => {
                for field in fields {
                    self.check_expression(field.expression(), span, context);
                }
            }
            Expression::DictLiteral(entries) => {
                for entry in entries {
                    self.check_expression(entry.value(), span, context);
                }
            }
            Expression::FieldAccess { base, .. }
                if !is_enum_member_reference(
                    expression,
                    self.enum_types,
                    self.current_module(context),
                ) =>
            {
                self.check_expression(base, span, context);
            }
            Expression::FieldAccess { .. } => {}
            Expression::IndexAccess { base, index } => {
                self.check_expression(base, span, context);
                self.check_expression(index, span, context);
            }
            Expression::Binary { left, right, .. } => {
                self.check_expression(left, span, context);
                self.check_expression(right, span, context);
            }
            Expression::Unary { expression, .. } => {
                self.check_expression(expression, span, context);
            }
            Expression::MultipleCondition(expressions) => {
                for expression in expressions {
                    self.check_expression(expression, span, context);
                }
            }
            Expression::DivertTarget(target) => {
                self.check_divert_target_value(target, span, context);
            }
            Expression::VariableReference(name) => {
                self.check_variable_reference(expression, name, span, context);
            }
            Expression::QualifiedReference(name) => {
                self.check_variable_reference(expression, name.as_str(), span, context);
            }
            Expression::StringContent(_)
            | Expression::String(_)
            | Expression::NumberInt(_)
            | Expression::NumberFloat(_)
            | Expression::NumberBool(_) => {}
        }
    }

    fn check_variable_reference(
        &mut self,
        expression: &Expression,
        name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if self
            .interface_module_literal_uses
            .contains_expression(expression)
        {
            return;
        }

        let current_flow_path = self.current_flow_path(context);
        if name.contains("::")
            && self
                .variable_scopes
                .qualified_constant_declared_type(name)
                .or_else(|| {
                    self.variable_scopes
                        .qualified_global_variable_declared_type(name)
                })
                .is_some()
        {
            return;
        }

        if name.contains('.') {
            return;
        }

        if self.variable_scopes.contains_visible_variable(
            name,
            self.current_module(context),
            current_flow_path,
        ) {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            span.clone(),
            format!("Unresolved variable: {name}"),
        ));
    }
}

impl ParsedVisitor for CallTargetChecker<'_> {
    fn visit_flow(&mut self, flow: &Flow, context: &VisitContext) {
        let Some(flow_path) = self.current_flow_path(context) else {
            return;
        };
        self.flow_contexts_by_path
            .entry(scoped_context_key(
                context.current_module.as_deref(),
                flow_path,
            ))
            .or_insert_with(|| FlowContext::new(flow.arguments().to_vec(), flow.is_function()));
    }

    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        match object {
            Object::Divert(divert) => {
                let arguments_checked_by_dynamic_interface_target = matches!(
                    divert.target(),
                    DivertTarget::Dynamic(Expression::DynamicInterfaceAccess { .. })
                );
                match divert.target() {
                    DivertTarget::Empty => self.diagnostics.push(Diagnostic::error(
                        divert.span().clone(),
                        "Empty diverts (->) are only valid on choices",
                    )),
                    DivertTarget::Dynamic(expression) => {
                        self.check_dynamic_divert_target(
                            expression,
                            divert.arguments(),
                            divert.span(),
                            context,
                        );
                    }
                    DivertTarget::Path(_) | DivertTarget::QualifiedPath(_)
                        if !self.current_flow_is_function(context) =>
                    {
                        self.check_plain_divert_target(divert, context);
                    }
                    DivertTarget::Path(_) | DivertTarget::QualifiedPath(_) => {}
                }
                if !arguments_checked_by_dynamic_interface_target {
                    for argument in divert.arguments() {
                        self.check_expression(argument, divert.span(), context);
                    }
                }
            }
            Object::ConstantDeclaration(declaration) => {
                self.check_expression(declaration.expression(), declaration.span(), context);
            }
            Object::Expression(expression) | Object::LogicLine(expression) => {
                self.check_expression(expression, &object_span(object), context);
            }
            Object::VariableAssignment(assignment) => {
                if let Some(expression) = assignment.expression() {
                    self.check_expression(expression, assignment.span(), context);
                }
            }
            Object::IncDec(inc_dec) => {
                self.check_expression(inc_dec.expression(), inc_dec.span(), context);
            }
            Object::Return(ret) => {
                if let Some(expression) = ret.returned_expression() {
                    self.check_expression(expression, ret.span(), context);
                }
            }
            Object::Conditional(conditional) => {
                let span = object_span(object);
                if let Some(condition) = conditional.initial_condition() {
                    self.check_expression(condition, &span, context);
                }
                for branch in conditional.branches() {
                    if let Some(condition) = branch.own_condition() {
                        self.check_expression(condition, &span, context);
                    }
                }
            }
            Object::ForLoop(for_loop) => {
                self.check_expression(for_loop.iterable(), for_loop.span(), context);
            }
            Object::Choice(choice) => {
                if let Some(condition) = choice.condition() {
                    self.check_expression(condition, choice.span(), context);
                }
            }
            Object::TunnelOnwards(tunnel_onwards) => {
                let arguments_checked_by_dynamic_interface_target = matches!(
                    tunnel_onwards.override_target(),
                    Some(DivertTarget::Dynamic(
                        Expression::DynamicInterfaceAccess { .. }
                    ))
                );
                if let Some(target) = tunnel_onwards.override_target() {
                    match target {
                        DivertTarget::Dynamic(expression) => {
                            self.check_dynamic_divert_target(
                                expression,
                                tunnel_onwards.arguments(),
                                tunnel_onwards.span(),
                                context,
                            );
                        }
                        DivertTarget::Path(_) | DivertTarget::QualifiedPath(_) => {
                            if let Some(target) = static_divert_target_name(target) {
                                self.check_cross_module_stitch_target(
                                    target,
                                    tunnel_onwards.span(),
                                    context,
                                );
                            }
                        }
                        DivertTarget::Empty => {}
                    }
                }
                if !arguments_checked_by_dynamic_interface_target {
                    for argument in tunnel_onwards.arguments() {
                        self.check_expression(argument, tunnel_onwards.span(), context);
                    }
                }
            }
            Object::AuthorWarning(_)
            | Object::ContentList(_)
            | Object::EnumDeclaration(_)
            | Object::ExternalDeclaration(_)
            | Object::Gather(_)
            | Object::Glue(_)
            | Object::StructDeclaration(_)
            | Object::Tag(_)
            | Object::Text(_)
            | Object::Weave(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::diagnostic::DiagnosticSeverity;

    use super::super::super::test_support::{assert_single_diagnostic, parse_story};
    use super::super::call_target_diagnostics;

    #[test]
    fn reports_missing_divert_targets() {
        let story = parse_story("-> missing");
        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "target not found: 'missing'",
        );
    }

    #[test]
    fn reports_non_function_call_targets() {
        let story = parse_story("~ knot()\n== knot ==\n");
        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "knot hasn't been marked as a function, but it's being called as one. Do you need to declare the knot as '== function knot =='?",
        );
    }

    #[test]
    fn accepts_typed_function_calls_and_return_type_use() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR source_player: Player = %Player{ hp: 10 }\n\
             VAR source_scores: int[] = [1]\n\
             VAR source_lookup: Dict<string, int> = %{\"ada\": 10}\n\
             VAR source_lookup_list: Dict<string, int>[] = [source_lookup]\n\
             VAR result: int = add(1, 2)\n\
             VAR copied_player: Player = echo_player(source_player)\n\
             VAR copied_scores: int[] = echo_scores(source_scores)\n\
             VAR copied_lookup: Dict<string, int> = echo_lookup(source_lookup)\n\
             VAR copied_lookup_list: Dict<string, int>[] = echo_lookup_list(source_lookup_list)\n\
             Done.\n\
             == function add(a: int, b: int) => int ==\n\
             ~ return a + b\n\
             == function echo_player(player: Player) => Player ==\n\
             ~ return player\n\
             == function echo_scores(values: int[]) => int[] ==\n\
             ~ return values\n\
             == function echo_lookup(values: Dict<string, int>) => Dict<string, int> ==\n\
             ~ return values\n\
             == function echo_lookup_list(values: Dict<string, int>[]) => Dict<string, int>[] ==\n\
             ~ return values",
        );

        assert_eq!(super::super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_typed_external_calls_and_return_type_use() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             EXTERNAL external_score(value: int) => int\n\
             EXTERNAL describe(player: Player, scores: int[]) => string\n\
             EXTERNAL copy_scores(scores: Dict<string, int>) => Dict<string, int>\n\
             VAR score: int = 1\n\
             VAR source_player: Player = %Player{ hp: 10 }\n\
             VAR scores: int[] = [score]\n\
             VAR score_lookup: Dict<string, int> = %{\"ada\": 10}\n\
             VAR adjusted_score: int = external_score(score) + LEN(scores)\n\
             VAR description: string = describe(source_player, scores)\n\
             VAR copied_scores: Dict<string, int> = copy_scores(score_lookup)\n\
             Done.",
        );

        assert_eq!(super::super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_dict_types_in_dynamic_interface_function_signatures() {
        let story = parse_story(
            "=== interface IScorer ===\n\
             == function copy_scores(scores: Dict<string, int>) => Dict<string, int> ==\n\
             === module game ===\n\
             FROM scorer\n\
             VAR route: interface<IScorer> = scorer\n\
             VAR source_scores: Dict<string, int> = %{\"ada\": 10}\n\
             VAR copied_scores: Dict<string, int> = {route}::copy_scores(source_scores)\n\
             == main ==\n\
             \n\
             === module scorer implements IScorer ===\n\
             == function copy_scores(scores: Dict<string, int>) => Dict<string, int> ==\n\
             ~ return scores",
        );

        assert_eq!(super::super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_typed_external_call_argument_count_mismatch() {
        let story = parse_story(
            "EXTERNAL external_score(value: int) => int\n\
             ~ external_score()\n\
             Done.",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Function 'external_score' expects 1 arguments but got 0",
        );
    }

    #[test]
    fn reports_typed_external_call_argument_type_mismatch() {
        let story = parse_story(
            "EXTERNAL external_score(value: int) => int\n\
             VAR label: string = \"x\"\n\
             ~ external_score(label)\n\
             ",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Argument 'value' for function 'external_score' has type string but expected int",
        );
    }

    #[test]
    fn external_return_type_participates_in_expression_type_checks() {
        let story = parse_story(
            "EXTERNAL external_label() => string\n\
             VAR score: int = external_label()\n\
             Done.",
        );

        let diagnostics = super::super::super::run_analysis_passes(&story);

        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == DiagnosticSeverity::Error
                    && diagnostic.message
                        == "Initializer for variable 'score' has type string but declared type is int"
            }),
            "{diagnostics:#?}"
        );
    }

    #[test]
    fn accepts_len_builtin_for_array_types() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR score: int = 1\n\
             VAR source_player: Player = %Player{ hp: 10 }\n\
             VAR scores: int[] = [score]\n\
             VAR players: Player[] = [source_player]\n\
             VAR nested_scores: int[][] = [scores]\n\
             VAR score_count: int = LEN(scores)\n\
             VAR player_count: int = LEN(players)\n\
             VAR row_count: int = LEN(nested_scores)\n\
             ",
        );

        assert_eq!(super::super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_len_builtin_non_array_arguments() {
        let cases = [
            (
                "VAR value: int = 1\n\
                 VAR count: int = LEN(value)\n\
                 ",
                "Argument for builtin 'LEN' has type int but expected array",
            ),
            (
                "VAR label: string = \"text\"\n\
                 VAR count: int = LEN(label)\n\
                 ",
                "Argument for builtin 'LEN' has type string but expected array",
            ),
            (
                "STRUCT Player {\n\
                 hp: int\n\
                 }\n\
                 VAR player: Player = %Player{ hp: 10 }\n\
                 VAR count: int = LEN(player)\n\
                 ",
                "Argument for builtin 'LEN' has type Player but expected array",
            ),
        ];

        for (source, expected_message) in cases {
            let story = parse_story(source);
            let diagnostics = call_target_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn reports_len_builtin_argument_count_mismatch() {
        let cases = [
            (
                "VAR values: int[] = [1]\n\
                 VAR count: int = LEN()\n\
                 ",
                "Builtin 'LEN' expects 1 argument but got 0",
            ),
            (
                "VAR values: int[] = [1]\n\
                 VAR count: int = LEN(values, values)\n\
                 ",
                "Builtin 'LEN' expects 1 argument but got 2",
            ),
        ];

        for (source, expected_message) in cases {
            let story = parse_story(source);
            let diagnostics = call_target_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn accepts_array_remove_builtin_for_array_types() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR score: int = 1\n\
             VAR source_player: Player = %Player{ hp: 10 }\n\
             VAR scores: int[] = [score]\n\
             VAR players: Player[] = [source_player]\n\
             VAR nested_scores: int[][] = [scores]\n\
             ~ ARRAY_REMOVE(scores, 0)\n\
             ~ ARRAY_REMOVE(players, 0)\n\
             ~ ARRAY_REMOVE(nested_scores, 0)\n\
             ",
        );

        assert_eq!(super::super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_array_push_and_insert_builtins_for_array_types() {
        let story = parse_story(
            "=== interface IRoute ===\n\
             == go ==\n\
             === module game ===\n\
             ENUM State { Idle Busy }\n\
             STRUCT Player {\n\
             hp: int\n\
             }\n\
             FROM left\n\
             VAR score: int = 1\n\
             VAR player: Player = %Player{ hp: 10 }\n\
             VAR scores: int[] = [score]\n\
             VAR players: Player[] = [player]\n\
             VAR lookups: Dict<string, int>[] = [%{\"ada\": 10}]\n\
             VAR states: State[] = [State.Idle]\n\
             VAR routes: interface<IRoute>[] = [left]\n\
             VAR nested_scores: int[][] = [scores]\n\
             == main ==\n\
             ~ ARRAY_PUSH(scores, 2)\n\
             ~ ARRAY_PUSH(players, player)\n\
             ~ ARRAY_PUSH(lookups, lookups[0])\n\
             ~ ARRAY_PUSH(states, State.Busy)\n\
             ~ ARRAY_PUSH(routes, routes[0])\n\
             ~ ARRAY_PUSH(nested_scores, scores)\n\
             ~ ARRAY_INSERT(scores, 0, 0)\n\
             ~ ARRAY_INSERT(scores, LEN(scores), 3)\n\
             Done.\n\
             === module left implements IRoute ===\n\
             == go ==\n\
             ",
        );

        assert_eq!(super::super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_array_remove_builtin_invalid_arguments() {
        let cases = [
            (
                "VAR value: int = 1\n\
                 ~ ARRAY_REMOVE(value, 0)\n\
                 ",
                "First argument for builtin 'ARRAY_REMOVE' has type int but expected array",
            ),
            (
                "VAR values: int[] = [1]\n\
                 VAR index: string = \"0\"\n\
                 ~ ARRAY_REMOVE(values, index)\n\
                 ",
                "Second argument for builtin 'ARRAY_REMOVE' has type string but expected int",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_REMOVE(values)\n\
                 ",
                "Builtin 'ARRAY_REMOVE' expects 2 arguments but got 1",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_REMOVE(values, 0, 1)\n\
                 ",
                "Builtin 'ARRAY_REMOVE' expects 2 arguments but got 3",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_REMOVE(copy_values(), 0)\n\
                 \n\
                 == function copy_values() => int[] ==\n\
                 ~ return values",
                "First argument for builtin 'ARRAY_REMOVE' must be a mutable lvalue",
            ),
        ];

        for (source, expected_message) in cases {
            let story = parse_story(source);
            let diagnostics = call_target_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn reports_array_push_and_insert_builtin_invalid_arguments() {
        let cases = [
            (
                "VAR value: int = 1\n\
                 ~ ARRAY_PUSH(value, 1)\n\
                 ",
                "First argument for builtin 'ARRAY_PUSH' has type int but expected array",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_PUSH(copy_values(), 1)\n\
                 \n\
                 == function copy_values() => int[] ==\n\
                 ~ return values",
                "First argument for builtin 'ARRAY_PUSH' must be a mutable lvalue",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_PUSH(values, \"two\")\n\
                 ",
                "Second argument for builtin 'ARRAY_PUSH' has type string but expected int",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_PUSH(values)\n\
                 ",
                "Builtin 'ARRAY_PUSH' expects 2 arguments but got 1",
            ),
            (
                "VAR values: int[] = [1]\n\
                 VAR index: string = \"0\"\n\
                 ~ ARRAY_INSERT(values, index, 2)\n\
                 ",
                "Second argument for builtin 'ARRAY_INSERT' has type string but expected int",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_INSERT(values, 0, \"two\")\n\
                 ",
                "Third argument for builtin 'ARRAY_INSERT' has type string but expected int",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_INSERT(values, 0)\n\
                 ",
                "Builtin 'ARRAY_INSERT' expects 3 arguments but got 2",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_INSERT(copy_values(), 0, 1)\n\
                 \n\
                 == function copy_values() => int[] ==\n\
                 ~ return values",
                "First argument for builtin 'ARRAY_INSERT' must be a mutable lvalue",
            ),
        ];

        for (source, expected_message) in cases {
            let story = parse_story(source);
            let diagnostics = call_target_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn accepts_dict_collection_builtins_for_dict_types() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR players: Dict<string, Player> = %{\"ada\": %Player{ hp: 10 }}\n\
             VAR names: Dict<int, string> = %{1: \"one\"}\n\
             VAR has_ada: bool = DICT_HAS(players, \"ada\")\n\
             VAR player_count: int = DICT_SIZE(players)\n\
             VAR player_keys: string[] = DICT_KEYS(players)\n\
             VAR name_keys: int[] = DICT_KEYS(names)\n\
             ~ DICT_REMOVE(players, \"ada\")\n\
             ~ DICT_REMOVE(names, 1)\n\
             ",
        );

        assert_eq!(super::super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_dict_collection_builtin_invalid_arguments() {
        let cases = [
            (
                "VAR value: int = 1\n\
                 VAR has_value: bool = DICT_HAS(value, \"ada\")\n\
                 ",
                "First argument for builtin 'DICT_HAS' has type int but expected Dict",
            ),
            (
                "VAR scores: Dict<string, int> = %{\"ada\": 10}\n\
                 VAR has_value: bool = DICT_HAS(scores, 1)\n\
                 ",
                "Second argument for builtin 'DICT_HAS' has type int but expected string",
            ),
            (
                "VAR value: int = 1\n\
                 VAR count: int = DICT_SIZE(value)\n\
                 ",
                "First argument for builtin 'DICT_SIZE' has type int but expected Dict",
            ),
            (
                "VAR scores: Dict<string, int> = %{\"ada\": 10}\n\
                 ~ DICT_REMOVE(scores, 1)\n\
                 ",
                "Second argument for builtin 'DICT_REMOVE' has type int but expected string",
            ),
            (
                "VAR scores: Dict<string, int> = %{\"ada\": 10}\n\
                 ~ DICT_REMOVE(copy_scores(), \"ada\")\n\
                 \n\
                 == function copy_scores() => Dict<string, int> ==\n\
                 ~ return scores",
                "First argument for builtin 'DICT_REMOVE' must be a mutable lvalue",
            ),
            (
                "VAR value: int = 1\n\
                 VAR keys: int[] = DICT_KEYS(value)\n\
                 ",
                "First argument for builtin 'DICT_KEYS' has type int but expected Dict",
            ),
            (
                "VAR scores: Dict<string, int> = %{\"ada\": 10}\n\
                 VAR keys: string[] = DICT_KEYS()\n\
                 ",
                "Builtin 'DICT_KEYS' expects 1 argument but got 0",
            ),
        ];

        for (source, expected_message) in cases {
            let story = parse_story(source);
            let diagnostics = call_target_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn reports_function_call_argument_count_mismatch() {
        let story = parse_story(
            "~ add(1)\n\
             \n\
             == function add(a: int, b: int) => int ==\n\
             ~ return a",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Function 'add' expects 2 arguments but got 1",
        );
    }

    #[test]
    fn reports_function_call_argument_type_mismatch() {
        let story = parse_story(
            "~ add(1, \"two\")\n\
             \n\
             == function add(a: int, b: int) => int ==\n\
             ~ return a",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Argument 'b' for function 'add' has type string but expected int",
        );
    }

    #[test]
    fn reports_composite_function_call_argument_type_mismatch() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR scores: int[] = [1]\n\
             ~ use_player(scores)\n\
             \n\
             == function use_player(player: Player) => void ==\n\
             ~ return",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Argument 'player' for function 'use_player' has type int[] but expected Player",
        );
    }

    #[test]
    fn function_return_type_participates_in_expression_type_checks() {
        let story = parse_story(
            "VAR score: int = label()\n\
             Done.\n\
             == function label() => string ==\n\
             ~ return \"ok\"",
        );

        let diagnostics = super::super::super::run_analysis_passes(&story);

        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.message
                    == "Initializer for variable 'score' has type string but declared type is int"
            }),
            "{diagnostics:#?}"
        );
    }

    #[test]
    fn module_function_lookup_uses_current_module_only() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             ~ helper()\n\
             \n\
             === module items ===\n\
             == function helper() => void ==\n\
             ~ return",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Function 'helper' is not declared",
        );
    }

    #[test]
    fn module_function_lookup_accepts_same_module_functions() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             ~ helper()\n\
             \n\
             == function helper() => void ==\n\
             ~ return\n\
             === module items ===\n\
             == function helper() => void ==\n\
             ~ return",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn qualified_module_function_calls_are_type_checked() {
        let story = parse_story(
            "=== module game ===\n\
             FROM math IMPORT add\n\
             == main ==\n\
             ~ temp total: int = math::add(1, 2)\n\
             \n\
             === module math ===\n\
             == function add(left: int, right: int) => int ==\n\
             ~ return left + right",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn qualified_module_function_calls_use_declaring_module_named_types() {
        let story = parse_story(
            "=== module game ===\n\
             FROM data IMPORT State, echo, DEFAULT_STATE\n\
             == main ==\n\
             ~ temp state: data::State = data::echo(data::DEFAULT_STATE)\n\
             \n\
             === module data ===\n\
             ENUM State { Idle Busy }\n\
             CONST DEFAULT_STATE: State = State.Idle\n\
             == function echo(value: State) => State ==\n\
             ~ return value",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn qualified_module_function_calls_report_argument_type_errors() {
        let story = parse_story(
            "=== module game ===\n\
             FROM math IMPORT add\n\
             == main ==\n\
             ~ temp total: int = math::add(1, \"two\")\n\
             \n\
             === module math ===\n\
             == function add(left: int, right: int) => int ==\n\
             ~ return left + right",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Argument 'right' for function 'math::add' has type string but expected int",
        );
    }

    #[test]
    fn qualified_external_calls_are_type_checked() {
        let story = parse_story(
            "=== module game ===\n\
             FROM audio IMPORT play\n\
             == main ==\n\
             ~ temp code: int = audio::play(\"intro\")\n\
             \n\
             === module audio ===\n\
             EXTERNAL play(name: string) => int\n\
             == helper ==\n\
             ",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn module_diverts_accept_same_module_knot_stitch_paths() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             -> scene.intro\n\
             == scene ==\n\
             = intro\n\
             ",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn module_diverts_accept_relative_stitch_shorthand() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             -> intro\n\
             = intro\n\
             ",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn module_diverts_accept_same_module_self_qualified_stitches() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             -> game::scene.intro\n\
             == scene ==\n\
             = intro\n\
             ",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn module_diverts_reject_cross_module_direct_stitch_paths() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT scene\n\
             == main ==\n\
             -> items::scene.intro\n\
             === module items ===\n\
             == scene ==\n\
             = intro\n\
             ",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cross-module direct stitch access is not allowed: 'items::scene.intro'. Import and reference the parent knot instead.",
        );
    }

    #[test]
    fn module_diverts_accept_cross_module_parent_knots_for_target_resolution() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT scene\n\
             == main ==\n\
             -> items::scene\n\
             === module items ===\n\
             == scene ==\n\
             = intro\n\
             ",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn module_tunnel_onwards_reject_cross_module_direct_stitch_paths() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT scene\n\
             == main ==\n\
             -> tunnel ->-> items::scene.intro\n\
             == tunnel ==\n\
             ->->\n\
             === module items ===\n\
             == scene ==\n\
             = intro\n\
             ",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cross-module direct stitch access is not allowed: 'items::scene.intro'. Import and reference the parent knot instead.",
        );
    }

    #[test]
    fn accepts_dynamic_interface_divert_targets() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target(amount: int) ==\n\
             === module game ===\n\
             FROM left\n\
             STRUCT RouteState {\n\
             current: interface<IItem>\n\
             routes: interface<IItem>[]\n\
             }\n\
             VAR route: interface<IItem> = left\n\
             VAR state: RouteState = %RouteState{ current: left, routes: [left] }\n\
             VAR routes: interface<IItem>[] = [left]\n\
             == main ==\n\
             -> {{route}::target}(1)\n\
             -> {{state.current}::target}(1)\n\
             -> {{routes[0]}::target}(1)\n\
             === module left implements IItem ===\n\
             == target(amount: int) ==\n\
             ",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn accepts_interface_module_literals_as_dynamic_interface_target_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === interface IRouter ===\n\
             == target(next: interface<IItem>) ==\n\
             === module game ===\n\
             FROM left\n\
             FROM router\n\
             VAR route: interface<IRouter> = router\n\
             == main ==\n\
             -> {{route}::target}(left)\n\
             === module router implements IRouter ===\n\
             == target(next: interface<IItem>) ==\n\
             \n\
             === module left implements IItem ===\n\
             == target ==\n\
             ",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn accepts_interface_module_literals_as_static_divert_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             == main ==\n\
             -> register(left)\n\
             == register(next: interface<IItem>) ==\n\
             \n\
             === module left implements IItem ===\n\
             == target ==\n\
             ",
        );

        assert_eq!(super::super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_interface_module_literals_as_function_call_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             == main ==\n\
             ~ temp route: interface<IItem> = select(left)\n\
             \n\
             == function select(next: interface<IItem>) => interface<IItem> ==\n\
             ~ return next\n\
             === module left implements IItem ===\n\
             == target ==\n\
             ",
        );

        assert_eq!(super::super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_interface_module_literals_as_qualified_static_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             FROM registry IMPORT register, select\n\
             == main ==\n\
             -> registry::register(left)\n\
             ~ temp route: interface<IItem> = registry::select(left)\n\
             \n\
             === module registry ===\n\
             == register(next: interface<IItem>) ==\n\
             \n\
             == function select(next: interface<IItem>) => interface<IItem> ==\n\
             ~ return next\n\
             === module left implements IItem ===\n\
             == target ==\n\
             ",
        );

        assert_eq!(super::super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn visible_variables_take_precedence_over_interface_module_literals_in_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             == main ==\n\
             ~ temp left: int = 1\n\
             ~ temp route: interface<IItem> = select(left)\n\
             \n\
             == function select(next: interface<IItem>) => interface<IItem> ==\n\
             ~ return next\n\
             === module left implements IItem ===\n\
             == target ==\n\
             ",
        );

        assert_single_diagnostic(
            &call_target_diagnostics(&story),
            DiagnosticSeverity::Error,
            "Argument 'next' for function 'select' has type int but expected interface<IItem>",
        );
    }

    #[test]
    fn reports_missing_import_for_interface_module_literals_in_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             == main ==\n\
             ~ temp route: interface<IItem> = select(left)\n\
             \n\
             == function select(next: interface<IItem>) => interface<IItem> ==\n\
             ~ return next\n\
             === module left implements IItem ===\n\
             == target ==\n\
             ",
        );

        assert_single_diagnostic(
            &call_target_diagnostics(&story),
            DiagnosticSeverity::Error,
            "Cannot type-check argument 'next' for function 'select': Module literal 'left' requires a bare import in module 'game': FROM left",
        );
    }

    #[test]
    fn reports_wrong_interface_for_interface_module_literals_in_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === interface IOther ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             == main ==\n\
             ~ temp route: interface<IItem> = select(left)\n\
             \n\
             == function select(next: interface<IItem>) => interface<IItem> ==\n\
             ~ return next\n\
             === module left implements IOther ===\n\
             == target ==\n\
             ",
        );

        assert_single_diagnostic(
            &call_target_diagnostics(&story),
            DiagnosticSeverity::Error,
            "Cannot type-check argument 'next' for function 'select': Module 'left' does not implement interface 'IItem'",
        );
    }

    #[test]
    fn accepts_dynamic_interface_function_calls() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === interface IScorer ===\n\
             == function score(item: interface<IItem>, amount: int) => int ==\n\
             === module game ===\n\
             FROM left\n\
             FROM scorer\n\
             VAR route: interface<IScorer> = scorer\n\
             == main ==\n\
             ~ temp value: int = {route}::score(left, 3)\n\
             \n\
             === module scorer implements IScorer ===\n\
             == function score(item: interface<IItem>, amount: int) => int ==\n\
             ~ return amount\n\
             === module left implements IItem ===\n\
             == target ==\n\
             ",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn reports_invalid_dynamic_interface_function_calls() {
        let cases = [
            (
                "~ temp value: int = {label}::score(1)",
                "Dynamic interface function 'score' has base type string but expected interface",
            ),
            (
                "~ temp value: int = {route}::missing()",
                "Interface 'IItem' does not declare member 'missing'",
            ),
            (
                "~ temp value: int = {route}::target(1)",
                "Interface 'IItem' member 'target' is a knot but dynamic function call requires a function",
            ),
            (
                "~ temp value: int = {route}::score()",
                "Dynamic interface function 'score' expects 1 arguments but got 0",
            ),
            (
                "~ temp value: int = {route}::score(\"bad\")",
                "Argument 'amount' for dynamic interface function 'score' has type string but expected int",
            ),
        ];

        for (logic, expected_message) in cases {
            let story = parse_story(&format!(
                "=== interface IItem ===\n\
                 == target(amount: int) ==\n\
                 == function score(amount: int) => int ==\n\
                 === module game ===\n\
                 FROM left\n\
                 VAR route: interface<IItem> = left\n\
                 VAR label: string = \"x\"\n\
                 == main ==\n\
                 {logic}\n\
                 \n\
                 === module left implements IItem ===\n\
                 == target(amount: int) ==\n\
                 \n\
                 == function score(amount: int) => int ==\n\
                 ~ return amount",
            ));
            let diagnostics = call_target_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn reports_invalid_dynamic_interface_divert_targets() {
        let cases = [
            (
                "-> {{label}::target}",
                "Dynamic interface target 'target' has base type string but expected interface",
            ),
            (
                "-> {{route}::missing}",
                "Interface 'IItem' does not declare member 'missing'",
            ),
            (
                "-> {{route}::score}",
                "Interface 'IItem' member 'score' is a function but dynamic target access requires a knot",
            ),
            (
                "-> {{route}::target}",
                "Dynamic interface target 'target' expects 1 arguments but got 0",
            ),
            (
                "-> {{route}::target}(\"bad\")",
                "Argument 'amount' for dynamic interface target 'target' has type string but expected int",
            ),
        ];

        for (divert, expected_message) in cases {
            let story = parse_story(&format!(
                "=== interface IItem ===\n\
                 == target(amount: int) ==\n\
                 == function score() => int ==\n\
                 === module game ===\n\
                 FROM left\n\
                 VAR route: interface<IItem> = left\n\
                 VAR label: string = \"x\"\n\
                 == main ==\n\
                 {divert}\n\
                 === module left implements IItem ===\n\
                 == target(amount: int) ==\n\
                 \n\
                 == function score() => int ==\n\
                 ~ return 1",
            ));
            let diagnostics = call_target_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn keeps_plain_dynamic_divert_targets_unchanged() {
        let story = parse_story(
            "VAR next: -> = -> done\n\
             -> {next}\n\
             == done ==\n\
             ",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn checks_plain_dynamic_divert_arguments() {
        let story = parse_story(
            "VAR next: -> = -> done\n\
             -> {next}(missing())\n\
             == done ==\n\
             ",
        );
        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Function 'missing' is not declared",
        );
    }
}
