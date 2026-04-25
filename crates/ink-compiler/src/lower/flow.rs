use std::collections::{HashMap, HashSet};

use crate::parsed::{ContentList, Expression, Flow, Object, Weave};

use super::context::ChoicePathMode;
use super::indexes::{CountedFlowPaths, ExternalSignatures, LoweringIndexes};
use super::ir::{Container, RuntimeObject};
use super::path::LabelIndex;
use super::weave::{
    lower_choice_weave, lower_linear_weave, lower_linear_weave_into_context, weave_has_choice,
    weave_has_weave_points,
};
use super::{done_container, ends_with_flow_terminator};

pub(super) fn lower_root_weave(
    weave: &Weave,
    indexes: &LoweringIndexes<'_>,
    count_all_visits: bool,
) -> Vec<RuntimeObject> {
    if weave_has_weave_points(weave) {
        lower_choice_weave(
            weave,
            ChoicePathMode::Root,
            &indexes.global_labels,
            &indexes.global_variables,
            &indexes.external_signatures,
            &indexes.constants,
            count_all_visits,
        )
    } else {
        let mut content = lower_linear_weave(
            weave,
            &indexes.global_labels,
            &indexes.global_variables,
            &indexes.external_signatures,
            &indexes.constants,
        );
        content.push(RuntimeObject::Container(done_container(
            "g-0",
            count_all_visits,
        )));
        content
    }
}

pub(super) fn lower_flow(
    flow: &Flow,
    indexes: &LoweringIndexes<'_>,
    count_all_visits: bool,
) -> Container {
    let child_stitch_names: Vec<String> = flow
        .child_flows()
        .iter()
        .map(|f| f.name().to_string())
        .collect();
    lower_flow_with_context(
        flow,
        None,
        &child_stitch_names,
        &indexes.global_labels,
        &indexes.global_variables,
        &indexes.external_signatures,
        &indexes.constants,
        &indexes.counted_flow_paths,
        count_all_visits,
    )
}

fn lower_flow_with_context(
    flow: &Flow,
    parent_knot_name: Option<&str>,
    sibling_stitch_names: &[String],
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    counted_flow_paths: &CountedFlowPaths,
    count_all_visits: bool,
) -> Container {
    let mut content = Vec::new();
    let flow_path = parent_knot_name
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    let local_variables = collect_flow_local_variables(flow);

    lower_flow_arguments_into(&mut content, flow);

    if weave_has_weave_points(flow.weave()) {
        let flow_container_path = parent_knot_name
            .map(|parent| format!("{parent}.{}.{}", flow.name(), content.len()))
            .unwrap_or_else(|| format!("{}.{}", flow.name(), content.len()));
        let path_mode = ChoicePathMode::Flow {
            flow_name: flow.name().to_string(),
            container_path: flow_container_path,
            parent_flow_name: parent_knot_name.map(|s| s.to_string()),
            sibling_stitch_names: sibling_stitch_names.to_vec(),
            local_variables: local_variables.clone(),
            self_target_relative: false,
            fallback_gather_target: None,
        };
        content.push(RuntimeObject::Container(Container {
            content: lower_choice_weave(
                flow.weave(),
                path_mode,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                count_all_visits,
            ),
            name: None,
            flags: None,
            merge_tail_metadata: true,
        }));
    } else if !flow.weave().content().is_empty() {
        let path_mode = ChoicePathMode::Flow {
            flow_name: flow.name().to_string(),
            container_path: flow_path.clone(),
            parent_flow_name: parent_knot_name.map(str::to_string),
            sibling_stitch_names: sibling_stitch_names.to_vec(),
            local_variables,
            self_target_relative: false,
            fallback_gather_target: None,
        };
        lower_linear_weave_into_context(
            &mut content,
            flow.weave(),
            global_labels,
            global_variables,
            external_signatures,
            constants,
            &path_mode,
        );
    }

    if !flow.child_flows().is_empty() {
        if !weave_has_choice(flow.weave()) && !ends_with_flow_terminator(&content) {
            let first_child_name = flow.child_flows()[0].name();
            content.push(RuntimeObject::Divert {
                target: format!(".^.{}", first_child_name),
                variable: false,
            });
        }

        let child_stitch_names: Vec<String> = flow
            .child_flows()
            .iter()
            .map(|f| f.name().to_string())
            .collect();

        let child_containers: Vec<Container> = flow
            .child_flows()
            .iter()
            .map(|child| {
                lower_flow_with_context(
                    child,
                    Some(flow.name()),
                    &child_stitch_names,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    counted_flow_paths,
                    count_all_visits,
                )
            })
            .collect();
        content.push(RuntimeObject::NamedContent(child_containers));
    }

    Container {
        content,
        name: Some(flow.name().to_string()),
        flags: flow_container_flags(
            counted_flow_paths.turns.contains(&flow_path),
            counted_flow_paths.visits.contains(&flow_path),
            count_all_visits,
        ),
        merge_tail_metadata: true,
    }
}

fn flow_container_flags(
    count_turns: bool,
    count_visits: bool,
    count_all_visits: bool,
) -> Option<i32> {
    match (count_turns, count_visits || count_all_visits) {
        (true, _) => Some(3),
        (false, true) => Some(1),
        (false, false) => None,
    }
}

fn lower_flow_arguments_into(content: &mut Vec<RuntimeObject>, flow: &Flow) {
    for argument in flow.arguments().iter().rev() {
        content.push(RuntimeObject::VariableAssignment(
            argument.name().to_string(),
        ));
    }
}

pub(super) fn collect_flow_local_variables(flow: &Flow) -> HashSet<String> {
    let mut local_variables = flow
        .arguments()
        .iter()
        .map(|argument| argument.name().to_string())
        .collect::<HashSet<_>>();
    collect_local_variables_in_weave(flow.weave(), &mut local_variables);
    local_variables
}

fn collect_local_variables_in_weave(weave: &Weave, local_variables: &mut HashSet<String>) {
    for object in weave.content() {
        collect_local_variables_in_object(object, local_variables);
    }
}

fn collect_local_variables_in_content_list(
    content_list: &ContentList,
    local_variables: &mut HashSet<String>,
) {
    for object in content_list.objects() {
        collect_local_variables_in_object(object, local_variables);
    }
}

fn collect_local_variables_in_object(object: &Object, local_variables: &mut HashSet<String>) {
    match object {
        Object::VariableAssignment(assignment) if assignment.is_temporary() => {
            local_variables.insert(assignment.name().to_string());
        }
        Object::ContentList(content_list) => {
            collect_local_variables_in_content_list(content_list, local_variables);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_local_variables_in_weave(branch.content(), local_variables);
            }
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_local_variables_in_content_list(content, local_variables);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_local_variables_in_content_list(content, local_variables);
            }
            collect_local_variables_in_content_list(choice.inner_content(), local_variables);
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_local_variables_in_content_list(element, local_variables);
            }
        }
        Object::Weave(weave) => collect_local_variables_in_weave(weave, local_variables),
        _ => {}
    }
}
