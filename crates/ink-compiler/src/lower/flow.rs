use std::collections::HashSet;

use ink_story_json_format::{Container, Object as RuntimeObject};

use crate::parsed::{ContentList, Flow, Object, Weave};

use super::context::{ChoicePathMode, LoweringContext};
use super::indexes::LoweringIndexes;
use super::path::LabelIndex;
use super::weave::{
    lower_choice_weave, lower_linear_weave_into_context, weave_has_choice, weave_has_weave_points,
};
use super::{ends_with_flow_terminator, named_container};

pub(super) fn lower_module_flow(
    module_name: &str,
    flow: &Flow,
    indexes: &LoweringIndexes<'_>,
) -> Container {
    lower_flow_in_module(Some(module_name), flow, indexes)
}

fn lower_flow_in_module(
    module_name: Option<&str>,
    flow: &Flow,
    indexes: &LoweringIndexes<'_>,
) -> Container {
    let child_stitch_names: Vec<String> = flow
        .child_flows()
        .iter()
        .map(|f| f.name().to_string())
        .collect();
    let context = FlowLoweringContext {
        module_name,
        parent_knot_name: None,
        sibling_stitch_names: &child_stitch_names,
        indexes,
    };
    lower_flow_with_context(flow, &context)
}

struct FlowLoweringContext<'a, 'idx> {
    module_name: Option<&'a str>,
    parent_knot_name: Option<&'a str>,
    sibling_stitch_names: &'a [String],
    indexes: &'a LoweringIndexes<'idx>,
}

fn lower_flow_with_context(flow: &Flow, context: &FlowLoweringContext<'_, '_>) -> Container {
    let mut content = Vec::new();
    let source_flow_path = context
        .parent_knot_name
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    let flow_path = context
        .module_name
        .map(|module| format!("{module}.{source_flow_path}"))
        .unwrap_or_else(|| source_flow_path.clone());
    let local_variables = collect_flow_local_variables(flow);

    lower_flow_arguments_into(&mut content, flow);

    if weave_has_weave_points(flow.weave()) {
        let flow_container_path = context
            .parent_knot_name
            .map(|parent| {
                context
                    .module_name
                    .map(|module| format!("{module}.{parent}.{}.{}", flow.name(), content.len()))
                    .unwrap_or_else(|| format!("{parent}.{}.{}", flow.name(), content.len()))
            })
            .unwrap_or_else(|| {
                context
                    .module_name
                    .map(|module| format!("{module}.{}.{}", flow.name(), content.len()))
                    .unwrap_or_else(|| format!("{}.{}", flow.name(), content.len()))
            });
        let path_mode = ChoicePathMode::Flow {
            module_name: context.module_name.map(str::to_string),
            flow_name: flow.name().to_string(),
            container_path: flow_container_path,
            parent_flow_name: context.parent_knot_name.map(|s| s.to_string()),
            sibling_stitch_names: context.sibling_stitch_names.to_vec(),
            local_variables: local_variables.clone(),
            self_target_relative: false,
            fallback_gather_target: None,
        };
        let choice_labels = LabelIndex::new();
        let lowering_context = LoweringContext::new(
            path_mode,
            &choice_labels,
            &context.indexes.global_labels,
            &context.indexes.global_variables,
            &context.indexes.external_signatures,
            &context.indexes.constants,
            &context.indexes.struct_definitions,
        );
        content.push(RuntimeObject::Container(lower_choice_weave(
            flow.weave(),
            &lowering_context,
        )));
    } else if !flow.weave().content().is_empty() {
        let path_mode = ChoicePathMode::Flow {
            module_name: context.module_name.map(str::to_string),
            flow_name: flow.name().to_string(),
            container_path: flow_path.clone(),
            parent_flow_name: context.parent_knot_name.map(str::to_string),
            sibling_stitch_names: context.sibling_stitch_names.to_vec(),
            local_variables,
            self_target_relative: false,
            fallback_gather_target: None,
        };
        let choice_labels = LabelIndex::new();
        let lowering_context = LoweringContext::new(
            path_mode,
            &choice_labels,
            &context.indexes.global_labels,
            &context.indexes.global_variables,
            &context.indexes.external_signatures,
            &context.indexes.constants,
            &context.indexes.struct_definitions,
        );
        lower_linear_weave_into_context(&mut content, flow.weave(), &lowering_context);
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

        let child_containers = flow
            .child_flows()
            .iter()
            .map(|child| {
                let child_context = FlowLoweringContext {
                    module_name: context.module_name,
                    parent_knot_name: Some(flow.name()),
                    sibling_stitch_names: &child_stitch_names,
                    indexes: context.indexes,
                };
                named_container(lower_flow_with_context(child, &child_context))
            })
            .collect();

        return Container {
            content,
            named_content: child_containers,
            name: Some(flow.name().to_string()),
            flags: None,
        };
    }

    Container {
        content,
        named_content: Vec::new(),
        name: Some(flow.name().to_string()),
        flags: None,
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
