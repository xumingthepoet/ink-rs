use crate::source::SourceSpan;

use super::push_indent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gather {
    span: SourceSpan,
    indentation_depth: usize,
}

impl Gather {
    pub fn new(span: SourceSpan, indentation_depth: usize) -> Self {
        Self {
            span,
            indentation_depth,
        }
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub fn indentation_depth(&self) -> usize {
        self.indentation_depth
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Gather(name=null, depth=");
        out.push_str(&self.indentation_depth.to_string());
        out.push(')');
    }
}
