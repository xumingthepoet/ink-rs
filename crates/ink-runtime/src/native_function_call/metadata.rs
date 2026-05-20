use ink_story_json_format::NativeFunction;

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
    DictHas,
    DictSize,
    DictRemove,
    DictKeys,
}

impl Op {
    pub fn from_format(function: NativeFunction) -> Self {
        match function {
            NativeFunction::Add => Self::Add,
            NativeFunction::Subtract => Self::Subtract,
            NativeFunction::Divide => Self::Divide,
            NativeFunction::Multiply => Self::Multiply,
            NativeFunction::Mod => Self::Mod,
            NativeFunction::Negate => Self::Negate,
            NativeFunction::Equal => Self::Equal,
            NativeFunction::Greater => Self::Greater,
            NativeFunction::Less => Self::Less,
            NativeFunction::GreaterThanOrEquals => Self::GreaterThanOrEquals,
            NativeFunction::LessThanOrEquals => Self::LessThanOrEquals,
            NativeFunction::NotEquals => Self::NotEquals,
            NativeFunction::Not => Self::Not,
            NativeFunction::And => Self::And,
            NativeFunction::Or => Self::Or,
            NativeFunction::Min => Self::Min,
            NativeFunction::Max => Self::Max,
            NativeFunction::Pow => Self::Pow,
            NativeFunction::Floor => Self::Floor,
            NativeFunction::Ceiling => Self::Ceiling,
            NativeFunction::Int => Self::Int,
            NativeFunction::Float => Self::Float,
            NativeFunction::Has => Self::Has,
            NativeFunction::Hasnt => Self::Hasnt,
            NativeFunction::FieldRead => Self::FieldRead,
            NativeFunction::IndexRead => Self::IndexRead,
            NativeFunction::FieldWrite => Self::FieldWrite,
            NativeFunction::IndexWrite => Self::IndexWrite,
            NativeFunction::Len => Self::Len,
            NativeFunction::ArrayRemove => Self::ArrayRemove,
            NativeFunction::DictHas => Self::DictHas,
            NativeFunction::DictSize => Self::DictSize,
            NativeFunction::DictRemove => Self::DictRemove,
            NativeFunction::DictKeys => Self::DictKeys,
        }
    }

    pub fn format_function(self) -> NativeFunction {
        match self {
            Self::Add => NativeFunction::Add,
            Self::Subtract => NativeFunction::Subtract,
            Self::Divide => NativeFunction::Divide,
            Self::Multiply => NativeFunction::Multiply,
            Self::Mod => NativeFunction::Mod,
            Self::Negate => NativeFunction::Negate,
            Self::Equal => NativeFunction::Equal,
            Self::Greater => NativeFunction::Greater,
            Self::Less => NativeFunction::Less,
            Self::GreaterThanOrEquals => NativeFunction::GreaterThanOrEquals,
            Self::LessThanOrEquals => NativeFunction::LessThanOrEquals,
            Self::NotEquals => NativeFunction::NotEquals,
            Self::Not => NativeFunction::Not,
            Self::And => NativeFunction::And,
            Self::Or => NativeFunction::Or,
            Self::Min => NativeFunction::Min,
            Self::Max => NativeFunction::Max,
            Self::Pow => NativeFunction::Pow,
            Self::Floor => NativeFunction::Floor,
            Self::Ceiling => NativeFunction::Ceiling,
            Self::Int => NativeFunction::Int,
            Self::Float => NativeFunction::Float,
            Self::Has => NativeFunction::Has,
            Self::Hasnt => NativeFunction::Hasnt,
            Self::FieldRead => NativeFunction::FieldRead,
            Self::IndexRead => NativeFunction::IndexRead,
            Self::FieldWrite => NativeFunction::FieldWrite,
            Self::IndexWrite => NativeFunction::IndexWrite,
            Self::Len => NativeFunction::Len,
            Self::ArrayRemove => NativeFunction::ArrayRemove,
            Self::DictHas => NativeFunction::DictHas,
            Self::DictSize => NativeFunction::DictSize,
            Self::DictRemove => NativeFunction::DictRemove,
            Self::DictKeys => NativeFunction::DictKeys,
        }
    }

    pub fn token(self) -> &'static str {
        self.format_function().token()
    }

    pub fn arity(self) -> usize {
        self.format_function().arity()
    }
}

#[cfg(test)]
mod tests {
    use super::Op;
    use ink_story_json_format::NativeFunction;

    #[test]
    fn runtime_operation_metadata_matches_format_native_functions() {
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
            (NativeFunction::DictHas, Op::DictHas),
            (NativeFunction::DictSize, Op::DictSize),
            (NativeFunction::DictRemove, Op::DictRemove),
            (NativeFunction::DictKeys, Op::DictKeys),
        ];

        assert_eq!(cases.len(), NativeFunction::ALL.len());
        for (function, op) in cases {
            assert_eq!(Op::from_format(function), op);
            assert_eq!(op.format_function(), function);
            assert_eq!(op.token(), function.token());
            assert_eq!(op.arity(), function.arity());
        }
    }
}
