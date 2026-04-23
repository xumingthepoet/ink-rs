mod choice;
mod content_list;
mod divert;
mod story;
mod text;
mod weave;

pub use choice::Choice;
pub use content_list::ContentList;
pub use divert::{Divert, DivertTarget};
pub use story::Story;
pub use text::Text;
pub use weave::Weave;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Text(Text),
    Choice(Choice),
    Divert(Divert),
}

impl Object {
    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        match self {
            Object::Text(text) => text.write_parse_snapshot(out, indent),
            Object::Choice(choice) => choice.write_parse_snapshot(out, indent),
            Object::Divert(divert) => divert.write_parse_snapshot(out, indent),
        }
    }
}

pub(crate) fn push_indent(out: &mut String, indent: usize) {
    out.push_str(&" ".repeat(indent));
}

pub(crate) fn escape_snapshot_text(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
