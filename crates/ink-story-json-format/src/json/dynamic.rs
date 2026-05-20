use std::collections::BTreeMap;

use serde_json::{Map, Value as JsonValue};

use crate::{DictKey, DictKeyType, DictValue, FormatError, Object, DICT_VALUE_MARKER};

use super::object::{object_from_value, object_to_value};
use super::scalar::{json_value_to_i32, json_value_to_string};

pub(super) fn dynamic_object_from_array_values(
    values: &[JsonValue],
) -> Result<Object, FormatError> {
    if let Some(object) = dict_from_array(values)? {
        return Ok(object);
    }
    let values = values
        .iter()
        .map(object_from_value)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Object::ValueArray(values))
}

pub(super) fn value_array_to_value(values: &[Object]) -> JsonValue {
    JsonValue::Array(values.iter().map(object_to_value).collect())
}

pub(super) fn value_object_from_map(obj: &Map<String, JsonValue>) -> Result<Object, FormatError> {
    let mut fields = BTreeMap::new();
    for (key, value) in obj {
        fields.insert(key.clone(), object_from_value(value)?);
    }
    Ok(Object::ValueObject(fields))
}

pub(super) fn value_object_to_value(fields: &BTreeMap<String, Object>) -> JsonValue {
    let mut obj = Map::new();
    for (key, value) in fields {
        obj.insert(key.clone(), object_to_value(value));
    }
    JsonValue::Object(obj)
}

fn dict_from_array(values: &[JsonValue]) -> Result<Option<Object>, FormatError> {
    let Some(JsonValue::String(marker)) = values.first() else {
        return Ok(None);
    };
    if marker != DICT_VALUE_MARKER {
        return Ok(None);
    }
    if values.len() != 3 {
        return Err(FormatError::new(
            "Dict value marker must be [\"dict\", keyType, entries]",
        ));
    }

    let key_type_token = json_value_to_string(&values[1], "Dict key type")?;
    let key_type = DictKeyType::from_token(key_type_token).ok_or_else(|| {
        FormatError::new(format!(
            "Dict key type must be 'string' or 'int', got {key_type_token}"
        ))
    })?;
    let entry_values = values[2]
        .as_array()
        .ok_or_else(|| FormatError::new("Dict entries must be an array"))?;
    let mut entries = BTreeMap::new();
    for (index, entry_value) in entry_values.iter().enumerate() {
        let entry = entry_value
            .as_array()
            .ok_or_else(|| FormatError::new(format!("Dict entry {index} must be an array")))?;
        if entry.len() != 2 {
            return Err(FormatError::new(format!(
                "Dict entry {index} must contain a key and value"
            )));
        }
        let key = dict_key_from_value(key_type, &entry[0], index)?;
        let value = object_from_value(&entry[1])?;
        if entries.insert(key.clone(), value).is_some() {
            return Err(FormatError::new(format!(
                "Dict entry {index} duplicates key {key:?}"
            )));
        }
    }

    Ok(Some(Object::ValueDict(DictValue { key_type, entries })))
}

fn dict_key_from_value(
    key_type: DictKeyType,
    value: &JsonValue,
    index: usize,
) -> Result<DictKey, FormatError> {
    match key_type {
        DictKeyType::String => Ok(DictKey::String(
            value
                .as_str()
                .ok_or_else(|| {
                    FormatError::new(format!("Dict entry {index} key must be a string"))
                })?
                .to_string(),
        )),
        DictKeyType::Int => Ok(DictKey::Int(json_value_to_i32(
            value,
            &format!("Dict entry {index} key"),
        )?)),
    }
}

pub(super) fn dict_to_value(value: &DictValue) -> JsonValue {
    let entries = value
        .entries
        .iter()
        .map(|(key, object)| {
            debug_assert_eq!(key.key_type(), value.key_type);
            JsonValue::Array(vec![dict_key_to_value(key), object_to_value(object)])
        })
        .collect();

    JsonValue::Array(vec![
        JsonValue::String(DICT_VALUE_MARKER.to_string()),
        JsonValue::String(value.key_type.token().to_string()),
        JsonValue::Array(entries),
    ])
}

fn dict_key_to_value(key: &DictKey) -> JsonValue {
    match key {
        DictKey::String(value) => JsonValue::String(value.clone()),
        DictKey::Int(value) => JsonValue::Number((*value).into()),
    }
}
