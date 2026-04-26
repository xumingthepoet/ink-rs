use crate::source::SourceSpan;

use super::{push_indent, Expression, TypeName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantDeclaration {
    name: String,
    declared_type: TypeName,
    expression: Expression,
    span: SourceSpan,
}

impl ConstantDeclaration {
    pub fn new(
        name: impl Into<String>,
        declared_type: TypeName,
        expression: Expression,
        span: SourceSpan,
    ) -> Self {
        Self {
            name: name.into(),
            declared_type,
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

    pub fn declared_type(&self) -> &TypeName {
        &self.declared_type
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("ConstantDeclaration(name=\"");
        out.push_str(&self.name);
        out.push_str("\", type=");
        out.push_str(&self.declared_type.snapshot_name());
        out.push(')');
        out.push('\n');
        self.expression.write_parse_snapshot(out, indent + 2);
    }
}
