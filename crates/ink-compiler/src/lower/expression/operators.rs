use ink_story_json_format::NativeFunction;

use crate::parsed::{BinaryOperator, UnaryOperator};

pub(super) fn native_function_for_binary_operator(operator: BinaryOperator) -> NativeFunction {
    match operator {
        BinaryOperator::And | BinaryOperator::AndSymbol => NativeFunction::And,
        BinaryOperator::Or | BinaryOperator::OrSymbol => NativeFunction::Or,
        BinaryOperator::Equals => NativeFunction::Equal,
        BinaryOperator::NotEquals => NativeFunction::NotEquals,
        BinaryOperator::GreaterThan => NativeFunction::Greater,
        BinaryOperator::LessThan => NativeFunction::Less,
        BinaryOperator::GreaterThanOrEquals => NativeFunction::GreaterThanOrEquals,
        BinaryOperator::LessThanOrEquals => NativeFunction::LessThanOrEquals,
        BinaryOperator::Has => NativeFunction::Has,
        BinaryOperator::Hasnt => NativeFunction::Hasnt,
        BinaryOperator::Add => NativeFunction::Add,
        BinaryOperator::Subtract => NativeFunction::Subtract,
        BinaryOperator::Multiply => NativeFunction::Multiply,
        BinaryOperator::Divide => NativeFunction::Divide,
        BinaryOperator::Modulo => NativeFunction::Mod,
    }
}

pub(super) fn native_function_for_unary_operator(operator: UnaryOperator) -> NativeFunction {
    match operator {
        UnaryOperator::Negate => NativeFunction::Negate,
        UnaryOperator::Not => NativeFunction::Not,
    }
}

pub(super) fn builtin_native_function(name: &str) -> Option<NativeFunction> {
    match name {
        "MIN" => Some(NativeFunction::Min),
        "MAX" => Some(NativeFunction::Max),
        "POW" => Some(NativeFunction::Pow),
        "FLOOR" => Some(NativeFunction::Floor),
        "CEILING" => Some(NativeFunction::Ceiling),
        "INT" => Some(NativeFunction::Int),
        "FLOAT" => Some(NativeFunction::Float),
        "to_str" => Some(NativeFunction::ToStr),
        "LEN" => Some(NativeFunction::Len),
        "ARRAY_PUSH" => Some(NativeFunction::ArrayPush),
        "ARRAY_INSERT" => Some(NativeFunction::ArrayInsert),
        "DICT_HAS" => Some(NativeFunction::DictHas),
        "DICT_SIZE" => Some(NativeFunction::DictSize),
        "DICT_REMOVE" => Some(NativeFunction::DictRemove),
        "DICT_KEYS" => Some(NativeFunction::DictKeys),
        _ => None,
    }
}
