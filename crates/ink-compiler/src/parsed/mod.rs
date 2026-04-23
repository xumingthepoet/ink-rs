mod choice;
mod content_list;
mod divert;
mod expression;
mod flow;
mod gather;
mod glue;
mod sequence;
mod story;
mod tag;
mod text;
mod weave;

pub use choice::Choice;
pub use content_list::ContentList;
pub use divert::{Divert, DivertTarget};
pub use expression::{BinaryOperator, Expression};
pub use flow::{Flow, FlowArgument, FlowLevel};
pub use gather::Gather;
pub use glue::Glue;
pub use sequence::{Sequence, SequenceType};
pub use story::Story;
pub use tag::Tag;
pub use text::Text;
pub use weave::Weave;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Text(Text),
    ContentList(ContentList),
    Glue(Glue),
    Choice(Choice),
    Divert(Divert),
    Gather(Gather),
    Tag(Tag),
    Sequence(Sequence),
    Weave(Weave),
}

impl Object {
    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        match self {
            Object::Text(text) => text.write_parse_snapshot(out, indent),
            Object::ContentList(content_list) => {
                out.push('\n');
                push_indent(out, indent);
                out.push_str("ContentList");
                content_list.write_parse_snapshot(out, indent + 2);
            }
            Object::Glue(glue) => glue.write_parse_snapshot(out, indent),
            Object::Choice(choice) => choice.write_parse_snapshot(out, indent),
            Object::Divert(divert) => divert.write_parse_snapshot(out, indent),
            Object::Gather(gather) => gather.write_parse_snapshot(out, indent),
            Object::Tag(tag) => tag.write_parse_snapshot(out, indent),
            Object::Sequence(sequence) => sequence.write_parse_snapshot(out, indent),
            Object::Weave(weave) => weave.write_parse_snapshot(out, indent),
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
