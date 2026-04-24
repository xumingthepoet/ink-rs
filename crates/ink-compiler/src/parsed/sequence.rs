use super::{push_indent, ContentList};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceType(u8);

impl SequenceType {
    pub const STOPPING: Self = Self(1);
    pub const CYCLE: Self = Self(1 << 1);
    pub const SHUFFLE: Self = Self(1 << 2);
    pub const ONCE: Self = Self(1 << 3);

    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
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
        out.push_str(&self.sequence_type.display_name());
        out.push(')');

        for element in &self.elements {
            out.push('\n');
            push_indent(out, indent + 2);
            if element.objects().is_empty() {
                out.push_str("ContentList");
            } else {
                out.push_str("Weave(baseIndent=0)");
                element.write_parse_snapshot(out, indent + 4);
            }
        }
    }
}

impl SequenceType {
    fn display_name(self) -> String {
        let mut parts = Vec::new();
        for (flag, name) in [
            (Self::STOPPING, "Stopping"),
            (Self::CYCLE, "Cycle"),
            (Self::SHUFFLE, "Shuffle"),
            (Self::ONCE, "Once"),
        ] {
            if self.contains(flag) {
                parts.push(name);
            }
        }
        parts.join(", ")
    }
}
