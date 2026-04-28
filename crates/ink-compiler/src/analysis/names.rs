use std::collections::{HashMap, HashSet};

use crate::{
    diagnostic::Diagnostic,
    parsed::{ContentList, Flow, Object, Story, Weave},
    source::SourceSpan,
};

use super::span::first_span_in_weave;

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

pub(super) fn naming_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let root_top_level_flows = top_level_flow_kinds(story.flows());
    let root_global_variables = scoped_global_variables(story.root_weave(), story.flows());

    check_weave_point_names(story.root_weave(), &root_global_variables, &mut diagnostics);
    for flow in story.flows() {
        check_subflow_and_weave_names(flow, &root_global_variables, &mut diagnostics);
        check_flow_arguments(
            flow,
            &root_top_level_flows,
            &root_global_variables,
            &mut diagnostics,
        );
        check_temporary_names_against_arguments(flow, &mut diagnostics);
    }

    for module in story.modules() {
        let module_top_level_flows = top_level_flow_kinds(module.flows());
        let module_global_variables = scoped_global_variables(module.weave(), module.flows());

        check_weave_point_names(module.weave(), &module_global_variables, &mut diagnostics);
        for flow in module.flows() {
            check_subflow_and_weave_names(flow, &module_global_variables, &mut diagnostics);
            check_flow_arguments(
                flow,
                &module_top_level_flows,
                &module_global_variables,
                &mut diagnostics,
            );
            check_temporary_names_against_arguments(flow, &mut diagnostics);
        }
    }

    diagnostics
}

fn top_level_flow_kinds(flows: &[Flow]) -> HashMap<String, SymbolKind> {
    flows
        .iter()
        .map(|flow| {
            let kind = if flow.is_function() {
                SymbolKind::Function
            } else {
                SymbolKind::Knot
            };
            (flow.name().to_string(), kind)
        })
        .collect()
}

fn scoped_global_variables(weave: &Weave, flows: &[Flow]) -> HashSet<String> {
    let mut global_variables = HashSet::new();
    collect_global_variables(weave, &mut global_variables);
    for flow in flows {
        collect_global_variables_in_flow(flow, &mut global_variables);
    }
    global_variables
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
        | Object::StructDeclaration(_)
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
        | Object::StructDeclaration(_)
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
            | Object::StructDeclaration(_)
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
            | Object::StructDeclaration(_)
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
    span: SourceSpan,
) -> Diagnostic {
    Diagnostic::error(
        span,
        format!(
            "{symbol_type} '{name}': name has already been used for a {}",
            existing_kind.display_name()
        ),
    )
}

#[cfg(test)]
mod tests {
    use crate::diagnostic::DiagnosticSeverity;

    use super::{
        super::test_support::{assert_single_diagnostic, parse_story},
        *,
    };

    #[test]
    fn reports_duplicate_flow_arguments() {
        let story = parse_story("== knot(a, a) ==\n-> DONE");
        let diagnostics = naming_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Multiple arguments with the same name: 'a'",
        );
    }

    #[test]
    fn reports_module_temporary_names_conflicting_with_arguments() {
        let story = parse_story(
            "=== module game ===\n\
             == main(arg: int) ==\n\
             ~ temp arg: int = 0\n\
             -> DONE",
        );
        let diagnostics = naming_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "temp 'arg': name has already been used for an argument to main",
        );
    }

    #[test]
    fn reports_module_argument_names_conflicting_with_module_globals() {
        let story = parse_story(
            "=== module game ===\n\
             VAR score: int = 0\n\
             == main(score: int) ==\n\
             -> DONE",
        );
        let diagnostics = naming_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "argument 'score': name has already been used for a var",
        );
    }
}
