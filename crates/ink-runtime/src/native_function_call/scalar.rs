use std::rc::Rc;

use super::{params, Op};
use crate::{object::RTObject, story_error::StoryError, value::Value, value_type::ValueType};

pub(super) fn call(op: Op, params: Vec<Rc<dyn RTObject>>) -> Result<Rc<dyn RTObject>, StoryError> {
    match op {
        Op::Add => return add(&params),
        Op::Equal => return equal(&params),
        Op::NotEquals => return not_equals(&params),
        _ => {}
    }

    let coerced_params = coerce_values_to_single_type(params)?;
    call_type(op, &coerced_params)
}

fn call_type(op: Op, coerced_params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
    match op {
        Op::Add => unreachable!("addition uses string-aware parameters"),
        Op::Subtract => subtract_op(coerced_params),
        Op::Divide => divide_op(coerced_params),
        Op::Multiply => multiply_op(coerced_params),
        Op::Mod => mod_op(coerced_params),
        Op::Negate => negate_op(coerced_params),
        Op::Equal => unreachable!("equality uses uncoerced parameters"),
        Op::Greater => greater_op(coerced_params),
        Op::Less => less_op(coerced_params),
        Op::GreaterThanOrEquals => greater_than_or_equals_op(coerced_params),
        Op::LessThanOrEquals => less_than_or_equals_op(coerced_params),
        Op::NotEquals => unreachable!("inequality uses uncoerced parameters"),
        Op::Not => not_op(coerced_params),
        Op::And => and_op(coerced_params),
        Op::Or => or_op(coerced_params),
        Op::Min => min_op(coerced_params),
        Op::Max => max_op(coerced_params),
        Op::Pow => pow_op(coerced_params),
        Op::Floor => floor_op(coerced_params),
        Op::Ceiling => ceiling_op(coerced_params),
        Op::Int => int_op(coerced_params),
        Op::Float => float_op(coerced_params),
        Op::Has => has(coerced_params),
        Op::Hasnt => hasnt(coerced_params),
        Op::FieldRead => unreachable!("field read uses uncoerced parameters"),
        Op::IndexRead => unreachable!("index read uses uncoerced parameters"),
        Op::FieldWrite => unreachable!("field write uses uncoerced parameters"),
        Op::IndexWrite => unreachable!("index write uses uncoerced parameters"),
        Op::Len => unreachable!("LEN uses uncoerced parameters"),
        Op::ArrayRemove => unreachable!("ARRAY_REMOVE uses uncoerced parameters"),
    }
}

fn add(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let left = params::value_for_operation(params, 0, "+")?;
    let right = params::value_for_operation(params, 1, "+")?;
    if matches!(left.value, ValueType::String(_)) || matches!(right.value, ValueType::String(_)) {
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

    let coerced_params = coerce_values_to_single_type(params.to_vec())?;
    add_op(&coerced_params)
}

fn equal(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let left = params::value_for_operation(params, 0, "==")?;
    let right = params::value_for_operation(params, 1, "==")?;
    let equal = if value_type_is_composite(&left.value) || value_type_is_composite(&right.value) {
        value_types_equal(&left.value, &right.value)
    } else {
        let coerced_params = coerce_values_to_single_type(params.to_vec())?;
        coerced_values_equal(&coerced_params)?
    };

    Ok(Rc::new(Value::new::<bool>(equal)))
}

fn not_equals(params: &[Rc<dyn RTObject>]) -> Result<Rc<dyn RTObject>, StoryError> {
    let equals_result = equal(params)?;
    let equals = Value::get_bool_value(equals_result.as_ref()).ok_or_else(|| {
        StoryError::InvalidStoryState("!= expected equality to return bool".to_owned())
    })?;

    Ok(Rc::new(Value::new::<bool>(!equals)))
}

fn coerce_values_to_single_type(
    params: Vec<Rc<dyn RTObject>>,
) -> Result<Vec<Rc<Value>>, StoryError> {
    let mut dest_type = 1; // Int
    let mut result: Vec<Rc<Value>> = Vec::new();

    for obj in params.iter() {
        // Find out what the output type is
        // "higher level" types infect both so that binary operations
        // use the same type on both sides. e.g. binary operation of
        // int and float causes the int to be casted to a float.
        if let Some(v) = params::runtime_value(obj.as_ref()) {
            if v.get_cast_ordinal() > dest_type {
                dest_type = v.get_cast_ordinal();
            }
        }
    }

    for obj in params.iter() {
        let value = params::value_object(obj.as_ref())?;
        match value.cast(dest_type)? {
            Some(casted_value) => result.push(Rc::new(casted_value)),
            None => {
                if let Ok(obj) = obj.clone().into_any().downcast::<Value>() {
                    result.push(obj);
                }
            }
        }
    }

    Ok(result)
}

fn and_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn greater_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn less_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn greater_than_or_equals_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn less_than_or_equals_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn subtract_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn add_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn divide_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn pow_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
    match params[0].value {
        ValueType::Int(op1) => match params[1].value {
            ValueType::Int(op2) => Ok(Rc::new(Value::new::<f32>((op1 as f32).powf(op2 as f32)))),
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

fn multiply_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn or_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn not_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
    match &params[0].value {
        ValueType::Int(op1) => Ok(Rc::new(Value::new::<bool>(*op1 == 0))),
        ValueType::Float(op1) => Ok(Rc::new(Value::new::<bool>(*op1 == 0.0))),
        _ => Err(StoryError::InvalidStoryState(
            "Operation not available for type.".to_owned(),
        )),
    }
}

fn min_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn max_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn mod_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn has(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn hasnt(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
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

fn negate_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
    match &params[0].value {
        ValueType::Int(op1) => Ok(Rc::new(Value::new::<i32>(-op1))),
        ValueType::Float(op1) => Ok(Rc::new(Value::new::<f32>(-op1))),
        _ => Err(StoryError::InvalidStoryState(
            "Operation not available for type.".to_owned(),
        )),
    }
}

fn floor_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
    match &params[0].value {
        ValueType::Int(op1) => Ok(Rc::new(Value::new::<i32>(*op1))),
        ValueType::Float(op1) => Ok(Rc::new(Value::new::<f32>(op1.floor()))),
        _ => Err(StoryError::InvalidStoryState(
            "Operation not available for type.".to_owned(),
        )),
    }
}

fn ceiling_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
    match &params[0].value {
        ValueType::Int(op1) => Ok(Rc::new(Value::new::<i32>(*op1))),
        ValueType::Float(op1) => Ok(Rc::new(Value::new::<f32>(op1.ceil()))),
        _ => Err(StoryError::InvalidStoryState(
            "Operation not available for type.".to_owned(),
        )),
    }
}

fn int_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
    match &params[0].value {
        ValueType::Int(op1) => Ok(Rc::new(Value::new::<i32>(*op1))),
        ValueType::Float(op1) => Ok(Rc::new(Value::new::<i32>(*op1 as i32))),
        _ => Err(StoryError::InvalidStoryState(
            "Operation not available for type.".to_owned(),
        )),
    }
}

fn float_op(params: &[Rc<Value>]) -> Result<Rc<dyn RTObject>, StoryError> {
    match &params[0].value {
        ValueType::Int(op1) => Ok(Rc::new(Value::new::<f32>(*op1 as f32))),
        ValueType::Float(op1) => Ok(Rc::new(Value::new::<f32>(*op1))),
        _ => Err(StoryError::InvalidStoryState(
            "Operation not available for type.".to_owned(),
        )),
    }
}

fn value_type_is_composite(value: &ValueType) -> bool {
    matches!(
        value,
        ValueType::Array(_) | ValueType::Object(_) | ValueType::Dict(_)
    )
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
        (ValueType::Dict(left), ValueType::Dict(right)) => {
            left.key_type() == right.key_type()
                && left.entries().len() == right.entries().len()
                && left.entries().iter().all(|(key, left_value)| {
                    right
                        .get(key)
                        .is_some_and(|right_value| value_types_equal(left_value, right_value))
                })
        }
        _ => false,
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

    fn float_value(value: f32) -> Rc<dyn RTObject> {
        Rc::new(Value::new::<f32>(value))
    }

    fn string_value(value: &str) -> Rc<dyn RTObject> {
        Rc::new(Value::new::<&str>(value))
    }

    fn dict_value(key_type: DictKeyType, entries: Vec<(DictKey, ValueType)>) -> Rc<dyn RTObject> {
        Rc::new(Value::new_value_type(dict_value_type(key_type, entries)))
    }

    fn dict_value_type(key_type: DictKeyType, entries: Vec<(DictKey, ValueType)>) -> ValueType {
        ValueType::Dict(
            DictValue::new(key_type, entries.into_iter().collect::<BTreeMap<_, _>>())
                .expect("test dict entries should match key type"),
        )
    }

    fn value_type(value: &dyn RTObject) -> &ValueType {
        &value
            .as_any()
            .downcast_ref::<Value>()
            .expect("expected runtime value")
            .value
    }

    #[test]
    fn scalar_operations_preserve_numeric_coercion() {
        let result = NativeFunctionCall::new(Op::Add)
            .call(vec![int_value(2), float_value(3.5)])
            .expect("mixed numeric addition should succeed");
        assert!(matches!(value_type(result.as_ref()), ValueType::Float(5.5)));
    }

    #[test]
    fn scalar_operations_preserve_string_addition_boundary() {
        let result = NativeFunctionCall::new(Op::Add)
            .call(vec![string_value("a"), string_value("b")])
            .expect("string addition should succeed");
        assert!(
            matches!(value_type(result.as_ref()), ValueType::String(value) if value.string == "ab")
        );

        let error =
            match NativeFunctionCall::new(Op::Add).call(vec![string_value("a"), int_value(1)]) {
                Ok(_) => panic!("expected mixed string addition to fail"),
                Err(StoryError::InvalidStoryState(message)) => message,
                Err(error) => panic!("expected invalid story state, got {error:?}"),
            };
        assert_eq!(error, "Operation not available for type.");
    }

    #[test]
    fn scalar_equality_preserves_composite_comparison() {
        let mut left_fields = BTreeMap::new();
        left_fields.insert("hp".to_string(), ValueType::Int(10));
        let mut right_fields = BTreeMap::new();
        right_fields.insert("hp".to_string(), ValueType::Int(10));

        let result = NativeFunctionCall::new(Op::Equal)
            .call(vec![
                Rc::new(Value::new_value_type(ValueType::Object(left_fields))),
                Rc::new(Value::new_value_type(ValueType::Object(right_fields))),
            ])
            .expect("composite equality should succeed");
        assert!(matches!(value_type(result.as_ref()), ValueType::Bool(true)));
    }

    #[test]
    fn scalar_equality_compares_dict_values_recursively() {
        let left = dict_value(
            DictKeyType::String,
            vec![
                (DictKey::String("hp".to_string()), ValueType::Int(10)),
                (
                    DictKey::String("nested".to_string()),
                    dict_value_type(DictKeyType::Int, vec![(DictKey::Int(1), ValueType::Int(1))]),
                ),
            ],
        );
        let right = dict_value(
            DictKeyType::String,
            vec![
                (DictKey::String("hp".to_string()), ValueType::Int(10)),
                (
                    DictKey::String("nested".to_string()),
                    dict_value_type(DictKeyType::Int, vec![(DictKey::Int(1), ValueType::Int(1))]),
                ),
            ],
        );
        let different_value = dict_value(
            DictKeyType::String,
            vec![
                (DictKey::String("hp".to_string()), ValueType::Int(11)),
                (
                    DictKey::String("nested".to_string()),
                    dict_value_type(DictKeyType::Int, vec![(DictKey::Int(1), ValueType::Int(1))]),
                ),
            ],
        );
        let different_key_type = dict_value(
            DictKeyType::Int,
            vec![(DictKey::Int(1), ValueType::Int(10))],
        );

        let equal = NativeFunctionCall::new(Op::Equal)
            .call(vec![left.clone(), right])
            .expect("dict equality should succeed");
        assert!(matches!(value_type(equal.as_ref()), ValueType::Bool(true)));

        let unequal_value = NativeFunctionCall::new(Op::Equal)
            .call(vec![left.clone(), different_value])
            .expect("dict equality should succeed");
        assert!(matches!(
            value_type(unequal_value.as_ref()),
            ValueType::Bool(false)
        ));

        let unequal_key_type = NativeFunctionCall::new(Op::Equal)
            .call(vec![left.clone(), different_key_type])
            .expect("dict equality should succeed");
        assert!(matches!(
            value_type(unequal_key_type.as_ref()),
            ValueType::Bool(false)
        ));

        let not_equal = NativeFunctionCall::new(Op::NotEquals)
            .call(vec![
                left,
                Rc::new(Value::new_value_type(ValueType::Array(vec![
                    ValueType::Int(10),
                ]))),
            ])
            .expect("cross-type dict inequality should succeed");
        assert!(matches!(
            value_type(not_equal.as_ref()),
            ValueType::Bool(true)
        ));
    }
}
