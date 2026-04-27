use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use ink_story_json_format as format;
use serde_json::Map;

use crate::{
    choice::Choice, choice_point::ChoicePoint, container::Container,
    control_command::ControlCommand, divert::Divert, glue::Glue,
    native_function_call::NativeFunctionCall, object::RTObject, path::Path, push_pop::PushPopType,
    story::INK_VERSION_CURRENT, story_error::StoryError, tag::Tag, value::Value,
    value_type::ValueType, variable_assigment::VariableAssignment,
    variable_reference::VariableReference, void::Void,
};

pub fn load_from_string(s: &str) -> Result<Rc<Container>, StoryError> {
    let program = format::Program::from_json_str(s)
        .map_err(|error| StoryError::BadJson(error.to_string()))?;
    let version = program.ink_version;

    if version != INK_VERSION_CURRENT {
        return Err(StoryError::BadJson(format!(
            "Story JSON format version mismatch: expected {}, got {}.",
            INK_VERSION_CURRENT, version
        )));
    }

    let main_content_container = format_container_to_runtime(&program.root)?;

    Ok(main_content_container)
}

fn format_container_to_runtime(container: &format::Container) -> Result<Rc<Container>, StoryError> {
    let mut content = Vec::with_capacity(container.content.len());
    for object in &container.content {
        content.push(format_object_to_runtime(object)?);
    }

    let mut named_content = HashMap::new();
    for named in &container.named_content {
        named_content.insert(
            named.name.clone(),
            format_container_to_runtime(&named.container)?,
        );
    }

    Ok(Container::new(
        container.name.clone(),
        container.flags.unwrap_or(0),
        content,
        named_content,
    ))
}

fn format_object_to_runtime(object: &format::Object) -> Result<Rc<dyn RTObject>, StoryError> {
    match object {
        format::Object::Container(container) => Ok(format_container_to_runtime(container)?),
        format::Object::String(value) => Ok(Rc::new(Value::new::<&str>(value))),
        format::Object::Bool(value) => Ok(Rc::new(Value::new::<bool>(*value))),
        format::Object::Int(value) => Ok(Rc::new(Value::new::<i32>(*value))),
        format::Object::Float(value) => Ok(Rc::new(Value::new::<f32>(*value as f32))),
        format::Object::DivertTarget(target) => Ok(Rc::new(Value::new::<Path>(
            Path::new_with_components_string(Some(target)),
        ))),
        format::Object::VariablePointer {
            name,
            context_index,
        } => Ok(Rc::new(Value::new_variable_pointer(name, *context_index))),
        format::Object::ControlCommand(command) => {
            let token = command.token();
            ControlCommand::new_from_name(token)
                .map(|command| Rc::new(command) as Rc<dyn RTObject>)
                .ok_or_else(|| {
                    StoryError::BadJson(format!("Unsupported control command token: {token}"))
                })
        }
        format::Object::NativeFunction(function) => {
            let token = function.token();
            NativeFunctionCall::new_from_name(token)
                .map(|function| Rc::new(function) as Rc<dyn RTObject>)
                .ok_or_else(|| {
                    StoryError::BadJson(format!("Unsupported native function token: {token}"))
                })
        }
        format::Object::Divert { target, variable } => {
            Ok(Rc::new(format_divert_to_runtime(format::Object::Divert {
                target: target.clone(),
                variable: *variable,
            })))
        }
        format::Object::TunnelDivert { target, variable } => Ok(Rc::new(format_divert_to_runtime(
            format::Object::TunnelDivert {
                target: target.clone(),
                variable: *variable,
            },
        ))),
        format::Object::FunctionDivert { target } => Ok(Rc::new(format_divert_to_runtime(
            format::Object::FunctionDivert {
                target: target.clone(),
            },
        ))),
        format::Object::ExternalFunction { target, args } => Ok(Rc::new(format_divert_to_runtime(
            format::Object::ExternalFunction {
                target: target.clone(),
                args: *args,
            },
        ))),
        format::Object::ConditionalDivert { target } => Ok(Rc::new(format_divert_to_runtime(
            format::Object::ConditionalDivert {
                target: target.clone(),
            },
        ))),
        format::Object::ChoicePoint { target, flags } => {
            Ok(Rc::new(ChoicePoint::new(*flags, target)))
        }
        format::Object::VariableAssignment(name) => {
            Ok(Rc::new(VariableAssignment::new(name, true, false)))
        }
        format::Object::GlobalVariableAssignment(name) => {
            Ok(Rc::new(VariableAssignment::new(name, true, true)))
        }
        format::Object::TempVariableReassignment(name) => {
            Ok(Rc::new(VariableAssignment::new(name, false, false)))
        }
        format::Object::VariableReassignment(name) => {
            Ok(Rc::new(VariableAssignment::new(name, false, true)))
        }
        format::Object::VariableReference(name) => Ok(Rc::new(VariableReference::new(name))),
        format::Object::ReadCount(target) => {
            Ok(Rc::new(VariableReference::from_path_for_count(target)))
        }
        format::Object::Glue => Ok(Rc::new(Glue::new())),
        format::Object::Tag { is_start } => {
            let token_value = format::Object::Tag {
                is_start: *is_start,
            }
            .to_json_value();
            let token = token_value
                .as_str()
                .expect("format tag object must serialize to a string token");
            ControlCommand::new_from_name(token)
                .map(|command| Rc::new(command) as Rc<dyn RTObject>)
                .ok_or_else(|| {
                    StoryError::BadJson(format!("Unsupported control command token: {token}"))
                })
        }
        format::Object::ValueArray(_) | format::Object::ValueObject(_) => Ok(Rc::new(
            Value::new_value_type(format_object_to_runtime_value(object)?),
        )),
        format::Object::Void => Ok(Rc::new(Void::new())),
    }
}

fn format_object_to_runtime_value(object: &format::Object) -> Result<ValueType, StoryError> {
    match object {
        format::Object::String(value) => Ok(ValueType::new(value.as_str())),
        format::Object::Bool(value) => Ok(ValueType::Bool(*value)),
        format::Object::Int(value) => Ok(ValueType::Int(*value)),
        format::Object::Float(value) => Ok(ValueType::Float(*value as f32)),
        format::Object::DivertTarget(target) => Ok(ValueType::DivertTarget(
            Path::new_with_components_string(Some(target)),
        )),
        format::Object::VariablePointer {
            name,
            context_index,
        } => Ok(Value::new_variable_pointer(name, *context_index).value),
        format::Object::ValueArray(values) => values
            .iter()
            .map(format_object_to_runtime_value)
            .collect::<Result<Vec<_>, _>>()
            .map(ValueType::Array),
        format::Object::ValueObject(fields) => fields
            .iter()
            .map(|(name, value)| Ok((name.clone(), format_object_to_runtime_value(value)?)))
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map(ValueType::Object),
        _ => Err(StoryError::BadJson(format!(
            "Unsupported value object in dynamic value: {:?}",
            object
        ))),
    }
}

fn format_divert_to_runtime(divert: format::Object) -> Divert {
    let (target, variable, conditional, external, external_args, pushes_to_stack, div_push_type) =
        match divert {
            format::Object::Divert { target, variable } => (
                target,
                variable,
                false,
                false,
                0,
                false,
                PushPopType::Function,
            ),
            format::Object::TunnelDivert { target, variable } => {
                (target, variable, false, false, 0, true, PushPopType::Tunnel)
            }
            format::Object::FunctionDivert { target } => {
                (target, false, false, false, 0, true, PushPopType::Function)
            }
            format::Object::ExternalFunction { target, args } => (
                target,
                false,
                false,
                true,
                args,
                false,
                PushPopType::Function,
            ),
            format::Object::ConditionalDivert { target } => {
                (target, false, true, false, 0, false, PushPopType::Function)
            }
            _ => unreachable!("format divert conversion requires a divert object"),
        };
    let var_divert_name = if variable { Some(target.clone()) } else { None };
    let target_path = if variable {
        None
    } else {
        Some(target.as_str())
    };

    Divert::new(
        pushes_to_stack,
        div_push_type,
        external,
        external_args,
        conditional,
        var_divert_name,
        target_path,
    )
}

pub fn jtoken_to_runtime_object(
    token: &serde_json::Value,
    name: Option<String>,
) -> Result<Rc<dyn RTObject>, StoryError> {
    if let serde_json::Value::Array(value) = token {
        if name.is_some() {
            return jarray_to_container(value, name);
        }
    }

    if let serde_json::Value::Object(obj) = token {
        if obj.get("originalChoicePath").is_some() {
            return jobject_to_choice(obj);
        }

        if let Some(prop_value) = obj.get("#") {
            return Ok(Rc::new(Tag::new(prop_value.as_str().unwrap())));
        }
    }

    let object = format::Object::from_json_value(token.clone()).map_err(|_| {
        StoryError::BadJson(format!(
            "Failed to convert token to runtime RTObject: {}",
            token
        ))
    })?;
    format_object_to_runtime(&object)
}

fn jarray_to_container(
    jarray: &[serde_json::Value],
    name: Option<String>,
) -> Result<Rc<dyn RTObject>, StoryError> {
    let container_value = serde_json::Value::Array(jarray.to_vec());
    let container = format::Container::from_json_value(container_value, name)
        .map_err(|error| StoryError::BadJson(error.to_string()))?;
    let runtime_container: Rc<dyn RTObject> = format_container_to_runtime(&container)?;
    Ok(runtime_container)
}

pub fn jarray_to_runtime_obj_list(
    jarray: &Vec<serde_json::Value>,
    skip_last: bool,
) -> Result<Vec<Rc<dyn RTObject>>, StoryError> {
    let mut count = jarray.len();

    if skip_last {
        count -= 1;
    }

    let mut list: Vec<Rc<dyn RTObject>> = Vec::with_capacity(jarray.len());

    for jtok in jarray.iter().take(count) {
        let runtime_obj = jtoken_to_runtime_object(jtok, None);
        list.push(runtime_obj?);
    }

    Ok(list)
}

fn jobject_to_choice(obj: &Map<String, serde_json::Value>) -> Result<Rc<dyn RTObject>, StoryError> {
    let text = obj.get("text").unwrap().as_str().unwrap();
    let index = obj.get("index").unwrap().as_u64().unwrap() as usize;
    let source_path = obj.get("originalChoicePath").unwrap().as_str().unwrap();
    let original_thread_index = obj.get("originalThreadIndex").unwrap().as_i64().unwrap() as usize;
    let path_string_on_choice = obj.get("targetPath").unwrap().as_str().unwrap();
    let choice_tags = jarray_to_tags(obj);

    Ok(Rc::new(Choice::new_from_json(
        path_string_on_choice,
        source_path.to_string(),
        text,
        index,
        original_thread_index,
        choice_tags,
    )))
}

fn jarray_to_tags(obj: &Map<String, serde_json::Value>) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();

    let prop_value = obj.get("tags");
    if let Some(pv) = prop_value {
        let tags_array = pv.as_array().unwrap();
        for tag in tags_array {
            tags.push(tag.as_str().unwrap().to_string());
        }
    }

    tags
}

pub(crate) fn jobject_to_hashmap_values(
    jobj: &Map<String, serde_json::Value>,
) -> Result<HashMap<String, Rc<Value>>, StoryError> {
    let mut dict: HashMap<String, Rc<Value>> = HashMap::new();

    for (k, v) in jobj.iter() {
        dict.insert(
            k.clone(),
            jtoken_to_runtime_object(v, None)?
                .into_any()
                .downcast::<Value>()
                .unwrap(),
        );
    }

    Ok(dict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn loads_current_story_json_version() {
        let json = r#"{"inkVersion":1,"root":["done",null]}"#;

        assert!(load_from_string(json).is_ok());
    }

    #[test]
    fn rejects_non_current_story_json_version() {
        let json = r#"{"inkVersion":21,"root":["done",null]}"#;

        let error = match load_from_string(json) {
            Ok(_) => panic!("expected version mismatch"),
            Err(error) => error,
        };

        assert!(error
            .to_string()
            .contains("Story JSON format version mismatch"));
    }

    #[test]
    fn loads_dynamic_array_values_from_story_json() {
        let json = r#"{
            "inkVersion": 1,
            "root": [
                [1, true, {"hp": 10}, ["^nested"]],
                "done",
                null
            ]
        }"#;

        let root = load_from_string(json).expect("story JSON should load");
        let value = root.content[0]
            .as_any()
            .downcast_ref::<Value>()
            .expect("first root object should be a value");

        let ValueType::Array(values) = &value.value else {
            panic!("expected array value");
        };
        assert!(matches!(values[0], ValueType::Int(1)));
        assert!(matches!(values[1], ValueType::Bool(true)));
        assert!(matches!(values[2], ValueType::Object(_)));
        assert!(matches!(values[3], ValueType::Array(_)));
    }

    #[test]
    fn loads_dynamic_object_values_from_runtime_tokens() {
        let runtime_object = jtoken_to_runtime_object(
            &json!({
                "name": "^Ada",
                "stats": {
                    "hp": 10,
                    "flags": [true, false]
                }
            }),
            None,
        )
        .expect("object token should load");
        let value = runtime_object
            .as_any()
            .downcast_ref::<Value>()
            .expect("runtime object should be a value");

        let ValueType::Object(fields) = &value.value else {
            panic!("expected object value");
        };
        assert!(matches!(fields.get("name"), Some(ValueType::String(_))));
        assert!(matches!(fields.get("stats"), Some(ValueType::Object(_))));
    }
}
