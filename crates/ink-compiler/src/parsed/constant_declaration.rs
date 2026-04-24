use crate::source::SourceSpan;

use super::{push_indent, Expression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantDeclaration {
    name: String,
    expression: Expression,
    span: SourceSpan,
}

impl ConstantDeclaration {
    pub fn new(name: impl Into<String>, expression: Expression, span: SourceSpan) -> Self {
        Self {
            name: name.into(),
            expression,
            span,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("ConstantDeclaration(name=\"");
        out.push_str(&self.name);
        out.push_str("\")");
        out.push('\n');
        self.expression.write_parse_snapshot(out, indent + 2);
    }
}
