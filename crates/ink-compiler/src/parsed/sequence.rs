use super::{push_indent, ContentList};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceType {
    Stopping,
    Cycle,
    Once,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    sequence_type: SequenceType,
    elements: Vec<ContentList>,
}

impl Sequence {
    pub fn new(sequence_type: SequenceType, elements: Vec<ContentList>) -> Self {
        Self {
            sequence_type,
            elements,
        }
    }

    pub fn sequence_type(&self) -> SequenceType {
        self.sequence_type
    }

    pub fn elements(&self) -> &[ContentList] {
        &self.elements
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Sequence(type=");
        out.push_str(match self.sequence_type {
            SequenceType::Stopping => "Stopping",
            SequenceType::Cycle => "Cycle",
            SequenceType::Once => "Once",
        });
        out.push(')');

        for element in &self.elements {
            out.push('\n');
            push_indent(out, indent + 2);
            out.push_str("Weave(baseIndent=0)");
            element.write_parse_snapshot(out, indent + 4);
        }
    }
}
