use std::{collections::HashMap, rc::Rc};

use ink_story_json_format as format;
use serde_json::{json, Map};

use crate::{
    choice::Choice,
    choice_point::ChoicePoint,
    container::Container,
    control_command::ControlCommand,
    divert::Divert,
    glue::Glue,
    native_function_call::NativeFunctionCall,
    object::RTObject,
    path::Path,
    push_pop::PushPopType,
    story_error::StoryError,
    tag::Tag,
    value::Value,
    value_type::{StringValue, VariablePointerValue},
    variable_assigment::VariableAssignment,
    variable_reference::VariableReference,
    void::Void,
};

pub fn write_dictionary_values(
    objs: &HashMap<String, Rc<Value>>,
) -> Result<serde_json::Value, StoryError> {
    let mut jobjs: Map<String, serde_json::Value> = Map::new();

    for (k, o) in objs {
        jobjs.insert(k.clone(), write_rtobject(o.clone())?);
    }

    Ok(serde_json::Value::Object(jobjs))
}

pub fn write_rtobject(o: Rc<dyn RTObject>) -> Result<serde_json::Value, StoryError> {
    if let Some(c) = o.as_any().downcast_ref::<Container>() {
        return write_rt_container(c, false);
    }

    if let Ok(divert) = o.clone().into_any().downcast::<Divert>() {
        let target_str = if divert.has_variable_target() {
            divert.variable_divert_name.clone().unwrap()
        } else {
            divert.get_target_path_string().unwrap()
        };

        let object = if divert.is_external {
            format::Object::ExternalFunction {
                target: target_str,
                args: divert.external_args,
            }
        } else if divert.pushes_to_stack && divert.stack_push_type == PushPopType::Function {
            format::Object::FunctionDivert { target: target_str }
        } else if divert.pushes_to_stack && divert.stack_push_type == PushPopType::Tunnel {
            format::Object::TunnelDivert {
                target: target_str,
                variable: divert.has_variable_target(),
            }
        } else if divert.is_conditional {
            format::Object::ConditionalDivert { target: target_str }
        } else {
            format::Object::Divert {
                target: target_str,
                variable: divert.has_variable_target(),
            }
        };

        return Ok(object.to_json_value());
    }

    if let Ok(cp) = o.clone().into_any().downcast::<ChoicePoint>() {
        return Ok(format::Object::ChoicePoint {
            target: ChoicePoint::get_path_string_on_choice(&cp),
            flags: cp.get_flags(),
        }
        .to_json_value());
    }

    if let Some(v) = Value::get_bool_value(o.as_ref()) {
        return Ok(format::Object::Bool(v).to_json_value());
    }

    if let Some(v) = Value::get_value::<i32>(o.as_ref()) {
        return Ok(format::Object::Int(v).to_json_value());
    }

    if let Some(v) = Value::get_value::<f32>(o.as_ref()) {
        return Ok(format::Object::Float(v as f64).to_json_value());
    }

    if let Some(v) = Value::get_value::<&StringValue>(o.as_ref()) {
        let text = if v.is_newline {
            "\n".to_string()
        } else {
            v.string.clone()
        };
        return Ok(format::Object::String(text).to_json_value());
    }

    if let Some(v) = Value::get_value::<&Path>(o.as_ref()) {
        return Ok(format::Object::DivertTarget(v.get_components_string()).to_json_value());
    }

    if let Some(v) = Value::get_value::<&VariablePointerValue>(o.as_ref()) {
        return Ok(format::Object::VariablePointer {
            name: v.variable_name.clone(),
            context_index: v.context_index,
        }
        .to_json_value());
    }

    if o.as_any().is::<Glue>() {
        return Ok(format::Object::Glue.to_json_value());
    }

    if let Some(cc) = o.as_any().downcast_ref::<ControlCommand>() {
        let name = ControlCommand::get_name(cc.command_type);
        let object = format::Object::from_json_value(serde_json::Value::String(name.clone()))
            .map_err(|_| {
                StoryError::BadJson(format!("Unsupported control command token: {name}"))
            })?;
        let object = match object {
            format::Object::ControlCommand(_) | format::Object::Tag { .. } => object,
            _ => {
                return Err(StoryError::BadJson(format!(
                    "Unsupported control command token: {name}"
                )))
            }
        };
        return Ok(object.to_json_value());
    }

    if let Some(f) = o.as_any().downcast_ref::<NativeFunctionCall>() {
        return Ok(
            format::Object::NativeFunction(NativeFunctionCall::get_name(f.op)).to_json_value(),
        );
    }

    if let Ok(var_ref) = o.clone().into_any().downcast::<VariableReference>() {
        if let Some(read_count_path) = var_ref.get_path_string_for_count() {
            return Ok(format::Object::ReadCount(read_count_path).to_json_value());
        } else {
            return Ok(format::Object::VariableReference(var_ref.name.clone()).to_json_value());
        }
    }

    if let Some(var_ass) = o.as_any().downcast_ref::<VariableAssignment>() {
        let object = if var_ass.is_new_declaration && var_ass.is_global {
            format::Object::GlobalVariableAssignment(var_ass.variable_name.clone())
        } else if var_ass.is_new_declaration {
            format::Object::VariableAssignment(var_ass.variable_name.clone())
        } else if var_ass.is_global {
            format::Object::VariableReassignment(var_ass.variable_name.clone())
        } else {
            format::Object::TempVariableReassignment(var_ass.variable_name.clone())
        };
        return Ok(object.to_json_value());
    }

    if o.as_any().is::<Void>() {
        return Ok(format::Object::Void.to_json_value());
    }

    if let Some(tag) = o.as_any().downcast_ref::<Tag>() {
        let mut jobj: Map<String, serde_json::Value> = Map::new();

        jobj.insert("#".to_owned(), json!(tag.get_text()));

        return Ok(serde_json::Value::Object(jobj));
    }

    if let Some(choice) = o.as_any().downcast_ref::<Choice>() {
        return Ok(write_choice(choice));
    }

    Err(StoryError::BadJson(format!(
        "Failed to write runtime object to JSON: {}",
        o
    )))
}

pub fn write_rt_container(
    container: &Container,
    without_name: bool,
) -> Result<serde_json::Value, StoryError> {
    let mut format_container = runtime_container_to_format(container, container.name.clone())?;
    if without_name {
        format_container.name = None;
    }

    Ok(format_container.to_json_value_with_name(!without_name))
}

fn runtime_container_to_format(
    container: &Container,
    name: Option<String>,
) -> Result<format::Container, StoryError> {
    let content = container
        .content
        .iter()
        .map(|object| runtime_object_to_format_object(object.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    let named_content = container
        .get_named_only_content()
        .iter()
        .map(|(name, container)| {
            runtime_container_to_format(container.as_ref(), Some(name.clone()))
                .map(|container| format::NamedContainer::new(name.clone(), container))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let flags = match container.get_count_flags() {
        0 => None,
        flags => Some(flags),
    };

    Ok(format::Container {
        content,
        named_content,
        name,
        flags,
    })
}

fn runtime_object_to_format_object(object: Rc<dyn RTObject>) -> Result<format::Object, StoryError> {
    let value = write_rtobject(object)?;
    format::Object::from_json_value(value).map_err(|error| StoryError::BadJson(error.to_string()))
}

pub fn write_choice(choice: &Choice) -> serde_json::Value {
    let mut jobj: Map<String, serde_json::Value> = Map::new();

    jobj.insert("text".to_owned(), json!(choice.text));
    jobj.insert("index".to_owned(), json!(*choice.index.borrow()));
    jobj.insert("originalChoicePath".to_owned(), json!(choice.source_path));
    jobj.insert(
        "originalThreadIndex".to_owned(),
        json!(choice.original_thread_index),
    );
    jobj.insert(
        "targetPath".to_owned(),
        json!(choice.target_path.to_string()),
    );

    jobj.insert("tags".to_owned(), write_choice_tags(choice));

    serde_json::Value::Object(jobj)
}

fn write_choice_tags(choice: &Choice) -> serde_json::Value {
    let mut tags: Vec<serde_json::Value> = Vec::new();
    for t in &choice.tags {
        tags.push(json!(t));
    }

    serde_json::Value::Array(tags)
}

pub(crate) fn write_list_rt_objs(
    objs: &[Rc<dyn RTObject>],
) -> Result<serde_json::Value, StoryError> {
    let mut c_array: Vec<serde_json::Value> = Vec::new();

    for o in objs {
        c_array.push(write_rtobject(o.clone())?);
    }

    Ok(serde_json::Value::Array(c_array))
}

pub(crate) fn write_int_dictionary(map: &HashMap<String, i32>) -> serde_json::Value {
    let mut jobj: Map<String, serde_json::Value> = Map::new();

    for (key, val) in map {
        jobj.insert(key.clone(), json!(*val));
    }

    serde_json::Value::Object(jobj)
}
