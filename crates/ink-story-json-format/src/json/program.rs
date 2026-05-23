use std::collections::BTreeMap;

use serde_json::{Map, Value as JsonValue};

use crate::{FormatError, Program, INTERFACES_METADATA_KEY};

use super::container::{container_from_value, container_to_value};
use super::metadata::{
    interfaces_from_value, interfaces_to_value, internal_functions_from_value,
    internal_functions_to_value,
};

pub(crate) fn program_from_str(input: &str) -> Result<Program, FormatError> {
    let value = serde_json::from_str(input)
        .map_err(|error| FormatError::new(format!("story JSON is not valid JSON: {error}")))?;
    program_from_value(value)
}

pub(crate) fn program_from_value(value: JsonValue) -> Result<Program, FormatError> {
    let obj = value
        .as_object()
        .ok_or_else(|| FormatError::new("compiled story JSON must be an object"))?;

    let root_value = obj
        .get("root")
        .ok_or_else(|| FormatError::new("compiled story JSON is missing root"))?;
    let root = container_from_value(root_value, None)?;
    let internal_functions = match obj.get("internalFunctions") {
        Some(value) => internal_functions_from_value(value)?,
        None => BTreeMap::new(),
    };
    let interfaces = match obj.get(INTERFACES_METADATA_KEY) {
        Some(value) => interfaces_from_value(value)?,
        None => BTreeMap::new(),
    };

    Ok(Program {
        root,
        internal_functions,
        interfaces,
    })
}

pub(crate) fn program_to_string(program: &Program) -> Result<String, FormatError> {
    serde_json::to_string(&program_to_value(program)).map_err(|error| {
        FormatError::new(format!("failed to serialize compiled story JSON: {error}"))
    })
}

pub(crate) fn program_to_value(program: &Program) -> JsonValue {
    let mut obj = Map::new();
    obj.insert("root".to_string(), container_to_value(&program.root, true));
    if !program.internal_functions.is_empty() {
        obj.insert(
            "internalFunctions".to_string(),
            internal_functions_to_value(&program.internal_functions),
        );
    }
    if !program.interfaces.is_empty() {
        obj.insert(
            INTERFACES_METADATA_KEY.to_string(),
            interfaces_to_value(&program.interfaces),
        );
    }
    JsonValue::Object(obj)
}
