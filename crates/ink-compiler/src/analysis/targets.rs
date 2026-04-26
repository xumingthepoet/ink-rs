use std::collections::HashMap;

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        DivertTarget, Expression, Flow, FlowArgument, Object, Story, TypeName,
    },
    source::SourceSpan,
};

use super::{
    context::{
        FlowContext, FlowSymbol, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex,
        VariableTargetIndex,
    },
    expression_types::{infer_expression_type, typed_builtin_return_type},
    span::object_span,
    structs::build_struct_type_index,
    target_symbols::{build_target_symbol_index, resolve_target_symbol},
    variable_targets::build_variable_target_index,
    variables::build_variable_scope_index,
};

pub(super) fn call_target_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let target_symbols = build_target_symbol_index(story);
    let variable_targets = build_variable_target_index(story);
    let variable_scopes = build_variable_scope_index(story);
    let struct_types = build_struct_type_index(story);
    let mut checker = CallTargetChecker::new(
        &target_symbols,
        &variable_targets,
        &variable_scopes,
        &struct_types,
    );
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct CallTargetChecker<'a> {
    target_symbols: &'a TargetSymbolIndex,
    variable_targets: &'a VariableTargetIndex,
    variable_scopes: &'a VariableScopeIndex,
    struct_types: &'a StructTypeIndex,
    diagnostics: Vec<Diagnostic>,
    flow_contexts_by_path: HashMap<String, FlowContext>,
}

impl<'a> CallTargetChecker<'a> {
    fn new(
        target_symbols: &'a TargetSymbolIndex,
        variable_targets: &'a VariableTargetIndex,
        variable_scopes: &'a VariableScopeIndex,
        struct_types: &'a StructTypeIndex,
    ) -> Self {
        Self {
            target_symbols,
            variable_targets,
            variable_scopes,
            struct_types,
            diagnostics: Vec::new(),
            flow_contexts_by_path: HashMap::new(),
        }
    }

    fn current_flow_path<'context>(
        &self,
        context: &'context VisitContext,
    ) -> Option<&'context str> {
        context.current_flow_path.as_deref()
    }

    fn current_flow_context(&self, context: &VisitContext) -> Option<&FlowContext> {
        self.current_flow_path(context)
            .and_then(|flow_path| self.flow_contexts_by_path.get(flow_path))
    }

    fn current_flow_arguments(&self, context: &VisitContext) -> Option<&[FlowArgument]> {
        self.current_flow_context(context)
            .map(FlowContext::arguments)
    }

    fn current_flow_is_function(&self, context: &VisitContext) -> bool {
        self.current_flow_context(context)
            .is_some_and(FlowContext::is_function)
    }

    fn check_plain_divert_target(
        &mut self,
        target: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let current_flow_path = self.current_flow_path(context);
        if let Some(symbol) = resolve_target_symbol(target, current_flow_path, self.target_symbols)
        {
            if symbol.is_function() {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "{target} can't be diverted to. It can only be called as a function since it's been marked as such: '{target}(...)'"
                    ),
                ));
            }
        } else if let Some((name, is_divert_target)) =
            resolve_current_flow_argument(target, self.current_flow_arguments(context))
                .map(|argument| (argument.name().to_string(), argument.is_divert_target()))
        {
            if !is_divert_target {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Since '{name}' is used as a variable divert target, it should be marked as: -> {name}"
                    ),
                ));
            }
        } else if !self.variable_targets.contains(target) {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("target not found: '{target}'"),
            ));
        }
    }

    fn check_expression(
        &mut self,
        expression: &Expression,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        match expression {
            Expression::FunctionCall { name, args } => {
                self.check_function_call(name, args, span, context);
            }
            Expression::ArrayLiteral(elements) => {
                for element in elements {
                    self.check_expression(element, span, context);
                }
            }
            Expression::StructLiteral(fields) => {
                for field in fields {
                    self.check_expression(field.expression(), span, context);
                }
            }
            Expression::FieldAccess { base, .. } => {
                self.check_expression(base, span, context);
            }
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
                self.check_variable_reference(name, span, context);
            }
            Expression::StringContent(_)
            | Expression::String(_)
            | Expression::NumberInt(_)
            | Expression::NumberFloat(_)
            | Expression::NumberBool(_) => {}
        }
    }

    fn check_function_call(
        &mut self,
        name: &str,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if typed_builtin_return_type(name).is_some() {
            self.check_typed_builtin_call(name, args, span, context);
            for arg in args {
                self.check_expression(arg, span, context);
            }
            return;
        }

        let symbol =
            resolve_target_symbol(name, self.current_flow_path(context), self.target_symbols)
                .cloned();

        if let Some(symbol) = symbol {
            if !symbol.is_function() {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "{name} hasn't been marked as a function, but it's being called as one. Do you need to declare the knot as '== function {name} =='?"
                    ),
                ));
            } else {
                self.check_function_call_signature(name, args, &symbol, span, context);
            }
        }

        for arg in args {
            self.check_expression(arg, span, context);
        }
    }

    fn check_typed_builtin_call(
        &mut self,
        name: &str,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        match name {
            "ARRAY_REMOVE" => self.check_array_remove_call(args, span, context),
            "LEN" => self.check_len_call(args, span, context),
            "READ_COUNT" | "TURNS_SINCE" => self.check_count_builtin_call(name, args, span),
            _ => {}
        }
    }

    fn check_count_builtin_call(&mut self, name: &str, args: &[Expression], span: &SourceSpan) {
        if args.len() != 1 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("Builtin '{name}' expects 1 argument but got {}", args.len()),
            ));
        }
    }

    fn check_array_remove_call(
        &mut self,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if args.len() != 2 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Builtin 'ARRAY_REMOVE' expects 2 arguments but got {}",
                    args.len()
                ),
            ));
            return;
        }

        if !is_mutable_lvalue(&args[0]) {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                "First argument for builtin 'ARRAY_REMOVE' must be a mutable lvalue",
            ));
        }

        match infer_expression_type(
            &args[0],
            self.variable_scopes,
            self.struct_types,
            self.target_symbols,
            self.current_flow_path(context),
        ) {
            Ok(argument_type) if argument_type.array_element_type().is_none() => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "First argument for builtin 'ARRAY_REMOVE' has type {} but expected array",
                        argument_type.display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check first argument for builtin 'ARRAY_REMOVE': {}",
                    error.message()
                ),
            )),
        }

        match infer_expression_type(
            &args[1],
            self.variable_scopes,
            self.struct_types,
            self.target_symbols,
            self.current_flow_path(context),
        ) {
            Ok(argument_type) if argument_type != TypeName::int() => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Second argument for builtin 'ARRAY_REMOVE' has type {} but expected int",
                        argument_type.display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check second argument for builtin 'ARRAY_REMOVE': {}",
                    error.message()
                ),
            )),
        }
    }

    fn check_len_call(&mut self, args: &[Expression], span: &SourceSpan, context: &VisitContext) {
        if args.len() != 1 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("Builtin 'LEN' expects 1 argument but got {}", args.len()),
            ));
            return;
        }

        match infer_expression_type(
            &args[0],
            self.variable_scopes,
            self.struct_types,
            self.target_symbols,
            self.current_flow_path(context),
        ) {
            Ok(argument_type) if argument_type.array_element_type().is_none() => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Argument for builtin 'LEN' has type {} but expected array",
                        argument_type.display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check argument for builtin 'LEN': {}",
                    error.message()
                ),
            )),
        }
    }

    fn check_function_call_signature(
        &mut self,
        name: &str,
        args: &[Expression],
        symbol: &FlowSymbol,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let parameters = symbol.arguments();
        if args.len() != parameters.len() {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Function '{name}' expects {} arguments but got {}",
                    parameters.len(),
                    args.len()
                ),
            ));
            return;
        }

        for (argument, parameter) in args.iter().zip(parameters) {
            let Some(expected_type) = parameter.declared_type() else {
                continue;
            };
            match infer_expression_type(
                argument,
                self.variable_scopes,
                self.struct_types,
                self.target_symbols,
                self.current_flow_path(context),
            ) {
                Ok(actual_type) if &actual_type != expected_type => {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Argument '{}' for function '{name}' has type {} but expected {}",
                            parameter.name(),
                            actual_type.display_name(),
                            expected_type.display_name()
                        ),
                    ));
                }
                Ok(_) => {}
                Err(error) => self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check argument '{}' for function '{name}': {}",
                        parameter.name(),
                        error.message()
                    ),
                )),
            }
        }
    }

    fn check_variable_reference(&mut self, name: &str, span: &SourceSpan, context: &VisitContext) {
        let current_flow_path = self.current_flow_path(context);
        if name.contains('.')
            || resolve_target_symbol(name, current_flow_path, self.target_symbols).is_some()
        {
            return;
        }

        if self
            .variable_scopes
            .contains_visible_variable(name, current_flow_path)
        {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            span.clone(),
            format!("Unresolved variable: {name}"),
        ));
    }

    fn check_divert_target_value(
        &mut self,
        target: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let variable_name = target.split('.').next().unwrap_or(target);
        if self
            .variable_scopes
            .contains_visible_variable(variable_name, self.current_flow_path(context))
        {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Since '{variable_name}' is a variable, it shouldn't be preceded by '->' here."
                ),
            ));
        }
    }
}

fn is_mutable_lvalue(expression: &Expression) -> bool {
    match expression {
        Expression::VariableReference(_) => true,
        Expression::FieldAccess { base, .. } | Expression::IndexAccess { base, .. } => {
            is_mutable_lvalue(base)
        }
        _ => false,
    }
}

impl ParsedVisitor for CallTargetChecker<'_> {
    fn visit_flow(&mut self, flow: &Flow, context: &VisitContext) {
        let Some(flow_path) = self.current_flow_path(context) else {
            return;
        };
        self.flow_contexts_by_path
            .entry(flow_path.to_string())
            .or_insert_with(|| FlowContext::new(flow.arguments().to_vec(), flow.is_function()));
    }

    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        match object {
            Object::Divert(divert) => {
                match divert.target() {
                    DivertTarget::Empty => self.diagnostics.push(Diagnostic::error(
                        divert.span().clone(),
                        "Empty diverts (->) are only valid on choices",
                    )),
                    DivertTarget::Path(target) if !self.current_flow_is_function(context) => {
                        self.check_plain_divert_target(target, divert.span(), context);
                    }
                    DivertTarget::Path(_) | DivertTarget::Done | DivertTarget::End => {}
                }
                for argument in divert.arguments() {
                    self.check_expression(argument, divert.span(), context);
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
            Object::Choice(choice) => {
                if let Some(condition) = choice.condition() {
                    self.check_expression(condition, choice.span(), context);
                }
            }
            Object::TunnelOnwards(tunnel_onwards) => {
                for argument in tunnel_onwards.arguments() {
                    self.check_expression(argument, tunnel_onwards.span(), context);
                }
            }
            Object::AuthorWarning(_)
            | Object::ContentList(_)
            | Object::ExternalDeclaration(_)
            | Object::Gather(_)
            | Object::Glue(_)
            | Object::Sequence(_)
            | Object::StructDeclaration(_)
            | Object::Tag(_)
            | Object::Text(_)
            | Object::Weave(_) => {}
        }
    }
}

fn resolve_current_flow_argument<'a>(
    target: &str,
    current_flow_arguments: Option<&'a [FlowArgument]>,
) -> Option<&'a FlowArgument> {
    let variable_target_name = target.split('.').next()?;
    current_flow_arguments?
        .iter()
        .find(|argument| argument.name() == variable_target_name)
}

#[cfg(test)]
mod tests {
    use crate::diagnostic::DiagnosticSeverity;

    use super::{
        super::test_support::{assert_single_diagnostic, parse_story},
        *,
    };

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
        let story = parse_story("~ knot()\n== knot ==\n-> DONE");
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
             VAR source_player: Player = { hp: 10 }\n\
             VAR source_scores: int[] = [1]\n\
             VAR result: int = add(1, 2)\n\
             VAR copied_player: Player = echo_player(source_player)\n\
             VAR copied_scores: int[] = echo_scores(source_scores)\n\
             -> DONE\n\
             == function add(a: int, b: int) -> int ==\n\
             ~ return a + b\n\
             == function echo_player(player: Player) -> Player ==\n\
             ~ return player\n\
             == function echo_scores(values: int[]) -> int[] ==\n\
             ~ return values",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_typed_external_calls_and_return_type_use() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             EXTERNAL external_score(value: int) -> int\n\
             EXTERNAL describe(player: Player, scores: int[]) -> string\n\
             VAR score: int = 1\n\
             VAR source_player: Player = { hp: 10 }\n\
             VAR scores: int[] = [score]\n\
             VAR adjusted_score: int = external_score(score) + LEN(scores)\n\
             VAR description: string = describe(source_player, scores)\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_typed_external_call_argument_count_mismatch() {
        let story = parse_story(
            "EXTERNAL external_score(value: int) -> int\n\
             ~ external_score()\n\
             -> DONE",
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
            "EXTERNAL external_score(value: int) -> int\n\
             VAR label: string = \"x\"\n\
             ~ external_score(label)\n\
             -> DONE",
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
            "EXTERNAL external_label() -> string\n\
             VAR score: int = external_label()\n\
             -> DONE",
        );

        let diagnostics = super::super::run_analysis_passes(&story);

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
             VAR source_player: Player = { hp: 10 }\n\
             VAR scores: int[] = [score]\n\
             VAR players: Player[] = [source_player]\n\
             VAR nested_scores: int[][] = [scores]\n\
             VAR score_count: int = LEN(scores)\n\
             VAR player_count: int = LEN(players)\n\
             VAR row_count: int = LEN(nested_scores)\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_count_builtins_as_int_expressions() {
        let story = parse_story(
            "VAR visits: int = READ_COUNT(-> knot)\n\
             VAR turns: int = TURNS_SINCE(-> knot)\n\
             -> DONE\n\
             == knot ==\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_count_builtin_argument_count_mismatch() {
        let story = parse_story(
            "VAR visits: int = READ_COUNT()\n\
             -> DONE",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Builtin 'READ_COUNT' expects 1 argument but got 0",
        );
    }

    #[test]
    fn reports_len_builtin_non_array_arguments() {
        let cases = [
            (
                "VAR value: int = 1\n\
                 VAR count: int = LEN(value)\n\
                 -> DONE",
                "Argument for builtin 'LEN' has type int but expected array",
            ),
            (
                "VAR label: string = \"text\"\n\
                 VAR count: int = LEN(label)\n\
                 -> DONE",
                "Argument for builtin 'LEN' has type string but expected array",
            ),
            (
                "STRUCT Player {\n\
                 hp: int\n\
                 }\n\
                 VAR player: Player = { hp: 10 }\n\
                 VAR count: int = LEN(player)\n\
                 -> DONE",
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
                 -> DONE",
                "Builtin 'LEN' expects 1 argument but got 0",
            ),
            (
                "VAR values: int[] = [1]\n\
                 VAR count: int = LEN(values, values)\n\
                 -> DONE",
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
             VAR source_player: Player = { hp: 10 }\n\
             VAR scores: int[] = [score]\n\
             VAR players: Player[] = [source_player]\n\
             VAR nested_scores: int[][] = [scores]\n\
             ~ ARRAY_REMOVE(scores, 0)\n\
             ~ ARRAY_REMOVE(players, 0)\n\
             ~ ARRAY_REMOVE(nested_scores, 0)\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_array_remove_builtin_invalid_arguments() {
        let cases = [
            (
                "VAR value: int = 1\n\
                 ~ ARRAY_REMOVE(value, 0)\n\
                 -> DONE",
                "First argument for builtin 'ARRAY_REMOVE' has type int but expected array",
            ),
            (
                "VAR values: int[] = [1]\n\
                 VAR index: string = \"0\"\n\
                 ~ ARRAY_REMOVE(values, index)\n\
                 -> DONE",
                "Second argument for builtin 'ARRAY_REMOVE' has type string but expected int",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_REMOVE(values)\n\
                 -> DONE",
                "Builtin 'ARRAY_REMOVE' expects 2 arguments but got 1",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_REMOVE(values, 0, 1)\n\
                 -> DONE",
                "Builtin 'ARRAY_REMOVE' expects 2 arguments but got 3",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_REMOVE(copy_values(), 0)\n\
                 -> DONE\n\
                 == function copy_values() -> int[] ==\n\
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
    fn reports_function_call_argument_count_mismatch() {
        let story = parse_story(
            "~ add(1)\n\
             -> DONE\n\
             == function add(a: int, b: int) -> int ==\n\
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
             -> DONE\n\
             == function add(a: int, b: int) -> int ==\n\
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
             -> DONE\n\
             == function use_player(player: Player) -> void ==\n\
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
             -> DONE\n\
             == function label() -> string ==\n\
             ~ return \"ok\"",
        );

        let diagnostics = super::super::run_analysis_passes(&story);

        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.message
                    == "Initializer for variable 'score' has type string but declared type is int"
            }),
            "{diagnostics:#?}"
        );
    }
}
