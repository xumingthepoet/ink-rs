use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use ink_story_json_format as format;
use serde_json::{json, Map};

use crate::{
    choice::Choice,
    choice_point::ChoicePoint,
    container::Container,
    control_command::{CommandType, ControlCommand},
    divert::Divert,
    glue::Glue,
    native_function_call::NativeFunctionCall,
    object::RTObject,
    push_pop::PushPopType,
    story_error::StoryError,
    tag::Tag,
    value::Value,
    value_type::ValueType,
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
    if let Some(tag) = o.as_any().downcast_ref::<Tag>() {
        let mut jobj: Map<String, serde_json::Value> = Map::new();

        jobj.insert("#".to_owned(), json!(tag.get_text()));

        return Ok(serde_json::Value::Object(jobj));
    }

    if let Some(choice) = o.as_any().downcast_ref::<Choice>() {
        return Ok(write_choice(choice));
    }

    if let Some(c) = o.as_any().downcast_ref::<Container>() {
        return write_rt_container(c, false);
    }

    Ok(runtime_object_to_format_object(o)?.to_json_value())
}

fn runtime_object_to_format_object(object: Rc<dyn RTObject>) -> Result<format::Object, StoryError> {
    if let Some(c) = object.as_any().downcast_ref::<Container>() {
        return runtime_container_to_format(c, c.name.clone()).map(format::Object::Container);
    }

    if let Ok(divert) = object.clone().into_any().downcast::<Divert>() {
        return divert_to_format_object(&divert);
    }

    if let Ok(cp) = object.clone().into_any().downcast::<ChoicePoint>() {
        return Ok(format::Object::ChoicePoint {
            target: ChoicePoint::get_path_string_on_choice(&cp),
            flags: cp.get_flags(),
        });
    }

    if let Some(v) = Value::get_bool_value(object.as_ref()) {
        return value_type_to_format_object(&ValueType::Bool(v));
    }

    if let Some(v) = object.as_any().downcast_ref::<Value>() {
        return value_type_to_format_object(&v.value);
    }

    if object.as_any().is::<Glue>() {
        return Ok(format::Object::Glue);
    }

    if let Some(cc) = object.as_any().downcast_ref::<ControlCommand>() {
        return control_command_to_format_object(cc);
    }

    if let Some(f) = object.as_any().downcast_ref::<NativeFunctionCall>() {
        return Ok(format::Object::NativeFunction(f.format_function()));
    }

    if let Ok(var_ref) = object.clone().into_any().downcast::<VariableReference>() {
        if let Some(read_count_path) = var_ref.get_path_string_for_count() {
            return Ok(format::Object::ReadCount(read_count_path));
        } else {
            return Ok(format::Object::VariableReference(var_ref.name.clone()));
        }
    }

    if let Some(var_ass) = object.as_any().downcast_ref::<VariableAssignment>() {
        return Ok(if var_ass.is_new_declaration && var_ass.is_global {
            format::Object::GlobalVariableAssignment(var_ass.variable_name.clone())
        } else if var_ass.is_new_declaration {
            format::Object::VariableAssignment(var_ass.variable_name.clone())
        } else if var_ass.is_global {
            format::Object::VariableReassignment(var_ass.variable_name.clone())
        } else {
            format::Object::TempVariableReassignment(var_ass.variable_name.clone())
        });
    }

    if object.as_any().is::<Void>() {
        return Ok(format::Object::Void);
    }

    Err(StoryError::BadJson(format!(
        "Failed to write runtime object to JSON: {}",
        object
    )))
}

fn divert_to_format_object(divert: &Rc<Divert>) -> Result<format::Object, StoryError> {
    let target_str = if divert.has_variable_target() {
        divert.variable_divert_name.clone().ok_or_else(|| {
            StoryError::InvalidStoryState(
                "Variable divert is missing its variable target name".to_owned(),
            )
        })?
    } else {
        divert.get_target_path_string().ok_or_else(|| {
            StoryError::InvalidStoryState("Divert is missing its target path".to_owned())
        })?
    };

    if divert.is_external {
        Ok(format::Object::ExternalFunction {
            target: target_str,
            args: divert.external_args,
        })
    } else if divert.pushes_to_stack && divert.stack_push_type == PushPopType::Function {
        Ok(format::Object::FunctionDivert { target: target_str })
    } else if divert.pushes_to_stack && divert.stack_push_type == PushPopType::Tunnel {
        Ok(format::Object::TunnelDivert {
            target: target_str,
            variable: divert.has_variable_target(),
        })
    } else if divert.is_conditional {
        Ok(format::Object::ConditionalDivert { target: target_str })
    } else {
        Ok(format::Object::Divert {
            target: target_str,
            variable: divert.has_variable_target(),
        })
    }
}

fn control_command_to_format_object(
    command: &ControlCommand,
) -> Result<format::Object, StoryError> {
    match command.command_type {
        CommandType::BeginTag => Ok(format::Object::Tag { is_start: true }),
        CommandType::EndTag => Ok(format::Object::Tag { is_start: false }),
        command_type => {
            let name = ControlCommand::get_name(command_type);
            format::ControlCommand::from_token(&name)
                .map(format::Object::ControlCommand)
                .ok_or_else(|| {
                    StoryError::BadJson(format!("Unsupported control command token: {name}"))
                })
        }
    }
}

fn value_type_to_format_object(value: &ValueType) -> Result<format::Object, StoryError> {
    match value {
        ValueType::Bool(value) => Ok(format::Object::Bool(*value)),
        ValueType::Int(value) => Ok(format::Object::Int(*value)),
        ValueType::Float(value) => Ok(format::Object::Float(*value as f64)),
        ValueType::String(value) => {
            let text = if value.is_newline {
                "\n".to_string()
            } else {
                value.string.clone()
            };
            Ok(format::Object::String(text))
        }
        ValueType::DivertTarget(value) => {
            Ok(format::Object::DivertTarget(value.get_components_string()))
        }
        ValueType::VariablePointer(value) => Ok(format::Object::VariablePointer {
            name: value.variable_name.clone(),
            context_index: value.context_index,
        }),
        ValueType::Array(values) => values
            .iter()
            .map(value_type_to_format_object)
            .collect::<Result<Vec<_>, _>>()
            .map(format::Object::ValueArray),
        ValueType::Object(fields) => fields
            .iter()
            .map(|(name, value)| Ok((name.clone(), value_type_to_format_object(value)?)))
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map(format::Object::ValueObject),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    use crate::native_function_call::{NativeFunctionCall, Op};
    use crate::object::Object;
    use crate::push_pop::PushPopType;

    #[test]
    fn writes_native_function_calls_through_format_native_functions() {
        let object: Rc<dyn RTObject> = Rc::new(NativeFunctionCall::new(Op::Len));

        assert_eq!(write_rtobject(object.clone()).unwrap(), json!("LEN"));
        assert_eq!(
            runtime_object_to_format_object(object).unwrap(),
            format::Object::NativeFunction(format::NativeFunction::Len)
        );
    }

    #[test]
    fn converts_runtime_containers_to_format_objects_without_json_reparse() {
        let named_child = Container::new(
            Some("knot".to_string()),
            0,
            vec![Rc::new(Value::new::<&str>("Nested")) as Rc<dyn RTObject>],
            HashMap::new(),
        );
        let mut named_content = HashMap::new();
        named_content.insert("knot".to_string(), named_child);
        let root = Container::new(
            Some("root".to_string()),
            1,
            vec![
                Rc::new(Value::new::<&str>("Line.")) as Rc<dyn RTObject>,
                Rc::new(ControlCommand::new(CommandType::BeginTag)) as Rc<dyn RTObject>,
                Rc::new(ControlCommand::new(CommandType::Done)) as Rc<dyn RTObject>,
            ],
            named_content,
        );
        let root_object: Rc<dyn RTObject> = root.clone();

        let format_object = runtime_object_to_format_object(root_object.clone())
            .expect("runtime container should convert to format object");

        assert_eq!(
            format_object.to_json_value(),
            write_rtobject(root_object).expect("runtime object should write")
        );
        assert_eq!(
            format_object.to_json_value(),
            write_rt_container(root.as_ref(), false).expect("runtime container should write")
        );
        let format::Object::Container(format_container) = format_object else {
            panic!("expected format container object");
        };
        assert_eq!(format_container.name.as_deref(), Some("root"));
        assert_eq!(format_container.flags, Some(1));
        assert_eq!(
            format_container.content,
            vec![
                format::Object::String("Line.".to_string()),
                format::Object::Tag { is_start: true },
                format::Object::ControlCommand(format::ControlCommand::Done),
            ]
        );
        assert_eq!(format_container.named_content.len(), 1);
        assert_eq!(format_container.named_content[0].name, "knot");
        assert_eq!(
            format_container.named_content[0].container.content,
            vec![format::Object::String("Nested".to_string())]
        );
    }

    #[test]
    fn value_dictionaries_write_and_read_save_state_values() {
        let mut fields = BTreeMap::new();
        fields.insert("hp".to_string(), ValueType::Int(10));
        let mut values = HashMap::new();
        values.insert("score".to_string(), Rc::new(Value::new::<i32>(7)));
        values.insert(
            "items".to_string(),
            Rc::new(Value::new_value_type(ValueType::Array(vec![
                ValueType::new("key"),
                ValueType::Bool(true),
            ]))),
        );
        values.insert(
            "player".to_string(),
            Rc::new(Value::new_value_type(ValueType::Object(fields))),
        );

        let written = write_dictionary_values(&values).expect("value dictionary should write");

        assert_eq!(written["score"], json!(7));
        assert_eq!(written["items"], json!(["^key", true]));
        assert_eq!(written["player"], json!({ "hp": 10 }));
        let read = crate::json::json_read::jobject_to_hashmap_values(
            written.as_object().expect("dictionary should be an object"),
        )
        .expect("value dictionary should read");

        assert!(matches!(
            read.get("score").map(|value| &value.value),
            Some(ValueType::Int(7))
        ));
        let Some(ValueType::Array(items)) = read.get("items").map(|value| &value.value) else {
            panic!("expected items array");
        };
        assert!(matches!(
            items.as_slice(),
            [ValueType::String(_), ValueType::Bool(true)]
        ));
        let Some(ValueType::Object(player)) = read.get("player").map(|value| &value.value) else {
            panic!("expected player object");
        };
        assert!(matches!(player.get("hp"), Some(ValueType::Int(10))));
    }

    #[test]
    fn unsupported_runtime_objects_still_report_bad_json() {
        let object: Rc<dyn RTObject> = Rc::new(UnsupportedObject::new());

        let error = match write_rtobject(object) {
            Ok(_) => panic!("expected unsupported runtime object to fail"),
            Err(StoryError::BadJson(message)) => message,
            Err(error) => panic!("expected BadJson, got {error:?}"),
        };

        assert!(error.contains("Failed to write runtime object to JSON: unsupported"));
    }

    #[test]
    fn malformed_json_write_divert_without_target_returns_error() {
        let object: Rc<dyn RTObject> = Rc::new(Divert::new(
            false,
            PushPopType::Tunnel,
            false,
            0,
            false,
            None,
            None,
        ));

        let error = match write_rtobject(object) {
            Ok(_) => panic!("missing divert target should fail"),
            Err(error) => error,
        };

        assert!(matches!(
            error,
            StoryError::InvalidStoryState(message)
                if message.contains("Divert is missing its target path")
        ));
    }

    struct UnsupportedObject {
        object: Object,
    }

    impl UnsupportedObject {
        fn new() -> Self {
            Self {
                object: Object::new(),
            }
        }
    }

    impl RTObject for UnsupportedObject {
        fn get_object(&self) -> &Object {
            &self.object
        }
    }

    impl fmt::Display for UnsupportedObject {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "unsupported")
        }
    }
}
