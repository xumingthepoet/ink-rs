use super::push_indent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    is_start: bool,
    in_choice: bool,
}

impl Tag {
    pub fn new(is_start: bool, in_choice: bool) -> Self {
        Self {
            is_start,
            in_choice,
        }
    }

    pub fn is_start(&self) -> bool {
        self.is_start
    }

    pub fn in_choice(&self) -> bool {
        self.in_choice
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Tag(start=");
        out.push_str(if self.is_start { "true" } else { "false" });
        out.push_str(", inChoice=");
        out.push_str(if self.in_choice { "true" } else { "false" });
        out.push(')');
    }
}
