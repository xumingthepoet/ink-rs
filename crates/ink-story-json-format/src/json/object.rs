use serde_json::{Map, Number, Value as JsonValue};

use crate::{
    FormatError, NativeFunction, Object, DYNAMIC_INTERFACE_ARGS_KEY,
    DYNAMIC_INTERFACE_FUNCTION_KEY, DYNAMIC_INTERFACE_NAME_KEY, DYNAMIC_INTERFACE_TARGET_KEY,
    TAG_END_TOKEN, TAG_START_TOKEN,
};

use super::container::{container_from_value, container_to_value};
use super::dynamic::{
    dict_to_value, dynamic_object_from_array_values, value_array_to_value, value_object_from_map,
    value_object_to_value,
};
use super::scalar::{
    json_value_to_i32, json_value_to_string, json_value_to_usize, required_json_string,
    required_usize,
};

pub(crate) fn object_from_value(value: &JsonValue) -> Result<Object, FormatError> {
    match value {
        JsonValue::Null => Err(FormatError::new(
            "null is only valid as a container terminator",
        )),
        JsonValue::Bool(value) => Ok(Object::Bool(*value)),
        JsonValue::Number(number) => {
            if let Some(value) = number.as_i64() {
                let value = i32::try_from(value)
                    .map_err(|_| FormatError::new("integer value is outside i32 range"))?;
                Ok(Object::Int(value))
            } else {
                let value = number
                    .as_f64()
                    .ok_or_else(|| FormatError::new("number value is not representable as f64"))?;
                Ok(Object::Float(value))
            }
        }
        JsonValue::String(value) => string_object_from_token(value),
        JsonValue::Array(_) => array_object_from_value(value),
        JsonValue::Object(obj) => object_from_map(obj),
    }
}

pub(crate) fn object_to_value(object: &Object) -> JsonValue {
    match object {
        Object::Container(container) => container_to_value(container, true),
        Object::String(text) if text == "\n" => JsonValue::String("\n".to_string()),
        Object::String(text) => JsonValue::String(format!("^{text}")),
        Object::ControlCommand(command) => JsonValue::String(command.token().to_string()),
        Object::Divert { target, variable } => {
            divert_to_value("->", target, *variable, false, None)
        }
        Object::TunnelDivert { target, variable } => {
            divert_to_value("->t->", target, *variable, false, None)
        }
        Object::FunctionDivert { target } => divert_to_value("f()", target, false, false, None),
        Object::ExternalFunction { target, args } => {
            divert_to_value("x()", target, false, false, Some(*args))
        }
        Object::DynamicInterfaceTarget { interface, member } => {
            dynamic_interface_to_value(DYNAMIC_INTERFACE_TARGET_KEY, interface, member, None)
        }
        Object::DynamicInterfaceFunctionCall {
            interface,
            member,
            args,
        } => dynamic_interface_to_value(
            DYNAMIC_INTERFACE_FUNCTION_KEY,
            interface,
            member,
            Some(*args),
        ),
        Object::ConditionalDivert { target } => divert_to_value("->", target, false, true, None),
        Object::DivertTarget(target) => {
            let mut obj = Map::new();
            obj.insert("^->".to_string(), JsonValue::String(target.clone()));
            JsonValue::Object(obj)
        }
        Object::ReadCount(target) => single_property_object("CNT?", target),
        Object::VariableAssignment(name) => single_property_object("temp=", name),
        Object::GlobalVariableAssignment(name) => single_property_object("VAR=", name),
        Object::TempVariableReassignment(name) => variable_assignment_to_value("temp=", name),
        Object::VariableReassignment(name) => variable_assignment_to_value("VAR=", name),
        Object::VariableReference(name) => single_property_object("VAR?", name),
        Object::VariablePointer {
            name,
            context_index,
        } => {
            let mut obj = Map::new();
            obj.insert("^var".to_string(), JsonValue::String(name.clone()));
            obj.insert("ci".to_string(), JsonValue::Number((*context_index).into()));
            JsonValue::Object(obj)
        }
        Object::ChoicePoint { target, flags } => {
            let mut obj = Map::new();
            obj.insert("*".to_string(), JsonValue::String(target.clone()));
            obj.insert("flg".to_string(), JsonValue::Number((*flags).into()));
            JsonValue::Object(obj)
        }
        Object::Glue => JsonValue::String("<>".to_string()),
        Object::Tag { is_start } => JsonValue::String(
            if *is_start {
                TAG_START_TOKEN
            } else {
                TAG_END_TOKEN
            }
            .to_string(),
        ),
        Object::Bool(value) => JsonValue::Bool(*value),
        Object::Int(value) => JsonValue::Number((*value).into()),
        Object::Float(value) => JsonValue::Number(
            Number::from_f64(*value).expect("compiled story float values must be finite"),
        ),
        Object::ValueArray(values) => value_array_to_value(values),
        Object::ValueObject(fields) => value_object_to_value(fields),
        Object::ValueDict(value) => dict_to_value(value),
        Object::Void => JsonValue::String("void".to_string()),
        Object::NativeFunction(function) => JsonValue::String(function.token().to_string()),
    }
}

fn array_object_from_value(value: &JsonValue) -> Result<Object, FormatError> {
    match container_from_value(value, None) {
        Ok(container) => Ok(Object::Container(container)),
        Err(container_error) => match value {
            JsonValue::Array(values) => dynamic_object_from_array_values(values),
            _ => Err(container_error),
        },
    }
}

fn string_object_from_token(token: &str) -> Result<Object, FormatError> {
    if let Some(text) = token.strip_prefix('^') {
        return Ok(Object::String(text.to_string()));
    }

    if token == "\n" {
        return Ok(Object::String("\n".to_string()));
    }

    if token == "<>" {
        return Ok(Object::Glue);
    }

    if token == "void" {
        return Ok(Object::Void);
    }

    if token == TAG_START_TOKEN {
        return Ok(Object::Tag { is_start: true });
    }

    if token == TAG_END_TOKEN {
        return Ok(Object::Tag { is_start: false });
    }

    if let Some(command) = crate::ControlCommand::from_token(token) {
        return Ok(Object::ControlCommand(command));
    }

    NativeFunction::from_token(token)
        .map(Object::NativeFunction)
        .ok_or_else(|| FormatError::new(format!("unsupported native function token: {token}")))
}

fn object_from_map(obj: &Map<String, JsonValue>) -> Result<Object, FormatError> {
    if let Some(target) = obj.get("^->") {
        return Ok(Object::DivertTarget(
            json_value_to_string(target, "^->")?.to_string(),
        ));
    }

    if let Some(name) = obj.get("^var") {
        let context_index = match obj.get("ci") {
            Some(value) => json_value_to_i32(value, "ci")?,
            None => -1,
        };
        return Ok(Object::VariablePointer {
            name: json_value_to_string(name, "^var")?.to_string(),
            context_index,
        });
    }

    if let Some(object) = dynamic_interface_from_map(obj)? {
        return Ok(object);
    }

    if let Some(object) = divert_from_map(obj)? {
        return Ok(object);
    }

    if let Some(target) = obj.get("*") {
        let flags = match obj.get("flg") {
            Some(value) => json_value_to_i32(value, "flg")?,
            None => 0,
        };
        return Ok(Object::ChoicePoint {
            target: json_value_to_string(target, "*")?.to_string(),
            flags,
        });
    }

    if let Some(name) = obj.get("VAR?") {
        return Ok(Object::VariableReference(
            json_value_to_string(name, "VAR?")?.to_string(),
        ));
    }

    if let Some(name) = obj.get("CNT?") {
        return Ok(Object::ReadCount(
            json_value_to_string(name, "CNT?")?.to_string(),
        ));
    }

    if let Some(assignment) = variable_assignment_from_map(obj)? {
        return Ok(assignment);
    }

    value_object_from_map(obj)
}

fn divert_from_map(obj: &Map<String, JsonValue>) -> Result<Option<Object>, FormatError> {
    if let Some(target) = obj.get("->") {
        let target = json_value_to_string(target, "->")?.to_string();
        let variable = obj.contains_key("var");
        let conditional = obj.contains_key("c");
        return Ok(Some(if conditional {
            Object::ConditionalDivert { target }
        } else {
            Object::Divert { target, variable }
        }));
    }

    if let Some(target) = obj.get("f()") {
        return Ok(Some(Object::FunctionDivert {
            target: json_value_to_string(target, "f()")?.to_string(),
        }));
    }

    if let Some(target) = obj.get("->t->") {
        return Ok(Some(Object::TunnelDivert {
            target: json_value_to_string(target, "->t->")?.to_string(),
            variable: obj.contains_key("var"),
        }));
    }

    if let Some(target) = obj.get("x()") {
        return Ok(Some(Object::ExternalFunction {
            target: json_value_to_string(target, "x()")?.to_string(),
            args: match obj.get("exArgs") {
                Some(value) => json_value_to_usize(value, "exArgs")?,
                None => 0,
            },
        }));
    }

    Ok(None)
}

fn dynamic_interface_from_map(obj: &Map<String, JsonValue>) -> Result<Option<Object>, FormatError> {
    if let Some(member) = obj.get(DYNAMIC_INTERFACE_TARGET_KEY) {
        return Ok(Some(Object::DynamicInterfaceTarget {
            interface: required_json_string(obj, DYNAMIC_INTERFACE_NAME_KEY)?.to_string(),
            member: json_value_to_string(member, DYNAMIC_INTERFACE_TARGET_KEY)?.to_string(),
        }));
    }

    if let Some(member) = obj.get(DYNAMIC_INTERFACE_FUNCTION_KEY) {
        return Ok(Some(Object::DynamicInterfaceFunctionCall {
            interface: required_json_string(obj, DYNAMIC_INTERFACE_NAME_KEY)?.to_string(),
            member: json_value_to_string(member, DYNAMIC_INTERFACE_FUNCTION_KEY)?.to_string(),
            args: required_usize(obj, DYNAMIC_INTERFACE_ARGS_KEY)?,
        }));
    }

    Ok(None)
}

fn dynamic_interface_to_value(
    key: &str,
    interface: &str,
    member: &str,
    args: Option<usize>,
) -> JsonValue {
    let mut obj = Map::new();
    obj.insert(key.to_string(), JsonValue::String(member.to_string()));
    obj.insert(
        DYNAMIC_INTERFACE_NAME_KEY.to_string(),
        JsonValue::String(interface.to_string()),
    );
    if let Some(args) = args {
        obj.insert(
            DYNAMIC_INTERFACE_ARGS_KEY.to_string(),
            JsonValue::Number(args.into()),
        );
    }
    JsonValue::Object(obj)
}

fn divert_to_value(
    key: &str,
    target: &str,
    variable: bool,
    conditional: bool,
    external_args: Option<usize>,
) -> JsonValue {
    let mut obj = Map::new();
    obj.insert(key.to_string(), JsonValue::String(target.to_string()));
    if variable {
        obj.insert("var".to_string(), JsonValue::Bool(true));
    }
    if conditional {
        obj.insert("c".to_string(), JsonValue::Bool(true));
    }
    if let Some(external_args) = external_args {
        if external_args > 0 {
            obj.insert(
                "exArgs".to_string(),
                JsonValue::Number(external_args.into()),
            );
        }
    }
    JsonValue::Object(obj)
}

fn variable_assignment_from_map(
    obj: &Map<String, JsonValue>,
) -> Result<Option<Object>, FormatError> {
    if let Some(name) = obj.get("VAR=") {
        let name = json_value_to_string(name, "VAR=")?.to_string();
        return Ok(Some(if obj.contains_key("re") {
            Object::VariableReassignment(name)
        } else {
            Object::GlobalVariableAssignment(name)
        }));
    }

    if let Some(name) = obj.get("temp=") {
        let name = json_value_to_string(name, "temp=")?.to_string();
        return Ok(Some(if obj.contains_key("re") {
            Object::TempVariableReassignment(name)
        } else {
            Object::VariableAssignment(name)
        }));
    }

    Ok(None)
}

fn single_property_object(key: &str, value: &str) -> JsonValue {
    let mut obj = Map::new();
    obj.insert(key.to_string(), JsonValue::String(value.to_string()));
    JsonValue::Object(obj)
}

fn variable_assignment_to_value(key: &str, name: &str) -> JsonValue {
    let mut obj = Map::new();
    obj.insert(key.to_string(), JsonValue::String(name.to_string()));
    obj.insert("re".to_string(), JsonValue::Bool(true));
    JsonValue::Object(obj)
}
