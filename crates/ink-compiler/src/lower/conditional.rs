use std::collections::HashSet;

use ink_story_json_format::{Container, ControlCommand, Object as RuntimeObject};

use crate::parsed::Conditional;

use super::context::ChoicePathMode;
use super::expression::lower_expression_into;
use super::indexes::{ConstantValues, ExternalSignatures, StructDefinitions};
use super::lower_object_into_with_context;
use super::path::LabelIndex;
use super::weave::{lower_choice_weave_with_initial_content, weave_has_choice};
use super::{named_container, named_content};

pub(super) fn lower_conditional_into(
    content: &mut Vec<RuntimeObject>,
    conditional: &Conditional,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    path_mode: &ChoicePathMode,
) {
    if let Some(condition) = conditional.initial_condition() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
        lower_expression_into(
            content,
            condition,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
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
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
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
            let lowered_branch = lower_choice_weave_with_initial_content(
                branch.content(),
                branch_path_mode,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
                false,
                initial_content,
            );
            let mut content_container = lowered_branch.content;
            let content_named_content = lowered_branch.named_content;
            content_container.push(RuntimeObject::Divert {
                target: branch_rejoin_target.clone(),
                variable: false,
            });
            let mut branch_container = Container::unnamed(branch_content);
            branch_container
                .named_content
                .push(named_container(Container {
                    content: content_container,
                    named_content: content_named_content,
                    name: Some("b".to_string()),
                    flags: None,
                }));
            content.push(RuntimeObject::Container(branch_container));
            continue;
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
                    struct_definitions,
                );
            }
            content_container.push(RuntimeObject::Divert {
                target: branch_rejoin_target.clone(),
                variable: false,
            });
            let mut branch_container = Container::unnamed(branch_content);
            branch_container
                .named_content
                .push(named_content("b", content_container));
            content.push(RuntimeObject::Container(branch_container));
            continue;
        }
    }

    if needs_fallthrough_pop {
        content.push(RuntimeObject::ControlCommand(ControlCommand::Pop));
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::NoOp));
}
