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

        let coerced_params = self.coerce_values_to_single_type(params)?;

        self.call_type(coerced_params)
    }

    fn call_type(&self, coerced_params: Vec<Rc<Value>>) -> Result<Rc<dyn RTObject>, StoryError> {
        match self.op {
            Op::Add => self.add_op(&coerced_params),
            Op::Subtract => self.subtract_op(&coerced_params),
            Op::Divide => self.divide_op(&coerced_params),
            Op::Multiply => self.multiply_op(&coerced_params),
            Op::Mod => self.mod_op(&coerced_params),
            Op::Negate => self.negate_op(&coerced_params),
            Op::Equal => self.equal_op(&coerced_params),
            Op::Greater => self.greater_op(&coerced_params),
            Op::Less => self.less_op(&coerced_params),
            Op::GreaterThanOrEquals => self.greater_than_or_equals_op(&coerced_params),
            Op::LessThanOrEquals => self.less_than_or_equals_op(&coerced_params),
            Op::NotEquals => self.not_equals_op(&coerced_params),
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

    fn equal_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Bool(op1) => match params[1].value {
                ValueType::Bool(op2) => Ok(Rc::new(Value::new::<bool>(*op1 == op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<bool>(*op1 == op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<bool>(*op1 == op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::String(op1) => match &params[1].value {
                ValueType::String(op2) => {
                    Ok(Rc::new(Value::new::<bool>(op1.string.eq(&op2.string))))
                }
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::DivertTarget(op1) => match &params[1].value {
                ValueType::DivertTarget(op2) => Ok(Rc::new(Value::new::<bool>(op1.eq(op2)))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            _ => Err(StoryError::InvalidStoryState(
                "Operation not available for type.".to_owned(),
            )),
        }
    }

    fn not_equals_op(&self, params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
        match &params[0].value {
            ValueType::Bool(op1) => match params[1].value {
                ValueType::Bool(op2) => Ok(Rc::new(Value::new::<bool>(*op1 != op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Int(op1) => match params[1].value {
                ValueType::Int(op2) => Ok(Rc::new(Value::new::<bool>(*op1 != op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::Float(op1) => match params[1].value {
                ValueType::Float(op2) => Ok(Rc::new(Value::new::<bool>(*op1 != op2))),
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::String(op1) => match &params[1].value {
                ValueType::String(op2) => {
                    Ok(Rc::new(Value::new::<bool>(!op1.string.eq(&op2.string))))
                }
                _ => Err(StoryError::InvalidStoryState(
                    "Operation not available for type.".to_owned(),
                )),
            },
            ValueType::DivertTarget(op1) => match &params[1].value {
                ValueType::DivertTarget(op2) => Ok(Rc::new(Value::new::<bool>(!op1.eq(op2)))),
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
