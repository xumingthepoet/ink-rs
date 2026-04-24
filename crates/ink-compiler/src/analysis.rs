use std::collections::{HashMap, HashSet};

use crate::{
    compiler::StageOutput,
    diagnostic::Diagnostic,
    parsed::{
        ContentList, DivertTarget, Expression, Flow, FlowLevel, Object, Return, Story, Weave,
    },
    source::SourceSpan,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedStory {
    pub parsed: Story,
}

pub(crate) fn analyze(parsed: Story) -> StageOutput<CheckedStory> {
    let mut diagnostics = naming_diagnostics(&parsed);
    diagnostics.extend(flow_diagnostics(&parsed));
    diagnostics.extend(call_target_diagnostics(&parsed));
    StageOutput {
        artifact: Some(CheckedStory { parsed }),
        diagnostics,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymbolKind {
    Function,
    Knot,
    Var,
}

impl SymbolKind {
    fn display_name(self) -> &'static str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Knot => "knot",
            SymbolKind::Var => "var",
        }
    }
}

fn naming_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let mut top_level_flows = HashMap::new();
    for flow in story.flows() {
        let kind = if flow.is_function() {
            SymbolKind::Function
        } else {
            SymbolKind::Knot
        };
        top_level_flows.insert(flow.name().to_string(), kind);
    }

    let mut global_variables = HashSet::new();
    collect_global_variables(story.root_weave(), &mut global_variables);
    for flow in story.flows() {
        collect_global_variables_in_flow(flow, &mut global_variables);
    }

    let mut diagnostics = Vec::new();
    for flow in story.flows() {
        check_flow_arguments(flow, &top_level_flows, &global_variables, &mut diagnostics);
    }
    diagnostics
}

fn collect_global_variables(weave: &Weave, global_variables: &mut HashSet<String>) {
    for object in weave.content() {
        collect_global_variables_in_object(object, global_variables);
    }
}

fn collect_global_variables_in_flow(flow: &Flow, global_variables: &mut HashSet<String>) {
    collect_global_variables(flow.weave(), global_variables);
    for child in flow.child_flows() {
        collect_global_variables_in_flow(child, global_variables);
    }
}

fn collect_global_variables_in_object(object: &Object, global_variables: &mut HashSet<String>) {
    match object {
        Object::VariableAssignment(assignment) if assignment.is_global() => {
            global_variables.insert(assignment.name().to_string());
        }
        Object::ConstantDeclaration(declaration) => {
            global_variables.insert(declaration.name().to_string());
        }
        Object::ContentList(content) => {
            for object in content.objects() {
                collect_global_variables_in_object(object, global_variables);
            }
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                for object in branch.content().content() {
                    collect_global_variables_in_object(object, global_variables);
                }
            }
        }
        Object::Sequence(sequence) => {
            for content in sequence.elements() {
                for object in content.objects() {
                    collect_global_variables_in_object(object, global_variables);
                }
            }
        }
        Object::Weave(weave) => collect_global_variables(weave, global_variables),
        Object::Choice(_)
        | Object::Divert(_)
        | Object::Expression(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::IncDec(_)
        | Object::LogicLine(_)
        | Object::Return(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::TunnelOnwards(_)
        | Object::VariableAssignment(_) => {}
    }
}

fn check_flow_arguments(
    flow: &Flow,
    top_level_flows: &HashMap<String, SymbolKind>,
    global_variables: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen_arguments = HashSet::new();
    for argument in flow.arguments() {
        if let Some(kind) = top_level_flows.get(argument.name()) {
            diagnostics.push(name_conflict_diagnostic(
                "argument",
                argument.name(),
                *kind,
                argument.span().clone(),
            ));
        } else if global_variables.contains(argument.name()) {
            diagnostics.push(name_conflict_diagnostic(
                "argument",
                argument.name(),
                SymbolKind::Var,
                argument.span().clone(),
            ));
        }

        if !seen_arguments.insert(argument.name()) {
            diagnostics.push(Diagnostic::error(
                argument.span().clone(),
                format!(
                    "Multiple arguments with the same name: '{}'",
                    argument.name()
                ),
            ));
        }
    }

    for child in flow.child_flows() {
        check_flow_arguments(child, top_level_flows, global_variables, diagnostics);
    }
}

fn name_conflict_diagnostic(
    symbol_type: &str,
    name: &str,
    existing_kind: SymbolKind,
    span: crate::source::SourceSpan,
) -> Diagnostic {
    Diagnostic::error(
        span,
        format!(
            "{symbol_type} '{name}': name has already been used for a {}",
            existing_kind.display_name()
        ),
    )
}

fn flow_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for flow in story.flows() {
        check_flow(flow, &mut diagnostics);
    }
    diagnostics
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FlowSymbol {
    is_function: bool,
}

fn call_target_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let flow_symbols = build_flow_symbol_index(story);
    let mut diagnostics = Vec::new();

    check_call_targets_in_weave(story.root_weave(), &flow_symbols, &mut diagnostics, false);
    for flow in story.flows() {
        check_call_targets_in_flow(flow, &flow_symbols, &mut diagnostics);
    }

    diagnostics
}

fn build_flow_symbol_index(story: &Story) -> HashMap<String, FlowSymbol> {
    let mut symbols = HashMap::new();
    for flow in story.flows() {
        collect_flow_symbol(flow, None, &mut symbols);
    }
    symbols
}

fn collect_flow_symbol(
    flow: &Flow,
    parent_path: Option<&str>,
    symbols: &mut HashMap<String, FlowSymbol>,
) {
    let path = parent_path
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    let symbol = FlowSymbol {
        is_function: flow.is_function(),
    };

    symbols.entry(path.clone()).or_insert(symbol);
    if parent_path.is_none() {
        symbols.entry(flow.name().to_string()).or_insert(symbol);
    }

    for child in flow.child_flows() {
        collect_flow_symbol(child, Some(&path), symbols);
    }
}

fn check_call_targets_in_flow(
    flow: &Flow,
    flow_symbols: &HashMap<String, FlowSymbol>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    check_call_targets_in_weave(flow.weave(), flow_symbols, diagnostics, flow.is_function());
    for child in flow.child_flows() {
        check_call_targets_in_flow(child, flow_symbols, diagnostics);
    }
}

fn check_call_targets_in_weave(
    weave: &Weave,
    flow_symbols: &HashMap<String, FlowSymbol>,
    diagnostics: &mut Vec<Diagnostic>,
    inside_function: bool,
) {
    for object in weave.content() {
        check_call_targets_in_object(object, flow_symbols, diagnostics, inside_function);
    }
}

fn check_call_targets_in_content_list(
    content: &ContentList,
    flow_symbols: &HashMap<String, FlowSymbol>,
    diagnostics: &mut Vec<Diagnostic>,
    inside_function: bool,
) {
    for object in content.objects() {
        check_call_targets_in_object(object, flow_symbols, diagnostics, inside_function);
    }
}

fn check_call_targets_in_object(
    object: &Object,
    flow_symbols: &HashMap<String, FlowSymbol>,
    diagnostics: &mut Vec<Diagnostic>,
    inside_function: bool,
) {
    match object {
        Object::Divert(divert) => {
            if !inside_function {
                if let DivertTarget::Path(target) = divert.target() {
                    check_plain_divert_target(target, divert.span(), flow_symbols, diagnostics);
                }
            }
            for argument in divert.arguments() {
                check_call_targets_in_expression(
                    argument,
                    divert.span(),
                    flow_symbols,
                    diagnostics,
                    inside_function,
                );
            }
        }
        Object::Expression(expression) | Object::LogicLine(expression) => {
            check_call_targets_in_expression(
                expression,
                &object_span(object),
                flow_symbols,
                diagnostics,
                inside_function,
            );
        }
        Object::VariableAssignment(assignment) => check_call_targets_in_expression(
            assignment.expression(),
            assignment.span(),
            flow_symbols,
            diagnostics,
            inside_function,
        ),
        Object::IncDec(inc_dec) => check_call_targets_in_expression(
            inc_dec.expression(),
            inc_dec.span(),
            flow_symbols,
            diagnostics,
            inside_function,
        ),
        Object::Return(ret) => {
            if let Some(expression) = ret.returned_expression() {
                check_call_targets_in_expression(
                    expression,
                    ret.span(),
                    flow_symbols,
                    diagnostics,
                    inside_function,
                );
            }
        }
        Object::ContentList(content) => {
            check_call_targets_in_content_list(content, flow_symbols, diagnostics, inside_function)
        }
        Object::Conditional(conditional) => {
            if let Some(condition) = conditional.initial_condition() {
                check_call_targets_in_expression(
                    condition,
                    &object_span(object),
                    flow_symbols,
                    diagnostics,
                    inside_function,
                );
            }
            for branch in conditional.branches() {
                if let Some(condition) = branch.own_condition() {
                    check_call_targets_in_expression(
                        condition,
                        &object_span(object),
                        flow_symbols,
                        diagnostics,
                        inside_function,
                    );
                }
                check_call_targets_in_weave(
                    branch.content(),
                    flow_symbols,
                    diagnostics,
                    inside_function,
                );
            }
        }
        Object::Choice(choice) => {
            if let Some(condition) = choice.condition() {
                check_call_targets_in_expression(
                    condition,
                    choice.span(),
                    flow_symbols,
                    diagnostics,
                    inside_function,
                );
            }
            if let Some(content) = choice.start_content() {
                check_call_targets_in_content_list(
                    content,
                    flow_symbols,
                    diagnostics,
                    inside_function,
                );
            }
            if let Some(content) = choice.choice_only_content() {
                check_call_targets_in_content_list(
                    content,
                    flow_symbols,
                    diagnostics,
                    inside_function,
                );
            }
            check_call_targets_in_content_list(
                choice.inner_content(),
                flow_symbols,
                diagnostics,
                inside_function,
            );
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                check_call_targets_in_content_list(
                    element,
                    flow_symbols,
                    diagnostics,
                    inside_function,
                );
            }
        }
        Object::Weave(weave) => {
            check_call_targets_in_weave(weave, flow_symbols, diagnostics, inside_function)
        }
        Object::ConstantDeclaration(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::TunnelOnwards(_) => {}
    }
}

fn check_plain_divert_target(
    target: &str,
    span: &SourceSpan,
    flow_symbols: &HashMap<String, FlowSymbol>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(symbol) = flow_symbols.get(target) {
        if symbol.is_function {
            diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "{target} can't be diverted to. It can only be called as a function since it's been marked as such: '{target}(...)'"
                ),
            ));
        }
    }
}

fn check_call_targets_in_expression(
    expression: &Expression,
    span: &SourceSpan,
    flow_symbols: &HashMap<String, FlowSymbol>,
    diagnostics: &mut Vec<Diagnostic>,
    inside_function: bool,
) {
    match expression {
        Expression::FunctionCall { name, args } => {
            if let Some(symbol) = flow_symbols.get(name) {
                if !symbol.is_function {
                    diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "{name} hasn't been marked as a function, but it's being called as one. Do you need to delcare the knot as '== function {name} =='?"
                        ),
                    ));
                }
            }
            for arg in args {
                check_call_targets_in_expression(
                    arg,
                    span,
                    flow_symbols,
                    diagnostics,
                    inside_function,
                );
            }
        }
        Expression::StringContent(content) => {
            check_call_targets_in_content_list(content, flow_symbols, diagnostics, inside_function)
        }
        Expression::Binary { left, right, .. } => {
            check_call_targets_in_expression(
                left,
                span,
                flow_symbols,
                diagnostics,
                inside_function,
            );
            check_call_targets_in_expression(
                right,
                span,
                flow_symbols,
                diagnostics,
                inside_function,
            );
        }
        Expression::Unary { expression, .. } => {
            check_call_targets_in_expression(
                expression,
                span,
                flow_symbols,
                diagnostics,
                inside_function,
            );
        }
        Expression::MultipleCondition(expressions) => {
            for expression in expressions {
                check_call_targets_in_expression(
                    expression,
                    span,
                    flow_symbols,
                    diagnostics,
                    inside_function,
                );
            }
        }
        Expression::String(_)
        | Expression::NumberInt(_)
        | Expression::NumberFloat(_)
        | Expression::NumberBool(_)
        | Expression::DivertTarget(_)
        | Expression::VariableReference(_) => {}
    }
}

fn check_flow(flow: &Flow, diagnostics: &mut Vec<Diagnostic>) {
    let found_return = find_return_in_flow(flow);

    if flow.is_function() {
        check_function_flow_control(flow, diagnostics);
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
        check_flow(child, diagnostics);
    }
}

fn check_function_flow_control(flow: &Flow, diagnostics: &mut Vec<Diagnostic>) {
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
    check_function_flow_control_in_weave(flow.weave(), diagnostics);
}

fn check_function_flow_control_in_weave(weave: &Weave, diagnostics: &mut Vec<Diagnostic>) {
    for object in weave.content() {
        check_function_flow_control_in_object(object, diagnostics);
    }
}

fn check_function_flow_control_in_content_list(
    content: &ContentList,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for object in content.objects() {
        check_function_flow_control_in_object(object, diagnostics);
    }
}

fn check_function_flow_control_in_object(object: &Object, diagnostics: &mut Vec<Diagnostic>) {
    match object {
        Object::Divert(divert) => diagnostics.push(Diagnostic::error(
            divert.span().clone(),
            format!(
                "Functions may not contain diverts, but saw '-> {}'",
                divert.target().to_snapshot_string()
            ),
        )),
        Object::Choice(choice) => diagnostics.push(Diagnostic::error(
            choice.span().clone(),
            "Functions may not contain choices",
        )),
        Object::ContentList(content) => {
            check_function_flow_control_in_content_list(content, diagnostics)
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                check_function_flow_control_in_weave(branch.content(), diagnostics);
            }
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                check_function_flow_control_in_content_list(element, diagnostics);
            }
        }
        Object::Weave(weave) => check_function_flow_control_in_weave(weave, diagnostics),
        Object::ConstantDeclaration(_)
        | Object::Expression(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::IncDec(_)
        | Object::LogicLine(_)
        | Object::Return(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::TunnelOnwards(_)
        | Object::VariableAssignment(_) => {}
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
        Object::Sequence(sequence) => sequence
            .elements()
            .iter()
            .find_map(find_return_in_content_list),
        Object::Weave(weave) => find_return_in_weave(weave),
        Object::Choice(choice) => choice
            .start_content()
            .and_then(find_return_in_content_list)
            .or_else(|| {
                choice
                    .choice_only_content()
                    .and_then(find_return_in_content_list)
            })
            .or_else(|| find_return_in_content_list(choice.inner_content())),
        Object::ConstantDeclaration(_)
        | Object::Divert(_)
        | Object::Expression(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::IncDec(_)
        | Object::LogicLine(_)
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
        || matches!(object, Object::ConstantDeclaration(_))
        || matches!(object, Object::ExternalDeclaration(_))
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
        Object::Sequence(sequence) => sequence.elements().iter().all(|element| {
            last_significant_object(element.objects()).is_some_and(object_terminates_flow)
        }),
        Object::ConstantDeclaration(_)
        | Object::Expression(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::IncDec(_)
        | Object::LogicLine(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::VariableAssignment(_) => false,
    }
}

fn first_span_in_weave(weave: &Weave) -> SourceSpan {
    weave
        .content()
        .first()
        .map(object_span)
        .unwrap_or_else(default_span)
}

fn object_span(object: &Object) -> SourceSpan {
    match object {
        Object::Choice(choice) => choice.span().clone(),
        Object::ConstantDeclaration(declaration) => declaration.span().clone(),
        Object::Divert(divert) => divert.span().clone(),
        Object::Gather(gather) => gather.span().clone(),
        Object::IncDec(inc_dec) => inc_dec.span().clone(),
        Object::Return(ret) => ret.span().clone(),
        Object::Text(text) => text.span().clone(),
        Object::TunnelOnwards(tunnel_onwards) => tunnel_onwards.span().clone(),
        Object::VariableAssignment(assignment) => assignment.span().clone(),
        Object::ContentList(content) => content
            .objects()
            .first()
            .map(object_span)
            .unwrap_or_else(default_span),
        Object::Conditional(conditional) => conditional
            .branches()
            .iter()
            .flat_map(|branch| branch.content().content())
            .next()
            .map(object_span)
            .unwrap_or_else(default_span),
        Object::Sequence(sequence) => sequence
            .elements()
            .iter()
            .flat_map(|element| element.objects())
            .next()
            .map(object_span)
            .unwrap_or_else(default_span),
        Object::Weave(weave) => first_span_in_weave(weave),
        Object::Expression(_)
        | Object::ExternalDeclaration(_)
        | Object::Glue(_)
        | Object::LogicLine(_)
        | Object::Tag(_) => default_span(),
    }
}

fn default_span() -> SourceSpan {
    SourceSpan::new(None, 1, 1)
}
