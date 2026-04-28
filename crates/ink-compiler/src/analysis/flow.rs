use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, walk_weave, ParsedVisitor, VisitContext},
        Choice, ContentList, DivertTarget, Expression, Flow, FlowLevel, Object, Return, Story,
        TypeName, Weave,
    },
    source::SourceSpan,
};

use super::{
    context::{StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    expression_types::{infer_expression_type, typed_builtin_return_type},
    span::{first_span_in_weave, object_span},
    structs::build_struct_type_index,
    target_symbols::{build_target_symbol_index, resolve_target_symbol},
    variables::build_variable_scope_index,
};

pub(super) fn flow_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let variable_scopes = build_variable_scope_index(story);
    let struct_types = build_struct_type_index(story);
    let target_symbols = build_target_symbol_index(story);
    let analysis = FlowAnalysisIndexes {
        variable_scopes: &variable_scopes,
        struct_types: &struct_types,
        target_symbols: &target_symbols,
    };
    let mut diagnostics = Vec::new();
    check_global_var_declaration_scope(story, &mut diagnostics);
    check_nested_choice_termination_in_weave(story.root_weave(), false, &mut diagnostics);
    for module in story.modules() {
        check_nested_choice_termination_in_weave(module.weave(), false, &mut diagnostics);
    }
    {
        let mut condition_checker = ConditionTypeChecker {
            diagnostics: &mut diagnostics,
            analysis: &analysis,
        };
        walk_story(story, &mut condition_checker);
    }
    for flow in story.flows() {
        check_flow(None, flow, flow.name(), &analysis, &mut diagnostics);
    }
    for module in story.modules() {
        for flow in module.flows() {
            check_flow(
                Some(module.name()),
                flow,
                flow.name(),
                &analysis,
                &mut diagnostics,
            );
        }
    }
    diagnostics
}

fn check_global_var_declaration_scope(story: &Story, diagnostics: &mut Vec<Diagnostic>) {
    for object in story.root_weave().content() {
        check_global_var_declaration_scope_in_object(object, true, diagnostics);
    }
    for flow in story.flows() {
        check_global_var_declaration_scope_in_flow(flow, diagnostics);
    }
    for module in story.modules() {
        for object in module.weave().content() {
            check_global_var_declaration_scope_in_object(object, true, diagnostics);
        }
        for flow in module.flows() {
            check_global_var_declaration_scope_in_flow(flow, diagnostics);
        }
    }
}

fn check_global_var_declaration_scope_in_flow(flow: &Flow, diagnostics: &mut Vec<Diagnostic>) {
    for object in flow.weave().content() {
        check_global_var_declaration_scope_in_object(object, false, diagnostics);
    }
    for child in flow.child_flows() {
        check_global_var_declaration_scope_in_flow(child, diagnostics);
    }
}

fn check_global_var_declaration_scope_in_object(
    object: &Object,
    is_story_top_level: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match object {
        Object::VariableAssignment(assignment) if assignment.is_global() && !is_story_top_level => {
            diagnostics.push(nested_global_var_declaration_diagnostic(
                assignment.span().clone(),
            ));
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                check_global_var_declaration_scope_in_content_list(content, diagnostics);
            }
            check_global_var_declaration_scope_in_content_list(choice.inner_content(), diagnostics);
        }
        Object::ContentList(content) => {
            check_global_var_declaration_scope_in_content_list(content, diagnostics);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                for object in branch.content().content() {
                    check_global_var_declaration_scope_in_object(object, false, diagnostics);
                }
            }
        }
        Object::Weave(weave) => {
            for object in weave.content() {
                check_global_var_declaration_scope_in_object(object, false, diagnostics);
            }
        }
        Object::AuthorWarning(_)
        | Object::ConstantDeclaration(_)
        | Object::Divert(_)
        | Object::Expression(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::IncDec(_)
        | Object::LogicLine(_)
        | Object::Return(_)
        | Object::StructDeclaration(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::TunnelOnwards(_)
        | Object::VariableAssignment(_) => {}
    }
}

fn check_global_var_declaration_scope_in_content_list(
    content: &ContentList,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for object in content.objects() {
        check_global_var_declaration_scope_in_object(object, false, diagnostics);
    }
}

fn nested_global_var_declaration_diagnostic(span: SourceSpan) -> Diagnostic {
    Diagnostic::error(
        span,
        "Global VAR declarations must appear at the story top level, outside knots, stitches, functions, choices, and conditionals.",
    )
}

struct FlowAnalysisIndexes<'a> {
    variable_scopes: &'a VariableScopeIndex,
    struct_types: &'a StructTypeIndex,
    target_symbols: &'a TargetSymbolIndex,
}

struct ConditionTypeChecker<'a> {
    diagnostics: &'a mut Vec<Diagnostic>,
    analysis: &'a FlowAnalysisIndexes<'a>,
}

impl ParsedVisitor for ConditionTypeChecker<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        match object {
            Object::Choice(choice) => {
                if let Some(condition) = choice.condition() {
                    self.check_condition("Choice", condition, choice.span(), context);
                }
            }
            Object::Conditional(conditional) => {
                let span = object_span(object);
                if let Some(condition) = conditional.initial_condition() {
                    self.check_condition("Conditional", condition, &span, context);
                }
                for branch in conditional.branches() {
                    if let Some(condition) = branch.own_condition() {
                        self.check_condition("Conditional", condition, &span, context);
                    }
                }
            }
            _ => {}
        }
    }
}

impl ConditionTypeChecker<'_> {
    fn check_condition(
        &mut self,
        label: &str,
        condition: &Expression,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let current_flow_path = context.current_flow_path.as_deref();
        let has_typed_signal = condition_type_signal(
            condition,
            self.analysis.variable_scopes,
            self.analysis.target_symbols,
            context.current_module.as_deref(),
            current_flow_path,
        )
        .is_some_and(|signal| signal == ConditionTypeSignal::Typed);

        match infer_expression_type(
            condition,
            self.analysis.variable_scopes,
            self.analysis.struct_types,
            self.analysis.target_symbols,
            context.current_module.as_deref(),
            current_flow_path,
        ) {
            Ok(condition_type) if condition_type == TypeName::bool() => {}
            Ok(condition_type) if has_typed_signal => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "{label} condition has type {} but expected bool",
                    condition_type.display_name()
                ),
            )),
            Ok(_) => {}
            Err(error) if has_typed_signal => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("Cannot type-check {label} condition: {}", error.message()),
            )),
            Err(_) => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConditionTypeSignal {
    LiteralOnly,
    Typed,
}

fn condition_type_signal(
    expression: &Expression,
    variable_scopes: &VariableScopeIndex,
    target_symbols: &TargetSymbolIndex,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
) -> Option<ConditionTypeSignal> {
    match expression {
        Expression::String(_)
        | Expression::StringContent(_)
        | Expression::NumberInt(_)
        | Expression::NumberFloat(_)
        | Expression::NumberBool(_)
        | Expression::ArrayLiteral(_)
        | Expression::StructLiteral(_) => Some(ConditionTypeSignal::LiteralOnly),
        Expression::VariableReference(name) => matches!(
            variable_scopes.visible_variable_declared_type(name, current_module, current_flow_path),
            Some(Some(_))
        )
        .then_some(ConditionTypeSignal::Typed),
        Expression::QualifiedReference(name) => matches!(
            variable_scopes
                .qualified_constant_declared_type(name.as_str())
                .or_else(|| variable_scopes.qualified_global_variable_declared_type(name.as_str())),
            Some(Some(_))
        )
        .then_some(ConditionTypeSignal::Typed),
        Expression::FunctionCall { name, .. } if typed_builtin_return_type(name).is_some() => {
            Some(ConditionTypeSignal::Typed)
        }
        Expression::FunctionCall { name, .. } => {
            resolve_target_symbol(name, current_module, current_flow_path, target_symbols)
                .is_some_and(|symbol| symbol.is_function() && symbol.has_typed_signature())
                .then_some(ConditionTypeSignal::Typed)
        }
        Expression::QualifiedFunctionCall { name, .. } => resolve_target_symbol(
            name.as_str(),
            current_module,
            current_flow_path,
            target_symbols,
        )
        .is_some_and(|symbol| symbol.is_function() && symbol.has_typed_signature())
        .then_some(ConditionTypeSignal::Typed),
        Expression::FieldAccess { base, .. } | Expression::IndexAccess { base, .. } => {
            condition_type_signal(
                base,
                variable_scopes,
                target_symbols,
                current_module,
                current_flow_path,
            )
            .filter(|signal| *signal == ConditionTypeSignal::Typed)
        }
        Expression::Unary { expression, .. } => condition_type_signal(
            expression,
            variable_scopes,
            target_symbols,
            current_module,
            current_flow_path,
        ),
        Expression::Binary { left, right, .. } => combine_condition_signals(
            condition_type_signal(
                left,
                variable_scopes,
                target_symbols,
                current_module,
                current_flow_path,
            ),
            condition_type_signal(
                right,
                variable_scopes,
                target_symbols,
                current_module,
                current_flow_path,
            ),
        ),
        Expression::MultipleCondition(expressions) => expressions
            .iter()
            .map(|expression| {
                condition_type_signal(
                    expression,
                    variable_scopes,
                    target_symbols,
                    current_module,
                    current_flow_path,
                )
            })
            .reduce(combine_condition_signals)
            .flatten(),
        Expression::DivertTarget(_) => None,
    }
}

fn combine_condition_signals(
    left: Option<ConditionTypeSignal>,
    right: Option<ConditionTypeSignal>,
) -> Option<ConditionTypeSignal> {
    match (left, right) {
        (Some(ConditionTypeSignal::Typed), _) | (_, Some(ConditionTypeSignal::Typed)) => {
            Some(ConditionTypeSignal::Typed)
        }
        (Some(ConditionTypeSignal::LiteralOnly), Some(ConditionTypeSignal::LiteralOnly)) => {
            Some(ConditionTypeSignal::LiteralOnly)
        }
        _ => None,
    }
}

fn check_flow(
    current_module: Option<&str>,
    flow: &Flow,
    current_flow_path: &str,
    analysis: &FlowAnalysisIndexes<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    check_nested_choice_termination_in_weave(flow.weave(), false, diagnostics);
    let found_return = find_return_in_flow(flow);

    if flow.is_function() {
        check_function_flow_control(
            current_module,
            flow,
            current_flow_path,
            analysis,
            diagnostics,
        );
    } else if let Some(found_return) = found_return {
        diagnostics.push(Diagnostic::error(
            found_return.span().clone(),
            format!(
                "Return statements can only be used in knots that are declared as functions: == function {} ==",
                flow.name()
            ),
        ));
    } else if let Some(span) = loose_end_warning_span(flow.weave()) {
        diagnostics.push(Diagnostic::warning(
            span,
            "Apparent loose end exists where the flow runs out. Do you need a '-> DONE' statement, choice or divert?",
        ));
    }

    for child in flow.child_flows() {
        let child_flow_path = format!("{current_flow_path}.{}", child.name());
        check_flow(
            current_module,
            child,
            &child_flow_path,
            analysis,
            diagnostics,
        );
    }
}

fn check_nested_choice_termination_in_weave(
    weave: &Weave,
    inside_sealed_content: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let objects = weave.content();
    for (index, object) in objects.iter().enumerate() {
        match object {
            Object::Choice(choice) => {
                if inside_sealed_content && !choice_flow_terminates(choice, &objects[index + 1..]) {
                    diagnostics.push(Diagnostic::error(
                        choice.span().clone(),
                        "Choices nested in conditionals need to explicitly divert afterwards.",
                    ));
                }
                if let Some(content) = choice.start_content() {
                    check_nested_choice_termination_in_content_list(
                        content,
                        inside_sealed_content,
                        diagnostics,
                    );
                }
                check_nested_choice_termination_in_content_list(
                    choice.inner_content(),
                    inside_sealed_content,
                    diagnostics,
                );
            }
            Object::ContentList(content) => check_nested_choice_termination_in_content_list(
                content,
                inside_sealed_content,
                diagnostics,
            ),
            Object::Conditional(conditional) => {
                for branch in conditional.branches() {
                    check_nested_choice_termination_in_weave(branch.content(), true, diagnostics);
                }
            }
            Object::Weave(weave) => {
                check_nested_choice_termination_in_weave(weave, inside_sealed_content, diagnostics)
            }
            Object::AuthorWarning(_)
            | Object::ConstantDeclaration(_)
            | Object::Divert(_)
            | Object::Expression(_)
            | Object::ExternalDeclaration(_)
            | Object::Gather(_)
            | Object::Glue(_)
            | Object::IncDec(_)
            | Object::LogicLine(_)
            | Object::Return(_)
            | Object::StructDeclaration(_)
            | Object::Tag(_)
            | Object::Text(_)
            | Object::TunnelOnwards(_)
            | Object::VariableAssignment(_) => {}
        }
    }
}

fn check_nested_choice_termination_in_content_list(
    content: &ContentList,
    inside_sealed_content: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for object in content.objects() {
        match object {
            Object::Conditional(conditional) => {
                for branch in conditional.branches() {
                    check_nested_choice_termination_in_weave(branch.content(), true, diagnostics);
                }
            }
            Object::ContentList(content) => check_nested_choice_termination_in_content_list(
                content,
                inside_sealed_content,
                diagnostics,
            ),
            Object::Weave(weave) => {
                check_nested_choice_termination_in_weave(weave, inside_sealed_content, diagnostics)
            }
            Object::AuthorWarning(_)
            | Object::Choice(_)
            | Object::ConstantDeclaration(_)
            | Object::Divert(_)
            | Object::Expression(_)
            | Object::ExternalDeclaration(_)
            | Object::Gather(_)
            | Object::Glue(_)
            | Object::IncDec(_)
            | Object::LogicLine(_)
            | Object::Return(_)
            | Object::StructDeclaration(_)
            | Object::Tag(_)
            | Object::Text(_)
            | Object::TunnelOnwards(_)
            | Object::VariableAssignment(_) => {}
        }
    }
}

fn choice_flow_terminates(choice: &Choice, following: &[Object]) -> bool {
    let mut terminating = last_significant_object(choice.inner_content().objects());
    for object in following {
        if matches!(
            object,
            Object::Choice(_) | Object::Gather(_) | Object::Weave(_)
        ) {
            break;
        }
        if !is_termination_ignored_object(object) {
            terminating = Some(object);
        }
    }

    terminating.is_some_and(object_terminates_flow)
}

fn check_function_flow_control(
    current_module: Option<&str>,
    flow: &Flow,
    current_flow_path: &str,
    analysis: &FlowAnalysisIndexes<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if flow.level() != FlowLevel::Knot {
        diagnostics.push(Diagnostic::error(
            first_span_in_weave(flow.weave()),
            "Functions cannot be stitches - i.e. they should be defined as '== function myFunc ==' rather than public to another knot.",
        ));
    }

    for child in flow.child_flows() {
        diagnostics.push(Diagnostic::error(
            first_span_in_weave(child.weave()),
            format!(
                "Functions may not contain stitches, but saw '{}' within the function '{}'",
                child.name(),
                flow.name()
            ),
        ));
    }

    let mut visitor = FunctionFlowControlVisitor {
        diagnostics,
        function_name: flow.name(),
        return_type: flow.return_type(),
        check_return_types: flow.has_typed_signature(),
        analysis,
    };
    let context = VisitContext {
        current_module: current_module.map(str::to_string),
        current_flow_path: Some(current_flow_path.to_string()),
        inside_function: true,
        ..VisitContext::default()
    };
    walk_weave(flow.weave(), &mut visitor, &context);
}

struct FunctionFlowControlVisitor<'a> {
    diagnostics: &'a mut Vec<Diagnostic>,
    function_name: &'a str,
    return_type: &'a TypeName,
    check_return_types: bool,
    analysis: &'a FlowAnalysisIndexes<'a>,
}

impl ParsedVisitor for FunctionFlowControlVisitor<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        if context.inside_choice_content || context.inside_expression {
            return;
        }

        match object {
            Object::Divert(divert) => self.diagnostics.push(Diagnostic::error(
                divert.span().clone(),
                format!(
                    "Functions may not contain diverts, but saw '-> {}'",
                    divert.target().to_snapshot_string()
                ),
            )),
            Object::Choice(choice) => self.diagnostics.push(Diagnostic::error(
                choice.span().clone(),
                "Functions may not contain choices",
            )),
            Object::Return(ret) if self.check_return_types => self.check_return(ret, context),
            _ => {}
        }
    }
}

impl FunctionFlowControlVisitor<'_> {
    fn check_return(&mut self, ret: &Return, context: &VisitContext) {
        match (ret.returned_expression(), self.return_type.is_void()) {
            (None, true) => {}
            (None, false) => self.diagnostics.push(Diagnostic::error(
                ret.span().clone(),
                format!(
                    "Function '{}' must return {} but return has no value",
                    self.function_name,
                    self.return_type.display_name()
                ),
            )),
            (Some(_), true) => self.diagnostics.push(Diagnostic::error(
                ret.span().clone(),
                format!(
                    "Function '{}' returns void but return has a value",
                    self.function_name
                ),
            )),
            (Some(expression), false) => self.check_return_expression(expression, ret, context),
        }
    }

    fn check_return_expression(
        &mut self,
        expression: &crate::parsed::Expression,
        ret: &Return,
        context: &VisitContext,
    ) {
        match infer_expression_type(
            expression,
            self.analysis.variable_scopes,
            self.analysis.struct_types,
            self.analysis.target_symbols,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            Ok(actual_type) if &actual_type != self.return_type => {
                self.diagnostics.push(Diagnostic::error(
                    ret.span().clone(),
                    format!(
                        "Function '{}' returns {} but declared return type is {}",
                        self.function_name,
                        actual_type.display_name(),
                        self.return_type.display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.push(Diagnostic::error(
                ret.span().clone(),
                format!(
                    "Cannot type-check return value for function '{}': {}",
                    self.function_name,
                    error.message()
                ),
            )),
        }
    }
}

fn find_return_in_flow(flow: &Flow) -> Option<&Return> {
    find_return_in_weave(flow.weave())
        .or_else(|| flow.child_flows().iter().find_map(find_return_in_flow))
}

fn find_return_in_weave(weave: &Weave) -> Option<&Return> {
    weave.content().iter().find_map(find_return_in_object)
}

fn find_return_in_content_list(content: &ContentList) -> Option<&Return> {
    content.objects().iter().find_map(find_return_in_object)
}

fn find_return_in_object(object: &Object) -> Option<&Return> {
    match object {
        Object::Return(ret) => Some(ret),
        Object::ContentList(content) => find_return_in_content_list(content),
        Object::Conditional(conditional) => conditional
            .branches()
            .iter()
            .find_map(|branch| find_return_in_weave(branch.content())),
        Object::Weave(weave) => find_return_in_weave(weave),
        Object::Choice(choice) => choice
            .start_content()
            .and_then(find_return_in_content_list)
            .or_else(|| find_return_in_content_list(choice.inner_content())),
        Object::AuthorWarning(_)
        | Object::ConstantDeclaration(_)
        | Object::Divert(_)
        | Object::Expression(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::IncDec(_)
        | Object::LogicLine(_)
        | Object::StructDeclaration(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::TunnelOnwards(_)
        | Object::VariableAssignment(_) => None,
    }
}

fn loose_end_warning_span(weave: &Weave) -> Option<SourceSpan> {
    let terminating = last_significant_object(weave.content())?;
    (!object_terminates_flow(terminating)).then(|| object_span(terminating))
}

fn last_significant_object(objects: &[Object]) -> Option<&Object> {
    objects
        .iter()
        .rev()
        .find(|object| !is_termination_ignored_object(object))
}

fn is_termination_ignored_object(object: &Object) -> bool {
    matches!(object, Object::Text(text) if text.text().trim().is_empty())
        || matches!(object, Object::AuthorWarning(_))
        || matches!(object, Object::ConstantDeclaration(_))
        || matches!(object, Object::ExternalDeclaration(_))
        || matches!(object, Object::StructDeclaration(_))
        || matches!(object, Object::VariableAssignment(assignment) if assignment.is_global())
}

fn object_terminates_flow(object: &Object) -> bool {
    match object {
        Object::Divert(divert) => {
            !divert.is_tunnel() && !matches!(divert.target(), DivertTarget::Empty)
        }
        Object::TunnelOnwards(_) | Object::Choice(_) | Object::Return(_) => true,
        Object::ContentList(content) => {
            last_significant_object(content.objects()).is_some_and(object_terminates_flow)
        }
        Object::Weave(weave) => {
            last_significant_object(weave.content()).is_some_and(object_terminates_flow)
        }
        Object::Conditional(conditional) => {
            conditional
                .branches()
                .last()
                .is_some_and(|branch| branch.is_else())
                && conditional.branches().iter().all(|branch| {
                    last_significant_object(branch.content().content())
                        .is_some_and(object_terminates_flow)
                })
        }
        Object::AuthorWarning(_)
        | Object::ConstantDeclaration(_)
        | Object::Expression(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::IncDec(_)
        | Object::LogicLine(_)
        | Object::StructDeclaration(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::VariableAssignment(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::diagnostic::DiagnosticSeverity;

    use super::{
        super::test_support::{assert_single_diagnostic, parse_story},
        *,
    };

    #[test]
    fn reports_loose_end_warnings_for_unterminated_knots() {
        let story = parse_story("== knot ==\nLine.");
        let diagnostics = flow_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Warning,
            "Apparent loose end exists where the flow runs out. Do you need a '-> DONE' statement, choice or divert?",
        );
    }

    #[test]
    fn accepts_bool_conditions() {
        let story = parse_story(
            "VAR ready: bool = true\n\
             { ready:\n\
               Conditional text.\n\
             }\n\
             * { ready } Choice text\n\
             -> DONE",
        );

        assert_eq!(flow_diagnostics(&story), []);
    }

    #[test]
    fn reports_global_var_declarations_outside_story_top_level() {
        let cases = ["{ true:\n\
               VAR score: int = 0\n\
             }\n\
             -> DONE"];

        for source in cases {
            let story = parse_story(source);
            let diagnostics = flow_diagnostics(&story);

            assert_single_diagnostic(
                &diagnostics,
                DiagnosticSeverity::Error,
                "Global VAR declarations must appear at the story top level, outside knots, stitches, functions, choices, and conditionals.",
            );
        }
    }

    #[test]
    fn reports_non_bool_typed_conditions() {
        let cases = [
            (
                "VAR value: int = 1\n\
                 { value:\n\
                   Text.\n\
                 }",
                "Conditional condition has type int but expected bool",
            ),
            (
                "VAR value: float = 1.0\n\
                 { value:\n\
                   Text.\n\
                 }",
                "Conditional condition has type float but expected bool",
            ),
            (
                "VAR label: string = \"yes\"\n\
                 * { label } Choice text",
                "Choice condition has type string but expected bool",
            ),
            (
                "VAR values: int[] = [1]\n\
                 { values:\n\
                   Text.\n\
                 }",
                "Conditional condition has type int[] but expected bool",
            ),
            (
                "STRUCT Player {\n\
                 hp: int\n\
                 }\n\
                 VAR player: Player = { hp: 10 }\n\
                 { player:\n\
                   Text.\n\
                 }",
                "Conditional condition has type Player but expected bool",
            ),
        ];

        for (source, expected_message) in cases {
            let story = parse_story(source);
            let diagnostics = flow_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn accepts_valid_primitive_and_composite_function_returns() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR default_player: Player = { hp: 10 }\n\
             VAR default_scores: int[] = [1]\n\
             == function add(a: int, b: int) => int ==\n\
             ~ return a + b\n\
             == function make_player() => Player ==\n\
             ~ return default_player\n\
             == function scores() => int[] ==\n\
             ~ return default_scores",
        );

        assert_eq!(flow_diagnostics(&story), []);
    }

    #[test]
    fn module_functions_reject_diverts() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             ~ helper()\n\
             -> DONE\n\
             == function helper() => void ==\n\
             -> DONE",
        );
        let diagnostics = flow_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Functions may not contain diverts, but saw '-> DONE'",
        );
    }

    #[test]
    fn module_flows_report_loose_end_warnings() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             Line.",
        );
        let diagnostics = flow_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Warning,
            "Apparent loose end exists where the flow runs out. Do you need a '-> DONE' statement, choice or divert?",
        );
    }

    #[test]
    fn module_function_return_types_use_module_scope() {
        let story = parse_story(
            "=== module game ===\n\
             VAR score: int = 1\n\
             == main ==\n\
             -> DONE\n\
             == function current_score() => int ==\n\
             ~ return score",
        );

        assert_eq!(flow_diagnostics(&story), []);
    }

    #[test]
    fn accepts_bare_return_in_void_function() {
        let story = parse_story(
            "== function log(message: string) => void ==\n\
             ~ return",
        );

        assert_eq!(flow_diagnostics(&story), []);
    }

    #[test]
    fn reports_missing_return_value_in_non_void_function() {
        let story = parse_story(
            "== function score() => int ==\n\
             ~ return",
        );

        let diagnostics = flow_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Function 'score' must return int but return has no value",
        );
    }

    #[test]
    fn reports_value_return_in_void_function() {
        let story = parse_story(
            "== function log() => void ==\n\
             ~ return \"ok\"",
        );

        let diagnostics = flow_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Function 'log' returns void but return has a value",
        );
    }

    #[test]
    fn reports_wrong_return_type() {
        let story = parse_story(
            "== function score(label: string) => int ==\n\
             ~ return label",
        );

        let diagnostics = flow_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Function 'score' returns string but declared return type is int",
        );
    }

    #[test]
    fn reports_wrong_composite_return_type() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR scores: int[] = [1]\n\
             == function make_player() => Player ==\n\
             ~ return scores",
        );

        let diagnostics = flow_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Function 'make_player' returns int[] but declared return type is Player",
        );
    }
}
