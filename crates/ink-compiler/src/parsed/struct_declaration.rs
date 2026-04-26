use crate::source::SourceSpan;

use super::{push_indent, TypeName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDeclaration {
    name: String,
    fields: Vec<StructField>,
    span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    name: String,
    type_name: TypeName,
    span: SourceSpan,
}

impl StructDeclaration {
    pub fn new(name: impl Into<String>, fields: Vec<StructField>, span: SourceSpan) -> Self {
        Self {
            name: name.into(),
            fields,
            span,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("StructDeclaration(name=\"");
        out.push_str(&self.name);
        out.push_str("\")");

        for field in &self.fields {
            field.write_parse_snapshot(out, indent + 2);
        }
    }
}

impl StructField {
    pub fn new(name: impl Into<String>, type_name: TypeName, span: SourceSpan) -> Self {
        Self {
            name: name.into(),
            type_name,
            span,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_name(&self) -> &TypeName {
        &self.type_name
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Field(name=\"");
        out.push_str(&self.name);
        out.push_str("\", type=");
        out.push_str(&self.type_name.snapshot_name());
        out.push(')');
    }
}
