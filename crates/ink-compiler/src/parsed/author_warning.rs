use crate::source::SourceSpan;

use super::push_indent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorWarning {
    message: String,
    span: SourceSpan,
}

impl AuthorWarning {
    pub fn new(message: String, span: SourceSpan) -> Self {
        Self { message, span }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("AuthorWarning(\"");
        out.push_str(&self.message.replace('"', "\\\""));
        out.push_str("\")");
    }
}
