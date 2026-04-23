use crate::source::SourceSpan;

use super::{escape_snapshot_text, push_indent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    text: String,
    span: SourceSpan,
}

impl Text {
    pub fn new(text: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            text: text.into(),
            span,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Text(\"");
        out.push_str(&escape_snapshot_text(&self.text));
        out.push_str("\")");
    }
}
