use std::collections::BTreeMap;

use serde_json::{Map, Value as JsonValue};

use crate::{
    FormatError, InterfaceDefinition, InterfaceMemberKind, InternalFunction,
    INTERFACE_IMPLEMENTATIONS_KEY, INTERFACE_MEMBERS_KEY,
};

use super::scalar::{json_value_to_string, required_i32, required_json_string};

pub(super) fn internal_functions_from_value(
    value: &JsonValue,
) -> Result<BTreeMap<String, InternalFunction>, FormatError> {
    let obj = value
        .as_object()
        .ok_or_else(|| FormatError::new("internalFunctions must be a JSON object"))?;
    let mut functions = BTreeMap::new();
    for (name, value) in obj {
        functions.insert(name.clone(), internal_function_from_value(name, value)?);
    }
    Ok(functions)
}

fn internal_function_from_value(
    name: &str,
    value: &JsonValue,
) -> Result<InternalFunction, FormatError> {
    let obj = value
        .as_object()
        .ok_or_else(|| FormatError::new(format!("internal function '{name}' must be an object")))?;
    let path = required_json_string(obj, "path")?.to_string();
    let return_type = required_json_string(obj, "returnType")?.to_string();
    let args = usize::try_from(required_i32(obj, "args")?)
        .map_err(|_| FormatError::new(format!("internal function '{name}' args must be >= 0")))?;
    let arg_types = match obj.get("argTypes") {
        Some(value) => string_array_from_value(value, "argTypes")?,
        None if args == 0 => Vec::new(),
        None => {
            return Err(FormatError::new(format!(
                "internal function '{name}' is missing argTypes"
            )));
        }
    };
    if arg_types.len() != args {
        return Err(FormatError::new(format!(
            "internal function '{name}' args does not match argTypes length"
        )));
    }
    Ok(InternalFunction {
        path,
        args,
        arg_types,
        return_type,
    })
}

fn string_array_from_value(value: &JsonValue, field: &str) -> Result<Vec<String>, FormatError> {
    value
        .as_array()
        .ok_or_else(|| FormatError::new(format!("{field} must be an array")))?
        .iter()
        .map(|value| Ok(json_value_to_string(value, field)?.to_string()))
        .collect()
}

pub(super) fn internal_functions_to_value(
    functions: &BTreeMap<String, InternalFunction>,
) -> JsonValue {
    let mut obj = Map::new();
    for (name, function) in functions {
        let mut function_obj = Map::new();
        function_obj.insert("path".to_string(), JsonValue::String(function.path.clone()));
        function_obj.insert(
            "args".to_string(),
            JsonValue::Number((function.args as u64).into()),
        );
        function_obj.insert(
            "argTypes".to_string(),
            JsonValue::Array(
                function
                    .arg_types
                    .iter()
                    .map(|arg| JsonValue::String(arg.clone()))
                    .collect(),
            ),
        );
        function_obj.insert(
            "returnType".to_string(),
            JsonValue::String(function.return_type.clone()),
        );
        obj.insert(name.clone(), JsonValue::Object(function_obj));
    }
    JsonValue::Object(obj)
}

pub(super) fn interfaces_from_value(
    value: &JsonValue,
) -> Result<BTreeMap<String, InterfaceDefinition>, FormatError> {
    let obj = value
        .as_object()
        .ok_or_else(|| FormatError::new("interfaces must be a JSON object"))?;
    let mut interfaces = BTreeMap::new();
    for (name, value) in obj {
        interfaces.insert(name.clone(), interface_definition_from_value(name, value)?);
    }
    Ok(interfaces)
}

fn interface_definition_from_value(
    name: &str,
    value: &JsonValue,
) -> Result<InterfaceDefinition, FormatError> {
    let obj = value
        .as_object()
        .ok_or_else(|| FormatError::new(format!("interface '{name}' must be an object")))?;
    let members_value = obj.get(INTERFACE_MEMBERS_KEY).ok_or_else(|| {
        FormatError::new(format!(
            "interface '{name}' is missing {INTERFACE_MEMBERS_KEY}"
        ))
    })?;
    let implementations_value = obj.get(INTERFACE_IMPLEMENTATIONS_KEY).ok_or_else(|| {
        FormatError::new(format!(
            "interface '{name}' is missing {INTERFACE_IMPLEMENTATIONS_KEY}"
        ))
    })?;
    Ok(InterfaceDefinition {
        members: interface_members_from_value(name, members_value)?,
        implementations: string_array_from_value(
            implementations_value,
            INTERFACE_IMPLEMENTATIONS_KEY,
        )?,
    })
}

fn interface_members_from_value(
    interface_name: &str,
    value: &JsonValue,
) -> Result<BTreeMap<String, InterfaceMemberKind>, FormatError> {
    let obj = value.as_object().ok_or_else(|| {
        FormatError::new(format!(
            "interface '{interface_name}' {INTERFACE_MEMBERS_KEY} must be a JSON object"
        ))
    })?;
    let mut members = BTreeMap::new();
    for (member, kind_value) in obj {
        let kind_token = json_value_to_string(kind_value, INTERFACE_MEMBERS_KEY)?;
        let kind = InterfaceMemberKind::from_token(kind_token).ok_or_else(|| {
            FormatError::new(format!(
                "interface '{interface_name}' member '{member}' has unsupported kind: {kind_token}"
            ))
        })?;
        members.insert(member.clone(), kind);
    }
    Ok(members)
}

pub(super) fn interfaces_to_value(interfaces: &BTreeMap<String, InterfaceDefinition>) -> JsonValue {
    let mut obj = Map::new();
    for (name, interface) in interfaces {
        let mut interface_obj = Map::new();
        interface_obj.insert(
            INTERFACE_MEMBERS_KEY.to_string(),
            interface_members_to_value(&interface.members),
        );
        interface_obj.insert(
            INTERFACE_IMPLEMENTATIONS_KEY.to_string(),
            JsonValue::Array(
                interface
                    .implementations
                    .iter()
                    .map(|implementation| JsonValue::String(implementation.clone()))
                    .collect(),
            ),
        );
        obj.insert(name.clone(), JsonValue::Object(interface_obj));
    }
    JsonValue::Object(obj)
}

fn interface_members_to_value(members: &BTreeMap<String, InterfaceMemberKind>) -> JsonValue {
    let mut obj = Map::new();
    for (member, kind) in members {
        obj.insert(member.clone(), JsonValue::String(kind.token().to_string()));
    }
    JsonValue::Object(obj)
}
