//! A combination of an ink-rs runtime value with its type.
use std::{collections::BTreeMap, fmt};

use crate::{path::Path, story_error::StoryError};

/// An ink-rs runtime value, tagged with its type.
#[repr(u8)]
#[derive(Clone, PartialEq)]
pub enum ValueType {
    Bool(bool),
    Int(i32),
    Float(f32),
    /// String value, constructed with [`new_string`](ValueType::new::<&str>)
    String(StringValue),
    /// Reference to an ink-rs divert.
    DivertTarget(Path),
    /// Reference to an ink-rs variable.
    VariablePointer(VariablePointerValue),
    /// Dynamic ink-rs array value.
    Array(Vec<ValueType>),
    /// Dynamic ink-rs object/struct value.
    Object(BTreeMap<String, ValueType>),
    /// Dynamic ink-rs dictionary value.
    Dict(DictValue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DictKeyType {
    String,
    Int,
}

impl fmt::Display for DictKeyType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => formatter.write_str("string"),
            Self::Int => formatter.write_str("int"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DictKey {
    String(String),
    Int(i32),
}

impl DictKey {
    pub fn key_type(&self) -> DictKeyType {
        match self {
            Self::String(_) => DictKeyType::String,
            Self::Int(_) => DictKeyType::Int,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct DictValue {
    key_type: DictKeyType,
    entries: BTreeMap<DictKey, ValueType>,
}

impl DictValue {
    pub fn new(
        key_type: DictKeyType,
        entries: BTreeMap<DictKey, ValueType>,
    ) -> Result<Self, StoryError> {
        if entries.keys().any(|key| key.key_type() != key_type) {
            return Err(StoryError::InvalidStoryState(
                "Dict entry key type does not match Dict key type".to_owned(),
            ));
        }

        Ok(Self { key_type, entries })
    }

    pub fn empty(key_type: DictKeyType) -> Self {
        Self {
            key_type,
            entries: BTreeMap::new(),
        }
    }

    pub fn key_type(&self) -> DictKeyType {
        self.key_type
    }

    pub fn entries(&self) -> &BTreeMap<DictKey, ValueType> {
        &self.entries
    }

    pub fn into_entries(self) -> BTreeMap<DictKey, ValueType> {
        self.entries
    }

    pub fn get(&self, key: &DictKey) -> Option<&ValueType> {
        if key.key_type() != self.key_type {
            return None;
        }

        self.entries.get(key)
    }
}

impl From<bool> for ValueType {
    fn from(value: bool) -> ValueType {
        ValueType::Bool(value)
    }
}

impl From<i32> for ValueType {
    fn from(value: i32) -> ValueType {
        ValueType::Int(value)
    }
}

impl From<f32> for ValueType {
    fn from(value: f32) -> ValueType {
        ValueType::Float(value)
    }
}

impl From<&str> for ValueType {
    fn from(value: &str) -> ValueType {
        let inline_ws = value.chars().all(|c| c == ' ' || c == '\t');

        ValueType::String(StringValue {
            string: value.to_string(),
            is_inline_whitespace: inline_ws,
            is_newline: value.eq("\n"),
        })
    }
}

impl From<Path> for ValueType {
    fn from(value: Path) -> ValueType {
        ValueType::DivertTarget(value)
    }
}

impl From<VariablePointerValue> for ValueType {
    fn from(value: VariablePointerValue) -> Self {
        ValueType::VariablePointer(value)
    }
}

impl From<Vec<ValueType>> for ValueType {
    fn from(value: Vec<ValueType>) -> Self {
        ValueType::Array(value)
    }
}

impl From<BTreeMap<String, ValueType>> for ValueType {
    fn from(value: BTreeMap<String, ValueType>) -> Self {
        ValueType::Object(value)
    }
}

impl From<DictValue> for ValueType {
    fn from(value: DictValue) -> Self {
        ValueType::Dict(value)
    }
}

impl TryFrom<&ValueType> for bool {
    type Error = ();
    fn try_from(value: &ValueType) -> Result<Self, Self::Error> {
        match value {
            ValueType::Bool(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

impl TryFrom<&ValueType> for i32 {
    type Error = ();
    fn try_from(value: &ValueType) -> Result<Self, Self::Error> {
        match value {
            ValueType::Int(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

impl TryFrom<&ValueType> for f32 {
    type Error = ();
    fn try_from(value: &ValueType) -> Result<Self, Self::Error> {
        match value {
            ValueType::Float(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

impl<'val> TryFrom<&'val ValueType> for &'val str {
    type Error = ();
    fn try_from(value: &'val ValueType) -> Result<Self, Self::Error> {
        match value {
            ValueType::String(v) => Ok(&v.string),
            _ => Err(()),
        }
    }
}

impl ValueType {
    pub fn new<T: Into<ValueType>>(v: T) -> Self {
        v.into()
    }

    pub fn get<'val, T>(&'val self) -> Option<T>
    where
        &'val Self: TryInto<T>,
    {
        self.try_into().ok()
    }

    /// Tries to convert the internal value of this `ValueType` to `i32`
    pub fn coerce_to_int(&self) -> Result<i32, StoryError> {
        match self {
            ValueType::Bool(v) => {
                if *v {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            ValueType::Int(v) => Ok(*v),
            ValueType::Float(v) => Ok(*v as i32),
            _ => Err(StoryError::BadArgument("Failed to cast to int".to_owned())),
        }
    }

    /// Tries to convert the internal value of this `ValueType` to `f32`
    pub fn coerce_to_float(&self) -> Result<f32, StoryError> {
        match self {
            ValueType::Bool(v) => {
                if *v {
                    Ok(1.0)
                } else {
                    Ok(0.0)
                }
            }
            ValueType::Int(v) => Ok(*v as f32),
            ValueType::Float(v) => Ok(*v),
            _ => Err(StoryError::BadArgument(
                "Failed to cast to float".to_owned(),
            )),
        }
    }

    /// Tries to convert the internal value of this `ValueType` to `bool`
    pub fn coerce_to_bool(&self) -> Result<bool, StoryError> {
        match self {
            ValueType::Bool(v) => Ok(*v),
            ValueType::Int(v) => {
                if *v == 1 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Err(StoryError::BadArgument(
                "Failed to cast to boolean".to_owned(),
            )),
        }
    }

    /// Tries to convert the internal value of this `ValueType` to `String`
    pub fn coerce_to_string(&self) -> Result<String, StoryError> {
        match self {
            ValueType::Bool(v) => Ok(v.to_string()),
            ValueType::Int(v) => Ok(v.to_string()),
            ValueType::Float(v) => Ok(v.to_string()),
            ValueType::String(v) => Ok(v.string.clone()),
            _ => Err(StoryError::BadArgument(
                "Failed to cast to float".to_owned(),
            )),
        }
    }
}

/// ink-rs runtime representation of a string.
#[derive(Clone, PartialEq)]
pub struct StringValue {
    /// The internal string value.
    pub string: String,
    pub(crate) is_inline_whitespace: bool,
    pub(crate) is_newline: bool,
}

impl StringValue {
    pub fn is_non_whitespace(&self) -> bool {
        !self.is_newline && !self.is_inline_whitespace
    }
}

/// ink-rs runtime representation of a reference to a variable.
#[derive(Clone, PartialEq)]
pub struct VariablePointerValue {
    pub(crate) variable_name: String,

    // Where the variable is located
    // -1 = default, unknown, yet to be determined
    // 0  = in global scope
    // 1+ = callstack element index + 1 (so that the first doesn't conflict with special global scope)
    pub(crate) context_index: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_values_clone_and_compare_by_contents() {
        let original = ValueType::Array(vec![
            ValueType::Int(1),
            ValueType::Array(vec![ValueType::Bool(true), ValueType::new("nested")]),
        ]);

        let cloned = original.clone();

        assert!(original == cloned);

        let ValueType::Array(mut edited) = cloned else {
            panic!("expected array value");
        };
        edited.push(ValueType::Int(2));

        assert!(original != ValueType::Array(edited));
    }

    #[test]
    fn object_values_clone_and_compare_by_contents() {
        let mut original_fields = BTreeMap::new();
        original_fields.insert("count".to_string(), ValueType::Int(3));
        original_fields.insert(
            "items".to_string(),
            ValueType::Array(vec![ValueType::new("a"), ValueType::new("b")]),
        );
        let original = ValueType::Object(original_fields);

        let cloned = original.clone();

        assert!(original == cloned);

        let ValueType::Object(mut edited_fields) = cloned else {
            panic!("expected object value");
        };
        edited_fields.insert("count".to_string(), ValueType::Int(4));

        assert!(original != ValueType::Object(edited_fields));
    }

    #[test]
    fn dict_values_clone_and_compare_by_contents_and_key_type() {
        let mut original_entries = BTreeMap::new();
        original_entries.insert(DictKey::String("score".to_string()), ValueType::Int(10));
        original_entries.insert(
            DictKey::String("items".to_string()),
            ValueType::Array(vec![ValueType::new("a"), ValueType::new("b")]),
        );
        let original = ValueType::Dict(
            DictValue::new(DictKeyType::String, original_entries).expect("valid string dict"),
        );

        let cloned = original.clone();

        assert!(original == cloned);

        let ValueType::Dict(cloned_dict) = cloned else {
            panic!("expected dict value");
        };
        let mut edited_entries = cloned_dict.into_entries();
        edited_entries.insert(DictKey::String("score".to_string()), ValueType::Int(11));

        assert!(
            original
                != ValueType::Dict(
                    DictValue::new(DictKeyType::String, edited_entries)
                        .expect("edited entries should keep key type"),
                )
        );
        assert!(original != ValueType::Dict(DictValue::empty(DictKeyType::Int)));
    }

    #[test]
    fn dict_values_reject_mismatched_key_type_entries() {
        let mut entries = BTreeMap::new();
        entries.insert(DictKey::String("score".to_string()), ValueType::Int(10));

        assert!(DictValue::new(DictKeyType::Int, entries).is_err());
    }
}
