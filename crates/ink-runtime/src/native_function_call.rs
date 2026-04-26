use std::{fmt, rc::Rc};

use crate::{
    object::{Object, RTObject},
    story_error::StoryError,
    value::Value,
    value_type::ValueType,
    void::Void,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Op {
    Add,
    Subtract,
    Divide,
    Multiply,
    Mod,
    Negate,

    Equal,
    Greater,
    Less,
    GreaterThanOrEquals,
    LessThanOrEquals,
    NotEquals,
    Not,

    And,
    Or,

    Min,
    Max,

    Pow,
    Floor,
    Ceiling,
    Int,
    Float,

    Has,
    Hasnt,

    FieldRead,
    IndexRead,
    FieldWrite,
    IndexWrite,
    Len,
    ArrayRemove,
}

const ADD_NAME: &str = "+";
const SUBTRACT_NAME: &str = "-";
const DIVIDE_NAME: &str = "/";
const MULTIPLY_NAME: &str = "*";
const MOD_NAME: &str = "%";
const NEGATE_NAME: &str = "_";
const EQUAL_NAME: &str = "==";
const GREATER_NAME: &str = ">";
const LESS_NAME: &str = "<";
const GREATER_THAN_OR_EQUALS_NAME: &str = ">=";
const LESS_THAN_OR_EQUALS_NAME: &str = "<=";
const NOT_EQUALS_NAME: &str = "!=";
const NOT_NAME: &str = "!";
const AND_NAME: &str = "&&";
const OR_NAME: &str = "||";
const MIN_NAME: &str = "MIN";
const MAX_NAME: &str = "MAX";
const POW_NAME: &str = "POW";
const FLOOR_NAME: &str = "FLOOR";
const CEILING_NAME: &str = "CEILING";
const INT_NAME: &str = "INT";
const FLOAT_NAME: &str = "FLOAT";
const HAS_NAME: &str = "?";
const HASNT_NAME: &str = "!?";
const FIELD_READ_NAME: &str = "FIELD";
const INDEX_READ_NAME: &str = "INDEX";
const FIELD_WRITE_NAME: &str = "SET_FIELD";
const INDEX_WRITE_NAME: &str = "SET_INDEX";
const LEN_NAME: &str = "LEN";
const ARRAY_REMOVE_NAME: &str = "ARRAY_REMOVE";

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

    pub fn new_from_name(name: &str) -> Option<Self> {
        match name {
            ADD_NAME => Some(Self::new(Op::Add)),
            SUBTRACT_NAME => Some(Self::new(Op::Subtract)),
            DIVIDE_NAME => Some(Self::new(Op::Divide)),
            MULTIPLY_NAME => Some(Self::new(Op::Multiply)),
            MOD_NAME => Some(Self::new(Op::Mod)),
            NEGATE_NAME => Some(Self::new(Op::Negate)),
            EQUAL_NAME => Some(Self::new(Op::Equal)),
            GREATER_NAME => Some(Self::new(Op::Greater)),
            LESS_NAME => Some(Self::new(Op::Less)),
            GREATER_THAN_OR_EQUALS_NAME => Some(Self::new(Op::GreaterThanOrEquals)),
            LESS_THAN_OR_EQUALS_NAME => Some(Self::new(Op::LessThanOrEquals)),
            NOT_EQUALS_NAME => Some(Self::new(Op::NotEquals)),
            NOT_NAME => Some(Self::new(Op::Not)),
            AND_NAME => Some(Self::new(Op::And)),
            OR_NAME => Some(Self::new(Op::Or)),
            MIN_NAME => Some(Self::new(Op::Min)),
            MAX_NAME => Some(Self::new(Op::Max)),
            POW_NAME => Some(Self::new(Op::Pow)),
            FLOOR_NAME => Some(Self::new(Op::Floor)),
            CEILING_NAME => Some(Self::new(Op::Ceiling)),
            INT_NAME => Some(Self::new(Op::Int)),
            FLOAT_NAME => Some(Self::new(Op::Float)),
            HAS_NAME => Some(Self::new(Op::Has)),
            HASNT_NAME => Some(Self::new(Op::Hasnt)),
            FIELD_READ_NAME => Some(Self::new(Op::FieldRead)),
            INDEX_READ_NAME => Some(Self::new(Op::IndexRead)),
            FIELD_WRITE_NAME => Some(Self::new(Op::FieldWrite)),
            INDEX_WRITE_NAME => Some(Self::new(Op::IndexWrite)),
            LEN_NAME => Some(Self::new(Op::Len)),
            ARRAY_REMOVE_NAME => Some(Self::new(Op::ArrayRemove)),
            _ => None,
        }
    }

    pub fn get_name(op: Op) -> String {
        match op {
            Op::Add => ADD_NAME.to_owned(),
            Op::Subtract => SUBTRACT_NAME.to_owned(),
            Op::Divide => DIVIDE_NAME.to_owned(),
            Op::Multiply => MULTIPLY_NAME.to_owned(),
            Op::Mod => MOD_NAME.to_owned(),
            Op::Negate => NEGATE_NAME.to_owned(),
            Op::Equal => EQUAL_NAME.to_owned(),
            Op::Greater => GREATER_NAME.to_owned(),
            Op::Less => LESS_NAME.to_owned(),
            Op::GreaterThanOrEquals => GREATER_THAN_OR_EQUALS_NAME.to_owned(),
            Op::LessThanOrEquals => LESS_THAN_OR_EQUALS_NAME.to_owned(),
            Op::NotEquals => NOT_EQUALS_NAME.to_owned(),
            Op::Not => NOT_NAME.to_owned(),
            Op::And => AND_NAME.to_owned(),
            Op::Or => OR_NAME.to_owned(),
            Op::Min => MIN_NAME.to_owned(),
            Op::Max => MAX_NAME.to_owned(),
            Op::Pow => POW_NAME.to_owned(),
            Op::Floor => FLOOR_NAME.to_owned(),
            Op::Ceiling => CEILING_NAME.to_owned(),
            Op::Int => INT_NAME.to_owned(),
            Op::Float => FLOAT_NAME.to_owned(),
            Op::Has => HAS_NAME.to_owned(),
            Op::Hasnt => HASNT_NAME.to_owned(),
            Op::FieldRead => FIELD_READ_NAME.to_owned(),
            Op::IndexRead => INDEX_READ_NAME.to_owned(),
            Op::FieldWrite => FIELD_WRITE_NAME.to_owned(),
            Op::IndexWrite => INDEX_WRITE_NAME.to_owned(),
            Op::Len => LEN_NAME.to_owned(),
            Op::ArrayRemove => ARRAY_REMOVE_NAME.to_owned(),
        }
    }

    pub fn get_number_of_parameters(&self) -> usize {
        match self.op {
            Op::Add => 2,
            Op::Subtract => 2,
            Op::Divide => 2,
            Op::Multiply => 2,
            Op::Mod => 2,
            Op::Negate => 1,
            Op::Equal => 2,
            Op::Greater => 2,
            Op::Less => 2,
            Op::GreaterThanOrEquals => 2,
            Op::LessThanOrEquals => 2,
            Op::NotEquals => 2,
            Op::Not => 1,
            Op::And => 2,
            Op::Or => 2,
            Op::Min => 2,
            Op::Max => 2,
            Op::Pow => 2,
            Op::Floor => 1,
            Op::Ceiling => 1,
            Op::Int => 1,
            Op::Float => 1,
            Op::Has => 2,
            Op::Hasnt => 2,
            Op::FieldRead => 2,
            Op::IndexRead => 2,
            Op::FieldWrite => 3,
            Op::IndexWrite => 3,
            Op::Len => 1,
            Op::ArrayRemove => 2,
        }
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
            Op::Add => return self.add(&params),
            Op::Equal => return self.equal(&params),
            Op::NotEquals => return self.not_equals(&params),
            Op::FieldRead => return self.field_read(&params),
            Op::IndexRead => return self.index_read(&params),
            Op::FieldWrite => return self.field_write(&params),
            Op::IndexWrite => return self.index_write(&params),
            Op::Len => return self.len(&params),
            Op::ArrayRemove => return self.array_remove(&params),
            _ => {}
        }

        let coerced_params = self.coerce_values_to_single_type(params)?;

        self.call_type(coerced_params)
    }

    fn call_type(&self, coerced_params: Vec<Rc<Value>>) -> Result<Rc<dyn RTObject>, StoryError> {
        match self.op {
            Op::Add => unreachable!("addition uses string-aware parameters"),
            Op::Subtract => self.subtract_op(&coerced_params),
            Op::Divide => self.divide_op(&coerced_params),
            Op::Multiply => self.multiply_op(&coerced_params),
            Op::Mod => self.mod_op(&coerced_params),
            Op::Negate => self.negate_op(&coerced_params),
            Op::Equal => unreachable!("equality uses uncoerced parameters"),
            Op::Greater => self.greater_op(&coerced_params),
            Op::Less => self.less_op(&coerced_params),
            Op::GreaterThanOrEquals => self.greater_than_or_equals_op(&coerced_params),
            Op::LessThanOrEquals => self.less_than_or_equals_op(&coerced_params),
            Op::NotEquals => unreachable!("inequality uses uncoerced parameters"),
            Op::Not => self.not_op(&coerced_params),
            Op::And => self.and_op(&coerced_params),
            Op::Or => self.or_op(&coerced_params),
            Op::Min => self.min_op(&coerced_params),
            Op::Max => self.max_op(&coerced_params),
            Op::Pow => self.pow_op(&coerced_params),
            Op::Floor => self.floor_op(&coerced_params),
            Op::Ceiling => self.ceiling_op(&coerced_params),
            Op::Int => self.int_op(&coerced_params),
            Op::Float => self.float_op(&coerced_params),
            Op::Has => self.has(&coerced_params),
            Op::Hasnt => self.hasnt(&coerced_params),
            Op::FieldRead => unreachable!("field read uses uncoerced parameters"),
            Op::IndexRead => unreachable!("index read uses uncoerced parameters"),
            Op::FieldWrite => unreachable!("field write uses uncoerced parameters"),
            Op::IndexWrite => unreachable!("index write uses uncoerced parameters"),
            Op::Len => unreachable!("LEN uses uncoerced parameters"),
            Op::ArrayRemove => unreachable!("ARRAY_REMOVE uses uncoerced parameters"),
        }
    }

    fn add(&self, params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
        let left = native_value_param(params, 0, "+")?;
        let right = native_value_param(params, 1, "+")?;
        if matches!(left.value, ValueType::String(_)) || matches!(right.value, ValueType::String(_))
        {
            return match (&left.value, &right.value) {
                (ValueType::String(left), ValueType::String(right)) => {
                    let mut output = String::new();
                    output.push_str(&left.string);
                    output.push_str(&right.string);
                    Ok(Rc::new(Value::new::<&str>(&output)))
                }
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            };
        }

        let coerced_params = self.coerce_values_to_single_type(params.to_vec())?;
        self.add_op(&coerced_params)
    }

    fn equal(&self, params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
        let left = native_value_param(params, 0, "==")?;
        let right = native_value_param(params, 1, "==")?;
        let equal = if value_type_is_composite(&left.value) || value_type_is_composite(&right.value)
        {
            value_types_equal(&left.value, &right.value)
        } else {
            let coerced_params = self.coerce_values_to_single_type(params.to_vec())?;
            coerced_values_equal(&coerced_params)?
        };

        Ok(Rc::new(Value::new::<bool>(equal)))
    }

    fn not_equals(&self, params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
        let equals_result = self.equal(params)?;
        let equals = Value::get_bool_value(equals_result.as_ref()).ok_or_else(|| {
            StoryError::InvalidStoryState("!= expected equality to return bool".to_owned())
        })?;

        Ok(Rc::new(Value::new::<bool>(!equals)))
    }

    fn field_read(&self, params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

    fn len(&self, params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
        let value = params[0]
            .as_ref()
            .as_any()
            .downcast_ref::<Value>()
            .ok_or_else(|| {
                StoryError::InvalidStoryState(
                    "LEN expected an array value as its parameter".to_owned(),
                )
            })?;

        match &value.value {
            ValueType::Array(values) => Ok(Rc::new(Value::new::<i32>(values.len() as i32))),
            _ => Err(StoryError::InvalidStoryState(
                "LEN expected an array value as its parameter".to_owned(),
            )),
        }
    }

    fn array_remove(&self, params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

    fn index_write(&self, params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

    fn field_write(&self, params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

    fn index_read(&self, params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

    fn coerce_values_to_single_type(
        &self,
        params: Vec<Rc<dyn RTObject>>,
    ) -> Result<Vec<Rc<Value>>, StoryError> {
        let mut dest_type = 1; // Int
        let mut result: Vec<Rc<Value>> = Vec::new();

        for obj in params.iter() {
            // Find out what the output type is
            // "higher level" types infect both so that binary operations
            // use the same type on both sides. e.g. binary operation of
            // int and float causes the int to be casted to a float.
            if let Some(v) = obj.as_ref().as_any().downcast_ref::<Value>() {
                if v.get_cast_ordinal() > dest_type {
                    dest_type = v.get_cast_ordinal();
                }
            }
        }

        for obj in params.iter() {
            if let Some(v) = obj.as_ref().as_any().downcast_ref::<Value>() {
                match v.cast(dest_type)? {
                    Some(casted_value) => result.push(Rc::new(casted_value)),
                    None => {
                        if let Ok(obj) = obj.clone().into_any().downcast::<Value>() {
                            result.push(obj);
                        }
                    }
                }
            } else {
                return Err(StoryError::InvalidStoryState(format!(
                    "RTObject of type Value expected: {}",
                    obj
                )));
            }
        }

        Ok(result)
    }

    fn and_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Bool(op1) => match params[1].value {
                ValueType::Bool(op2) => Ok(Rc::new(Value::new::<bool>(*op1 && op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<bool>(*op1 != 0 && op2 != 0))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<bool>(*op1 != 0.0 && op2 != 0.0))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn greater_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<bool>(*op1 > op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<bool>(*op1 > op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn less_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<bool>(*op1 < op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<bool>(*op1 < op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn greater_than_or_equals_op(
        &self,
        params: &[Rc<Value>],
    ) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<bool>(*op1 >= op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<bool>(*op1 >= op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn less_than_or_equals_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<bool>(*op1 <= op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<bool>(*op1 <= op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn subtract_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<i32>(*op1 - op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<f32>(*op1 - op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn add_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<i32>(op1 + op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<f32>(op1 + op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::String(op1) => match &params[1].value {
                ValueType::String(op2) => {
                    let mut sb = String::new();
                    sb.push_str(&op1.string);
                    sb.push_str(&op2.string);
                    Ok(Rc::new(Value::new::<&str>(&sb)))
                }
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn divide_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match params[0].value {
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<i32>(op1 / op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<f32>(op1 / op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn pow_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match params[0].value {
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => {
                    Ok(Rc::new(Value::new::<f32>((op1 as f32).powf(op2 as f32))))
                }
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<f32>(op1.powf(op2)))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn multiply_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match params[0].value {
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<i32>(op1 * op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<f32>(op1 * op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn or_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Bool(op1) => match params[1].value {
                ValueType::Bool(op2) => Ok(Rc::new(Value::new::<bool>(*op1 || op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<bool>(*op1 != 0 || op2 != 0))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<bool>(*op1 != 0.0 || op2 != 0.0))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn not_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => Ok(Rc::new(Value::new::<bool>(*op1 == 0))),
            ValueType::Float(op1) => Ok(Rc::new(Value::new::<bool>(*op1 == 0.0))),
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn min_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<i32>(i32::min(*op1, op2)))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<f32>(f32::min(*op1, op2)))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn max_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<i32>(i32::max(*op1, op2)))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<f32>(f32::max(*op1, op2)))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn mod_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match params[0].value {
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<i32>(op1 % op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<f32>(op1 % op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn has(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::String(op1) => match &params[1].value {
                ValueType::String(op2) => Ok(Rc::new(Value::new::<bool>(
                    op1.string.contains(&op2.string),
                ))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn hasnt(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::String(op1) => match &params[1].value {
                ValueType::String(op2) => Ok(Rc::new(Value::new::<bool>(
                    !op1.string.contains(&op2.string),
                ))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn negate_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => Ok(Rc::new(Value::new::<i32>(-op1))),
            ValueType::Float(op1) => Ok(Rc::new(Value::new::<f32>(-op1))),
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn floor_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => Ok(Rc::new(Value::new::<i32>(*op1))),
            ValueType::Float(op1) => Ok(Rc::new(Value::new::<f32>(op1.floor()))),
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn ceiling_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => Ok(Rc::new(Value::new::<i32>(*op1))),
            ValueType::Float(op1) => Ok(Rc::new(Value::new::<f32>(op1.ceil()))),
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn int_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => Ok(Rc::new(Value::new::<i32>(*op1))),
            ValueType::Float(op1) => Ok(Rc::new(Value::new::<i32>(*op1 as i32))),
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn float_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Int(op1) => Ok(Rc::new(Value::new::<f32>(*op1 as f32))),
            ValueType::Float(op1) => Ok(Rc::new(Value::new::<f32>(*op1))),
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }
}

fn native_value_param<'a>(
    params: &'a [Rc<dyn RTObject>],
    index: usize,
    op_name: &str,
) -> Result<&'a Value, StoryError> {
    params[index]
        .as_ref()
        .as_any()
        .downcast_ref::<Value>()
        .ok_or_else(|| {
            StoryError::InvalidStoryState(format!("{op_name} expected value parameters"))
        })
}

fn value_type_is_composite(value: &ValueType) -> bool {
    matches!(value, ValueType::Array(_) | ValueType::Object(_))
}

fn coerced_values_equal(params: &[Rc<Value>]) -> Result<bool, StoryError> {
    match &params[0].value {
        ValueType::Bool(op1) => match params[1].value {
            ValueType::Bool(op2) => Ok(*op1 == op2),
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        },
        ValueType::Int(op1) => match params[1].value {
            ValueType::Int(op2) => Ok(*op1 == op2),
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        },
        ValueType::Float(op1) => match params[1].value {
            ValueType::Float(op2) => Ok(*op1 == op2),
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        },
        ValueType::String(op1) => match &params[1].value {
            ValueType::String(op2) => Ok(op1.string.eq(&op2.string)),
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        },
        ValueType::DivertTarget(op1) => match &params[1].value {
            ValueType::DivertTarget(op2) => Ok(op1.eq(op2)),
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        },
        _ => Err(StoryError::InvalidStoryState(
            "Operation not available for type.".to_owned(),
        )),
    }
}

fn value_types_equal(left: &ValueType, right: &ValueType) -> bool {
    match (left, right) {
        (ValueType::Bool(left), ValueType::Bool(right)) => left == right,
        (ValueType::Int(left), ValueType::Int(right)) => left == right,
        (ValueType::Float(left), ValueType::Float(right)) => left == right,
        (ValueType::String(left), ValueType::String(right)) => left.string == right.string,
        (ValueType::DivertTarget(left), ValueType::DivertTarget(right)) => left == right,
        (ValueType::VariablePointer(left), ValueType::VariablePointer(right)) => left == right,
        (ValueType::Array(left), ValueType::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| value_types_equal(left, right))
        }
        (ValueType::Object(left), ValueType::Object(right)) => {
            left.len() == right.len()
                && left.iter().all(|(field_name, left_value)| {
                    right
                        .get(field_name)
                        .is_some_and(|right_value| value_types_equal(left_value, right_value))
                })
        }
        _ => false,
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
