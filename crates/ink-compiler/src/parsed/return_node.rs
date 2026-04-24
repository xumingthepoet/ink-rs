use super::push_indent;
use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Return {
    returned_expression: Option<Expression>,
}

impl Return {
    pub fn new(returned_expression: Option<Expression>) -> Self {
        Self {
            returned_expression,
        }
    }

    pub fn returned_expression(&self) -> Option<&Expression> {
        self.returned_expression.as_ref()
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Return");
        if let Some(expr) = &self.returned_expression {
            out.push('\n');
            expr.write_parse_snapshot(out, indent + 2);
        }
    }
}
