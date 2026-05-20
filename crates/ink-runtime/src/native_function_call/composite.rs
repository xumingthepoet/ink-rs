use std::rc::Rc;

use super::params;
use crate::{
    object::RTObject,
    story_error::StoryError,
    value::Value,
    value_type::{DictKey, DictKeyType, DictValue, ValueType},
};

const FIELD_OBJECT: &str = "FIELD expected an object value as its first parameter";
const FIELD_NAME: &str = "FIELD expected a string field name as its second parameter";
const INDEX_ARRAY: &str = "INDEX expected an array or dict value as its first parameter";
const INDEX_VALUE: &str = "INDEX expected an int index as its second parameter";
const SET_FIELD_OBJECT: &str = "SET_FIELD expected an object value as its first parameter";
const SET_FIELD_NAME: &str = "SET_FIELD expected a string field name as its second parameter";
const SET_FIELD_VALUE: &str = "SET_FIELD expected a value as its third parameter";
const SET_INDEX_ARRAY: &str = "SET_INDEX expected an array or dict value as its first parameter";
const SET_INDEX_VALUE: &str = "SET_INDEX expected an int index as its second parameter";
const SET_INDEX_NEW_VALUE: &str = "SET_INDEX expected a value as its third parameter";
const LEN_ARRAY: &str = "LEN expected an array value as its parameter";
const ARRAY_REMOVE_ARRAY: &str = "ARRAY_REMOVE expected an array value as its first parameter";
const ARRAY_REMOVE_INDEX: &str = "ARRAY_REMOVE expected an int index as its second parameter";
const ARRAY_PUSH_ARRAY: &str = "ARRAY_PUSH expected an array value as its first parameter";
const ARRAY_PUSH_VALUE: &str = "ARRAY_PUSH expected a value as its second parameter";
const ARRAY_INSERT_ARRAY: &str = "ARRAY_INSERT expected an array value as its first parameter";
const ARRAY_INSERT_INDEX: &str = "ARRAY_INSERT expected an int index as its second parameter";
const ARRAY_INSERT_VALUE: &str = "ARRAY_INSERT expected a value as its third parameter";
const DICT_HAS_DICT: &str = "DICT_HAS expected a dict value as its first parameter";
const DICT_SIZE_DICT: &str = "DICT_SIZE expected a dict value as its parameter";
const DICT_REMOVE_DICT: &str = "DICT_REMOVE expected a dict value as its first parameter";
const DICT_KEYS_DICT: &str = "DICT_KEYS expected a dict value as its parameter";

pub(super) fn field_read(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params::value(params, 0, FIELD_OBJECT)?;
    let field_name = params::string(params, 1, FIELD_NAME)?;
    let fields = params::object_fields(value, FIELD_OBJECT)?;

    fields
        .get(field_name)
        .cloned()
        .map(Value::new_value_type)
        .map(|value| Rc::new(value) as Rc<dyn RTObject>)
        .ok_or_else(|| {
            StoryError::InvalidStoryState(format!("Object field not found: '{field_name}'"))
        })
}

pub(super) fn index_read(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params::value(params, 0, INDEX_ARRAY)?;
    match &value.value {
        ValueType::Array(values) => {
            let index = params::int(params, 1, INDEX_VALUE)?;
            let array_index = params::array_index_in_bounds(index, values.len())?;
            Ok(Rc::new(Value::new_value_type(values[array_index].clone())))
        }
        ValueType::Dict(dict) => {
            let key = dict_key_param(params, 1, dict.key_type(), "INDEX")?;
            dict.get(&key)
                .cloned()
                .map(Value::new_value_type)
                .map(|value| Rc::new(value) as Rc<dyn RTObject>)
                .ok_or_else(|| {
                    StoryError::InvalidStoryState(format!(
                        "Dict key not found: {}",
                        dict_key_description(&key)
                    ))
                })
        }
        _ => Err(StoryError::InvalidStoryState(INDEX_ARRAY.to_owned())),
    }
}

pub(super) fn field_write(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params::value(params, 0, SET_FIELD_OBJECT)?;
    let field_name = params::string(params, 1, SET_FIELD_NAME)?;
    let new_value = params::value(params, 2, SET_FIELD_VALUE)?;
    let fields = params::object_fields(value, SET_FIELD_OBJECT)?;

    if !fields.contains_key(field_name) {
        return Err(StoryError::InvalidStoryState(format!(
            "Object field not found: '{field_name}'"
        )));
    }

    let mut updated_fields = fields.clone();
    updated_fields.insert(field_name.to_owned(), new_value.value.clone());
    Ok(Rc::new(Value::new_value_type(ValueType::Object(
        updated_fields,
    ))))
}

pub(super) fn index_write(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params::value(params, 0, SET_INDEX_ARRAY)?;
    match &value.value {
        ValueType::Array(values) => {
            let index = params::int(params, 1, SET_INDEX_VALUE)?;
            let new_value = params::value(params, 2, SET_INDEX_NEW_VALUE)?;
            let array_index = params::array_index_in_bounds(index, values.len())?;

            let mut updated_values = values.to_vec();
            updated_values[array_index] = new_value.value.clone();
            Ok(Rc::new(Value::new_value_type(ValueType::Array(
                updated_values,
            ))))
        }
        ValueType::Dict(dict) => {
            let key = dict_key_param(params, 1, dict.key_type(), "SET_INDEX")?;
            let new_value = params::value(params, 2, SET_INDEX_NEW_VALUE)?;
            let mut updated_entries = dict.entries().clone();
            updated_entries.insert(key, new_value.value.clone());
            let updated_dict = DictValue::new(dict.key_type(), updated_entries)?;
            Ok(Rc::new(Value::new_value_type(ValueType::Dict(
                updated_dict,
            ))))
        }
        _ => Err(StoryError::InvalidStoryState(SET_INDEX_ARRAY.to_owned())),
    }
}

fn dict_key_param(
    params: &[Rc<dyn RTObject>],
    index: usize,
    key_type: DictKeyType,
    op_name: &str,
) -> Result<DictKey, StoryError> {
    let value = params::value(params, index, "Dict key expected")?;
    match (key_type, &value.value) {
        (DictKeyType::String, ValueType::String(value)) => {
            Ok(DictKey::String(value.string.clone()))
        }
        (DictKeyType::Int, ValueType::Int(value)) => Ok(DictKey::Int(*value)),
        _ => Err(StoryError::InvalidStoryState(format!(
            "{op_name} expected {} Dict key as its second parameter",
            dict_key_type_name_with_article(key_type)
        ))),
    }
}

fn dict_key_type_name_with_article(key_type: DictKeyType) -> &'static str {
    match key_type {
        DictKeyType::String => "a string",
        DictKeyType::Int => "an int",
    }
}

fn dict_key_description(key: &DictKey) -> String {
    match key {
        DictKey::String(value) => format!("\"{value}\""),
        DictKey::Int(value) => value.to_string(),
    }
}

pub(super) fn len(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params::value(params, 0, LEN_ARRAY)?;
    let values = params::array_values(value, LEN_ARRAY)?;

    Ok(Rc::new(Value::new::<i32>(values.len() as i32)))
}

pub(super) fn array_remove(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params::value(params, 0, ARRAY_REMOVE_ARRAY)?;
    let index = params::int(params, 1, ARRAY_REMOVE_INDEX)?;
    let values = params::array_values(value, ARRAY_REMOVE_ARRAY)?;
    let array_index = params::array_index_in_bounds(index, values.len())?;

    let mut updated_values = values.to_vec();
    updated_values.remove(array_index);
    Ok(Rc::new(Value::new_value_type(ValueType::Array(
        updated_values,
    ))))
}

pub(super) fn array_push(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params::value(params, 0, ARRAY_PUSH_ARRAY)?;
    let values = params::array_values(value, ARRAY_PUSH_ARRAY)?;
    let new_value = params::value(params, 1, ARRAY_PUSH_VALUE)?;

    let mut updated_values = values.to_vec();
    updated_values.push(new_value.value.clone());
    Ok(Rc::new(Value::new_value_type(ValueType::Array(
        updated_values,
    ))))
}

pub(super) fn array_insert(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let value = params::value(params, 0, ARRAY_INSERT_ARRAY)?;
    let index = params::int(params, 1, ARRAY_INSERT_INDEX)?;
    let values = params::array_values(value, ARRAY_INSERT_ARRAY)?;
    let new_value = params::value(params, 2, ARRAY_INSERT_VALUE)?;
    let array_index = array_insert_index_in_bounds(index, values.len())?;

    let mut updated_values = values.to_vec();
    updated_values.insert(array_index, new_value.value.clone());
    Ok(Rc::new(Value::new_value_type(ValueType::Array(
        updated_values,
    ))))
}

fn array_insert_index_in_bounds(index: i32, len: usize) -> Result<usize, StoryError> {
    let array_index =
        usize::try_from(index).map_err(|_| array_insert_index_out_of_bounds(index))?;
    if array_index > len {
        return Err(array_insert_index_out_of_bounds(index));
    }
    Ok(array_index)
}

fn array_insert_index_out_of_bounds(index: i32) -> StoryError {
    StoryError::InvalidStoryState(format!("Array insert index out of bounds: {index}"))
}

pub(super) fn dict_has(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let dict = dict_param(params, 0, DICT_HAS_DICT)?;
    let key = dict_key_param(params, 1, dict.key_type(), "DICT_HAS")?;

    Ok(Rc::new(Value::new::<bool>(dict.get(&key).is_some())))
}

pub(super) fn dict_size(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let dict = dict_param(params, 0, DICT_SIZE_DICT)?;

    Ok(Rc::new(Value::new::<i32>(dict.entries().len() as i32)))
}

pub(super) fn dict_remove(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let dict = dict_param(params, 0, DICT_REMOVE_DICT)?;
    let key = dict_key_param(params, 1, dict.key_type(), "DICT_REMOVE")?;
    let mut updated_entries = dict.entries().clone();
    updated_entries.remove(&key);
    let updated_dict = DictValue::new(dict.key_type(), updated_entries)?;

    Ok(Rc::new(Value::new_value_type(ValueType::Dict(
        updated_dict,
    ))))
}

pub(super) fn dict_keys(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let dict = dict_param(params, 0, DICT_KEYS_DICT)?;
    let keys = dict
        .entries()
        .keys()
        .map(|key| match key {
            DictKey::String(value) => ValueType::from(value.as_str()),
            DictKey::Int(value) => ValueType::Int(*value),
        })
        .collect();

    Ok(Rc::new(Value::new_value_type(ValueType::Array(keys))))
}

fn dict_param<'a>(
    params: &'a [Rc<dyn RTObject>],
    index: usize,
    expected_message: &'static str,
) -> Result<&'a DictValue, StoryError> {
    match &params::value(params, index, expected_message)?.value {
        ValueType::Dict(dict) => Ok(dict),
        _ => Err(StoryError::InvalidStoryState(expected_message.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, rc::Rc};

    use super::super::{NativeFunctionCall, Op};
    use crate::{
        object::RTObject,
        story_error::StoryError,
        value::Value,
        value_type::{DictKey, DictKeyType, DictValue, ValueType},
    };

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

    fn dict_value(key_type: DictKeyType, entries: Vec<(DictKey, ValueType)>) -> Rc<dyn RTObject> {
        Rc::new(Value::new_value_type(ValueType::Dict(
            DictValue::new(key_type, entries.into_iter().collect::<BTreeMap<_, _>>())
                .expect("test dict entries should match key type"),
        )))
    }

    fn non_value_object() -> Rc<dyn RTObject> {
        Rc::new(NativeFunctionCall::new(Op::Add))
    }

    fn value_type(value: &dyn RTObject) -> &ValueType {
        &value
            .as_any()
            .downcast_ref::<Value>()
            .expect("expected runtime value")
            .value
    }

    fn invalid_state(result: Result<Rc<dyn RTObject>, StoryError>) -> String {
        match result {
            Ok(_) => panic!("expected invalid story state"),
            Err(StoryError::InvalidStoryState(message)) => message,
            Err(error) => panic!("expected invalid story state, got {error:?}"),
        }
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
    fn array_push_and_insert_return_array_copies() {
        let original = array_value(vec![ValueType::Int(1), ValueType::Int(3)]);
        let pushed = NativeFunctionCall::new(Op::ArrayPush)
            .call(vec![original.clone(), int_value(4)])
            .expect("ARRAY_PUSH should succeed");
        let inserted_head = NativeFunctionCall::new(Op::ArrayInsert)
            .call(vec![pushed.clone(), int_value(0), int_value(0)])
            .expect("ARRAY_INSERT should insert at the head");
        let inserted_middle = NativeFunctionCall::new(Op::ArrayInsert)
            .call(vec![inserted_head.clone(), int_value(2), int_value(2)])
            .expect("ARRAY_INSERT should insert in the middle");
        let inserted_tail = NativeFunctionCall::new(Op::ArrayInsert)
            .call(vec![inserted_middle.clone(), int_value(5), int_value(5)])
            .expect("ARRAY_INSERT should insert at LEN(array)");

        assert!(
            matches!(value_type(original.as_ref()), ValueType::Array(values)
                if matches!(values.as_slice(), [ValueType::Int(1), ValueType::Int(3)]))
        );
        assert!(
            matches!(value_type(pushed.as_ref()), ValueType::Array(values)
                if matches!(values.as_slice(), [ValueType::Int(1), ValueType::Int(3), ValueType::Int(4)]))
        );
        assert!(
            matches!(value_type(inserted_tail.as_ref()), ValueType::Array(values)
            if matches!(values.as_slice(), [
                ValueType::Int(0),
                ValueType::Int(1),
                ValueType::Int(2),
                ValueType::Int(3),
                ValueType::Int(4),
                ValueType::Int(5)
            ]))
        );
    }

    #[test]
    fn array_insert_reports_out_of_bounds_indexes() {
        for index in [-1, 3] {
            let error = invalid_state(NativeFunctionCall::new(Op::ArrayInsert).call(vec![
                array_value(vec![ValueType::Int(1), ValueType::Int(2)]),
                int_value(index),
                int_value(3),
            ]));
            assert_eq!(error, format!("Array insert index out of bounds: {index}"));
        }
    }

    #[test]
    fn dict_index_read_supports_string_and_int_keys() {
        let string_dict = dict_value(
            DictKeyType::String,
            vec![(DictKey::String("ada".to_string()), ValueType::Int(10))],
        );
        let int_dict = dict_value(
            DictKeyType::Int,
            vec![(DictKey::Int(1), ValueType::new("one"))],
        );

        let string_read = NativeFunctionCall::new(Op::IndexRead)
            .call(vec![string_dict, string_value("ada")])
            .expect("string-key dict read should succeed");
        assert!(matches!(
            value_type(string_read.as_ref()),
            ValueType::Int(10)
        ));

        let int_read = NativeFunctionCall::new(Op::IndexRead)
            .call(vec![int_dict, int_value(1)])
            .expect("int-key dict read should succeed");
        assert!(
            matches!(value_type(int_read.as_ref()), ValueType::String(value) if value.string == "one")
        );
    }

    #[test]
    fn dict_index_write_inserts_and_replaces_without_mutating_original() {
        let original = dict_value(
            DictKeyType::String,
            vec![(DictKey::String("ada".to_string()), ValueType::Int(10))],
        );
        let inserted = NativeFunctionCall::new(Op::IndexWrite)
            .call(vec![original.clone(), string_value("bea"), int_value(11)])
            .expect("dict insert should succeed");
        let replaced = NativeFunctionCall::new(Op::IndexWrite)
            .call(vec![inserted.clone(), string_value("ada"), int_value(12)])
            .expect("dict replace should succeed");

        assert!(
            matches!(value_type(original.as_ref()), ValueType::Dict(dict) if matches!(dict.get(&DictKey::String("bea".to_string())), None))
        );
        assert!(
            matches!(value_type(inserted.as_ref()), ValueType::Dict(dict) if matches!(dict.get(&DictKey::String("bea".to_string())), Some(ValueType::Int(11))))
        );
        assert!(
            matches!(value_type(replaced.as_ref()), ValueType::Dict(dict) if matches!(dict.get(&DictKey::String("ada".to_string())), Some(ValueType::Int(12))))
        );
    }

    #[test]
    fn dict_collection_helpers_read_size_presence_and_keys() {
        let string_dict = dict_value(
            DictKeyType::String,
            vec![
                (DictKey::String("bea".to_string()), ValueType::Int(11)),
                (DictKey::String("ada".to_string()), ValueType::Int(10)),
            ],
        );
        let int_dict = dict_value(
            DictKeyType::Int,
            vec![
                (DictKey::Int(2), ValueType::new("two")),
                (DictKey::Int(1), ValueType::new("one")),
            ],
        );

        let has_ada = NativeFunctionCall::new(Op::DictHas)
            .call(vec![string_dict.clone(), string_value("ada")])
            .expect("DICT_HAS should succeed");
        assert!(matches!(
            value_type(has_ada.as_ref()),
            ValueType::Bool(true)
        ));

        let has_missing = NativeFunctionCall::new(Op::DictHas)
            .call(vec![string_dict.clone(), string_value("missing")])
            .expect("DICT_HAS should succeed for missing keys");
        assert!(matches!(
            value_type(has_missing.as_ref()),
            ValueType::Bool(false)
        ));

        let size = NativeFunctionCall::new(Op::DictSize)
            .call(vec![string_dict.clone()])
            .expect("DICT_SIZE should succeed");
        assert!(matches!(value_type(size.as_ref()), ValueType::Int(2)));

        let string_keys = NativeFunctionCall::new(Op::DictKeys)
            .call(vec![string_dict])
            .expect("DICT_KEYS should succeed for string keys");
        assert!(
            matches!(value_type(string_keys.as_ref()), ValueType::Array(values)
                if matches!(values.as_slice(), [ValueType::String(left), ValueType::String(right)] if left.string == "ada" && right.string == "bea"))
        );

        let int_keys = NativeFunctionCall::new(Op::DictKeys)
            .call(vec![int_dict])
            .expect("DICT_KEYS should succeed for int keys");
        assert!(
            matches!(value_type(int_keys.as_ref()), ValueType::Array(values)
                if matches!(values.as_slice(), [ValueType::Int(1), ValueType::Int(2)]))
        );
    }

    #[test]
    fn dict_remove_returns_updated_dict_and_missing_key_is_noop() {
        let original = dict_value(
            DictKeyType::String,
            vec![
                (DictKey::String("ada".to_string()), ValueType::Int(10)),
                (DictKey::String("bea".to_string()), ValueType::Int(11)),
            ],
        );
        let removed = NativeFunctionCall::new(Op::DictRemove)
            .call(vec![original.clone(), string_value("ada")])
            .expect("DICT_REMOVE should remove existing keys");
        let nooped = NativeFunctionCall::new(Op::DictRemove)
            .call(vec![removed.clone(), string_value("missing")])
            .expect("DICT_REMOVE should ignore missing keys");

        assert!(
            matches!(value_type(original.as_ref()), ValueType::Dict(dict) if dict.entries().len() == 2)
        );
        assert!(matches!(value_type(removed.as_ref()), ValueType::Dict(dict)
                if dict.entries().len() == 1 && dict.get(&DictKey::String("ada".to_string())).is_none()));
        assert!(
            matches!(value_type(nooped.as_ref()), ValueType::Dict(dict) if dict.entries().len() == 1)
        );
    }

    #[test]
    fn dict_index_operations_reject_wrong_keys_and_missing_reads() {
        let string_dict = dict_value(
            DictKeyType::String,
            vec![(DictKey::String("ada".to_string()), ValueType::Int(10))],
        );
        let int_dict = dict_value(
            DictKeyType::Int,
            vec![(DictKey::Int(1), ValueType::Int(10))],
        );

        let wrong_string_key = invalid_state(
            NativeFunctionCall::new(Op::IndexRead).call(vec![string_dict.clone(), int_value(1)]),
        );
        assert_eq!(
            wrong_string_key,
            "INDEX expected a string Dict key as its second parameter"
        );

        let wrong_int_key = invalid_state(NativeFunctionCall::new(Op::IndexWrite).call(vec![
            int_dict,
            string_value("1"),
            int_value(11),
        ]));
        assert_eq!(
            wrong_int_key,
            "SET_INDEX expected an int Dict key as its second parameter"
        );

        let missing_key = invalid_state(
            NativeFunctionCall::new(Op::IndexRead).call(vec![string_dict, string_value("missing")]),
        );
        assert_eq!(missing_key, "Dict key not found: \"missing\"");
    }

    #[test]
    fn index_write_reports_key_errors_before_new_value_errors() {
        let array_error = invalid_state(NativeFunctionCall::new(Op::IndexWrite).call(vec![
            array_value(vec![ValueType::Int(1)]),
            string_value("0"),
            non_value_object(),
        ]));
        assert_eq!(
            array_error,
            "SET_INDEX expected an int index as its second parameter"
        );

        let dict_error = invalid_state(NativeFunctionCall::new(Op::IndexWrite).call(vec![
            dict_value(
                DictKeyType::Int,
                vec![(DictKey::Int(1), ValueType::Int(10))],
            ),
            string_value("1"),
            non_value_object(),
        ]));
        assert_eq!(
            dict_error,
            "SET_INDEX expected an int Dict key as its second parameter"
        );
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

    #[test]
    fn composite_operations_keep_invalid_parameter_type_errors() {
        let object_error = invalid_state(
            NativeFunctionCall::new(Op::FieldRead)
                .call(vec![non_value_object(), string_value("hp")]),
        );
        assert_eq!(
            object_error,
            "FIELD expected an object value as its first parameter"
        );

        let field_name_error = invalid_state(NativeFunctionCall::new(Op::FieldRead).call(vec![
            object_value(vec![("hp", ValueType::Int(7))]),
            int_value(0),
        ]));
        assert_eq!(
            field_name_error,
            "FIELD expected a string field name as its second parameter"
        );

        let index_error = invalid_state(NativeFunctionCall::new(Op::IndexRead).call(vec![
            array_value(vec![ValueType::Int(1)]),
            string_value("0"),
        ]));
        assert_eq!(
            index_error,
            "INDEX expected an int index as its second parameter"
        );

        let new_value_error = invalid_state(NativeFunctionCall::new(Op::FieldWrite).call(vec![
            object_value(vec![("hp", ValueType::Int(7))]),
            string_value("hp"),
            non_value_object(),
        ]));
        assert_eq!(
            new_value_error,
            "SET_FIELD expected a value as its third parameter"
        );

        let negative_index = invalid_state(
            NativeFunctionCall::new(Op::IndexRead)
                .call(vec![array_value(vec![ValueType::Int(1)]), int_value(-1)]),
        );
        assert_eq!(negative_index, "Array index out of bounds: -1");
    }

    fn object_value_type(fields: Vec<(&str, ValueType)>) -> ValueType {
        let mut object_fields = BTreeMap::new();
        for (name, value) in fields {
            object_fields.insert(name.to_string(), value);
        }
        ValueType::Object(object_fields)
    }
}
