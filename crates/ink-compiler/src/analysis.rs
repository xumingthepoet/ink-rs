use std::collections::{HashMap, HashSet};

use crate::{
    compiler::StageOutput,
    diagnostic::Diagnostic,
    parsed::{
        ContentList, DivertTarget, Expression, Flow, FlowArgument, FlowLevel, Object, Return,
        Story, Weave,
    },
    source::SourceSpan,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedStory {
    pub parsed: Story,
}

pub(crate) fn analyze(parsed: Story) -> StageOutput<CheckedStory> {
    let mut diagnostics = constant_redefinition_diagnostics(&parsed);
    diagnostics.extend(author_warning_diagnostics(&parsed));
    diagnostics.extend(naming_diagnostics(&parsed));
    diagnostics.extend(flow_diagnostics(&parsed));
    diagnostics.extend(call_target_diagnostics(&parsed));
    StageOutput {
        artifact: Some(CheckedStory { parsed }),
        diagnostics,
    }
}

fn author_warning_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    collect_author_warnings_in_weave(story.root_weave(), &mut diagnostics);
    for flow in story.flows() {
        collect_author_warnings_in_flow(flow, &mut diagnostics);
    }
    diagnostics
}

fn collect_author_warnings_in_flow(flow: &Flow, diagnostics: &mut Vec<Diagnostic>) {
    collect_author_warnings_in_weave(flow.weave(), diagnostics);
    for child in flow.child_flows() {
        collect_author_warnings_in_flow(child, diagnostics);
    }
}

fn collect_author_warnings_in_weave(weave: &Weave, diagnostics: &mut Vec<Diagnostic>) {
    for object in weave.content() {
        collect_author_warnings_in_object(object, diagnostics);
    }
}

fn collect_author_warnings_in_content_list(
    content: &ContentList,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for object in content.objects() {
        collect_author_warnings_in_object(object, diagnostics);
    }
}

fn collect_author_warnings_in_object(object: &Object, diagnostics: &mut Vec<Diagnostic>) {
    match object {
        Object::AuthorWarning(author_warning) => diagnostics.push(Diagnostic::author(
            author_warning.span().clone(),
            author_warning.message().to_string(),
        )),
        Object::ContentList(content) => {
            collect_author_warnings_in_content_list(content, diagnostics)
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_author_warnings_in_weave(branch.content(), diagnostics);
            }
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_author_warnings_in_content_list(content, diagnostics);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_author_warnings_in_content_list(content, diagnostics);
            }
            collect_author_warnings_in_content_list(choice.inner_content(), diagnostics);
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_author_warnings_in_content_list(element, diagnostics);
            }
        }
        Object::Weave(weave) => collect_author_warnings_in_weave(weave, diagnostics),
        Object::ConstantDeclaration(_)
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

fn constant_redefinition_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut constants = HashMap::new();
    collect_constant_redefinition_diagnostics_in_weave(
        story.root_weave(),
        &mut constants,
        &mut diagnostics,
    );
    for flow in story.flows() {
        collect_constant_redefinition_diagnostics_in_flow(flow, &mut constants, &mut diagnostics);
    }
    diagnostics
}

fn collect_constant_redefinition_diagnostics_in_flow(
    flow: &Flow,
    constants: &mut HashMap<String, Expression>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    collect_constant_redefinition_diagnostics_in_weave(flow.weave(), constants, diagnostics);
    for child in flow.child_flows() {
        collect_constant_redefinition_diagnostics_in_flow(child, constants, diagnostics);
    }
}

fn collect_constant_redefinition_diagnostics_in_weave(
    weave: &Weave,
    constants: &mut HashMap<String, Expression>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for object in weave.content() {
        collect_constant_redefinition_diagnostics_in_object(object, constants, diagnostics);
    }
}

fn collect_constant_redefinition_diagnostics_in_content_list(
    content: &ContentList,
    constants: &mut HashMap<String, Expression>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for object in content.objects() {
        collect_constant_redefinition_diagnostics_in_object(object, constants, diagnostics);
    }
}

fn collect_constant_redefinition_diagnostics_in_object(
    object: &Object,
    constants: &mut HashMap<String, Expression>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match object {
        Object::ConstantDeclaration(declaration) => {
            if let Some(existing) = constants.get(declaration.name()) {
                if existing != declaration.expression() {
                    diagnostics.push(Diagnostic::error(
                        declaration.span().clone(),
                        format!(
                            "CONST '{}' has been redefined with a different value",
                            declaration.name()
                        ),
                    ));
                }
            }
            constants.insert(
                declaration.name().to_string(),
                declaration.expression().clone(),
            );
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_constant_redefinition_diagnostics_in_content_list(
                    content,
                    constants,
                    diagnostics,
                );
            }
            if let Some(content) = choice.choice_only_content() {
                collect_constant_redefinition_diagnostics_in_content_list(
                    content,
                    constants,
                    diagnostics,
                );
            }
            collect_constant_redefinition_diagnostics_in_content_list(
                choice.inner_content(),
                constants,
                diagnostics,
            );
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_constant_redefinition_diagnostics_in_weave(
                    branch.content(),
                    constants,
                    diagnostics,
                );
            }
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_constant_redefinition_diagnostics_in_content_list(
                    element,
                    constants,
                    diagnostics,
                );
            }
        }
        Object::ContentList(content) => collect_constant_redefinition_diagnostics_in_content_list(
            content,
            constants,
            diagnostics,
        ),
        Object::Weave(weave) => {
            collect_constant_redefinition_diagnostics_in_weave(weave, constants, diagnostics)
        }
        Object::AuthorWarning(_)
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
    check_weave_point_names(story.root_weave(), &global_variables, &mut diagnostics);
    for flow in story.flows() {
        check_subflow_and_weave_names(flow, &global_variables, &mut diagnostics);
        check_flow_arguments(flow, &top_level_flows, &global_variables, &mut diagnostics);
        check_temporary_names_against_arguments(flow, &mut diagnostics);
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
        Object::AuthorWarning(_)
        | Object::Choice(_)
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

fn check_subflow_and_weave_names(
    flow: &Flow,
    global_variables: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    check_weave_point_names(flow.weave(), global_variables, diagnostics);

    for child in flow.child_flows() {
        if global_variables.contains(child.name()) {
            diagnostics.push(name_conflict_diagnostic(
                "stitch",
                child.name(),
                SymbolKind::Var,
                first_span_in_weave(child.weave()),
            ));
        }
        check_subflow_and_weave_names(child, global_variables, diagnostics);
    }
}

fn check_temporary_names_against_arguments(flow: &Flow, diagnostics: &mut Vec<Diagnostic>) {
    let argument_names: HashSet<String> = flow
        .arguments()
        .iter()
        .map(|argument| argument.name().to_string())
        .collect();
    if !argument_names.is_empty() {
        check_temporary_names_against_arguments_in_weave(
            flow.weave(),
            flow.name(),
            &argument_names,
            diagnostics,
        );
    }

    for child in flow.child_flows() {
        check_temporary_names_against_arguments(child, diagnostics);
    }
}

fn check_temporary_names_against_arguments_in_weave(
    weave: &Weave,
    flow_name: &str,
    argument_names: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for object in weave.content() {
        check_temporary_names_against_arguments_in_object(
            object,
            flow_name,
            argument_names,
            diagnostics,
        );
    }
}

fn check_temporary_names_against_arguments_in_content_list(
    content: &ContentList,
    flow_name: &str,
    argument_names: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for object in content.objects() {
        check_temporary_names_against_arguments_in_object(
            object,
            flow_name,
            argument_names,
            diagnostics,
        );
    }
}

fn check_temporary_names_against_arguments_in_object(
    object: &Object,
    flow_name: &str,
    argument_names: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match object {
        Object::VariableAssignment(assignment)
            if assignment.is_temporary() && argument_names.contains(assignment.name()) =>
        {
            diagnostics.push(Diagnostic::error(
                assignment.span().clone(),
                format!(
                    "temp '{}': name has already been used for an argument to {}",
                    assignment.name(),
                    flow_name
                ),
            ));
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                check_temporary_names_against_arguments_in_content_list(
                    content,
                    flow_name,
                    argument_names,
                    diagnostics,
                );
            }
            if let Some(content) = choice.choice_only_content() {
                check_temporary_names_against_arguments_in_content_list(
                    content,
                    flow_name,
                    argument_names,
                    diagnostics,
                );
            }
            check_temporary_names_against_arguments_in_content_list(
                choice.inner_content(),
                flow_name,
                argument_names,
                diagnostics,
            );
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                check_temporary_names_against_arguments_in_weave(
                    branch.content(),
                    flow_name,
                    argument_names,
                    diagnostics,
                );
            }
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                check_temporary_names_against_arguments_in_content_list(
                    element,
                    flow_name,
                    argument_names,
                    diagnostics,
                );
            }
        }
        Object::ContentList(content) => check_temporary_names_against_arguments_in_content_list(
            content,
            flow_name,
            argument_names,
            diagnostics,
        ),
        Object::Weave(weave) => check_temporary_names_against_arguments_in_weave(
            weave,
            flow_name,
            argument_names,
            diagnostics,
        ),
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
        | Object::Tag(_)
        | Object::Text(_)
        | Object::TunnelOnwards(_)
        | Object::VariableAssignment(_) => {}
    }
}

fn check_weave_point_names(
    weave: &Weave,
    global_variables: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut local_names: HashMap<String, SourceSpan> = HashMap::new();
    for object in weave.content() {
        match object {
            Object::Choice(choice) => {
                if let Some(name) = choice.identifier() {
                    check_weave_point_name(
                        "choice",
                        name,
                        choice.span().clone(),
                        &mut local_names,
                        global_variables,
                        diagnostics,
                    );
                }
                if let Some(content) = choice.start_content() {
                    check_weave_point_names_in_content_list(content, global_variables, diagnostics);
                }
                if let Some(content) = choice.choice_only_content() {
                    check_weave_point_names_in_content_list(content, global_variables, diagnostics);
                }
                check_weave_point_names_in_content_list(
                    choice.inner_content(),
                    global_variables,
                    diagnostics,
                );
            }
            Object::Gather(gather) => {
                if let Some(name) = gather.identifier() {
                    check_weave_point_name(
                        "gather",
                        name,
                        gather.span().clone(),
                        &mut local_names,
                        global_variables,
                        diagnostics,
                    );
                }
            }
            Object::Conditional(conditional) => {
                for branch in conditional.branches() {
                    check_weave_point_names(branch.content(), global_variables, diagnostics);
                }
            }
            Object::Sequence(sequence) => {
                for element in sequence.elements() {
                    check_weave_point_names_in_content_list(element, global_variables, diagnostics);
                }
            }
            Object::ContentList(content) => {
                check_weave_point_names_in_content_list(content, global_variables, diagnostics)
            }
            Object::Weave(weave) => check_weave_point_names(weave, global_variables, diagnostics),
            Object::AuthorWarning(_)
            | Object::ConstantDeclaration(_)
            | Object::Divert(_)
            | Object::Expression(_)
            | Object::ExternalDeclaration(_)
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
}

fn check_weave_point_names_in_content_list(
    content: &ContentList,
    global_variables: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for object in content.objects() {
        match object {
            Object::Conditional(conditional) => {
                for branch in conditional.branches() {
                    check_weave_point_names(branch.content(), global_variables, diagnostics);
                }
            }
            Object::Sequence(sequence) => {
                for element in sequence.elements() {
                    check_weave_point_names_in_content_list(element, global_variables, diagnostics);
                }
            }
            Object::ContentList(content) => {
                check_weave_point_names_in_content_list(content, global_variables, diagnostics)
            }
            Object::Weave(weave) => check_weave_point_names(weave, global_variables, diagnostics),
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
            | Object::Tag(_)
            | Object::Text(_)
            | Object::TunnelOnwards(_)
            | Object::VariableAssignment(_) => {}
        }
    }
}

fn check_weave_point_name(
    symbol_type: &str,
    name: &str,
    span: SourceSpan,
    local_names: &mut HashMap<String, SourceSpan>,
    global_variables: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if global_variables.contains(name) {
        diagnostics.push(name_conflict_diagnostic(
            symbol_type,
            name,
            SymbolKind::Var,
            span,
        ));
    } else if local_names.insert(name.to_string(), span.clone()).is_some() {
        diagnostics.push(Diagnostic::error(
            span,
            format!("{symbol_type} with the same label '{name}' already exists in this context"),
        ));
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
    check_nested_choice_termination_in_weave(story.root_weave(), false, &mut diagnostics);
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
    let target_symbols = build_target_symbol_index(story);
    let variable_targets = build_variable_target_index(story);
    let mut diagnostics = Vec::new();

    check_call_targets_in_weave(
        story.root_weave(),
        &target_symbols,
        &variable_targets,
        &mut diagnostics,
        None,
        None,
        false,
    );
    for flow in story.flows() {
        check_call_targets_in_flow(
            flow,
            None,
            &target_symbols,
            &variable_targets,
            &mut diagnostics,
        );
    }

    diagnostics
}

fn build_target_symbol_index(story: &Story) -> HashMap<String, FlowSymbol> {
    let mut symbols = HashMap::new();
    for flow in story.flows() {
        collect_flow_symbol(flow, None, &mut symbols);
    }
    collect_target_symbols_in_weave(story.root_weave(), None, &mut symbols);
    for flow in story.flows() {
        collect_target_symbols_in_flow(flow, None, &mut symbols);
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

fn collect_target_symbols_in_flow(
    flow: &Flow,
    parent_path: Option<&str>,
    symbols: &mut HashMap<String, FlowSymbol>,
) {
    let flow_path = parent_path
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    collect_target_symbols_in_weave(flow.weave(), Some(&flow_path), symbols);
    for child in flow.child_flows() {
        collect_target_symbols_in_flow(child, Some(&flow_path), symbols);
    }
}

fn collect_target_symbols_in_weave(
    weave: &Weave,
    flow_path: Option<&str>,
    symbols: &mut HashMap<String, FlowSymbol>,
) {
    for object in weave.content() {
        collect_target_symbols_in_object(object, flow_path, symbols);
    }
}

fn collect_target_symbols_in_content_list(
    content: &ContentList,
    flow_path: Option<&str>,
    symbols: &mut HashMap<String, FlowSymbol>,
) {
    for object in content.objects() {
        collect_target_symbols_in_object(object, flow_path, symbols);
    }
}

fn collect_target_symbols_in_object(
    object: &Object,
    flow_path: Option<&str>,
    symbols: &mut HashMap<String, FlowSymbol>,
) {
    match object {
        Object::Choice(choice) => {
            if let Some(identifier) = choice.identifier() {
                insert_label_symbol(symbols, identifier, flow_path);
            }
            if let Some(content) = choice.start_content() {
                collect_target_symbols_in_content_list(content, flow_path, symbols);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_target_symbols_in_content_list(content, flow_path, symbols);
            }
            collect_target_symbols_in_content_list(choice.inner_content(), flow_path, symbols);
        }
        Object::Gather(gather) => {
            if let Some(identifier) = gather.identifier() {
                insert_label_symbol(symbols, identifier, flow_path);
            }
        }
        Object::ContentList(content) => {
            collect_target_symbols_in_content_list(content, flow_path, symbols)
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_target_symbols_in_weave(branch.content(), flow_path, symbols);
            }
        }
        Object::Sequence(sequence) => {
            for content in sequence.elements() {
                collect_target_symbols_in_content_list(content, flow_path, symbols);
            }
        }
        Object::Weave(weave) => collect_target_symbols_in_weave(weave, flow_path, symbols),
        Object::AuthorWarning(_)
        | Object::ConstantDeclaration(_)
        | Object::Divert(_)
        | Object::Expression(_)
        | Object::ExternalDeclaration(_)
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

fn insert_label_symbol(
    symbols: &mut HashMap<String, FlowSymbol>,
    identifier: &str,
    flow_path: Option<&str>,
) {
    let symbol = FlowSymbol { is_function: false };
    symbols.entry(identifier.to_string()).or_insert(symbol);
    if let Some(flow_path) = flow_path {
        symbols
            .entry(format!("{flow_path}.{identifier}"))
            .or_insert(symbol);
    }
}

fn build_variable_target_index(story: &Story) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_variable_targets_in_weave(story.root_weave(), &mut names);
    for flow in story.flows() {
        collect_variable_targets_in_flow(flow, &mut names);
    }
    names
}

fn collect_variable_targets_in_flow(flow: &Flow, names: &mut HashSet<String>) {
    for argument in flow.arguments() {
        names.insert(argument.name().to_string());
    }
    collect_variable_targets_in_weave(flow.weave(), names);
    for child in flow.child_flows() {
        collect_variable_targets_in_flow(child, names);
    }
}

fn collect_variable_targets_in_weave(weave: &Weave, names: &mut HashSet<String>) {
    for object in weave.content() {
        collect_variable_targets_in_object(object, names);
    }
}

fn collect_variable_targets_in_content_list(content: &ContentList, names: &mut HashSet<String>) {
    for object in content.objects() {
        collect_variable_targets_in_object(object, names);
    }
}

fn collect_variable_targets_in_object(object: &Object, names: &mut HashSet<String>) {
    match object {
        Object::VariableAssignment(assignment) => {
            names.insert(assignment.name().to_string());
            collect_variable_targets_in_expression(assignment.expression(), names);
        }
        Object::Choice(choice) => {
            if let Some(condition) = choice.condition() {
                collect_variable_targets_in_expression(condition, names);
            }
            if let Some(content) = choice.start_content() {
                collect_variable_targets_in_content_list(content, names);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_variable_targets_in_content_list(content, names);
            }
            collect_variable_targets_in_content_list(choice.inner_content(), names);
        }
        Object::ContentList(content) => collect_variable_targets_in_content_list(content, names),
        Object::Conditional(conditional) => {
            if let Some(condition) = conditional.initial_condition() {
                collect_variable_targets_in_expression(condition, names);
            }
            for branch in conditional.branches() {
                if let Some(condition) = branch.own_condition() {
                    collect_variable_targets_in_expression(condition, names);
                }
                collect_variable_targets_in_weave(branch.content(), names);
            }
        }
        Object::Expression(expression) | Object::LogicLine(expression) => {
            collect_variable_targets_in_expression(expression, names);
        }
        Object::IncDec(inc_dec) => {
            collect_variable_targets_in_expression(inc_dec.expression(), names)
        }
        Object::Return(ret) => {
            if let Some(expression) = ret.returned_expression() {
                collect_variable_targets_in_expression(expression, names);
            }
        }
        Object::Sequence(sequence) => {
            for content in sequence.elements() {
                collect_variable_targets_in_content_list(content, names);
            }
        }
        Object::Weave(weave) => collect_variable_targets_in_weave(weave, names),
        Object::AuthorWarning(_)
        | Object::ConstantDeclaration(_)
        | Object::Divert(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::TunnelOnwards(_) => {}
    }
}

fn collect_variable_targets_in_expression(expression: &Expression, names: &mut HashSet<String>) {
    match expression {
        Expression::StringContent(content) => {
            collect_variable_targets_in_content_list(content, names)
        }
        Expression::FunctionCall { args, .. } => {
            for arg in args {
                collect_variable_targets_in_expression(arg, names);
            }
        }
        Expression::Binary { left, right, .. } => {
            collect_variable_targets_in_expression(left, names);
            collect_variable_targets_in_expression(right, names);
        }
        Expression::Unary { expression, .. } => {
            collect_variable_targets_in_expression(expression, names)
        }
        Expression::MultipleCondition(expressions) => {
            for expression in expressions {
                collect_variable_targets_in_expression(expression, names);
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

fn check_call_targets_in_flow(
    flow: &Flow,
    parent_path: Option<&str>,
    target_symbols: &HashMap<String, FlowSymbol>,
    variable_targets: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let flow_path = parent_path
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    check_call_targets_in_weave(
        flow.weave(),
        target_symbols,
        variable_targets,
        diagnostics,
        Some(&flow_path),
        Some(flow),
        flow.is_function(),
    );
    for child in flow.child_flows() {
        check_call_targets_in_flow(
            child,
            Some(&flow_path),
            target_symbols,
            variable_targets,
            diagnostics,
        );
    }
}

fn check_call_targets_in_weave(
    weave: &Weave,
    target_symbols: &HashMap<String, FlowSymbol>,
    variable_targets: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
    current_flow_path: Option<&str>,
    current_flow: Option<&Flow>,
    inside_function: bool,
) {
    for object in weave.content() {
        check_call_targets_in_object(
            object,
            target_symbols,
            variable_targets,
            diagnostics,
            current_flow_path,
            current_flow,
            inside_function,
        );
    }
}

fn check_call_targets_in_content_list(
    content: &ContentList,
    target_symbols: &HashMap<String, FlowSymbol>,
    variable_targets: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
    current_flow_path: Option<&str>,
    current_flow: Option<&Flow>,
    inside_function: bool,
) {
    for object in content.objects() {
        check_call_targets_in_object(
            object,
            target_symbols,
            variable_targets,
            diagnostics,
            current_flow_path,
            current_flow,
            inside_function,
        );
    }
}

fn check_call_targets_in_object(
    object: &Object,
    target_symbols: &HashMap<String, FlowSymbol>,
    variable_targets: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
    current_flow_path: Option<&str>,
    current_flow: Option<&Flow>,
    inside_function: bool,
) {
    match object {
        Object::Divert(divert) => {
            match divert.target() {
                DivertTarget::Empty => diagnostics.push(Diagnostic::error(
                    divert.span().clone(),
                    "Empty diverts (->) are only valid on choices",
                )),
                DivertTarget::Path(target) if !inside_function => {
                    check_plain_divert_target(
                        target,
                        divert.span(),
                        target_symbols,
                        variable_targets,
                        diagnostics,
                        current_flow_path,
                        current_flow,
                    );
                }
                DivertTarget::Path(_) | DivertTarget::Done | DivertTarget::End => {}
            }
            for argument in divert.arguments() {
                check_call_targets_in_expression(
                    argument,
                    divert.span(),
                    target_symbols,
                    variable_targets,
                    diagnostics,
                    current_flow_path,
                    current_flow,
                    inside_function,
                );
            }
        }
        Object::Expression(expression) | Object::LogicLine(expression) => {
            check_call_targets_in_expression(
                expression,
                &object_span(object),
                target_symbols,
                variable_targets,
                diagnostics,
                current_flow_path,
                current_flow,
                inside_function,
            );
        }
        Object::VariableAssignment(assignment) => check_call_targets_in_expression(
            assignment.expression(),
            assignment.span(),
            target_symbols,
            variable_targets,
            diagnostics,
            current_flow_path,
            current_flow,
            inside_function,
        ),
        Object::IncDec(inc_dec) => check_call_targets_in_expression(
            inc_dec.expression(),
            inc_dec.span(),
            target_symbols,
            variable_targets,
            diagnostics,
            current_flow_path,
            current_flow,
            inside_function,
        ),
        Object::Return(ret) => {
            if let Some(expression) = ret.returned_expression() {
                check_call_targets_in_expression(
                    expression,
                    ret.span(),
                    target_symbols,
                    variable_targets,
                    diagnostics,
                    current_flow_path,
                    current_flow,
                    inside_function,
                );
            }
        }
        Object::ContentList(content) => check_call_targets_in_content_list(
            content,
            target_symbols,
            variable_targets,
            diagnostics,
            current_flow_path,
            current_flow,
            inside_function,
        ),
        Object::Conditional(conditional) => {
            if let Some(condition) = conditional.initial_condition() {
                check_call_targets_in_expression(
                    condition,
                    &object_span(object),
                    target_symbols,
                    variable_targets,
                    diagnostics,
                    current_flow_path,
                    current_flow,
                    inside_function,
                );
            }
            for branch in conditional.branches() {
                if let Some(condition) = branch.own_condition() {
                    check_call_targets_in_expression(
                        condition,
                        &object_span(object),
                        target_symbols,
                        variable_targets,
                        diagnostics,
                        current_flow_path,
                        current_flow,
                        inside_function,
                    );
                }
                check_call_targets_in_weave(
                    branch.content(),
                    target_symbols,
                    variable_targets,
                    diagnostics,
                    current_flow_path,
                    current_flow,
                    inside_function,
                );
            }
        }
        Object::Choice(choice) => {
            if let Some(condition) = choice.condition() {
                check_call_targets_in_expression(
                    condition,
                    choice.span(),
                    target_symbols,
                    variable_targets,
                    diagnostics,
                    current_flow_path,
                    current_flow,
                    inside_function,
                );
            }
            if let Some(content) = choice.start_content() {
                check_call_targets_in_content_list(
                    content,
                    target_symbols,
                    variable_targets,
                    diagnostics,
                    current_flow_path,
                    current_flow,
                    inside_function,
                );
            }
            if let Some(content) = choice.choice_only_content() {
                check_call_targets_in_content_list(
                    content,
                    target_symbols,
                    variable_targets,
                    diagnostics,
                    current_flow_path,
                    current_flow,
                    inside_function,
                );
            }
            check_call_targets_in_content_list(
                choice.inner_content(),
                target_symbols,
                variable_targets,
                diagnostics,
                current_flow_path,
                current_flow,
                inside_function,
            );
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                check_call_targets_in_content_list(
                    element,
                    target_symbols,
                    variable_targets,
                    diagnostics,
                    current_flow_path,
                    current_flow,
                    inside_function,
                );
            }
        }
        Object::Weave(weave) => check_call_targets_in_weave(
            weave,
            target_symbols,
            variable_targets,
            diagnostics,
            current_flow_path,
            current_flow,
            inside_function,
        ),
        Object::AuthorWarning(_)
        | Object::ConstantDeclaration(_)
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
    target_symbols: &HashMap<String, FlowSymbol>,
    variable_targets: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
    current_flow_path: Option<&str>,
    current_flow: Option<&Flow>,
) {
    if let Some(symbol) = resolve_target_symbol(target, current_flow_path, target_symbols) {
        if symbol.is_function {
            diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "{target} can't be diverted to. It can only be called as a function since it's been marked as such: '{target}(...)'"
                ),
            ));
        }
    } else if let Some(argument) = resolve_current_flow_argument(target, current_flow) {
        if !argument.is_divert_target() {
            diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Since '{}' is used as a variable divert target, it should be marked as: -> {}",
                    argument.name(),
                    argument.name()
                ),
            ));
        }
    } else if !variable_targets.contains(target) {
        diagnostics.push(Diagnostic::error(
            span.clone(),
            format!("target not found: '{target}'"),
        ));
    }
}

fn resolve_current_flow_argument<'a>(
    target: &str,
    current_flow: Option<&'a Flow>,
) -> Option<&'a FlowArgument> {
    let variable_target_name = target.split('.').next()?;
    current_flow?
        .arguments()
        .iter()
        .find(|argument| argument.name() == variable_target_name)
}

fn resolve_target_symbol<'a>(
    target: &str,
    current_flow_path: Option<&str>,
    target_symbols: &'a HashMap<String, FlowSymbol>,
) -> Option<&'a FlowSymbol> {
    if target.contains('.') {
        return target_symbols.get(target);
    }

    if let Some(flow_path) = current_flow_path {
        if let Some(symbol) = target_symbols.get(&format!("{flow_path}.{target}")) {
            return Some(symbol);
        }
        if let Some((parent_flow_path, _)) = flow_path.rsplit_once('.') {
            if let Some(symbol) = target_symbols.get(&format!("{parent_flow_path}.{target}")) {
                return Some(symbol);
            }
        }
    }

    target_symbols.get(target)
}

fn check_call_targets_in_expression(
    expression: &Expression,
    span: &SourceSpan,
    target_symbols: &HashMap<String, FlowSymbol>,
    variable_targets: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
    current_flow_path: Option<&str>,
    current_flow: Option<&Flow>,
    inside_function: bool,
) {
    match expression {
        Expression::FunctionCall { name, args } => {
            if let Some(symbol) = resolve_target_symbol(name, current_flow_path, target_symbols) {
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
                    target_symbols,
                    variable_targets,
                    diagnostics,
                    current_flow_path,
                    current_flow,
                    inside_function,
                );
            }
        }
        Expression::StringContent(content) => check_call_targets_in_content_list(
            content,
            target_symbols,
            variable_targets,
            diagnostics,
            current_flow_path,
            current_flow,
            inside_function,
        ),
        Expression::Binary { left, right, .. } => {
            check_call_targets_in_expression(
                left,
                span,
                target_symbols,
                variable_targets,
                diagnostics,
                current_flow_path,
                current_flow,
                inside_function,
            );
            check_call_targets_in_expression(
                right,
                span,
                target_symbols,
                variable_targets,
                diagnostics,
                current_flow_path,
                current_flow,
                inside_function,
            );
        }
        Expression::Unary { expression, .. } => {
            check_call_targets_in_expression(
                expression,
                span,
                target_symbols,
                variable_targets,
                diagnostics,
                current_flow_path,
                current_flow,
                inside_function,
            );
        }
        Expression::MultipleCondition(expressions) => {
            for expression in expressions {
                check_call_targets_in_expression(
                    expression,
                    span,
                    target_symbols,
                    variable_targets,
                    diagnostics,
                    current_flow_path,
                    current_flow,
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
    check_nested_choice_termination_in_weave(flow.weave(), false, diagnostics);
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
                        "Choices nested in conditionals or sequences need to explicitly divert afterwards.",
                    ));
                }
                if let Some(content) = choice.start_content() {
                    check_nested_choice_termination_in_content_list(
                        content,
                        inside_sealed_content,
                        diagnostics,
                    );
                }
                if let Some(content) = choice.choice_only_content() {
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
            Object::Sequence(sequence) => {
                for element in sequence.elements() {
                    check_nested_choice_termination_in_content_list(element, true, diagnostics);
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
            Object::Sequence(sequence) => {
                for element in sequence.elements() {
                    check_nested_choice_termination_in_content_list(element, true, diagnostics);
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
            | Object::Tag(_)
            | Object::Text(_)
            | Object::TunnelOnwards(_)
            | Object::VariableAssignment(_) => {}
        }
    }
}

fn choice_flow_terminates(choice: &crate::parsed::Choice, following: &[Object]) -> bool {
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
        Object::AuthorWarning(_)
        | Object::ConstantDeclaration(_)
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
        Object::AuthorWarning(_)
        | Object::ConstantDeclaration(_)
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
        || matches!(object, Object::AuthorWarning(_))
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
        Object::AuthorWarning(_)
        | Object::ConstantDeclaration(_)
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
        Object::AuthorWarning(author_warning) => author_warning.span().clone(),
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
