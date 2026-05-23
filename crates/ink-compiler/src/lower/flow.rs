use std::collections::{HashMap, HashSet};

use ink_story_json_format::{Container, Object as RuntimeObject};

use crate::parsed::{ContentList, DictKeyType, Flow, Object, TypeName, Weave};

use super::context::{ChoicePathMode, LoweringContext};
use super::expression::infer_lowered_expression_type;
use super::indexes::LoweringIndexes;
use super::named_container;
use super::path::LabelIndex;
use super::weave::{lower_choice_weave, lower_linear_weave_into_context, weave_has_weave_points};

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
    let local_variable_types = collect_flow_local_variable_types(
        flow,
        context.module_name,
        &source_flow_path,
        context.indexes,
    );

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
            local_variable_types: local_variable_types.clone(),
            self_target_relative: false,
            fallback_gather_target: None,
        };
        let choice_labels = LabelIndex::new();
        let lowering_context = LoweringContext::new(
            path_mode,
            &choice_labels,
            &context.indexes.global_labels,
            &context.indexes.global_variables,
            &context.indexes.global_variable_types,
            &context.indexes.external_signatures,
            &context.indexes.interface_members,
            &context.indexes.constants,
            &context.indexes.struct_definitions,
            &context.indexes.enum_definitions,
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
            local_variable_types,
            self_target_relative: false,
            fallback_gather_target: None,
        };
        let choice_labels = LabelIndex::new();
        let lowering_context = LoweringContext::new(
            path_mode,
            &choice_labels,
            &context.indexes.global_labels,
            &context.indexes.global_variables,
            &context.indexes.global_variable_types,
            &context.indexes.external_signatures,
            &context.indexes.interface_members,
            &context.indexes.constants,
            &context.indexes.struct_definitions,
            &context.indexes.enum_definitions,
        );
        lower_linear_weave_into_context(&mut content, flow.weave(), &lowering_context);
    }

    if !flow.child_flows().is_empty() {
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
        };
    }

    Container {
        content,
        named_content: Vec::new(),
        name: Some(flow.name().to_string()),
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

fn collect_flow_local_variable_types(
    flow: &Flow,
    module_name: Option<&str>,
    source_flow_path: &str,
    indexes: &LoweringIndexes<'_>,
) -> HashMap<String, TypeName> {
    let mut local_variable_types = flow
        .arguments()
        .iter()
        .filter_map(|argument| {
            argument
                .declared_type()
                .cloned()
                .map(|declared_type| (argument.name().to_string(), declared_type))
        })
        .collect::<HashMap<_, _>>();
    collect_local_variable_types_in_weave(flow.weave(), &mut local_variable_types);
    collect_loop_variable_types_in_weave(
        flow.weave(),
        flow.name(),
        module_name,
        source_flow_path,
        indexes,
        &mut local_variable_types,
    );
    local_variable_types
}

fn collect_local_variables_in_weave(weave: &Weave, local_variables: &mut HashSet<String>) {
    for object in weave.content() {
        collect_local_variables_in_object(object, local_variables);
    }
}

fn collect_local_variable_types_in_weave(
    weave: &Weave,
    local_variable_types: &mut HashMap<String, TypeName>,
) {
    for object in weave.content() {
        collect_local_variable_types_in_object(object, local_variable_types);
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

fn collect_local_variable_types_in_content_list(
    content_list: &ContentList,
    local_variable_types: &mut HashMap<String, TypeName>,
) {
    for object in content_list.objects() {
        collect_local_variable_types_in_object(object, local_variable_types);
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
        Object::ForLoop(for_loop) => {
            local_variables.insert(for_loop.index_name());
            local_variables.insert(for_loop.limit_name());
            local_variables.insert(for_loop.keys_name());
            for variable in for_loop.variables() {
                local_variables.insert(variable.runtime_name().to_string());
            }
            collect_local_variables_in_weave(for_loop.body(), local_variables);
        }
        Object::Choice(choice) => {
            if let Some(binding) = choice.dynamic_binding() {
                local_variables.insert(binding.array_name().to_string());
                local_variables.insert(binding.index_name().to_string());
                local_variables.insert(binding.limit_name().to_string());
                for variable in binding.variables() {
                    local_variables.insert(variable.runtime_name().to_string());
                }
            }
            if let Some(content) = choice.start_content() {
                collect_local_variables_in_content_list(content, local_variables);
            }
            collect_local_variables_in_content_list(choice.inner_content(), local_variables);
        }
        Object::Weave(weave) => collect_local_variables_in_weave(weave, local_variables),
        _ => {}
    }
}

fn collect_local_variable_types_in_object(
    object: &Object,
    local_variable_types: &mut HashMap<String, TypeName>,
) {
    match object {
        Object::VariableAssignment(assignment) if assignment.is_temporary() => {
            if let Some(declared_type) = assignment.declared_type() {
                local_variable_types.insert(assignment.name().to_string(), declared_type.clone());
            }
        }
        Object::ContentList(content_list) => {
            collect_local_variable_types_in_content_list(content_list, local_variable_types);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_local_variable_types_in_weave(branch.content(), local_variable_types);
            }
        }
        Object::ForLoop(for_loop) => {
            local_variable_types.insert(for_loop.index_name(), TypeName::int());
            local_variable_types.insert(for_loop.limit_name(), TypeName::int());
            collect_local_variable_types_in_weave(for_loop.body(), local_variable_types);
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_local_variable_types_in_content_list(content, local_variable_types);
            }
            collect_local_variable_types_in_content_list(
                choice.inner_content(),
                local_variable_types,
            );
        }
        Object::Weave(weave) => {
            collect_local_variable_types_in_weave(weave, local_variable_types);
        }
        _ => {}
    }
}

fn collect_loop_variable_types_in_weave(
    weave: &Weave,
    flow_name: &str,
    module_name: Option<&str>,
    source_flow_path: &str,
    indexes: &LoweringIndexes<'_>,
    local_variable_types: &mut HashMap<String, TypeName>,
) {
    for object in weave.content() {
        collect_loop_variable_types_in_object(
            object,
            flow_name,
            module_name,
            source_flow_path,
            indexes,
            local_variable_types,
        );
    }
}

fn collect_loop_variable_types_in_content_list(
    content_list: &ContentList,
    flow_name: &str,
    module_name: Option<&str>,
    source_flow_path: &str,
    indexes: &LoweringIndexes<'_>,
    local_variable_types: &mut HashMap<String, TypeName>,
) {
    for object in content_list.objects() {
        collect_loop_variable_types_in_object(
            object,
            flow_name,
            module_name,
            source_flow_path,
            indexes,
            local_variable_types,
        );
    }
}

fn collect_loop_variable_types_in_object(
    object: &Object,
    flow_name: &str,
    module_name: Option<&str>,
    source_flow_path: &str,
    indexes: &LoweringIndexes<'_>,
    local_variable_types: &mut HashMap<String, TypeName>,
) {
    match object {
        Object::ForLoop(for_loop) => {
            if let Some(iterable_type) = infer_loop_iterable_type(
                for_loop,
                flow_name,
                module_name,
                source_flow_path,
                indexes,
                local_variable_types,
            ) {
                insert_loop_variable_types(for_loop, &iterable_type, local_variable_types);
            }
            collect_loop_variable_types_in_weave(
                for_loop.body(),
                flow_name,
                module_name,
                source_flow_path,
                indexes,
                local_variable_types,
            );
        }
        Object::Choice(choice) => {
            if let Some(binding) = choice.dynamic_binding() {
                if let Some(iterable_type) = infer_dynamic_choice_iterable_type(
                    binding.iterable(),
                    flow_name,
                    module_name,
                    source_flow_path,
                    indexes,
                    local_variable_types,
                ) {
                    insert_dynamic_choice_variable_types(
                        binding,
                        &iterable_type,
                        local_variable_types,
                    );
                }
            }
            if let Some(content) = choice.start_content() {
                collect_loop_variable_types_in_content_list(
                    content,
                    flow_name,
                    module_name,
                    source_flow_path,
                    indexes,
                    local_variable_types,
                );
            }
            collect_loop_variable_types_in_content_list(
                choice.inner_content(),
                flow_name,
                module_name,
                source_flow_path,
                indexes,
                local_variable_types,
            );
        }
        Object::ContentList(content_list) => collect_loop_variable_types_in_content_list(
            content_list,
            flow_name,
            module_name,
            source_flow_path,
            indexes,
            local_variable_types,
        ),
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_loop_variable_types_in_weave(
                    branch.content(),
                    flow_name,
                    module_name,
                    source_flow_path,
                    indexes,
                    local_variable_types,
                );
            }
        }
        Object::Weave(weave) => collect_loop_variable_types_in_weave(
            weave,
            flow_name,
            module_name,
            source_flow_path,
            indexes,
            local_variable_types,
        ),
        _ => {}
    }
}

fn infer_loop_iterable_type(
    for_loop: &crate::parsed::ForLoop,
    flow_name: &str,
    module_name: Option<&str>,
    source_flow_path: &str,
    indexes: &LoweringIndexes<'_>,
    local_variable_types: &HashMap<String, TypeName>,
) -> Option<TypeName> {
    let local_variables = local_variable_types.keys().cloned().collect::<HashSet<_>>();
    let path_mode = ChoicePathMode::Flow {
        module_name: module_name.map(str::to_string),
        flow_name: flow_name.to_string(),
        container_path: source_flow_path.to_string(),
        parent_flow_name: source_flow_path
            .rsplit_once('.')
            .map(|(parent, _)| parent.to_string()),
        sibling_stitch_names: Vec::new(),
        local_variables,
        local_variable_types: local_variable_types.clone(),
        self_target_relative: false,
        fallback_gather_target: None,
    };
    let choice_labels = LabelIndex::new();
    let context = LoweringContext::new(
        path_mode,
        &choice_labels,
        &indexes.global_labels,
        &indexes.global_variables,
        &indexes.global_variable_types,
        &indexes.external_signatures,
        &indexes.interface_members,
        &indexes.constants,
        &indexes.struct_definitions,
        &indexes.enum_definitions,
    );
    infer_lowered_expression_type(for_loop.iterable(), &context)
}

fn infer_dynamic_choice_iterable_type(
    iterable: &crate::parsed::Expression,
    flow_name: &str,
    module_name: Option<&str>,
    source_flow_path: &str,
    indexes: &LoweringIndexes<'_>,
    local_variable_types: &HashMap<String, TypeName>,
) -> Option<TypeName> {
    let local_variables = local_variable_types.keys().cloned().collect::<HashSet<_>>();
    let path_mode = ChoicePathMode::Flow {
        module_name: module_name.map(str::to_string),
        flow_name: flow_name.to_string(),
        container_path: source_flow_path.to_string(),
        parent_flow_name: source_flow_path
            .rsplit_once('.')
            .map(|(parent, _)| parent.to_string()),
        sibling_stitch_names: Vec::new(),
        local_variables,
        local_variable_types: local_variable_types.clone(),
        self_target_relative: false,
        fallback_gather_target: None,
    };
    let choice_labels = LabelIndex::new();
    let context = LoweringContext::new(
        path_mode,
        &choice_labels,
        &indexes.global_labels,
        &indexes.global_variables,
        &indexes.global_variable_types,
        &indexes.external_signatures,
        &indexes.interface_members,
        &indexes.constants,
        &indexes.struct_definitions,
        &indexes.enum_definitions,
    );
    infer_lowered_expression_type(iterable, &context)
}

fn insert_loop_variable_types(
    for_loop: &crate::parsed::ForLoop,
    iterable_type: &TypeName,
    local_variable_types: &mut HashMap<String, TypeName>,
) {
    local_variable_types.insert(for_loop.index_name(), TypeName::int());
    local_variable_types.insert(for_loop.limit_name(), TypeName::int());

    if let Some(element_type) = iterable_type.array_element_type() {
        match for_loop.variables() {
            [item] => {
                local_variable_types.insert(item.runtime_name().to_string(), element_type.clone());
            }
            [index, item] => {
                local_variable_types.insert(index.runtime_name().to_string(), TypeName::int());
                local_variable_types.insert(item.runtime_name().to_string(), element_type.clone());
            }
            _ => {}
        }
        return;
    }

    if let Some((key_type, value_type)) = iterable_type.dict_key_value_types() {
        local_variable_types.insert(
            for_loop.keys_name(),
            TypeName::array(dict_key_type_name(key_type)),
        );
        if let [key, value] = for_loop.variables() {
            local_variable_types
                .insert(key.runtime_name().to_string(), dict_key_type_name(key_type));
            local_variable_types.insert(value.runtime_name().to_string(), value_type.clone());
        }
    }
}

fn insert_dynamic_choice_variable_types(
    binding: &crate::parsed::DynamicChoiceBinding,
    iterable_type: &TypeName,
    local_variable_types: &mut HashMap<String, TypeName>,
) {
    local_variable_types.insert(binding.array_name().to_string(), iterable_type.clone());
    local_variable_types.insert(binding.index_name().to_string(), TypeName::int());
    local_variable_types.insert(binding.limit_name().to_string(), TypeName::int());

    let Some(element_type) = iterable_type.array_element_type() else {
        return;
    };
    match binding.variables() {
        [item] => {
            local_variable_types.insert(item.runtime_name().to_string(), element_type.clone());
        }
        [index, item] => {
            local_variable_types.insert(index.runtime_name().to_string(), TypeName::int());
            local_variable_types.insert(item.runtime_name().to_string(), element_type.clone());
        }
        _ => {}
    }
}

fn dict_key_type_name(key_type: DictKeyType) -> TypeName {
    match key_type {
        DictKeyType::String => TypeName::string(),
        DictKeyType::Int => TypeName::int(),
    }
}
