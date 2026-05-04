use crate::source::SourceSpan;

use super::push_indent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDeclaration {
    name: String,
    members: Vec<EnumMember>,
    span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumMember {
    name: String,
    span: SourceSpan,
}

impl EnumDeclaration {
    pub fn new(name: impl Into<String>, members: Vec<EnumMember>, span: SourceSpan) -> Self {
        Self {
            name: name.into(),
            members,
            span,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn members(&self) -> &[EnumMember] {
        &self.members
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("EnumDeclaration(name=\"");
        out.push_str(&self.name);
        out.push_str("\")");

        for member in &self.members {
            member.write_parse_snapshot(out, indent + 2);
        }
    }
}

impl EnumMember {
    pub fn new(name: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Member(name=\"");
        out.push_str(&self.name);
        out.push_str("\")");
    }
}
