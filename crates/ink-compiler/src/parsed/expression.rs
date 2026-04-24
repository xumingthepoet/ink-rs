use super::{push_indent, ContentList, Object};

#[derive(Debug, Clone, Copy)]
pub struct FloatLiteral(f64);

impl FloatLiteral {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

impl PartialEq for FloatLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for FloatLiteral {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    String(String),
    StringContent(ContentList),
    NumberInt(i32),
    NumberFloat(FloatLiteral),
    NumberBool(bool),
    DivertTarget(String),
    VariableReference(String),
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        expression: Box<Expression>,
    },
    MultipleCondition(Vec<Expression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    And,
    AndSymbol,
    Or,
    OrSymbol,
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEquals,
    LessThanOrEquals,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
}

impl Expression {
    pub fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        push_indent(out, indent);
        match self {
            Expression::String(value) => {
                out.push_str("String(\"");
                out.push_str(&super::escape_snapshot_text(value));
                out.push_str("\")");
            }
            Expression::StringContent(content) => {
                out.push_str("String(\"");
                out.push_str(&super::escape_snapshot_text(&content_list_display(content)));
                out.push_str("\")");
            }
            Expression::NumberInt(value) => {
                out.push_str("Number(");
                out.push_str(&value.to_string());
                out.push(')');
            }
            Expression::NumberFloat(value) => {
                out.push_str("Number(");
                out.push_str(&format_float(value.value()));
                out.push(')');
            }
            Expression::NumberBool(value) => {
                out.push_str("Number(");
                out.push_str(if *value { "true" } else { "false" });
                out.push(')');
            }
            Expression::DivertTarget(target) => {
                out.push_str("DivertTarget(-> ");
                out.push_str(target);
                out.push(')');
            }
            Expression::VariableReference(name) => {
                out.push_str("VariableReference(");
                out.push_str(name);
                out.push(')');
            }
            Expression::FunctionCall { name, args } => {
                out.push_str("FunctionCall(");
                out.push_str(name);
                out.push_str(", args=");
                out.push_str(&args.len().to_string());
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
            Expression::Unary {
                operator,
                expression,
            } => {
                out.push_str("Unary(");
                out.push_str(operator.snapshot_name());
                out.push_str(", ");
                expression.write_parse_snapshot(out, 0);
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

fn content_list_display(content: &ContentList) -> String {
    let mut output = String::from("ContentList(");
    for (index, object) in content.objects().iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&object_display(object));
    }
    output.push(')');
    output
}

fn object_display(object: &Object) -> String {
    match object {
        Object::Text(text) => text.text().to_string(),
        Object::Expression(expression) => expression_display(expression),
        Object::ContentList(content) => content_list_display(content),
        Object::Divert(divert) => divert.target().to_snapshot_string(),
        Object::Glue(_) => "<>".to_string(),
        Object::Tag(_) => "#".to_string(),
        Object::Return(_) => "Return".to_string(),
        Object::Conditional(_) => "Conditional".to_string(),
        Object::LogicLine(expression) => expression_display(expression),
        Object::IncDec(_) => "IncDec".to_string(),
        Object::Choice(_) => "Choice".to_string(),
        Object::Gather(_) => "Gather".to_string(),
        Object::Sequence(_) => "Sequence".to_string(),
        Object::TunnelOnwards(_) => "TunnelOnwards".to_string(),
        Object::ConstantDeclaration(declaration) => declaration.name().to_string(),
        Object::VariableAssignment(assignment) => assignment.name().to_string(),
        Object::ExternalDeclaration(external) => external.name().to_string(),
        Object::Weave(_) => "Weave".to_string(),
    }
}

fn expression_display(expression: &Expression) -> String {
    match expression {
        Expression::String(value) => value.clone(),
        Expression::StringContent(content) => content_list_display(content),
        Expression::NumberInt(value) => value.to_string(),
        Expression::NumberFloat(value) => format_float(value.value()),
        Expression::NumberBool(value) => value.to_string(),
        Expression::DivertTarget(target) => format!("-> {target}"),
        Expression::VariableReference(name) => name.clone(),
        Expression::FunctionCall { name, args } => {
            let args = args
                .iter()
                .map(expression_display)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}({args})")
        }
        Expression::Binary {
            operator,
            left,
            right,
        } => format!(
            "({} {} {})",
            expression_display(left),
            operator.snapshot_name(),
            expression_display(right)
        ),
        Expression::Unary {
            operator,
            expression,
        } => format!(
            "{}{}",
            operator.snapshot_name(),
            expression_display(expression)
        ),
        Expression::MultipleCondition(expressions) => expressions
            .iter()
            .map(expression_display)
            .collect::<Vec<_>>()
            .join(", "),
    }
}

impl BinaryOperator {
    pub fn runtime_name(self) -> &'static str {
        match self {
            BinaryOperator::And => "&&",
            BinaryOperator::AndSymbol => "&&",
            BinaryOperator::Or => "||",
            BinaryOperator::OrSymbol => "||",
            BinaryOperator::Equals => "==",
            BinaryOperator::NotEquals => "!=",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::LessThan => "<",
            BinaryOperator::GreaterThanOrEquals => ">=",
            BinaryOperator::LessThanOrEquals => "<=",
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Modulo => "%",
        }
    }

    fn snapshot_name(self) -> &'static str {
        match self {
            BinaryOperator::And => "and",
            BinaryOperator::AndSymbol => "&&",
            BinaryOperator::Or => "or",
            BinaryOperator::OrSymbol => "||",
            BinaryOperator::Equals => "==",
            BinaryOperator::NotEquals => "!=",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::LessThan => "<",
            BinaryOperator::GreaterThanOrEquals => ">=",
            BinaryOperator::LessThanOrEquals => "<=",
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Modulo => "%",
        }
    }
}

impl UnaryOperator {
    pub fn runtime_name(self) -> &'static str {
        match self {
            UnaryOperator::Negate => "_",
            UnaryOperator::Not => "!",
        }
    }

    fn snapshot_name(self) -> &'static str {
        match self {
            UnaryOperator::Negate => "-",
            UnaryOperator::Not => "not",
        }
    }
}

fn format_float(value: f64) -> String {
    let formatted = value.to_string();
    if formatted == "-0" {
        "0".to_string()
    } else {
        formatted
    }
}
