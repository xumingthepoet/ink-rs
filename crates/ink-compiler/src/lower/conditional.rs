use std::collections::{HashMap, HashSet};

use crate::parsed::{Conditional, Expression};

use super::context::ChoicePathMode;
use super::expression::lower_expression_into;
use super::indexes::ExternalSignatures;
use super::ir::{Container, ControlCommand, RuntimeObject};
use super::lower_object_into_with_context;
use super::weave::{lower_choice_weave_with_initial_content, weave_has_choice};

pub(super) fn lower_conditional_into(
    content: &mut Vec<RuntimeObject>,
    conditional: &Conditional,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
) {
    if let Some(condition) = conditional.initial_condition() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
        lower_expression_into(
            content,
            condition,
            choice_labels,
            global_labels,
            external_signatures,
            constants,
            path_mode,
            false,
        );
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    }

    let has_initial_condition = conditional.initial_condition().is_some();
    let switch_like = has_initial_condition
        && conditional
            .branches()
            .iter()
            .any(|branch| branch.own_condition().is_some());
    let needs_fallthrough_pop = switch_like
        && !conditional
            .branches()
            .last()
            .is_some_and(|branch| branch.is_else());
    let rejoin_index =
        content.len() + conditional.branches().len() + usize::from(needs_fallthrough_pop);
    let rejoin_target = path_mode.runtime_index_path(rejoin_index);
    let branch_rejoin_target = rejoin_target;

    for branch in conditional.branches() {
        let branch_path_mode = path_mode.for_conditional_branch(content.len());
        let mut branch_content = Vec::new();
        let duplicates_stack_value = switch_like && !branch.is_else();
        if duplicates_stack_value {
            branch_content.push(RuntimeObject::ControlCommand(ControlCommand::Duplicate));
        }

        if !branch.is_true_branch() && !branch.is_else() {
            let needs_eval = branch.own_condition().is_some();
            if needs_eval {
                branch_content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
            }
            if let Some(condition) = branch.own_condition() {
                lower_expression_into(
                    &mut branch_content,
                    condition,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    false,
                );
            }
            if switch_like {
                branch_content.push(RuntimeObject::NativeFunction("==".to_string()));
            }
            if needs_eval {
                branch_content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
            }
        }

        if branch.is_else() {
            branch_content.push(RuntimeObject::Divert {
                target: ".^.b".to_string(),
                variable: false,
            });
        } else {
            branch_content.push(RuntimeObject::ConditionalDivert {
                target: ".^.b".to_string(),
            });
        }

        if weave_has_choice(branch.content()) {
            let initial_content = if branch.is_inline() {
                if duplicates_stack_value || (branch.is_else() && switch_like) {
                    vec![RuntimeObject::ControlCommand(ControlCommand::Pop)]
                } else {
                    Vec::new()
                }
            } else {
                let mut initial_content = Vec::new();
                if duplicates_stack_value || (branch.is_else() && switch_like) {
                    initial_content.push(RuntimeObject::ControlCommand(ControlCommand::Pop));
                }
                initial_content.push(RuntimeObject::String("\n".to_string()));
                initial_content
            };
            let mut content_container = Vec::new();
            let mut lowered_branch = lower_choice_weave_with_initial_content(
                branch.content(),
                branch_path_mode,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                false,
                initial_content,
            );
            let trailing_named_content =
                if matches!(lowered_branch.last(), Some(RuntimeObject::NamedContent(_))) {
                    lowered_branch.pop()
                } else {
                    None
                };
            content_container.extend(lowered_branch);
            content_container.push(RuntimeObject::Divert {
                target: branch_rejoin_target.clone(),
                variable: false,
            });
            if let Some(named_content) = trailing_named_content {
                content_container.push(named_content);
            }
            branch_content.push(RuntimeObject::NamedContent(vec![Container {
                content: content_container,
                name: Some("b".to_string()),
                flags: None,
                merge_tail_metadata: true,
            }]));
        } else {
            let mut content_container = Vec::new();
            if duplicates_stack_value || (branch.is_else() && switch_like) {
                content_container.push(RuntimeObject::ControlCommand(ControlCommand::Pop));
            }
            if !branch.is_inline() {
                content_container.push(RuntimeObject::String("\n".to_string()));
            }
            for object in branch.content().content() {
                lower_object_into_with_context(
                    &mut content_container,
                    object,
                    &branch_path_mode,
                    choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                );
            }
            content_container.push(RuntimeObject::Divert {
                target: branch_rejoin_target.clone(),
                variable: false,
            });
            branch_content.push(RuntimeObject::NamedContent(vec![Container {
                content: content_container,
                name: Some("b".to_string()),
                flags: None,
                merge_tail_metadata: true,
            }]));
        }

        content.push(RuntimeObject::Container(Container {
            content: branch_content,
            name: None,
            flags: None,
            merge_tail_metadata: true,
        }));
    }

    if needs_fallthrough_pop {
        content.push(RuntimeObject::ControlCommand(ControlCommand::Pop));
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::NoOp));
}
