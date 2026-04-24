use crate::source::SourceSpan;

use super::push_indent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gather {
    span: SourceSpan,
    identifier: Option<String>,
    indentation_depth: usize,
}

impl Gather {
    pub fn new(span: SourceSpan, indentation_depth: usize) -> Self {
        Self {
            span,
            identifier: None,
            indentation_depth,
        }
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub fn indentation_depth(&self) -> usize {
        self.indentation_depth
    }

    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    pub fn set_identifier(&mut self, identifier: Option<String>) {
        self.identifier = identifier;
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Gather(name=");
        match &self.identifier {
            Some(name) => {
                out.push('"');
                out.push_str(name);
                out.push('"');
            }
            None => out.push_str("null"),
        }
        out.push_str(", depth=");
        out.push_str(&self.indentation_depth.to_string());
        out.push(')');
    }
}
