use ink_story_json_format::{Container, ControlCommand, NativeFunction, Object as RuntimeObject};

use crate::parsed::{ForLoop, TypeName};

use super::context::LoweringContext;
use super::expression::{infer_lowered_expression_type, lower_expression_into};
use super::lower_object_into_with_context;
use super::named_content;

enum IterableKind {
    Array,
    Dict,
}

pub(super) fn lower_for_loop_into(
    content: &mut Vec<RuntimeObject>,
    for_loop: &ForLoop,
    context: &LoweringContext<'_>,
) {
    let Some(iterable_type) = infer_lowered_expression_type(for_loop.iterable(), context) else {
        return;
    };
    let Some(kind) = iterable_kind(&iterable_type) else {
        return;
    };

    match kind {
        IterableKind::Array => lower_array_for_loop_into(content, for_loop, context),
        IterableKind::Dict => lower_dict_for_loop_into(content, for_loop, context),
    }
}

fn iterable_kind(iterable_type: &TypeName) -> Option<IterableKind> {
    if iterable_type.array_element_type().is_some() {
        return Some(IterableKind::Array);
    }
    if iterable_type.dict_key_value_types().is_some() {
        return Some(IterableKind::Dict);
    }
    None
}

fn lower_array_for_loop_into(
    content: &mut Vec<RuntimeObject>,
    for_loop: &ForLoop,
    context: &LoweringContext<'_>,
) {
    assign_temp_with(content, &for_loop.limit_name(), |content| {
        lower_expression_into(content, for_loop.iterable(), context, false);
        content.push(RuntimeObject::NativeFunction(NativeFunction::Len));
    });
    assign_int_temp(content, &for_loop.index_name(), 0);

    let loop_index = content.len();
    let loop_target = context.path_mode().runtime_index_path(loop_index);
    let rejoin_target = context.path_mode().runtime_index_path(loop_index + 1);
    let body_context =
        context.with_path_mode(context.path_mode().for_conditional_branch(loop_index));

    let mut body_content = Vec::new();
    match for_loop.variables() {
        [item] => assign_array_item(&mut body_content, item.runtime_name(), for_loop, context),
        [index, item] => {
            assign_temp_from_variable(
                &mut body_content,
                index.runtime_name(),
                &for_loop.index_name(),
            );
            assign_array_item(&mut body_content, item.runtime_name(), for_loop, context);
        }
        _ => {}
    }
    lower_loop_body_into(&mut body_content, for_loop, &body_context);
    increment_loop_index(&mut body_content, &for_loop.index_name());
    body_content.push(RuntimeObject::Divert {
        target: loop_target,
        variable: false,
    });

    content.push(RuntimeObject::Container(loop_container(
        for_loop,
        body_content,
        rejoin_target,
    )));
    content.push(RuntimeObject::ControlCommand(ControlCommand::NoOp));
}

fn lower_dict_for_loop_into(
    content: &mut Vec<RuntimeObject>,
    for_loop: &ForLoop,
    context: &LoweringContext<'_>,
) {
    assign_temp_with(content, &for_loop.keys_name(), |content| {
        lower_expression_into(content, for_loop.iterable(), context, false);
        content.push(RuntimeObject::NativeFunction(NativeFunction::DictKeys));
    });
    assign_temp_with(content, &for_loop.limit_name(), |content| {
        content.push(RuntimeObject::VariableReference(for_loop.keys_name()));
        content.push(RuntimeObject::NativeFunction(NativeFunction::Len));
    });
    assign_int_temp(content, &for_loop.index_name(), 0);

    let loop_index = content.len();
    let loop_target = context.path_mode().runtime_index_path(loop_index);
    let rejoin_target = context.path_mode().runtime_index_path(loop_index + 1);
    let body_context =
        context.with_path_mode(context.path_mode().for_conditional_branch(loop_index));

    let mut body_content = Vec::new();
    if let [key, value] = for_loop.variables() {
        assign_dict_key(&mut body_content, key.runtime_name(), for_loop);
        assign_dict_value(
            &mut body_content,
            value.runtime_name(),
            key.runtime_name(),
            for_loop,
            context,
        );
    }
    lower_loop_body_into(&mut body_content, for_loop, &body_context);
    increment_loop_index(&mut body_content, &for_loop.index_name());
    body_content.push(RuntimeObject::Divert {
        target: loop_target,
        variable: false,
    });

    content.push(RuntimeObject::Container(loop_container(
        for_loop,
        body_content,
        rejoin_target,
    )));
    content.push(RuntimeObject::ControlCommand(ControlCommand::NoOp));
}

fn loop_container(
    for_loop: &ForLoop,
    body_content: Vec<RuntimeObject>,
    rejoin_target: String,
) -> Container {
    let mut container = Container::unnamed(vec![
        RuntimeObject::ControlCommand(ControlCommand::EvalStart),
        RuntimeObject::VariableReference(for_loop.index_name()),
        RuntimeObject::VariableReference(for_loop.limit_name()),
        RuntimeObject::NativeFunction(NativeFunction::Less),
        RuntimeObject::ControlCommand(ControlCommand::EvalEnd),
        RuntimeObject::ConditionalDivert {
            target: ".^.b".to_string(),
        },
        RuntimeObject::Divert {
            target: rejoin_target,
            variable: false,
        },
    ]);
    container
        .named_content
        .push(named_content("b", body_content));
    container
}

fn lower_loop_body_into(
    content: &mut Vec<RuntimeObject>,
    for_loop: &ForLoop,
    context: &LoweringContext<'_>,
) {
    for object in for_loop.body().content() {
        lower_object_into_with_context(content, object, context);
    }
}

fn assign_array_item(
    content: &mut Vec<RuntimeObject>,
    target_name: &str,
    for_loop: &ForLoop,
    context: &LoweringContext<'_>,
) {
    assign_temp_with(content, target_name, |content| {
        lower_expression_into(content, for_loop.iterable(), context, false);
        content.push(RuntimeObject::VariableReference(for_loop.index_name()));
        content.push(RuntimeObject::NativeFunction(NativeFunction::IndexRead));
    });
}

fn assign_dict_key(content: &mut Vec<RuntimeObject>, target_name: &str, for_loop: &ForLoop) {
    assign_temp_with(content, target_name, |content| {
        content.push(RuntimeObject::VariableReference(for_loop.keys_name()));
        content.push(RuntimeObject::VariableReference(for_loop.index_name()));
        content.push(RuntimeObject::NativeFunction(NativeFunction::IndexRead));
    });
}

fn assign_dict_value(
    content: &mut Vec<RuntimeObject>,
    target_name: &str,
    key_name: &str,
    for_loop: &ForLoop,
    context: &LoweringContext<'_>,
) {
    assign_temp_with(content, target_name, |content| {
        lower_expression_into(content, for_loop.iterable(), context, false);
        content.push(RuntimeObject::VariableReference(key_name.to_string()));
        content.push(RuntimeObject::NativeFunction(NativeFunction::IndexRead));
    });
}

fn assign_int_temp(content: &mut Vec<RuntimeObject>, name: &str, value: i32) {
    assign_temp_with(content, name, |content| {
        content.push(RuntimeObject::Int(value));
    });
}

fn assign_temp_from_variable(
    content: &mut Vec<RuntimeObject>,
    target_name: &str,
    source_name: &str,
) {
    assign_temp_with(content, target_name, |content| {
        content.push(RuntimeObject::VariableReference(source_name.to_string()));
    });
}

fn assign_temp_with(
    content: &mut Vec<RuntimeObject>,
    name: &str,
    build_value: impl FnOnce(&mut Vec<RuntimeObject>),
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    build_value(content);
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::VariableAssignment(name.to_string()));
}

fn increment_loop_index(content: &mut Vec<RuntimeObject>, index_name: &str) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    content.push(RuntimeObject::VariableReference(index_name.to_string()));
    content.push(RuntimeObject::Int(1));
    content.push(RuntimeObject::NativeFunction(NativeFunction::Add));
    content.push(RuntimeObject::TempVariableReassignment(
        index_name.to_string(),
    ));
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
}
