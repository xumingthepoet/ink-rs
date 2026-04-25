use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_weave, ParsedVisitor, VisitContext},
        Choice, ContentList, DivertTarget, Flow, FlowLevel, Object, Return, Story, Weave,
    },
    source::SourceSpan,
};

use super::span::{first_span_in_weave, object_span};

pub(super) fn flow_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    check_nested_choice_termination_in_weave(story.root_weave(), false, &mut diagnostics);
    for flow in story.flows() {
        check_flow(flow, &mut diagnostics);
    }
    diagnostics
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

    let mut visitor = FunctionFlowControlVisitor { diagnostics };
    walk_weave(flow.weave(), &mut visitor, &VisitContext::default());
}

struct FunctionFlowControlVisitor<'a> {
    diagnostics: &'a mut Vec<Diagnostic>,
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
            _ => {}
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
