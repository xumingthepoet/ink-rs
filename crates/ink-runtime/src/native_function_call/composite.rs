use std::rc::Rc;

use crate::{object::RTObject, story_error::StoryError, value::Value, value_type::ValueType};

pub(super) fn field_read(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params[0]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(
                "FIELD expected an object value as its first parameter".to_owned(),
            )
        })?;
    let field = params[1]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(
                "FIELD expected a string field name as its second parameter".to_owned(),
            )
        })?;

    let field_name = match &field.value {
        ValueType::String(field_name) => &field_name.string,
        _ => {
            return Err(StoryError::InvalidStoryState(
                "FIELD expected a string field name as its second parameter".to_owned(),
            ));
        }
    };

    match &value.value {
        ValueType::Object(fields) => fields
            .get(field_name)
            .cloned()
            .map(Value::new_value_type)
            .map(|value| Rc::new(value) as Rc<dyn RTObject>)
            .ok_or_else(|| {
                StoryError::InvalidStoryState(format!("Object field not found: '{field_name}'"))
            }),
        _ => Err(StoryError::InvalidStoryState(
            "FIELD expected an object value as its first parameter".to_owned(),
        )),
    }
}

pub(super) fn index_read(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params[0]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(
                "INDEX expected an array value as its first parameter".to_owned(),
            )
        })?;
    let index = params[1]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(
                "INDEX expected an int index as its second parameter".to_owned(),
            )
        })?;

    let index = match index.value {
        ValueType::Int(index) => index,
        _ => {
            return Err(StoryError::InvalidStoryState(
                "INDEX expected an int index as its second parameter".to_owned(),
            ));
        }
    };

    match &value.value {
        ValueType::Array(values) => {
            let array_index = usize::try_from(index).map_err(|_| {
                StoryError::InvalidStoryState(format!("Array index out of bounds: {index}"))
            })?;
            values
                .get(array_index)
                .cloned()
                .map(Value::new_value_type)
                .map(|value| Rc::new(value) as Rc<dyn RTObject>)
                .ok_or_else(|| {
                    StoryError::InvalidStoryState(format!("Array index out of bounds: {index}"))
                })
        }
        _ => Err(StoryError::InvalidStoryState(
            "INDEX expected an array value as its first parameter".to_owned(),
        )),
    }
}

pub(super) fn field_write(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params[0]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(
                "SET_FIELD expected an object value as its first parameter".to_owned(),
            )
        })?;
    let field = params[1]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(
                "SET_FIELD expected a string field name as its second parameter".to_owned(),
            )
        })?;
    let new_value = params[2]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(
                "SET_FIELD expected a value as its third parameter".to_owned(),
            )
        })?;

    let field_name = match &field.value {
        ValueType::String(field_name) => &field_name.string,
        _ => {
            return Err(StoryError::InvalidStoryState(
                "SET_FIELD expected a string field name as its second parameter".to_owned(),
            ));
        }
    };

    match &value.value {
        ValueType::Object(fields) => {
            if !fields.contains_key(field_name) {
                return Err(StoryError::InvalidStoryState(format!(
                    "Object field not found: '{field_name}'"
                )));
            }

            let mut updated_fields = fields.clone();
            updated_fields.insert(field_name.clone(), new_value.value.clone());
            Ok(Rc::new(Value::new_value_type(ValueType::Object(
                updated_fields,
            ))))
        }
        _ => Err(StoryError::InvalidStoryState(
            "SET_FIELD expected an object value as its first parameter".to_owned(),
        )),
    }
}

pub(super) fn index_write(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params[0]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(
                "SET_INDEX expected an array value as its first parameter".to_owned(),
            )
        })?;
    let index = params[1]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(
                "SET_INDEX expected an int index as its second parameter".to_owned(),
            )
        })?;
    let new_value = params[2]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(
                "SET_INDEX expected a value as its third parameter".to_owned(),
            )
        })?;

    let index = match index.value {
        ValueType::Int(index) => index,
        _ => {
            return Err(StoryError::InvalidStoryState(
                "SET_INDEX expected an int index as its second parameter".to_owned(),
            ));
        }
    };

    match &value.value {
        ValueType::Array(values) => {
            let array_index = usize::try_from(index).map_err(|_| {
                StoryError::InvalidStoryState(format!("Array index out of bounds: {index}"))
            })?;
            if array_index >= values.len() {
                return Err(StoryError::InvalidStoryState(format!(
                    "Array index out of bounds: {index}"
                )));
            }

            let mut updated_values = values.clone();
            updated_values[array_index] = new_value.value.clone();
            Ok(Rc::new(Value::new_value_type(ValueType::Array(
                updated_values,
            ))))
        }
        _ => Err(StoryError::InvalidStoryState(
            "SET_INDEX expected an array value as its first parameter".to_owned(),
        )),
    }
}

pub(super) fn len(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params[0]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState("LEN expected an array value as its parameter".to_owned())
        })?;

    match &value.value {
        ValueType::Array(values) => Ok(Rc::new(Value::new::<i32>(values.len() as i32))),
        _ => Err(StoryError::InvalidStoryState(
            "LEN expected an array value as its parameter".to_owned(),
        )),
    }
}

pub(super) fn array_remove(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params[0]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(
                "ARRAY_REMOVE expected an array value as its first parameter".to_owned(),
            )
        })?;
    let index = params[1]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(
                "ARRAY_REMOVE expected an int index as its second parameter".to_owned(),
            )
        })?;

    let index = match index.value {
        ValueType::Int(index) => index,
        _ => {
            return Err(StoryError::InvalidStoryState(
                "ARRAY_REMOVE expected an int index as its second parameter".to_owned(),
            ));
        }
    };

    match &value.value {
        ValueType::Array(values) => {
            let array_index = usize::try_from(index).map_err(|_| {
                StoryError::InvalidStoryState(format!("Array index out of bounds: {index}"))
            })?;
            if array_index >= values.len() {
                return Err(StoryError::InvalidStoryState(format!(
                    "Array index out of bounds: {index}"
                )));
            }

            let mut updated_values = values.clone();
            updated_values.remove(array_index);
            Ok(Rc::new(Value::new_value_type(ValueType::Array(
                updated_values,
            ))))
        }
        _ => Err(StoryError::InvalidStoryState(
            "ARRAY_REMOVE expected an array value as its first parameter".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, rc::Rc};

    use super::super::{NativeFunctionCall, Op};
    use crate::{object::RTObject, story_error::StoryError, value::Value, value_type::ValueType};

    fn int_value(value: i32) -> Rc<dyn RTObject> {
        Rc::new(Value::new::<i32>(value))
    }

    fn string_value(value: &str) -> Rc<dyn RTObject> {
        Rc::new(Value::new::<&str>(value))
    }

    fn array_value(values: Vec<ValueType>) -> Rc<dyn RTObject> {
        Rc::new(Value::new_value_type(ValueType::Array(values)))
    }

    fn object_value(fields: Vec<(&str, ValueType)>) -> Rc<dyn RTObject> {
        let mut object_fields = BTreeMap::new();
        for (name, value) in fields {
            object_fields.insert(name.to_string(), value);
        }
        Rc::new(Value::new_value_type(ValueType::Object(object_fields)))
    }

    fn value_type(value: &dyn RTObject) -> &ValueType {
        &value
            .as_any()
            .downcast_ref::<Value>()
            .expect("expected runtime value")
            .value
    }

    #[test]
    fn field_operations_return_object_copies() {
        let original = object_value(vec![("hp", ValueType::Int(7))]);
        let updated = NativeFunctionCall::new(Op::FieldWrite)
            .call(vec![original.clone(), string_value("hp"), int_value(9)])
            .expect("field write should succeed");

        assert!(
            matches!(value_type(original.as_ref()), ValueType::Object(fields) if matches!(fields.get("hp"), Some(ValueType::Int(7))))
        );
        assert!(
            matches!(value_type(updated.as_ref()), ValueType::Object(fields) if matches!(fields.get("hp"), Some(ValueType::Int(9))))
        );

        let read = NativeFunctionCall::new(Op::FieldRead)
            .call(vec![updated, string_value("hp")])
            .expect("field read should succeed");
        assert!(matches!(value_type(read.as_ref()), ValueType::Int(9)));
    }

    #[test]
    fn index_operations_return_array_copies_and_lengths() {
        let original = array_value(vec![ValueType::Int(1), ValueType::Int(2)]);
        let updated = NativeFunctionCall::new(Op::IndexWrite)
            .call(vec![original.clone(), int_value(1), int_value(5)])
            .expect("index write should succeed");

        assert!(
            matches!(value_type(original.as_ref()), ValueType::Array(values) if matches!(values.as_slice(), [ValueType::Int(1), ValueType::Int(2)]))
        );
        assert!(
            matches!(value_type(updated.as_ref()), ValueType::Array(values) if matches!(values.as_slice(), [ValueType::Int(1), ValueType::Int(5)]))
        );

        let read = NativeFunctionCall::new(Op::IndexRead)
            .call(vec![updated.clone(), int_value(1)])
            .expect("index read should succeed");
        assert!(matches!(value_type(read.as_ref()), ValueType::Int(5)));

        let len = NativeFunctionCall::new(Op::Len)
            .call(vec![updated])
            .expect("len should succeed");
        assert!(matches!(value_type(len.as_ref()), ValueType::Int(2)));
    }

    #[test]
    fn array_remove_preserves_nested_values() {
        let original = array_value(vec![
            ValueType::Int(1),
            ValueType::Array(vec![ValueType::Int(2)]),
            object_value_type(vec![("hp", ValueType::Int(3))]),
        ]);
        let updated = NativeFunctionCall::new(Op::ArrayRemove)
            .call(vec![original.clone(), int_value(0)])
            .expect("array remove should succeed");

        assert!(
            matches!(value_type(original.as_ref()), ValueType::Array(values) if values.len() == 3)
        );
        assert!(
            matches!(value_type(updated.as_ref()), ValueType::Array(values) if matches!(values.as_slice(), [ValueType::Array(_), ValueType::Object(_)]))
        );
    }

    #[test]
    fn composite_operations_keep_missing_field_and_bounds_errors() {
        let missing_field = match NativeFunctionCall::new(Op::FieldRead).call(vec![
            object_value(vec![("hp", ValueType::Int(7))]),
            string_value("mp"),
        ]) {
            Ok(_) => panic!("expected missing field"),
            Err(StoryError::InvalidStoryState(message)) => message,
            Err(error) => panic!("expected invalid story state, got {error:?}"),
        };
        assert_eq!(missing_field, "Object field not found: 'mp'");

        let out_of_bounds = match NativeFunctionCall::new(Op::IndexRead)
            .call(vec![array_value(vec![ValueType::Int(1)]), int_value(4)])
        {
            Ok(_) => panic!("expected out-of-bounds index"),
            Err(StoryError::InvalidStoryState(message)) => message,
            Err(error) => panic!("expected invalid story state, got {error:?}"),
        };
        assert_eq!(out_of_bounds, "Array index out of bounds: 4");
    }

    fn object_value_type(fields: Vec<(&str, ValueType)>) -> ValueType {
        let mut object_fields = BTreeMap::new();
        for (name, value) in fields {
            object_fields.insert(name.to_string(), value);
        }
        ValueType::Object(object_fields)
    }
}
