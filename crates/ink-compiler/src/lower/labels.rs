use std::collections::HashSet;

use crate::parsed::{Flow, Object, Story, Weave};

use super::indexes::{ConstantValues, RuntimeLenEstimator, StructDefinitions};
use super::path::{child_path, LabelIndex, RuntimePath};
use super::weave::weave_has_weave_points;

pub(super) fn build_label_index(
    story: &Story,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    global_variables: &HashSet<String>,
    estimator: &RuntimeLenEstimator,
) -> LabelIndex {
    let mut labels = LabelIndex::new();
    collect_weave_labels(
        story.root_weave(),
        "0",
        None,
        constants,
        struct_definitions,
        global_variables,
        estimator,
        &mut labels,
    );
    for flow in story.flows() {
        collect_flow_labels(
            flow,
            None,
            None,
            constants,
            struct_definitions,
            global_variables,
            estimator,
            &mut labels,
        );
    }
    for module in story.modules() {
        for flow in module.flows() {
            collect_flow_labels(
                flow,
                Some(module.name()),
                None,
                constants,
                struct_definitions,
                global_variables,
                estimator,
                &mut labels,
            );
        }
    }
    labels
}

#[expect(
    clippy::too_many_arguments,
    reason = "label collection still passes explicit recursion context"
)]
fn collect_flow_labels(
    flow: &Flow,
    module_name: Option<&str>,
    parent_flow_name: Option<&str>,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    global_variables: &HashSet<String>,
    estimator: &RuntimeLenEstimator,
    labels: &mut LabelIndex,
) {
    let source_flow_path = parent_flow_name
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    let flow_path = module_name
        .map(|module| format!("{module}.{source_flow_path}"))
        .unwrap_or_else(|| source_flow_path.clone());
    labels.insert(flow_path.clone(), RuntimePath::new(flow_path.clone()));
    if parent_flow_name.is_none() && module_name.is_none() {
        labels.insert(flow.name().to_string(), RuntimePath::new(flow_path.clone()));
    }
    let weave_container_path = if weave_has_weave_points(flow.weave()) {
        format!("{}.{}", flow_path, flow.arguments().len())
    } else {
        format!("{flow_path}.0")
    };
    collect_weave_labels(
        flow.weave(),
        &weave_container_path,
        Some(&flow_path),
        constants,
        struct_definitions,
        global_variables,
        estimator,
        labels,
    );
    for child in flow.child_flows() {
        collect_flow_labels(
            child,
            module_name,
            Some(&source_flow_path),
            constants,
            struct_definitions,
            global_variables,
            estimator,
            labels,
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "weave label collection still passes explicit recursion context"
)]
fn collect_weave_labels(
    weave: &Weave,
    container_path: &str,
    flow_alias_prefix: Option<&str>,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    global_variables: &HashSet<String>,
    estimator: &RuntimeLenEstimator,
    labels: &mut LabelIndex,
) {
    let mut choice_count = 0;
    let mut gather_count = 0;
    let mut current_container_path = container_path.to_string();
    let mut last_section_had_choice = false;
    let mut previous_choice_content: Option<(String, usize)> = None;
    for object in weave.content() {
        match object {
            Object::Choice(choice) => {
                if let Some(identifier) = choice.identifier() {
                    let target_path = format!("{current_container_path}.c-{choice_count}");
                    labels.insert_scoped_aliases(
                        identifier,
                        container_path,
                        flow_alias_prefix,
                        RuntimePath::new(target_path),
                    );
                }
                let choice_path = format!("{current_container_path}.c-{choice_count}");
                previous_choice_content = Some((
                    choice_path,
                    (estimator.choice_content_len)(
                        choice,
                        constants,
                        struct_definitions,
                        global_variables,
                    ),
                ));
                choice_count += 1;
                last_section_had_choice = true;
            }
            Object::Gather(gather) => {
                previous_choice_content = None;
                let gather_name = gather.identifier().map(str::to_string).unwrap_or_else(|| {
                    let name = format!("g-{gather_count}");
                    gather_count += 1;
                    name
                });
                let gather_path = if last_section_had_choice {
                    format!("{container_path}.{gather_name}")
                } else {
                    format!("{current_container_path}.{gather_name}")
                };
                if let Some(identifier) = gather.identifier() {
                    labels.insert_scoped_aliases(
                        identifier,
                        container_path,
                        flow_alias_prefix,
                        RuntimePath::new(gather_path.as_str()),
                    );
                }
                current_container_path = gather_path;
                last_section_had_choice = false;
            }
            Object::Weave(weave) => {
                if let Some((choice_path, next_index)) = previous_choice_content.as_mut() {
                    let nested_container_path = child_path(choice_path, &next_index.to_string());
                    collect_weave_labels(
                        weave,
                        &nested_container_path,
                        flow_alias_prefix,
                        constants,
                        struct_definitions,
                        global_variables,
                        estimator,
                        labels,
                    );
                    *next_index += 1;
                } else {
                    collect_weave_labels(
                        weave,
                        &current_container_path,
                        flow_alias_prefix,
                        constants,
                        struct_definitions,
                        global_variables,
                        estimator,
                        labels,
                    );
                }
            }
            _ => {
                if let Some((_, next_index)) = previous_choice_content.as_mut() {
                    *next_index += (estimator.object_len)(
                        object,
                        constants,
                        struct_definitions,
                        global_variables,
                    );
                }
            }
        }
    }
}
