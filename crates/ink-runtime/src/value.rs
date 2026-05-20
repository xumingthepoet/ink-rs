use std::fmt;

use crate::{
    object::{Object, RTObject},
    path::Path,
    story_error::StoryError,
    value_type::{DictKey, StringValue, ValueType, VariablePointerValue},
};

const CAST_BOOL: u8 = 0;
const CAST_INT: u8 = 1;
const CAST_FLOAT: u8 = 2;
const CAST_STRING: u8 = 3;
const CAST_DIVERT_TARGET: u8 = 4;
const CAST_VARIABLE_POINTER: u8 = 5;
const CAST_ARRAY: u8 = 6;
const CAST_OBJECT: u8 = 7;
const CAST_DICT: u8 = 8;

pub struct Value {
    obj: Object,
    pub value: ValueType,
}

impl RTObject for Value {
    fn get_object(&self) -> &Object {
        &self.obj
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_value_type(&self.value, f)
    }
}

fn fmt_value_type(value: &ValueType, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match value {
        ValueType::Bool(v) => write!(f, "{}", v),
        ValueType::Int(v) => write!(f, "{}", v),
        ValueType::Float(v) => write!(f, "{}", v),
        ValueType::String(v) => write!(f, "{}", v.string),
        ValueType::DivertTarget(p) => write!(f, "DivertTargetValue({})", p),
        ValueType::VariablePointer(v) => write!(f, "VariablePointerValue({})", v.variable_name),
        ValueType::Array(values) => {
            write!(f, "[")?;
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                fmt_value_type(value, f)?;
            }
            write!(f, "]")
        }
        ValueType::Object(fields) => {
            write!(f, "{{")?;
            for (index, (name, value)) in fields.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}: ", name)?;
                fmt_value_type(value, f)?;
            }
            write!(f, "}}")
        }
        ValueType::Dict(dict) => {
            write!(f, "Dict<{}>{{", dict.key_type())?;
            for (index, (key, value)) in dict.entries().iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                fmt_dict_key(key, f)?;
                write!(f, ": ")?;
                fmt_value_type(value, f)?;
            }
            write!(f, "}}")
        }
    }
}

fn fmt_dict_key(key: &DictKey, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match key {
        DictKey::String(value) => write!(f, "\"{}\"", value),
        DictKey::Int(value) => write!(f, "{}", value),
    }
}

impl<T: Into<ValueType>> From<T> for Value {
    fn from(value: T) -> Self {
        Self::new_value_type(value.into())
    }
}

impl<'val> TryFrom<&'val dyn RTObject> for &'val StringValue {
    type Error = ();
    fn try_from(o: &dyn RTObject) -> Result<&StringValue, Self::Error> {
        match o.as_any().downcast_ref::<Value>() {
            Some(v) => match &v.value {
                ValueType::String(v) => Ok(v),
                _ => Err(()),
            },
            None => Err(()),
        }
    }
}
impl<'val> TryFrom<&'val dyn RTObject> for &'val VariablePointerValue {
    type Error = ();
    fn try_from(o: &dyn RTObject) -> Result<&VariablePointerValue, Self::Error> {
        match o.as_any().downcast_ref::<Value>() {
            Some(v) => match &v.value {
                ValueType::VariablePointer(v) => Ok(v),
                _ => Err(()),
            },
            None => Err(()),
        }
    }
}
impl<'val> TryFrom<&'val dyn RTObject> for &'val Path {
    type Error = ();
    fn try_from(o: &dyn RTObject) -> Result<&Path, Self::Error> {
        match o.as_any().downcast_ref::<Value>() {
            Some(v) => match &v.value {
                ValueType::DivertTarget(p) => Ok(p),
                _ => Err(()),
            },
            None => Err(()),
        }
    }
}
impl TryFrom<&dyn RTObject> for i32 {
    type Error = ();
    fn try_from(o: &dyn RTObject) -> Result<i32, Self::Error> {
        match o.as_any().downcast_ref::<Value>() {
            Some(v) => match &v.value {
                ValueType::Int(v) => Ok(*v),
                _ => Err(()),
            },
            None => Err(()),
        }
    }
}
impl TryFrom<&dyn RTObject> for f32 {
    type Error = ();
    fn try_from(o: &dyn RTObject) -> Result<f32, Self::Error> {
        match o.as_any().downcast_ref::<Value>() {
            Some(v) => match &v.value {
                ValueType::Float(v) => Ok(*v),
                _ => Err(()),
            },
            None => Err(()),
        }
    }
}
impl Value {
    pub fn new_value_type(valuetype: ValueType) -> Self {
        Self {
            obj: Object::new(),
            value: valuetype,
        }
    }

    pub fn new<T: Into<Value>>(v: T) -> Self {
        v.into()
    }

    pub fn new_variable_pointer(variable_name: &str, context_index: i32) -> Self {
        Self {
            obj: Object::new(),
            value: ValueType::VariablePointer(VariablePointerValue {
                variable_name: variable_name.to_string(),
                context_index,
            }),
        }
    }

    pub fn get_value<'val, T>(o: &'val dyn RTObject) -> Option<T>
    where
        &'val dyn RTObject: TryInto<T>,
    {
        o.try_into().ok()
    }

    pub(crate) fn get_bool_value(o: &dyn RTObject) -> Option<bool> {
        match o.as_any().downcast_ref::<Value>() {
            Some(v) => match &v.value {
                ValueType::Bool(v) => Some(*v),
                _ => None,
            },
            None => None,
        }
    }

    pub fn is_truthy(&self) -> Result<bool, StoryError> {
        match &self.value {
            ValueType::Bool(v) => Ok(*v),
            ValueType::Int(v) => Ok(*v != 0),
            ValueType::Float(v) => Ok(*v != 0.0),
            ValueType::String(v) => Ok(!v.string.is_empty()),
            ValueType::DivertTarget(_) => Err(StoryError::InvalidStoryState(
                "Shouldn't be checking the truthiness of a divert target".to_owned(),
            )),
            ValueType::VariablePointer(_) => Err(StoryError::InvalidStoryState(
                "Shouldn't be checking the truthiness of a variable pointer".to_owned(),
            )),
            ValueType::Array(_) => Err(StoryError::InvalidStoryState(
                "Shouldn't be checking the truthiness of an array".to_owned(),
            )),
            ValueType::Object(_) => Err(StoryError::InvalidStoryState(
                "Shouldn't be checking the truthiness of an object".to_owned(),
            )),
            ValueType::Dict(_) => Err(StoryError::InvalidStoryState(
                "Shouldn't be checking the truthiness of a dict".to_owned(),
            )),
        }
    }

    pub fn get_cast_ordinal(&self) -> u8 {
        let v = &self.value;

        // SAFETY: `ValueType` is `repr(u8)` so every variant has the layout
        // of a struct with its first field being the `u8` discriminant,
        // ensuring the `u8` can be read from a pointer to the enum.
        // See e.g. https://doc.rust-lang.org/std/mem/fn.discriminant.html#accessing-the-numeric-value-of-the-discriminant
        let ptr_to_option = (v as *const ValueType) as *const u8;
        unsafe { *ptr_to_option }
    }

    // If None is returned means that casting is not needed
    pub fn cast(&self, cast_dest_type: u8) -> Result<Option<Value>, StoryError> {
        match &self.value {
            ValueType::Bool(v) => match cast_dest_type {
                CAST_BOOL => Ok(None),
                CAST_INT => {
                    if *v {
                        Ok(Some(Self::new::<i32>(1)))
                    } else {
                        Ok(Some(Self::new::<i32>(0)))
                    }
                }
                CAST_FLOAT => {
                    if *v {
                        Ok(Some(Self::new::<f32>(1.0)))
                    } else {
                        Ok(Some(Self::new::<f32>(0.0)))
                    }
                }
                CAST_STRING => {
                    if *v {
                        Ok(Some(Self::new::<&str>("true")))
                    } else {
                        Ok(Some(Self::new::<&str>("false")))
                    }
                }
                _ => Err(StoryError::InvalidStoryState(
                    "Cast not allowed for bool".to_owned(),
                )),
            },
            ValueType::Int(v) => match cast_dest_type {
                CAST_BOOL => {
                    if *v == 0 {
                        Ok(Some(Self::new::<bool>(false)))
                    } else {
                        Ok(Some(Self::new::<bool>(true)))
                    }
                }
                CAST_INT => Ok(None),
                CAST_FLOAT => Ok(Some(Self::new::<f32>(*v as f32))),
                CAST_STRING => Ok(Some(Self::new::<&str>(&v.to_string()))),
                _ => Err(StoryError::InvalidStoryState(
                    "Cast not allowed for int".to_owned(),
                )),
            },
            ValueType::Float(v) => match cast_dest_type {
                CAST_BOOL => {
                    if *v == 0.0 {
                        Ok(Some(Self::new::<bool>(false)))
                    } else {
                        Ok(Some(Self::new::<bool>(true)))
                    }
                }
                CAST_INT => Ok(Some(Self::new::<i32>(*v as i32))),
                CAST_FLOAT => Ok(None),
                CAST_STRING => Ok(Some(Self::new::<&str>(&v.to_string()))),
                _ => Err(StoryError::InvalidStoryState(
                    "Cast not allowed for float".to_owned(),
                )),
            },
            ValueType::String(v) => match cast_dest_type {
                CAST_INT => v
                    .string
                    .parse::<i32>()
                    .map(Self::new::<i32>)
                    .map(Some)
                    .map_err(|_| {
                        StoryError::InvalidStoryState(format!(
                            "Failed to cast string '{}' to int",
                            v.string
                        ))
                    }),
                CAST_FLOAT => v
                    .string
                    .parse::<f32>()
                    .map(Self::new::<f32>)
                    .map(Some)
                    .map_err(|_| {
                        StoryError::InvalidStoryState(format!(
                            "Failed to cast string '{}' to float",
                            v.string
                        ))
                    }),
                CAST_STRING => Ok(None),
                _ => Err(StoryError::InvalidStoryState(
                    "Cast not allowed for string".to_owned(),
                )),
            },
            ValueType::DivertTarget(_) => match cast_dest_type {
                CAST_DIVERT_TARGET => Ok(None),
                _ => Err(StoryError::InvalidStoryState(
                    "Cast not allowed for divert".to_owned(),
                )),
            },
            ValueType::VariablePointer(_) => match cast_dest_type {
                CAST_VARIABLE_POINTER => Ok(None),
                _ => Err(StoryError::InvalidStoryState(
                    "Cast not allowed for variable pointer".to_owned(),
                )),
            },
            ValueType::Array(_) => match cast_dest_type {
                CAST_ARRAY => Ok(None),
                _ => Err(StoryError::InvalidStoryState(
                    "Cast not allowed for array".to_owned(),
                )),
            },
            ValueType::Object(_) => match cast_dest_type {
                CAST_OBJECT => Ok(None),
                _ => Err(StoryError::InvalidStoryState(
                    "Cast not allowed for object".to_owned(),
                )),
            },
            ValueType::Dict(_) => match cast_dest_type {
                CAST_DICT => Ok(None),
                _ => Err(StoryError::InvalidStoryState(
                    "Cast not allowed for dict".to_owned(),
                )),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_type::{DictKeyType, DictValue};
    use std::collections::BTreeMap;

    #[test]
    fn invalid_string_cast_to_int_returns_error() {
        let value = Value::new::<&str>("not an int");
        let error = match value.cast(CAST_INT) {
            Ok(_) => panic!("invalid int parse should be reported"),
            Err(error) => error,
        };

        assert!(matches!(
            error,
            StoryError::InvalidStoryState(message)
                if message.contains("Failed to cast string 'not an int' to int")
        ));
    }

    #[test]
    fn invalid_string_cast_to_float_returns_error() {
        let value = Value::new::<&str>("not a float");
        let error = match value.cast(CAST_FLOAT) {
            Ok(_) => panic!("invalid float parse should be reported"),
            Err(error) => error,
        };

        assert!(matches!(
            error,
            StoryError::InvalidStoryState(message)
                if message.contains("Failed to cast string 'not a float' to float")
        ));
    }

    #[test]
    fn valid_string_numeric_casts_still_succeed() {
        let int_value = Value::new::<&str>("42")
            .cast(CAST_INT)
            .expect("valid int parse should succeed")
            .expect("string to int should produce a new value");
        assert!(matches!(int_value.value, ValueType::Int(42)));

        let float_value = Value::new::<&str>("3.5")
            .cast(CAST_FLOAT)
            .expect("valid float parse should succeed")
            .expect("string to float should produce a new value");
        assert!(matches!(float_value.value, ValueType::Float(value) if value == 3.5));
    }

    #[test]
    fn dict_values_display_with_key_type_and_reject_truthiness_and_casts() {
        let mut entries = BTreeMap::new();
        entries.insert(DictKey::String("ada".to_string()), ValueType::Int(10));
        entries.insert(
            DictKey::String("items".to_string()),
            ValueType::Array(vec![ValueType::Int(1)]),
        );
        let value = Value::new_value_type(ValueType::Dict(
            DictValue::new(DictKeyType::String, entries).expect("valid dict"),
        ));

        assert_eq!(
            value.to_string(),
            "Dict<string>{\"ada\": 10, \"items\": [1]}"
        );
        assert!(matches!(
            value.is_truthy(),
            Err(StoryError::InvalidStoryState(message))
                if message.contains("truthiness of a dict")
        ));
        assert!(matches!(value.cast(CAST_DICT), Ok(None)));
        assert!(matches!(
            value.cast(CAST_INT),
            Err(StoryError::InvalidStoryState(message))
                if message.contains("Cast not allowed for dict")
        ));
    }
}
