use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use ink_story_json_format as format;
use serde_json::{Map, Value as JsonValue};

use crate::{
    choice_point::ChoicePoint,
    container::Container,
    control_command::ControlCommand,
    divert::Divert,
    dynamic_interface::{
        DynamicInterfaceFunctionCall, DynamicInterfaceRegistry, DynamicInterfaceTarget,
    },
    glue::Glue,
    native_function_call::NativeFunctionCall,
    object::RTObject,
    path::Path,
    push_pop::PushPopType,
    story::INK_VERSION_CURRENT,
    story_error::StoryError,
    tag::Tag,
    value::Value,
    value_type::{DictKey, DictKeyType, DictValue, ValueType},
    variable_assigment::VariableAssignment,
    variable_reference::VariableReference,
    void::Void,
};

pub(crate) struct LoadedProgram {
    pub(crate) main_content_container: Rc<Container>,
    pub(crate) internal_functions: BTreeMap<String, format::InternalFunction>,
    pub(crate) dynamic_interfaces: DynamicInterfaceRegistry,
}

#[cfg(test)]
pub fn load_from_string(s: &str) -> Result<Rc<Container>, StoryError> {
    Ok(load_program_from_string(s)?.main_content_container)
}

pub(crate) fn load_program_from_string(s: &str) -> Result<LoadedProgram, StoryError> {
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

    Ok(LoadedProgram {
        main_content_container,
        internal_functions: program.internal_functions,
        dynamic_interfaces: DynamicInterfaceRegistry::from_format(program.interfaces),
    })
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
            Ok(Rc::new(NativeFunctionCall::new_from_format(*function)))
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
        format::Object::DynamicInterfaceTarget { interface, member } => Ok(Rc::new(
            DynamicInterfaceTarget::new(interface.clone(), member.clone()),
        )),
        format::Object::DynamicInterfaceFunctionCall {
            interface,
            member,
            args,
        } => Ok(Rc::new(DynamicInterfaceFunctionCall::new(
            interface.clone(),
            member.clone(),
            *args,
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
            let token = if *is_start {
                format::TAG_START_TOKEN
            } else {
                format::TAG_END_TOKEN
            };
            ControlCommand::new_from_name(token)
                .map(|command| Rc::new(command) as Rc<dyn RTObject>)
                .ok_or_else(|| {
                    StoryError::BadJson(format!("Unsupported control command token: {token}"))
                })
        }
        format::Object::ValueArray(_)
        | format::Object::ValueObject(_)
        | format::Object::ValueDict(_) => Ok(Rc::new(Value::new_value_type(
            format_object_to_runtime_value(object)?,
        ))),
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
        format::Object::ValueDict(dict) => format_dict_to_runtime_value(dict),
        _ => Err(StoryError::BadJson(format!(
            "Unsupported value object in dynamic value: {:?}",
            object
        ))),
    }
}

fn format_dict_to_runtime_value(dict: &format::DictValue) -> Result<ValueType, StoryError> {
    let key_type = format_dict_key_type_to_runtime(dict.key_type);
    let entries = dict
        .entries
        .iter()
        .map(|(key, value)| -> Result<(DictKey, ValueType), StoryError> {
            Ok((
                format_dict_key_to_runtime(key),
                format_object_to_runtime_value(value)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    DictValue::new(key_type, entries).map(ValueType::Dict)
}

fn format_dict_key_type_to_runtime(key_type: format::DictKeyType) -> DictKeyType {
    match key_type {
        format::DictKeyType::String => DictKeyType::String,
        format::DictKeyType::Int => DictKeyType::Int,
    }
}

fn format_dict_key_to_runtime(key: &format::DictKey) -> DictKey {
    match key {
        format::DictKey::String(value) => DictKey::String(value.clone()),
        format::DictKey::Int(value) => DictKey::Int(*value),
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
    token: &JsonValue,
    name: Option<String>,
) -> Result<Rc<dyn RTObject>, StoryError> {
    if token.is_array() && name.is_some() {
        return jarray_to_container(json_array(token, "container token")?, name);
    }

    if token.is_object() {
        let obj = json_object(token, "runtime object token")?;
        if obj.get("#").is_some() {
            return Ok(Rc::new(Tag::new(required_string(obj, "#")?)));
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
    jarray: &[JsonValue],
    name: Option<String>,
) -> Result<Rc<dyn RTObject>, StoryError> {
    let container_value = JsonValue::Array(jarray.to_vec());
    let container = format::Container::from_json_value(container_value, name)
        .map_err(|error| StoryError::BadJson(error.to_string()))?;
    let runtime_container: Rc<dyn RTObject> = format_container_to_runtime(&container)?;
    Ok(runtime_container)
}

pub(crate) fn jobject_to_hashmap_values(
    jobj: &Map<String, serde_json::Value>,
) -> Result<HashMap<String, Rc<Value>>, StoryError> {
    let mut dict: HashMap<String, Rc<Value>> = HashMap::new();

    for (k, v) in jobj.iter() {
        let value = jtoken_to_runtime_object(v, None)?
            .into_any()
            .downcast::<Value>()
            .map_err(|_| {
                StoryError::BadJson(format!("JSON field '{k}' must decode to a runtime value"))
            })?;
        dict.insert(k.clone(), value);
    }

    Ok(dict)
}

fn required_value<'a>(
    obj: &'a Map<String, JsonValue>,
    field: &str,
) -> Result<&'a JsonValue, StoryError> {
    obj.get(field)
        .ok_or_else(|| StoryError::BadJson(format!("Missing required JSON field '{field}'")))
}

fn required_string<'a>(
    obj: &'a Map<String, JsonValue>,
    field: &str,
) -> Result<&'a str, StoryError> {
    required_value(obj, field)?
        .as_str()
        .ok_or_else(|| StoryError::BadJson(format!("JSON field '{field}' must be a string")))
}

fn json_array<'a>(value: &'a JsonValue, context: &str) -> Result<&'a Vec<JsonValue>, StoryError> {
    value
        .as_array()
        .ok_or_else(|| StoryError::BadJson(format!("{context} must be an array")))
}

fn json_object<'a>(
    value: &'a JsonValue,
    context: &str,
) -> Result<&'a Map<String, JsonValue>, StoryError> {
    value
        .as_object()
        .ok_or_else(|| StoryError::BadJson(format!("{context} must be an object")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic_interface::{
        DynamicInterfaceFunctionCall, DynamicInterfaceMemberKind, DynamicInterfaceTarget,
    };
    use crate::native_function_call::{NativeFunctionCall, Op};
    use crate::story::Story;
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
    fn loads_native_function_tokens_as_runtime_calls() {
        let json = r#"{"inkVersion":1,"root":["LEN","done",null]}"#;

        let root = load_from_string(json).expect("story JSON should load");
        let function = root.content[0]
            .as_any()
            .downcast_ref::<NativeFunctionCall>()
            .expect("first root object should be a native function");

        assert_eq!(function.op, Op::Len);
    }

    #[test]
    fn loads_dynamic_interface_objects_and_metadata() {
        let json = r#"{
            "inkVersion": 1,
            "root": [
                "^left",
                {"i->": "target", "interface": "IItem"},
                {"i()": "score", "interface": "IItem", "args": 1},
                "done",
                null
            ],
            "interfaces": {
                "IItem": {
                    "members": {
                        "target": "knot",
                        "score": "function"
                    },
                    "implementations": ["left", "right"]
                }
            }
        }"#;

        let loaded = load_program_from_string(json).expect("story JSON should load");
        let target = loaded.main_content_container.content[1]
            .as_any()
            .downcast_ref::<DynamicInterfaceTarget>()
            .expect("target instruction should load");
        let call = loaded.main_content_container.content[2]
            .as_any()
            .downcast_ref::<DynamicInterfaceFunctionCall>()
            .expect("function instruction should load");

        assert_eq!(target.interface(), "IItem");
        assert_eq!(target.member(), "target");
        assert_eq!(call.interface(), "IItem");
        assert_eq!(call.member(), "score");
        assert_eq!(call.args(), 1);
        assert_eq!(
            loaded
                .dynamic_interfaces
                .interface("IItem")
                .and_then(|interface| interface.member_kind("target")),
            Some(DynamicInterfaceMemberKind::Knot)
        );
        assert_eq!(
            loaded
                .dynamic_interfaces
                .interface("IItem")
                .and_then(|interface| interface.member_kind("score")),
            Some(DynamicInterfaceMemberKind::Function)
        );
        let _ = Story::new(json).expect("public Story loader should accept dynamic interfaces");
    }

    #[test]
    fn rejects_unknown_native_function_tokens_from_story_json() {
        let json = r#"{"inkVersion":1,"root":["UNKNOWN_NATIVE",null]}"#;

        let error = match load_from_string(json) {
            Ok(_) => panic!("expected unsupported native token"),
            Err(error) => error,
        };

        assert!(error
            .to_string()
            .contains("unsupported native function token: UNKNOWN_NATIVE"));
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
    fn loads_dynamic_dict_values_from_story_json() {
        let json = r#"{
            "inkVersion": 1,
            "root": [
                ["dict", "string", [["ada", 10], ["items", [1, true]]]],
                ["dict", "int", [[1, "^one"]]],
                "done",
                null
            ]
        }"#;

        let root = load_from_string(json).expect("story JSON should load");
        let string_value = root.content[0]
            .as_any()
            .downcast_ref::<Value>()
            .expect("first root object should be a value");
        let int_value = root.content[1]
            .as_any()
            .downcast_ref::<Value>()
            .expect("second root object should be a value");

        let ValueType::Dict(string_dict) = &string_value.value else {
            panic!("expected string-key dict value");
        };
        assert_eq!(string_dict.key_type(), DictKeyType::String);
        assert!(matches!(
            string_dict.get(&DictKey::String("ada".to_string())),
            Some(ValueType::Int(10))
        ));
        assert!(matches!(
            string_dict.get(&DictKey::String("items".to_string())),
            Some(ValueType::Array(_))
        ));

        let ValueType::Dict(int_dict) = &int_value.value else {
            panic!("expected int-key dict value");
        };
        assert_eq!(int_dict.key_type(), DictKeyType::Int);
        assert!(
            matches!(int_dict.get(&DictKey::Int(1)), Some(ValueType::String(value)) if value.string == "one")
        );
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

    #[test]
    fn rejects_non_string_save_state_tag_text() {
        let error = runtime_object_error(&json!({ "#": 7 }));

        assert!(error.contains("JSON field '#' must be a string"));
    }

    #[test]
    fn loads_value_dictionary_save_state_entries() {
        let value = json!({
            "health": 3,
            "items": [1, true],
            "player": {
                "hp": 10
            }
        });
        let dictionary = jobject_to_hashmap_values(value.as_object().expect("object"))
            .expect("dictionary should load");

        assert!(matches!(
            dictionary.get("health").map(|value| &value.value),
            Some(ValueType::Int(3))
        ));
        assert!(matches!(
            dictionary.get("items").map(|value| &value.value),
            Some(ValueType::Array(_))
        ));
        assert!(matches!(
            dictionary.get("player").map(|value| &value.value),
            Some(ValueType::Object(_))
        ));
    }

    #[test]
    fn rejects_non_value_dictionary_save_state_entries() {
        let value = json!({
            "bad": "done"
        });
        let error = match jobject_to_hashmap_values(value.as_object().expect("object")) {
            Ok(_) => panic!("expected non-value dictionary entry to fail"),
            Err(StoryError::BadJson(message)) => message,
            Err(error) => panic!("expected BadJson, got {error:?}"),
        };

        assert!(error.contains("JSON field 'bad' must decode to a runtime value"));
    }

    fn runtime_object_error(token: &serde_json::Value) -> String {
        match jtoken_to_runtime_object(token, None) {
            Ok(_) => panic!("expected malformed runtime object JSON to fail"),
            Err(StoryError::BadJson(message)) => message,
            Err(error) => panic!("expected BadJson, got {error:?}"),
        }
    }
}
