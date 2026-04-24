use super::push_indent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    NumberInt(i32),
    NumberBool(bool),
    VariableReference(String),
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    MultipleCondition(Vec<Expression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    And,
}

impl Expression {
    pub fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        push_indent(out, indent);
        match self {
            Expression::NumberInt(value) => {
                out.push_str("Number(");
                out.push_str(&value.to_string());
                out.push(')');
            }
            Expression::NumberBool(value) => {
                out.push_str("Number(");
                out.push_str(if *value { "true" } else { "false" });
                out.push(')');
            }
            Expression::VariableReference(name) => {
                out.push_str("VariableReference(");
                out.push_str(name);
                out.push(')');
            }
            Expression::Binary {
                operator,
                left,
                right,
            } => {
                out.push_str("Binary(");
                out.push_str(operator.snapshot_name());
                out.push_str(", ");
                left.write_parse_snapshot(out, 0);
                out.push_str(", ");
                right.write_parse_snapshot(out, 0);
                out.push(')');
            }
            Expression::MultipleCondition(expressions) => {
                out.push_str("MultipleCondition(");
                for (index, expression) in expressions.iter().enumerate() {
                    if index > 0 {
                        out.push_str(", ");
                    }
                    expression.write_parse_snapshot(out, 0);
                }
                out.push(')');
            }
        }
    }

    pub fn multiple_condition(expressions: Vec<Expression>) -> Option<Self> {
        match expressions.len() {
            0 => None,
            1 => expressions.into_iter().next(),
            _ => Some(Self::MultipleCondition(expressions)),
        }
    }
}

impl BinaryOperator {
    pub fn runtime_name(self) -> &'static str {
        match self {
            BinaryOperator::And => "&&",
        }
    }

    fn snapshot_name(self) -> &'static str {
        match self {
            BinaryOperator::And => "and",
        }
    }
}
