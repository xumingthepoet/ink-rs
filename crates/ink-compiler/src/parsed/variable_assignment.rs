use crate::source::SourceSpan;

use super::{push_indent, Expression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableAssignment {
    name: String,
    expression: Expression,
    is_global: bool,
    is_temporary: bool,
    span: SourceSpan,
}

impl VariableAssignment {
    pub fn new(
        name: impl Into<String>,
        expression: Expression,
        is_global: bool,
        is_temporary: bool,
        span: SourceSpan,
    ) -> Self {
        Self {
            name: name.into(),
            expression,
            is_global,
            is_temporary,
            span,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }

    pub fn is_temporary(&self) -> bool {
        self.is_temporary
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("VariableAssignment(name=\"");
        out.push_str(&self.name);
        out.push_str("\", global=");
        out.push_str(if self.is_global { "true" } else { "false" });
        out.push_str(", temp=");
        out.push_str(if self.is_temporary { "true" } else { "false" });
        out.push(')');
        out.push('\n');
        self.expression.write_parse_snapshot(out, indent + 2);
    }
}
