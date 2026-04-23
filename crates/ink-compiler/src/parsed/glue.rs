use super::push_indent;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Glue;

impl Glue {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Glue");
    }
}
