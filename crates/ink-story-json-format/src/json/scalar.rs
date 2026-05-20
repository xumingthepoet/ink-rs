use serde_json::{Map, Value as JsonValue};

use crate::FormatError;

pub(super) fn required_i32(obj: &Map<String, JsonValue>, key: &str) -> Result<i32, FormatError> {
    let value = obj
        .get(key)
        .ok_or_else(|| FormatError::new(format!("compiled story JSON is missing {key}")))?;
    json_value_to_i32(value, key)
}

pub(super) fn required_json_string<'a>(
    obj: &'a Map<String, JsonValue>,
    key: &str,
) -> Result<&'a str, FormatError> {
    let value = obj
        .get(key)
        .ok_or_else(|| FormatError::new(format!("compiled story JSON is missing {key}")))?;
    json_value_to_string(value, key)
}

pub(super) fn required_usize(
    obj: &Map<String, JsonValue>,
    key: &str,
) -> Result<usize, FormatError> {
    let value = obj
        .get(key)
        .ok_or_else(|| FormatError::new(format!("compiled story JSON is missing {key}")))?;
    json_value_to_usize(value, key)
}

pub(super) fn json_value_to_string<'a>(
    value: &'a JsonValue,
    key: &str,
) -> Result<&'a str, FormatError> {
    value
        .as_str()
        .ok_or_else(|| FormatError::new(format!("{key} must be a string")))
}

pub(super) fn json_value_to_i32(value: &JsonValue, key: &str) -> Result<i32, FormatError> {
    let value = value
        .as_i64()
        .ok_or_else(|| FormatError::new(format!("{key} must be an integer")))?;
    i32::try_from(value).map_err(|_| FormatError::new(format!("{key} is outside i32 range")))
}

pub(super) fn json_value_to_usize(value: &JsonValue, key: &str) -> Result<usize, FormatError> {
    let value = value
        .as_u64()
        .ok_or_else(|| FormatError::new(format!("{key} must be a non-negative integer")))?;
    usize::try_from(value).map_err(|_| FormatError::new(format!("{key} is outside usize range")))
}
