use std::collections::{HashMap, HashSet};

use crate::{
    compiler::StageOutput,
    diagnostic::Diagnostic,
    parsed::{Flow, Object, Story, Weave},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedStory {
    pub parsed: Story,
}

pub(crate) fn analyze(parsed: Story) -> StageOutput<CheckedStory> {
    let diagnostics = naming_diagnostics(&parsed);
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
