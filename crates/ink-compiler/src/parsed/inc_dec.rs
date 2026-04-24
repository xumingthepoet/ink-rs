use crate::source::SourceSpan;

use super::{push_indent, Expression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncDec {
    name: String,
    expression: Expression,
    is_increment: bool,
    span: SourceSpan,
}

impl IncDec {
    pub fn new(
        name: impl Into<String>,
        expression: Expression,
        is_increment: bool,
        span: SourceSpan,
    ) -> Self {
        Self {
            name: name.into(),
            expression,
            is_increment,
            span,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn is_increment(&self) -> bool {
        self.is_increment
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("IncDec(");
        out.push_str(&self.name);
        out.push_str(", inc=");
        out.push_str(if self.is_increment { "true" } else { "false" });
        out.push(')');
    }
}
