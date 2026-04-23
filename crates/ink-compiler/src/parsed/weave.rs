use super::{push_indent, Object};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Weave {
    content: Vec<Object>,
    base_indent: usize,
}

impl Weave {
    pub fn new(content: Vec<Object>, base_indent: usize) -> Self {
        Self {
            content,
            base_indent,
        }
    }

    pub fn content(&self) -> &[Object] {
        &self.content
    }

    pub fn base_indent(&self) -> usize {
        self.base_indent
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Weave(baseIndent=");
        out.push_str(&self.base_indent.to_string());
        out.push(')');

        for object in &self.content {
            object.write_parse_snapshot(out, indent + 2);
        }
    }
}
