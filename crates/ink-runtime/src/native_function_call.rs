use std::{fmt, rc::Rc};

use ink_story_json_format::NativeFunction;

mod composite;
mod metadata;
mod scalar;
pub use metadata::Op;

use crate::{
    object::{Object, RTObject},
    story_error::StoryError,
    void::Void,
};

pub struct NativeFunctionCall {
    obj: Object,
    pub op: Op,
}

impl NativeFunctionCall {
    pub fn new(op: Op) -> Self {
        Self {
            obj: Object::new(),
            op,
        }
    }

    pub fn new_from_format(function: NativeFunction) -> Self {
        Self::new_from_name(function.token())
            .expect("format native function token should map to a runtime operation")
    }

    pub fn new_from_name(name: &str) -> Option<Self> {
        NativeFunction::from_token(name).map(|function| Self::new(Op::from_format(function)))
    }

    pub fn get_name(op: Op) -> String {
        op.token().to_owned()
    }

    pub fn format_function(&self) -> NativeFunction {
        self.op.format_function()
    }

    pub fn get_number_of_parameters(&self) -> usize {
        self.op.arity()
    }

    pub(crate) fn call(
        &self,
        params: Vec<Rc<dyn RTObject>>,
    ) -> Result<Rc<dyn RTObject>, StoryError> {
        if self.get_number_of_parameters() != params.len() {
            return Err(StoryError::InvalidStoryState(
                "Unexpected number of parameters".to_owned(),
            ));
        }

        for p in &params {
            if p.as_ref().as_any().is::<Void>() {
                return Err(StoryError::InvalidStoryState(format!("Attempting to perform {} on a void value. Did you forget to 'return' a value from a function you called here?", Self::get_name(self.op))));
            }
        }

        match self.op {
            Op::FieldRead => composite::field_read(&params),
            Op::IndexRead => composite::index_read(&params),
            Op::FieldWrite => composite::field_write(&params),
            Op::IndexWrite => composite::index_write(&params),
            Op::Len => composite::len(&params),
            Op::ArrayRemove => composite::array_remove(&params),
            _ => scalar::call(self.op, params),
        }
    }
}

impl RTObject for NativeFunctionCall {
    fn get_object(&self) -> &Object {
        &self.obj
    }
}

impl fmt::Display for NativeFunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Native '{:?}'", self.op)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, rc::Rc};

    use ink_story_json_format::NativeFunction;

    use crate::{object::RTObject, value::Value, value_type::ValueType};

    use super::{NativeFunctionCall, Op};

    fn object_with_fields(fields: BTreeMap<String, ValueType>) -> Rc<dyn RTObject> {
        Rc::new(Value::new_value_type(ValueType::Object(fields)))
    }

    fn array_value(values: Vec<ValueType>) -> Rc<dyn RTObject> {
        Rc::new(Value::new_value_type(ValueType::Array(values)))
    }

    fn int_value(value: i32) -> Rc<dyn RTObject> {
        Rc::new(Value::new::<i32>(value))
    }

    fn bool_value(value: bool) -> Rc<dyn RTObject> {
        Rc::new(Value::new::<bool>(value))
    }

    fn float_value(value: f32) -> Rc<dyn RTObject> {
        Rc::new(Value::new::<f32>(value))
    }

    fn string_value(value: &str) -> Rc<dyn RTObject> {
        Rc::new(Value::new::<&str>(value))
    }

    #[test]
    fn maps_format_native_functions_to_runtime_ops_and_back() {
        let cases = [
            (NativeFunction::Add, Op::Add),
            (NativeFunction::Subtract, Op::Subtract),
            (NativeFunction::Divide, Op::Divide),
            (NativeFunction::Multiply, Op::Multiply),
            (NativeFunction::Mod, Op::Mod),
            (NativeFunction::Negate, Op::Negate),
            (NativeFunction::Equal, Op::Equal),
            (NativeFunction::Greater, Op::Greater),
            (NativeFunction::Less, Op::Less),
            (NativeFunction::GreaterThanOrEquals, Op::GreaterThanOrEquals),
            (NativeFunction::LessThanOrEquals, Op::LessThanOrEquals),
            (NativeFunction::NotEquals, Op::NotEquals),
            (NativeFunction::Not, Op::Not),
            (NativeFunction::And, Op::And),
            (NativeFunction::Or, Op::Or),
            (NativeFunction::Min, Op::Min),
            (NativeFunction::Max, Op::Max),
            (NativeFunction::Pow, Op::Pow),
            (NativeFunction::Floor, Op::Floor),
            (NativeFunction::Ceiling, Op::Ceiling),
            (NativeFunction::Int, Op::Int),
            (NativeFunction::Float, Op::Float),
            (NativeFunction::Has, Op::Has),
            (NativeFunction::Hasnt, Op::Hasnt),
            (NativeFunction::FieldRead, Op::FieldRead),
            (NativeFunction::IndexRead, Op::IndexRead),
            (NativeFunction::FieldWrite, Op::FieldWrite),
            (NativeFunction::IndexWrite, Op::IndexWrite),
            (NativeFunction::Len, Op::Len),
            (NativeFunction::ArrayRemove, Op::ArrayRemove),
        ];

        assert_eq!(cases.len(), NativeFunction::ALL.len());
        for (function, op) in cases {
            let runtime_function = NativeFunctionCall::new_from_format(function);
            assert_eq!(runtime_function.op, op);
            assert_eq!(runtime_function.format_function(), function);
            assert_eq!(NativeFunctionCall::get_name(op), function.token());
            assert_eq!(
                NativeFunctionCall::new_from_name(function.token()).map(|call| call.op),
                Some(op)
            );
            assert_eq!(
                runtime_function.get_number_of_parameters(),
                function.arity()
            );
        }
    }

    fn object_value_type(fields: Vec<(&str, ValueType)>) -> ValueType {
        let mut object_fields = BTreeMap::new();
        for (name, value) in fields {
            object_fields.insert(name.to_string(), value);
        }
        ValueType::Object(object_fields)
    }

    fn object_value(fields: Vec<(&str, ValueType)>) -> Rc<dyn RTObject> {
        Rc::new(Value::new_value_type(object_value_type(fields)))
    }

    fn native_bool_result(op: Op, left: Rc<dyn RTObject>, right: Rc<dyn RTObject>) -> bool {
        let result = NativeFunctionCall::new(op)
            .call(vec![left, right])
            .expect("native bool operation should succeed");
        Value::get_bool_value(result.as_ref()).expect("result should be bool")
    }

    fn object_field_int(value: &dyn RTObject, field_name: &str) -> Option<i32> {
        let value = value.as_any().downcast_ref::<Value>()?;
        let ValueType::Object(fields) = &value.value else {
            return None;
        };
        let ValueType::Int(field_value) = fields.get(field_name)? else {
            return None;
        };
        Some(*field_value)
    }

    fn array_item_int(value: &dyn RTObject, index: usize) -> Option<i32> {
        let value = value.as_any().downcast_ref::<Value>()?;
        let ValueType::Array(values) = &value.value else {
            return None;
        };
        let ValueType::Int(item_value) = values.get(index)? else {
            return None;
        };
        Some(*item_value)
    }

    fn array_len(value: &dyn RTObject) -> Option<usize> {
        let value = value.as_any().downcast_ref::<Value>()?;
        let ValueType::Array(values) = &value.value else {
            return None;
        };
        Some(values.len())
    }

    fn array_item_object_field_int(
        value: &dyn RTObject,
        index: usize,
        field_name: &str,
    ) -> Option<i32> {
        let value = value.as_any().downcast_ref::<Value>()?;
        let ValueType::Array(values) = &value.value else {
            return None;
        };
        let ValueType::Object(fields) = values.get(index)? else {
            return None;
        };
        let ValueType::Int(field_value) = fields.get(field_name)? else {
            return None;
        };
        Some(*field_value)
    }

    fn nested_array_item_int(value: &dyn RTObject, outer: usize, inner: usize) -> Option<i32> {
        let value = value.as_any().downcast_ref::<Value>()?;
        let ValueType::Array(values) = &value.value else {
            return None;
        };
        let ValueType::Array(inner_values) = values.get(outer)? else {
            return None;
        };
        let ValueType::Int(item_value) = inner_values.get(inner)? else {
            return None;
        };
        Some(*item_value)
    }

    #[test]
    fn field_read_returns_object_field_value() {
        let mut fields = BTreeMap::new();
        fields.insert("hp".to_string(), ValueType::Int(7));

        let result = NativeFunctionCall::new(Op::FieldRead)
            .call(vec![object_with_fields(fields), string_value("hp")])
            .expect("field read should succeed");

        assert_eq!(Value::get_value::<i32>(result.as_ref()), Some(7));
    }

    #[test]
    fn equality_recurses_through_nested_arrays_and_structs() {
        let left = array_value(vec![
            ValueType::Int(1),
            ValueType::Array(vec![
                ValueType::Int(2),
                object_value_type(vec![
                    ("hp", ValueType::Int(3)),
                    ("ready", ValueType::Bool(true)),
                ]),
            ]),
        ]);
        let right = array_value(vec![
            ValueType::Int(1),
            ValueType::Array(vec![
                ValueType::Int(2),
                object_value_type(vec![
                    ("hp", ValueType::Int(3)),
                    ("ready", ValueType::Bool(true)),
                ]),
            ]),
        ]);

        assert!(native_bool_result(Op::Equal, left, right));

        let left = object_value(vec![(
            "stats",
            object_value_type(vec![
                ("hp", ValueType::Int(3)),
                ("ready", ValueType::Bool(true)),
            ]),
        )]);
        let right = object_value(vec![(
            "stats",
            object_value_type(vec![
                ("hp", ValueType::Int(3)),
                ("ready", ValueType::Bool(true)),
            ]),
        )]);

        assert!(native_bool_result(Op::Equal, left, right));
    }

    #[test]
    fn equality_detects_array_length_and_object_field_mismatches() {
        assert!(!native_bool_result(
            Op::Equal,
            array_value(vec![ValueType::Int(1)]),
            array_value(vec![ValueType::Int(1), ValueType::Int(2)])
        ));
        assert!(native_bool_result(
            Op::NotEquals,
            array_value(vec![ValueType::Int(1)]),
            array_value(vec![ValueType::Int(1), ValueType::Int(2)])
        ));

        let left = object_value(vec![("hp", ValueType::Int(3))]);
        let right = object_value(vec![
            ("hp", ValueType::Int(3)),
            ("ready", ValueType::Bool(true)),
        ]);

        assert!(!native_bool_result(Op::Equal, left, right));
    }

    #[test]
    fn equality_compares_arrays_of_structs_and_primitive_fields() {
        let left = array_value(vec![object_value_type(vec![
            ("hp", ValueType::Int(7)),
            ("name", ValueType::new("Ada")),
            ("ready", ValueType::Bool(true)),
        ])]);
        let right = array_value(vec![object_value_type(vec![
            ("hp", ValueType::Int(7)),
            ("name", ValueType::new("Ada")),
            ("ready", ValueType::Bool(true)),
        ])]);

        assert!(native_bool_result(Op::Equal, left, right));

        let left = object_value(vec![
            ("hp", ValueType::Int(7)),
            ("ready", ValueType::Bool(true)),
        ]);
        let right = object_value(vec![
            ("hp", ValueType::Int(8)),
            ("ready", ValueType::Bool(true)),
        ]);

        assert!(native_bool_result(Op::NotEquals, left, right));
    }

    #[test]
    fn primitive_equality_regressions_keep_legacy_semantics() {
        assert!(native_bool_result(
            Op::Equal,
            bool_value(true),
            bool_value(true)
        ));
        assert!(native_bool_result(Op::Equal, int_value(1), int_value(1)));
        assert!(native_bool_result(
            Op::Equal,
            float_value(1.5),
            float_value(1.5)
        ));
        assert!(native_bool_result(
            Op::Equal,
            string_value("Ada"),
            string_value("Ada")
        ));
        assert!(native_bool_result(
            Op::NotEquals,
            int_value(1),
            int_value(2)
        ));
        assert!(native_bool_result(
            Op::Equal,
            int_value(1),
            float_value(1.0)
        ));
        assert!(!native_bool_result(
            Op::Equal,
            array_value(vec![ValueType::Int(1)]),
            array_value(vec![ValueType::Float(1.0)])
        ));
    }

    #[test]
    fn string_add_concatenates_strings_and_keeps_numeric_addition() {
        let string_result = NativeFunctionCall::new(Op::Add)
            .call(vec![string_value("Ada"), string_value("!")])
            .expect("string addition should succeed");
        assert_eq!(string_result.to_string(), "Ada!");

        let int_result = NativeFunctionCall::new(Op::Add)
            .call(vec![int_value(1), int_value(2)])
            .expect("int addition should succeed");
        assert_eq!(Value::get_value::<i32>(int_result.as_ref()), Some(3));

        let float_result = NativeFunctionCall::new(Op::Add)
            .call(vec![float_value(1.0), float_value(2.5)])
            .expect("float addition should succeed");
        assert_eq!(Value::get_value::<f32>(float_result.as_ref()), Some(3.5));
    }

    #[test]
    fn string_add_reports_mixed_string_operands() {
        for params in [
            vec![string_value("Ada"), int_value(1)],
            vec![int_value(1), string_value("Ada")],
        ] {
            let error = match NativeFunctionCall::new(Op::Add).call(params) {
                Ok(_) => panic!("mixed string addition should fail"),
                Err(error) => error,
            };
            let message = error.to_string();

            assert!(
                message.contains("Operation not available for type."),
                "unexpected error: {message}"
            );
        }
    }

    #[test]
    fn field_read_reports_missing_fields() {
        let mut fields = BTreeMap::new();
        fields.insert("hp".to_string(), ValueType::Int(7));

        let error = match NativeFunctionCall::new(Op::FieldRead)
            .call(vec![object_with_fields(fields), string_value("mp")])
        {
            Ok(_) => panic!("missing field should fail"),
            Err(error) => error,
        };
        let message = error.to_string();

        assert!(
            message.contains("Object field not found: 'mp'"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn field_write_returns_object_copy_with_updated_field() {
        let mut fields = BTreeMap::new();
        fields.insert("hp".to_string(), ValueType::Int(7));
        fields.insert("mp".to_string(), ValueType::Int(3));
        let original = object_with_fields(fields);

        let result = NativeFunctionCall::new(Op::FieldWrite)
            .call(vec![original.clone(), string_value("hp"), int_value(5)])
            .expect("field write should succeed");

        assert_eq!(object_field_int(result.as_ref(), "hp"), Some(5));
        assert_eq!(object_field_int(result.as_ref(), "mp"), Some(3));
        assert_eq!(object_field_int(original.as_ref(), "hp"), Some(7));
    }

    #[test]
    fn field_write_reports_missing_fields() {
        let mut fields = BTreeMap::new();
        fields.insert("hp".to_string(), ValueType::Int(7));

        let error = match NativeFunctionCall::new(Op::FieldWrite).call(vec![
            object_with_fields(fields),
            string_value("mp"),
            int_value(5),
        ]) {
            Ok(_) => panic!("missing field should fail"),
            Err(error) => error,
        };
        let message = error.to_string();

        assert!(
            message.contains("Object field not found: 'mp'"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn index_read_uses_zero_based_array_indexes() {
        let result = NativeFunctionCall::new(Op::IndexRead)
            .call(vec![
                array_value(vec![ValueType::Int(11), ValueType::Int(22)]),
                int_value(0),
            ])
            .expect("index read should succeed");

        assert_eq!(Value::get_value::<i32>(result.as_ref()), Some(11));
    }

    #[test]
    fn index_read_reports_out_of_bounds_indexes() {
        let error = match NativeFunctionCall::new(Op::IndexRead)
            .call(vec![array_value(vec![ValueType::Int(11)]), int_value(1)])
        {
            Ok(_) => panic!("out-of-bounds index should fail"),
            Err(error) => error,
        };
        let message = error.to_string();

        assert!(
            message.contains("Array index out of bounds: 1"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn index_write_returns_array_copy_with_updated_item() {
        let original = array_value(vec![ValueType::Int(11), ValueType::Int(22)]);

        let result = NativeFunctionCall::new(Op::IndexWrite)
            .call(vec![original.clone(), int_value(1), int_value(33)])
            .expect("index write should succeed");

        assert_eq!(array_item_int(result.as_ref(), 0), Some(11));
        assert_eq!(array_item_int(result.as_ref(), 1), Some(33));
        assert_eq!(array_item_int(original.as_ref(), 1), Some(22));
    }

    #[test]
    fn index_write_reports_out_of_bounds_indexes() {
        let error = match NativeFunctionCall::new(Op::IndexWrite).call(vec![
            array_value(vec![ValueType::Int(11)]),
            int_value(1),
            int_value(33),
        ]) {
            Ok(_) => panic!("out-of-bounds index should fail"),
            Err(error) => error,
        };
        let message = error.to_string();

        assert!(
            message.contains("Array index out of bounds: 1"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn len_returns_array_lengths() {
        let mut fields = BTreeMap::new();
        fields.insert("hp".to_string(), ValueType::Int(7));

        let cases = [
            (array_value(vec![]), 0),
            (array_value(vec![ValueType::Int(1), ValueType::Int(2)]), 2),
            (
                array_value(vec![
                    ValueType::Object(fields),
                    ValueType::Object(BTreeMap::new()),
                ]),
                2,
            ),
            (
                array_value(vec![
                    ValueType::Array(vec![ValueType::Int(1)]),
                    ValueType::Array(vec![]),
                ]),
                2,
            ),
        ];

        for (value, expected_len) in cases {
            let result = NativeFunctionCall::new(Op::Len)
                .call(vec![value])
                .expect("LEN should succeed");
            assert_eq!(Value::get_value::<i32>(result.as_ref()), Some(expected_len));
        }
    }

    #[test]
    fn len_reports_non_array_values() {
        let error = match NativeFunctionCall::new(Op::Len).call(vec![int_value(1)]) {
            Ok(_) => panic!("LEN on non-array should fail"),
            Err(error) => error,
        };
        let message = error.to_string();

        assert!(
            message.contains("LEN expected an array value as its parameter"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn array_remove_removes_items_by_index() {
        let cases = [(0, [2, 3]), (1, [1, 3]), (2, [1, 2])];

        for (index, expected_items) in cases {
            let result = NativeFunctionCall::new(Op::ArrayRemove)
                .call(vec![
                    array_value(vec![
                        ValueType::Int(1),
                        ValueType::Int(2),
                        ValueType::Int(3),
                    ]),
                    int_value(index),
                ])
                .expect("ARRAY_REMOVE should succeed");

            assert_eq!(array_len(result.as_ref()), Some(2));
            assert_eq!(array_item_int(result.as_ref(), 0), Some(expected_items[0]));
            assert_eq!(array_item_int(result.as_ref(), 1), Some(expected_items[1]));
        }
    }

    #[test]
    fn array_remove_supports_struct_and_nested_arrays() {
        let mut first_fields = BTreeMap::new();
        first_fields.insert("hp".to_string(), ValueType::Int(1));
        let mut second_fields = BTreeMap::new();
        second_fields.insert("hp".to_string(), ValueType::Int(2));

        let struct_result = NativeFunctionCall::new(Op::ArrayRemove)
            .call(vec![
                array_value(vec![
                    ValueType::Object(first_fields),
                    ValueType::Object(second_fields),
                ]),
                int_value(0),
            ])
            .expect("ARRAY_REMOVE should remove from struct arrays");

        assert_eq!(array_len(struct_result.as_ref()), Some(1));
        assert_eq!(
            array_item_object_field_int(struct_result.as_ref(), 0, "hp"),
            Some(2)
        );

        let nested_result = NativeFunctionCall::new(Op::ArrayRemove)
            .call(vec![
                array_value(vec![
                    ValueType::Array(vec![ValueType::Int(1)]),
                    ValueType::Array(vec![ValueType::Int(2), ValueType::Int(3)]),
                ]),
                int_value(0),
            ])
            .expect("ARRAY_REMOVE should remove from nested arrays");

        assert_eq!(array_len(nested_result.as_ref()), Some(1));
        assert_eq!(nested_array_item_int(nested_result.as_ref(), 0, 0), Some(2));
    }

    #[test]
    fn array_remove_reports_out_of_bounds_indexes() {
        for index in [-1, 1] {
            let error = match NativeFunctionCall::new(Op::ArrayRemove).call(vec![
                array_value(vec![ValueType::Int(11)]),
                int_value(index),
            ]) {
                Ok(_) => panic!("out-of-bounds index should fail"),
                Err(error) => error,
            };
            let message = error.to_string();

            assert!(
                message.contains(&format!("Array index out of bounds: {index}")),
                "unexpected error: {message}"
            );
        }
    }
}
