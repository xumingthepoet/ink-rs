use crate::source::SourceSpan;

use super::{push_indent, DivertTarget};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TunnelOnwards {
    override_target: Option<DivertTarget>,
    span: SourceSpan,
}

impl TunnelOnwards {
    pub fn new(override_target: Option<DivertTarget>, span: SourceSpan) -> Self {
        Self {
            override_target,
            span,
        }
    }

    pub fn override_target(&self) -> Option<&DivertTarget> {
        self.override_target.as_ref()
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("TunnelOnwards");
    }
}
