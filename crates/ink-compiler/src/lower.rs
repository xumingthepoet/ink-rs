use std::collections::{HashMap, HashSet};

mod conditional;
mod context;
mod expression;
mod flow;
mod indexes;
mod labels;
mod path;
mod sequence;
mod value;
mod weave;

use ink_story_json_format::{
    Container, ControlCommand, NamedContainer, Object as RuntimeObject, Program as RuntimeProgram,
};

use crate::{
    analysis::CheckedStory,
    compiler::StageOutput,
    parsed::{AssignmentTarget, Choice, Divert, DivertTarget, Expression, Object},
};

use conditional::lower_conditional_into;
use context::{ChoicePathMode, LoweringContext};
use expression::{
    lower_expression_into, lower_function_arg_into, lower_logic_line_into,
    lower_output_expression_into,
};
use flow::{lower_flow, lower_module_flow, lower_root_weave};
use indexes::{
    ConstantValues, ExternalSignatures, LoweringIndexes, RuntimeLenEstimator, StructDefinitions,
};
use path::{compact_path_strings_in_container, LabelIndex};
use sequence::lower_sequence;
use value::{lower_value_literal, runtime_default_for_type};
use weave::{lower_choice_weave, lower_content_list_into_context};

pub(crate) fn lower(story: &CheckedStory, count_all_visits: bool) -> StageOutput<RuntimeProgram> {
    let indexes = LoweringIndexes::build(
        &story.parsed,
        RuntimeLenEstimator {
            choice_content_len: estimated_choice_content_len,
            object_len: estimated_runtime_len_for_label_collection,
        },
    );
    if !story.parsed.modules().is_empty() {
        return lower_module_story(story, &indexes, count_all_visits);
    }

    let root_weave = story.parsed.root_weave();
    let main_container = lower_root_weave(root_weave, &indexes, count_all_visits);

    let root_content = vec![
        RuntimeObject::Container(main_container),
        RuntimeObject::ControlCommand(ControlCommand::Done),
    ];

    let mut named_containers = story
        .parsed
        .flows()
        .iter()
        .map(|flow| named_container(lower_flow(flow, &indexes, count_all_visits)))
        .collect::<Vec<_>>();
    if let Some(global_declarations) = lower_global_declarations(&indexes, None) {
        named_containers.push(named_container(global_declarations));
    }

    let mut root = Container {
        content: root_content,
        named_content: named_containers,
        name: None,
        flags: count_all_visits.then_some(1),
    };
    compact_path_strings_in_container(&mut root);

    StageOutput {
        artifact: Some(RuntimeProgram::new(root)),
        diagnostics: Vec::new(),
    }
}

fn lower_module_story(
    story: &CheckedStory,
    indexes: &LoweringIndexes<'_>,
    count_all_visits: bool,
) -> StageOutput<RuntimeProgram> {
    let mut named_containers = story
        .parsed
        .modules()
        .iter()
        .filter(|module| story.module_reachability.is_reachable(module.name()))
        .map(|module| named_container(lower_module(module, indexes, count_all_visits)))
        .collect::<Vec<_>>();
    if let Some(global_declarations) =
        lower_global_declarations(indexes, Some(&story.module_reachability))
    {
        named_containers.push(named_container(global_declarations));
    }

    let root_content = if let Some(entry_point) = &story.entry_point {
        vec![
            RuntimeObject::Divert {
                target: format!("{}.{}", entry_point.module, entry_point.knot),
                variable: false,
            },
            RuntimeObject::ControlCommand(ControlCommand::Done),
        ]
    } else {
        vec![RuntimeObject::ControlCommand(ControlCommand::Done)]
    };

    let mut root = Container {
        content: root_content,
        named_content: named_containers,
        name: None,
        flags: count_all_visits.then_some(1),
    };
    compact_path_strings_in_container(&mut root);

    StageOutput {
        artifact: Some(RuntimeProgram::new(root)),
        diagnostics: Vec::new(),
    }
}

fn lower_module(
    module: &crate::parsed::Module,
    indexes: &LoweringIndexes<'_>,
    count_all_visits: bool,
) -> Container {
    let named_content = module
        .flows()
        .iter()
        .map(|flow| {
            named_container(lower_module_flow(
                module.name(),
                flow,
                indexes,
                count_all_visits,
            ))
        })
        .collect::<Vec<_>>();

    Container {
        content: Vec::new(),
        named_content,
        name: Some(module.name().to_string()),
        flags: count_all_visits.then_some(1),
    }
}

pub(super) fn named_container(container: Container) -> NamedContainer {
    let name = container
        .name
        .clone()
        .expect("named content containers must carry a container name");
    NamedContainer::new(name, container)
}

pub(super) fn named_content(
    name: impl Into<String>,
    content: Vec<RuntimeObject>,
) -> NamedContainer {
    let name = name.into();
    NamedContainer::new(
        name.clone(),
        Container {
            content,
            named_content: Vec::new(),
            name: Some(name),
            flags: None,
        },
    )
}

fn lower_global_declarations(
    indexes: &LoweringIndexes<'_>,
    reachability: Option<&crate::analysis::ModuleReachability>,
) -> Option<Container> {
    let visible_declarations = indexes
        .variable_declarations
        .iter()
        .filter(
            |declaration| match (declaration.module_name(), reachability) {
                (Some(module), Some(reachability)) => reachability.is_reachable(module),
                _ => true,
            },
        )
        .collect::<Vec<_>>();

    if visible_declarations.is_empty() {
        return None;
    }

    let choice_labels = LabelIndex::new();
    let mut content = vec![RuntimeObject::ControlCommand(ControlCommand::EvalStart)];
    for declaration in visible_declarations
        .iter()
        .copied()
        .filter(|declaration| declaration.assignment().is_global())
    {
        let path_mode = declaration
            .module_name()
            .map(|module_name| ChoicePathMode::Module {
                module_name: module_name.to_string(),
            })
            .unwrap_or(ChoicePathMode::Root);
        let context = LoweringContext::new(
            path_mode,
            &choice_labels,
            &indexes.global_labels,
            &indexes.global_variables,
            &indexes.external_signatures,
            &indexes.constants,
            &indexes.struct_definitions,
        );
        if lower_assignment_initializer_with_context(
            &mut content,
            declaration.assignment(),
            &context,
        ) {
            content.push(RuntimeObject::GlobalVariableAssignment(
                declaration.runtime_name().to_string(),
            ));
        }
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::ControlCommand(ControlCommand::End));

    Some(Container {
        content,
        named_content: Vec::new(),
        name: Some("global decl".to_string()),
        flags: None,
    })
}

fn estimated_choice_content_len(
    choice: &Choice,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    global_variables: &HashSet<String>,
) -> usize {
    let mut content = Vec::new();
    lower_content_list_into_context(
        &mut content,
        choice.inner_content(),
        &ChoicePathMode::Root,
        &LabelIndex::new(),
        &LabelIndex::new(),
        global_variables,
        &HashMap::new(),
        constants,
        struct_definitions,
    );
    content.len()
}

fn estimated_runtime_len_for_label_collection(
    object: &Object,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    global_variables: &HashSet<String>,
) -> usize {
    if matches!(object, Object::Weave(_)) {
        return 1;
    }

    let mut content = Vec::new();
    lower_object_into_with_context_count(
        &mut content,
        object,
        &ChoicePathMode::Root,
        &LabelIndex::new(),
        &LabelIndex::new(),
        global_variables,
        &HashMap::new(),
        constants,
        struct_definitions,
        false,
    );
    content.len()
}

fn lower_object_into_with_context(
    content: &mut Vec<RuntimeObject>,
    object: &Object,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
) {
    lower_object_into_with_context_count(
        content,
        object,
        path_mode,
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
        false,
    );
}

fn lower_object_into_with_context_count(
    content: &mut Vec<RuntimeObject>,
    object: &Object,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    count_all_visits: bool,
) {
    let lowering_context = LoweringContext::new(
        path_mode.clone(),
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
    );
    match object {
        Object::Text(text) => content.push(RuntimeObject::String(text.text().to_string())),
        Object::AuthorWarning(_) => {}
        Object::ContentList(content_list) => {
            lower_content_list_into_context(
                content,
                content_list,
                path_mode,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
            );
        }
        Object::Expression(expression) => {
            lower_output_expression_into(content, expression, &lowering_context)
        }
        Object::Conditional(conditional) => lower_conditional_into(
            content,
            conditional,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
            path_mode,
        ),
        Object::LogicLine(expression) => {
            lower_logic_line_into(content, expression, &lowering_context);
        }
        Object::Glue(_) => content.push(RuntimeObject::Glue),
        Object::Divert(divert) => push_divert_with_context(
            content,
            divert,
            path_mode,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
        ),
        Object::TunnelOnwards(tunnel_onwards) => {
            lower_tunnel_onwards_into(
                content,
                tunnel_onwards,
                path_mode,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
            );
        }
        Object::Choice(_) => {}
        Object::ConstantDeclaration(_) => {}
        Object::Gather(_) => {} // Handled in lower_choice_weave
        Object::StructDeclaration(_) => {}
        Object::VariableAssignment(assignment) => {
            lower_variable_assignment_into(
                content,
                assignment,
                path_mode,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
            );
        }
        Object::IncDec(inc_dec) => {
            lower_inc_dec_into(
                content,
                inc_dec,
                path_mode,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
            );
        }
        Object::Return(ret) => {
            if lower_tail_recursive_return_into(
                content,
                ret,
                path_mode,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
            ) {
                return;
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
            if let Some(expr) = ret.returned_expression() {
                lower_expression_into(content, expr, &lowering_context, false);
            } else {
                content.push(RuntimeObject::Void);
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
            content.push(RuntimeObject::ControlCommand(ControlCommand::PopFunction));
        }
        Object::Tag(tag) => content.push(RuntimeObject::Tag {
            is_start: tag.is_start(),
        }),
        Object::Sequence(sequence) => content.push(RuntimeObject::Container(lower_sequence(
            sequence,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
            path_mode,
            &path_mode.sequence_container_path(content.len()),
        ))),
        Object::Weave(weave) => {
            let nested_path_mode = path_mode.for_nested_weave(content.len());
            content.push(RuntimeObject::Container(lower_choice_weave(
                weave,
                nested_path_mode,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
                count_all_visits,
            )));
        }
        Object::ExternalDeclaration(_) => {}
    }
}

fn lower_assignment_initializer_into(
    content: &mut Vec<RuntimeObject>,
    assignment: &crate::parsed::VariableAssignment,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    path_mode: &ChoicePathMode,
    struct_definitions: &StructDefinitions,
) -> bool {
    if let Some(expression) = assignment.expression() {
        if let Expression::ArrayLiteral(_) | Expression::StructLiteral(_) = expression {
            if let Some(value) = lower_value_literal(
                expression,
                assignment.declared_type(),
                struct_definitions,
                choice_labels,
                global_labels,
                path_mode,
            ) {
                content.push(value);
                return true;
            }
        }

        let context = LoweringContext::new(
            path_mode.clone(),
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
        );
        lower_expression_into(content, expression, &context, false);
        return true;
    }

    assignment
        .declared_type()
        .and_then(|declared_type| {
            runtime_default_for_type(
                declared_type,
                struct_definitions,
                path_mode.current_module_name(),
            )
        })
        .map(|default_value| content.push(default_value))
        .is_some()
}

fn lower_assignment_initializer_with_context(
    content: &mut Vec<RuntimeObject>,
    assignment: &crate::parsed::VariableAssignment,
    context: &LoweringContext<'_>,
) -> bool {
    lower_assignment_initializer_into(
        content,
        assignment,
        context.choice_labels(),
        context.global_labels(),
        context.global_variables(),
        context.external_signatures(),
        context.constants(),
        context.path_mode(),
        context.struct_definitions(),
    )
}

enum AssignmentPathComponent<'a> {
    Field(&'a str),
    Index(&'a Expression),
    CachedIndex(String),
}

#[derive(Clone, Copy)]
enum AssignmentUpdateValue<'a> {
    Expression(&'a Expression),
    Compound {
        expression: &'a Expression,
        operator: &'static str,
    },
    ArrayRemove {
        index: &'a Expression,
    },
}

fn collect_assignment_path<'a>(
    target: &'a AssignmentTarget,
    components: &mut Vec<AssignmentPathComponent<'a>>,
) -> Option<&'a str> {
    match target {
        AssignmentTarget::Variable(name) => Some(name),
        AssignmentTarget::QualifiedVariable(name) => Some(name.as_str()),
        AssignmentTarget::FieldAccess { base, field } => {
            let root = collect_assignment_path(base, components)?;
            components.push(AssignmentPathComponent::Field(field));
            Some(root)
        }
        AssignmentTarget::IndexAccess { base, index } => {
            let root = collect_assignment_path(base, components)?;
            components.push(AssignmentPathComponent::Index(index));
            Some(root)
        }
    }
}

fn lower_assignment_path_component_key_into(
    content: &mut Vec<RuntimeObject>,
    component: &AssignmentPathComponent<'_>,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    path_mode: &ChoicePathMode,
    struct_definitions: &StructDefinitions,
) {
    let context = LoweringContext::new(
        path_mode.clone(),
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
    );
    match component {
        AssignmentPathComponent::Field(field) => {
            content.push(RuntimeObject::String((*field).to_string()));
        }
        AssignmentPathComponent::CachedIndex(name) => {
            content.push(RuntimeObject::VariableReference(name.clone()));
        }
        AssignmentPathComponent::Index(index) => {
            lower_expression_into(content, index, &context, false)
        }
    }
}

fn lower_assignment_path_read_into(
    content: &mut Vec<RuntimeObject>,
    root_name: &str,
    components: &[AssignmentPathComponent<'_>],
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    path_mode: &ChoicePathMode,
    struct_definitions: &StructDefinitions,
) {
    content.push(RuntimeObject::VariableReference(
        resolve_runtime_variable_name(root_name, path_mode, global_variables),
    ));
    for component in components {
        lower_assignment_path_component_key_into(
            content,
            component,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            path_mode,
            struct_definitions,
        );
        let read_operation = match component {
            AssignmentPathComponent::Field(_) => "FIELD",
            AssignmentPathComponent::Index(_) | AssignmentPathComponent::CachedIndex(_) => "INDEX",
        };
        content.push(RuntimeObject::NativeFunction(read_operation.to_string()));
    }
}

fn lower_assignment_update_value_into(
    content: &mut Vec<RuntimeObject>,
    root_name: &str,
    components: &[AssignmentPathComponent<'_>],
    value: AssignmentUpdateValue<'_>,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    path_mode: &ChoicePathMode,
    struct_definitions: &StructDefinitions,
) {
    let context = LoweringContext::new(
        path_mode.clone(),
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
    );
    match value {
        AssignmentUpdateValue::Expression(expression) => {
            lower_expression_into(content, expression, &context, false)
        }
        AssignmentUpdateValue::Compound {
            expression,
            operator,
        } => {
            lower_assignment_path_read_into(
                content,
                root_name,
                components,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                path_mode,
                struct_definitions,
            );
            lower_expression_into(content, expression, &context, false);
            content.push(RuntimeObject::NativeFunction(operator.to_string()));
        }
        AssignmentUpdateValue::ArrayRemove { index } => {
            lower_assignment_path_read_into(
                content,
                root_name,
                components,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                path_mode,
                struct_definitions,
            );
            lower_expression_into(content, index, &context, false);
            content.push(RuntimeObject::NativeFunction("ARRAY_REMOVE".to_string()));
        }
    }
}

fn lower_assignment_path_update_value_into(
    content: &mut Vec<RuntimeObject>,
    root_name: &str,
    components: &[AssignmentPathComponent<'_>],
    component_index: usize,
    value: AssignmentUpdateValue<'_>,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    path_mode: &ChoicePathMode,
    struct_definitions: &StructDefinitions,
) {
    lower_assignment_path_read_into(
        content,
        root_name,
        &components[..component_index],
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        path_mode,
        struct_definitions,
    );
    lower_assignment_path_component_key_into(
        content,
        &components[component_index],
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        path_mode,
        struct_definitions,
    );

    if component_index + 1 == components.len() {
        lower_assignment_update_value_into(
            content,
            root_name,
            components,
            value,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            path_mode,
            struct_definitions,
        );
    } else {
        lower_assignment_path_update_value_into(
            content,
            root_name,
            components,
            component_index + 1,
            value,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            path_mode,
            struct_definitions,
        );
    }

    let write_operation = match &components[component_index] {
        AssignmentPathComponent::Field(_) => "SET_FIELD",
        AssignmentPathComponent::Index(_) | AssignmentPathComponent::CachedIndex(_) => "SET_INDEX",
    };
    content.push(RuntimeObject::NativeFunction(write_operation.to_string()));
}

fn lower_cached_assignment_indexes_into<'a>(
    content: &mut Vec<RuntimeObject>,
    components: &[AssignmentPathComponent<'a>],
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    path_mode: &ChoicePathMode,
    struct_definitions: &StructDefinitions,
) -> Vec<AssignmentPathComponent<'a>> {
    let mut cached_components = Vec::with_capacity(components.len());
    let mut next_index = 0;
    let context = LoweringContext::new(
        path_mode.clone(),
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
    );
    for component in components {
        match component {
            AssignmentPathComponent::Field(field) => {
                cached_components.push(AssignmentPathComponent::Field(field));
            }
            AssignmentPathComponent::CachedIndex(name) => {
                cached_components.push(AssignmentPathComponent::CachedIndex(name.clone()));
            }
            AssignmentPathComponent::Index(index) => {
                let temp_name = format!("$lvalue{next_index}");
                next_index += 1;
                lower_expression_into(content, index, &context, false);
                content.push(RuntimeObject::VariableAssignment(temp_name.clone()));
                cached_components.push(AssignmentPathComponent::CachedIndex(temp_name));
            }
        }
    }
    cached_components
}

fn push_reassignment_for_name(
    content: &mut Vec<RuntimeObject>,
    name: &str,
    path_mode: &ChoicePathMode,
) {
    if path_mode.is_local_variable(name) {
        content.push(RuntimeObject::TempVariableReassignment(name.to_string()));
    } else {
        content.push(RuntimeObject::VariableReassignment(name.to_string()));
    }
}

fn resolve_runtime_variable_name(
    name: &str,
    path_mode: &ChoicePathMode,
    global_variables: &HashSet<String>,
) -> String {
    if name.contains("::") || path_mode.is_local_variable(name) {
        return name.to_string();
    }

    path_mode
        .current_module_name()
        .map(|module_name| format!("{module_name}::{name}"))
        .filter(|qualified_name| global_variables.contains(qualified_name))
        .unwrap_or_else(|| name.to_string())
}

fn lower_tail_recursive_return_into(
    content: &mut Vec<RuntimeObject>,
    ret: &crate::parsed::Return,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
) -> bool {
    let Some(flow_name) = path_mode.current_flow_name() else {
        return false;
    };
    let signature_name = path_mode
        .current_module_name()
        .map(|module_name| format!("{module_name}::{flow_name}"))
        .filter(|qualified_name| external_signatures.contains_key(qualified_name))
        .unwrap_or_else(|| flow_name.to_string());
    let Some(tail_args) = ret.direct_self_tail_call_args(flow_name) else {
        return false;
    };
    let Some(indexes::CallSignature::Ink {
        args: expected_args,
        ..
    }) = external_signatures.get(&signature_name)
    else {
        return false;
    };
    if expected_args.len() != tail_args.len() {
        return false;
    }
    let Some(body_start_target) = path_mode.current_flow_body_start_target(expected_args.len())
    else {
        return false;
    };

    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    let mut visiting_constants = HashSet::new();
    let context = LoweringContext::new(
        path_mode.clone(),
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
    );
    for (index, arg) in tail_args.iter().enumerate() {
        lower_function_arg_into(
            content,
            arg,
            expected_args.get(index),
            &context,
            false,
            &mut visiting_constants,
        );
    }
    for argument in expected_args.iter().rev() {
        content.push(RuntimeObject::VariableAssignment(
            argument.name().to_string(),
        ));
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::Divert {
        target: body_start_target,
        variable: false,
    });

    true
}

fn lower_variable_assignment_into(
    content: &mut Vec<RuntimeObject>,
    assignment: &crate::parsed::VariableAssignment,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
) {
    if assignment.is_global() {
        return;
    }

    let Some(name) = assignment.target().variable_name() else {
        let Some(expression) = assignment.expression() else {
            return;
        };
        let mut components = Vec::new();
        let Some(root_name) = collect_assignment_path(assignment.target(), &mut components) else {
            return;
        };
        if components.is_empty() {
            return;
        }

        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
        let resolved_root_name =
            resolve_runtime_variable_name(root_name, path_mode, global_variables);
        lower_assignment_path_update_value_into(
            content,
            resolved_root_name.as_str(),
            &components,
            0,
            AssignmentUpdateValue::Expression(expression),
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            path_mode,
            struct_definitions,
        );
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
        push_reassignment_for_name(content, resolved_root_name.as_str(), path_mode);
        return;
    };

    let mut initializer = Vec::new();
    if !lower_assignment_initializer_into(
        &mut initializer,
        assignment,
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        path_mode,
        struct_definitions,
    ) {
        return;
    }

    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    content.extend(initializer);
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));

    if assignment.is_temporary() {
        content.push(RuntimeObject::VariableAssignment(name.to_string()));
    } else {
        let resolved_name = resolve_runtime_variable_name(name, path_mode, global_variables);
        push_reassignment_for_name(content, resolved_name.as_str(), path_mode);
    }
}

fn lower_inc_dec_into(
    content: &mut Vec<RuntimeObject>,
    inc_dec: &crate::parsed::IncDec,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
) {
    let Some(name) = inc_dec.target().variable_name() else {
        let mut components = Vec::new();
        let Some(root_name) = collect_assignment_path(inc_dec.target(), &mut components) else {
            return;
        };
        if components.is_empty() {
            return;
        }
        let operator = if inc_dec.is_increment() { "+" } else { "-" };

        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
        let resolved_root_name =
            resolve_runtime_variable_name(root_name, path_mode, global_variables);
        let cached_components = lower_cached_assignment_indexes_into(
            content,
            &components,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            path_mode,
            struct_definitions,
        );
        lower_assignment_path_update_value_into(
            content,
            resolved_root_name.as_str(),
            &cached_components,
            0,
            AssignmentUpdateValue::Compound {
                expression: inc_dec.expression(),
                operator,
            },
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            path_mode,
            struct_definitions,
        );
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
        push_reassignment_for_name(content, resolved_root_name.as_str(), path_mode);
        return;
    };
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    let resolved_name = resolve_runtime_variable_name(name, path_mode, global_variables);
    content.push(RuntimeObject::VariableReference(resolved_name.clone()));
    let context = LoweringContext::new(
        path_mode.clone(),
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
    );
    lower_expression_into(content, inc_dec.expression(), &context, false);
    content.push(RuntimeObject::NativeFunction(
        if inc_dec.is_increment() { "+" } else { "-" }.to_string(),
    ));
    if path_mode.is_local_variable(name) {
        content.push(RuntimeObject::TempVariableReassignment(resolved_name));
    } else {
        content.push(RuntimeObject::VariableReassignment(resolved_name));
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
}

fn push_divert_with_context(
    content: &mut Vec<RuntimeObject>,
    divert: &Divert,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
) {
    if let DivertTarget::Dynamic(expression) = divert.target() {
        push_dynamic_divert_with_context(
            content,
            divert,
            DynamicDivertTarget {
                expression: expression.clone(),
                divert_arguments: divert.arguments().to_vec(),
            },
            path_mode,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
        );
        return;
    }

    if !divert.arguments().is_empty() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
        let context = LoweringContext::new(
            path_mode.clone(),
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
        );
        for argument in divert.arguments() {
            lower_expression_into(content, argument, &context, false);
        }
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    }

    if divert.is_thread() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::StartThread));
    }

    match divert.target() {
        DivertTarget::Done => content.push(RuntimeObject::ControlCommand(ControlCommand::Done)),
        DivertTarget::End => content.push(RuntimeObject::ControlCommand(ControlCommand::End)),
        DivertTarget::Dynamic(_) => {
            unreachable!("dynamic divert targets return before static lowering")
        }
        DivertTarget::Path(target) => {
            let resolved_target = if let Some(choice_target) = choice_labels.get(target) {
                runtime_divert(choice_target.to_string(), false, divert.is_tunnel())
            } else if let Some(label_target) = path_mode
                .scoped_label_target(target, global_labels)
                .filter(|label_target| *label_target != target)
            {
                runtime_divert(
                    path_mode.resolve_label_target(label_target),
                    false,
                    divert.is_tunnel(),
                )
            } else {
                runtime_divert(
                    path_mode.resolve_divert_target(target),
                    false,
                    divert.is_tunnel(),
                )
            };
            content.push(resolved_target);
        }
        DivertTarget::QualifiedPath(target) => {
            let target = target.as_str();
            let resolved_target = if let Some(choice_target) = choice_labels.get(target) {
                runtime_divert(choice_target.to_string(), false, divert.is_tunnel())
            } else if let Some(label_target) = path_mode
                .scoped_label_target(target, global_labels)
                .filter(|label_target| *label_target != target)
            {
                runtime_divert(
                    path_mode.resolve_label_target(label_target),
                    false,
                    divert.is_tunnel(),
                )
            } else {
                runtime_divert(
                    path_mode.resolve_divert_target(target),
                    false,
                    divert.is_tunnel(),
                )
            };
            content.push(resolved_target);
        }
        DivertTarget::Empty => {
            content.push(runtime_divert(String::new(), false, divert.is_tunnel()))
        }
    }
}

struct DynamicDivertTarget {
    expression: Expression,
    divert_arguments: Vec<Expression>,
}

fn push_dynamic_divert_with_context(
    content: &mut Vec<RuntimeObject>,
    divert: &Divert,
    dynamic_target: DynamicDivertTarget,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
) {
    const DYNAMIC_DIVERT_TARGET_TEMP: &str = "$divertTarget";

    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    let context = LoweringContext::new(
        path_mode.clone(),
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
    );
    for argument in &dynamic_target.divert_arguments {
        lower_expression_into(content, argument, &context, false);
    }
    lower_expression_into(content, &dynamic_target.expression, &context, false);
    content.push(RuntimeObject::VariableAssignment(
        DYNAMIC_DIVERT_TARGET_TEMP.to_string(),
    ));
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));

    if divert.is_thread() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::StartThread));
    }
    content.push(runtime_divert(
        DYNAMIC_DIVERT_TARGET_TEMP.to_string(),
        true,
        divert.is_tunnel(),
    ));
}

fn runtime_divert(target: String, variable: bool, is_tunnel: bool) -> RuntimeObject {
    if is_tunnel {
        RuntimeObject::TunnelDivert { target, variable }
    } else {
        RuntimeObject::Divert { target, variable }
    }
}

fn lower_tunnel_onwards_into(
    content: &mut Vec<RuntimeObject>,
    tunnel_onwards: &crate::parsed::TunnelOnwards,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    let context = LoweringContext::new(
        path_mode.clone(),
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
    );
    for argument in tunnel_onwards.arguments() {
        lower_expression_into(content, argument, &context, false);
    }
    if let Some(target) = tunnel_onwards.override_target() {
        match target {
            DivertTarget::Dynamic(expression) => {
                lower_expression_into(content, expression, &context, false);
            }
            DivertTarget::Path(target) => {
                if let Some(choice_target) = choice_labels.get(target) {
                    content.push(RuntimeObject::DivertTarget(choice_target.to_string()));
                } else if let Some(label_target) = path_mode
                    .scoped_label_target(target, global_labels)
                    .filter(|label_target| *label_target != target)
                {
                    content.push(RuntimeObject::DivertTarget(
                        path_mode.resolve_label_target(label_target),
                    ));
                } else if path_mode.is_local_variable(target) || global_variables.contains(target) {
                    content.push(RuntimeObject::VariableReference(target.clone()));
                } else {
                    content.push(RuntimeObject::DivertTarget(
                        path_mode.resolve_divert_target(target),
                    ));
                }
            }
            DivertTarget::QualifiedPath(target) => {
                let target = target.as_str();
                if let Some(choice_target) = choice_labels.get(target) {
                    content.push(RuntimeObject::DivertTarget(choice_target.to_string()));
                } else if let Some(label_target) = path_mode
                    .scoped_label_target(target, global_labels)
                    .filter(|label_target| *label_target != target)
                {
                    content.push(RuntimeObject::DivertTarget(
                        path_mode.resolve_label_target(label_target),
                    ));
                } else if path_mode.is_local_variable(target) || global_variables.contains(target) {
                    content.push(RuntimeObject::VariableReference(target.to_string()));
                } else {
                    content.push(RuntimeObject::DivertTarget(
                        path_mode.resolve_divert_target(target),
                    ));
                }
            }
            DivertTarget::Done => content.push(RuntimeObject::DivertTarget("DONE".to_string())),
            DivertTarget::End => content.push(RuntimeObject::DivertTarget("END".to_string())),
            DivertTarget::Empty => content.push(RuntimeObject::Void),
        }
    } else {
        content.push(RuntimeObject::Void);
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::ControlCommand(ControlCommand::PopTunnel));
}

fn ends_with_flow_terminator(content: &[RuntimeObject]) -> bool {
    content
        .iter()
        .rev()
        .find(|object| !matches!(object, RuntimeObject::String(text) if text == "\n"))
        .is_some_and(|object| {
            matches!(
                object,
                RuntimeObject::Divert { .. }
                    | RuntimeObject::ControlCommand(ControlCommand::End | ControlCommand::Done)
            )
        })
}

fn ends_with_end_or_done(content: &[RuntimeObject]) -> bool {
    content
        .iter()
        .rev()
        .find(|object| !matches!(object, RuntimeObject::String(text) if text == "\n"))
        .is_some_and(|object| {
            matches!(
                object,
                RuntimeObject::ControlCommand(ControlCommand::End | ControlCommand::Done)
            )
        })
}

fn done_container(name: &str, count_all_visits: bool) -> Container {
    Container {
        content: vec![RuntimeObject::ControlCommand(ControlCommand::Done)],
        named_content: Vec::new(),
        name: Some(name.to_string()),
        flags: count_all_visits.then_some(5),
    }
}

#[cfg(test)]
mod tests {
    use ink_story_json_format::Object as RuntimeObject;
    use serde_json::{json, Value};

    use crate::{compiler::Compiler, source::SourceInput};

    use super::lower;

    #[test]
    fn module_lowering_uses_checked_entry_point_and_reachability() {
        let source = concat!(
            "=== module game ===\n",
            "IMPORT helper FROM support\n",
            "== main ==\n",
            "-> support::helper\n",
            "=== module support ===\n",
            "== helper ==\n",
            "-> END\n",
            "=== module unused ===\n",
            "== spare ==\n",
            "-> END\n",
        );
        let compiler = Compiler::default();
        let parsed = compiler
            .parse(SourceInput::new(source))
            .artifact
            .expect("parse should succeed");
        let checked = compiler
            .analyze(parsed)
            .artifact
            .expect("analysis should produce checked story");

        assert!(checked.module_reachability.is_reachable("support"));
        assert!(!checked.module_reachability.is_reachable("unused"));

        let program = lower(&checked, false)
            .artifact
            .expect("lowering should produce program");

        assert!(matches!(
            program.root.content.first(),
            Some(RuntimeObject::Divert { target, variable: false }) if target == "game.main"
        ));
        let named_modules = program
            .root
            .named_content
            .iter()
            .map(|named| named.name.as_str())
            .collect::<Vec<_>>();
        assert!(named_modules.contains(&"game"));
        assert!(named_modules.contains(&"support"));
        assert!(!named_modules.contains(&"unused"));
    }

    #[test]
    fn module_lowering_json_contains_root_entry_and_reachable_module_containers() {
        let source = concat!(
            "=== module game ===\n",
            "IMPORT helper FROM support\n",
            "== main ==\n",
            "Game.\n",
            "-> END\n",
            "=== module support ===\n",
            "== helper ==\n",
            "Support.\n",
            "-> END\n",
            "=== module unused ===\n",
            "== main_unused ==\n",
            "Unused.\n",
            "-> END\n",
        );
        let compiler = Compiler::default();
        let compiled = compiler.compile(SourceInput::new(source));

        assert!(
            compiled.artifact.is_some(),
            "module story should compile with warnings only: {:#?}",
            compiled.diagnostics
        );
        let json = compiled
            .artifact
            .expect("compiled story")
            .program
            .to_json_value();
        let root = json
            .get("root")
            .and_then(Value::as_array)
            .expect("root should encode as JSON array");

        assert_eq!(root.first(), Some(&json!({"->": "game.main"})));
        let named = root
            .last()
            .and_then(Value::as_object)
            .expect("root should end with named content object");
        assert!(named.contains_key("game"), "{json:#}");
        assert!(named.contains_key("support"), "{json:#}");
        assert!(!named.contains_key("unused"), "{json:#}");
        assert!(container_json_has_named_content(
            named.get("game").expect("game module"),
            "main"
        ));
        assert!(container_json_has_named_content(
            named.get("support").expect("support module"),
            "helper"
        ));
    }

    #[test]
    fn module_lowering_json_excludes_unreachable_module_globals() {
        let source = concat!(
            "=== module game ===\n",
            "IMPORT helper FROM support\n",
            "== main ==\n",
            "-> support::helper\n",
            "=== module support ===\n",
            "VAR shown: int = 1\n",
            "== helper ==\n",
            "-> END\n",
            "=== module unused ===\n",
            "VAR hidden: int = 2\n",
            "== spare ==\n",
            "-> END\n",
        );
        let compiled = Compiler::default().compile(SourceInput::new(source));

        assert!(
            compiled.artifact.is_some(),
            "module story should compile with unreachable warning only: {:#?}",
            compiled.diagnostics
        );
        let json = compiled
            .artifact
            .expect("compiled story")
            .program
            .to_json_value();
        let json_text = json.to_string();

        assert!(json_text.contains("support::shown"), "{json:#}");
        assert!(!json_text.contains("unused"), "{json:#}");
        assert!(!json_text.contains("unused::hidden"), "{json:#}");
    }

    #[test]
    fn module_global_initializer_lowering_uses_scoped_runtime_names() {
        let source = concat!(
            "=== module game ===\n",
            "IMPORT helper FROM support\n",
            "== main ==\n",
            "-> support::helper\n",
            "=== module support ===\n",
            "VAR shown: int = 7\n",
            "== helper ==\n",
            "-> END\n",
            "=== module unused ===\n",
            "VAR hidden: int = 9\n",
            "== spare ==\n",
            "-> END\n",
        );
        let compiled = Compiler::default().compile(SourceInput::new(source));

        assert!(
            compiled.artifact.is_some(),
            "module story should compile with unreachable warning only: {:#?}",
            compiled.diagnostics
        );
        let json = compiled
            .artifact
            .expect("compiled story")
            .program
            .to_json_value();
        let root = json
            .get("root")
            .and_then(Value::as_array)
            .expect("root should encode as JSON array");
        let named = root
            .last()
            .and_then(Value::as_object)
            .expect("root should end with named content object");
        let global_decl = named
            .get("global decl")
            .and_then(Value::as_array)
            .expect("global declarations should encode as a container");

        assert!(global_decl.contains(&json!(7)), "{json:#}");
        assert!(
            global_decl.contains(&json!({"VAR=": "support::shown"})),
            "{json:#}"
        );
        assert!(
            !global_decl.contains(&json!({"VAR=": "unused::hidden"})),
            "{json:#}"
        );
    }

    #[test]
    fn module_compilation_fails_when_unreachable_module_has_errors() {
        let source = concat!(
            "=== module game ===\n",
            "== main ==\n",
            "-> END\n",
            "=== module unused ===\n",
            "VAR broken: int = \"wrong\"\n",
            "== spare ==\n",
            "-> END\n",
        );
        let compiled = Compiler::default().compile(SourceInput::new(source));

        assert!(compiled.artifact.is_none());
        assert!(
            compiled
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(
                    "Initializer for variable 'broken' has type string but declared type is int"
                )),
            "unreachable module errors should fail compilation: {:#?}",
            compiled.diagnostics
        );
    }

    #[test]
    fn expression_lowering_context_preserves_current_expression_json() {
        let source = concat!(
            "=== module game ===\n",
            "IMPORT add FROM math\n",
            "STRUCT Stats {\n",
            "hp: int\n",
            "}\n",
            "CONST LIMIT: int = 3\n",
            "VAR state: Stats = { hp: 4 }\n",
            "VAR items: int[] = [5, 8]\n",
            "== main ==\n",
            "{math::add(LIMIT, state.hp)}|{items[1]}|{\"HP {state.hp}\"}\n",
            "-> END\n",
            "=== module math ===\n",
            "== function add(left: int, right: int) => int ==\n",
            "~ return left + right\n",
        );
        let compiled = Compiler::default().compile(SourceInput::new(source));

        assert!(
            compiled.artifact.is_some(),
            "module story should compile: {:#?}",
            compiled.diagnostics
        );
        let json = compiled
            .artifact
            .expect("compiled story")
            .program
            .to_json_value();

        assert!(json_contains_sequence(
            &json,
            &[
                json!(3),
                json!({"VAR?": "game::state"}),
                json!("^hp"),
                json!("FIELD")
            ]
        ));
        assert!(json_contains_sequence(
            &json,
            &[json!({"VAR?": "game::items"}), json!(1), json!("INDEX")]
        ));
        assert!(json_contains_sequence(
            &json,
            &[
                json!("str"),
                json!("^HP "),
                json!("ev"),
                json!({"VAR?": "game::state"}),
                json!("^hp"),
                json!("FIELD"),
                json!("out"),
                json!("/ev"),
                json!("/str")
            ]
        ));
        assert!(
            json.to_string().contains("\"f()\":\"math.add\""),
            "{json:#}"
        );
    }

    fn container_json_has_named_content(container: &Value, name: &str) -> bool {
        container
            .as_array()
            .and_then(|items| items.last())
            .and_then(Value::as_object)
            .is_some_and(|named| named.contains_key(name))
    }

    fn json_contains_sequence(value: &Value, expected: &[Value]) -> bool {
        match value {
            Value::Array(items) => {
                items
                    .windows(expected.len())
                    .any(|window| window == expected)
                    || items
                        .iter()
                        .any(|item| json_contains_sequence(item, expected))
            }
            Value::Object(object) => object
                .values()
                .any(|item| json_contains_sequence(item, expected)),
            _ => false,
        }
    }
}
