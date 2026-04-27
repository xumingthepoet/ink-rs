use std::{collections::BTreeMap, rc::Rc};

use crate::{object::RTObject, story_error::StoryError, value::Value, value_type::ValueType};

pub(super) fn runtime_value(object: &dyn RTObject) -> Option<&Value> {
    object.as_any().downcast_ref::<Value>()
}

pub(super) fn value<'a>(
    params: &'a [Rc<dyn RTObject>],
    index: usize,
    expected_message: &'static str,
) -> Result<&'a Value, StoryError> {
    runtime_value(params[index].as_ref()).ok_or_else(|| invalid(expected_message))
}

pub(super) fn value_for_operation<'a>(
    params: &'a [Rc<dyn RTObject>],
    index: usize,
    op_name: &str,
) -> Result<&'a Value, StoryError> {
    runtime_value(params[index].as_ref())
        .ok_or_else(|| invalid(format!("{op_name} expected value parameters")))
}

pub(super) fn value_object(object: &dyn RTObject) -> Result<&Value, StoryError> {
    runtime_value(object)
        .ok_or_else(|| invalid(format!("RTObject of type Value expected: {object}")))
}

pub(super) fn string<'a>(
    params: &'a [Rc<dyn RTObject>],
    index: usize,
    expected_message: &'static str,
) -> Result<&'a str, StoryError> {
    match &value(params, index, expected_message)?.value {
        ValueType::String(value) => Ok(&value.string),
        _ => Err(invalid(expected_message)),
    }
}

pub(super) fn int(
    params: &[Rc<dyn RTObject>],
    index: usize,
    expected_message: &'static str,
) -> Result<i32, StoryError> {
    match value(params, index, expected_message)?.value {
        ValueType::Int(index) => Ok(index),
        _ => Err(invalid(expected_message)),
    }
}

pub(super) fn object_fields<'a>(
    value: &'a Value,
    expected_message: &'static str,
) -> Result<&'a BTreeMap<String, ValueType>, StoryError> {
    match &value.value {
        ValueType::Object(fields) => Ok(fields),
        _ => Err(invalid(expected_message)),
    }
}

pub(super) fn array_values<'a>(
    value: &'a Value,
    expected_message: &'static str,
) -> Result<&'a [ValueType], StoryError> {
    match &value.value {
        ValueType::Array(values) => Ok(values),
        _ => Err(invalid(expected_message)),
    }
}

pub(super) fn array_index_in_bounds(index: i32, len: usize) -> Result<usize, StoryError> {
    let array_index = usize::try_from(index).map_err(|_| array_index_out_of_bounds(index))?;
    if array_index >= len {
        return Err(array_index_out_of_bounds(index));
    }
    Ok(array_index)
}

fn array_index_out_of_bounds(index: i32) -> StoryError {
    invalid(format!("Array index out of bounds: {index}"))
}

fn invalid(message: impl Into<String>) -> StoryError {
    StoryError::InvalidStoryState(message.into())
}
