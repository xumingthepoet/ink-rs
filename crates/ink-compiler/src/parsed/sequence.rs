use std::fmt;

use super::{ContentList, Object, ObjectRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceType {
    Stopping,
    Cycle,
    Shuffle,
    Once,
}

impl fmt::Display for SequenceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            SequenceType::Stopping => "Stopping",
            SequenceType::Cycle => "Cycle",
            SequenceType::Shuffle => "Shuffle",
            SequenceType::Once => "Once",
        };

        f.write_str(label)
    }
}

#[derive(Debug, Clone)]
pub struct Sequence {
    object: ObjectRef,
    sequence_type: SequenceType,
}

impl Sequence {
    pub fn new(element_content_lists: Vec<ContentList>, sequence_type: SequenceType) -> Self {
        let object = Object::new_ref();
        object.borrow_mut().set_sequence_kind(sequence_type);

        for element_content_list in element_content_lists {
            Object::add_content(&object, element_content_list.object());
        }

        Self {
            object,
            sequence_type,
        }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn sequence_type(&self) -> SequenceType {
        self.sequence_type
    }

    pub fn elements(&self) -> Vec<ObjectRef> {
        self.object.borrow().content().to_vec()
    }
}

impl fmt::Display for Sequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sequence(type={})", self.sequence_type)
    }
}

#[cfg(test)]
mod tests {
    use super::{Sequence, SequenceType};
    use crate::parsed::{ContentList, ObjectKind, Text};

    #[test]
    fn sequence_wraps_element_lists_and_type() {
        let first = ContentList::new();
        first.add_content(Text::new("One").object());

        let second = ContentList::new();
        second.add_content(Text::new("Two").object());

        let sequence = Sequence::new(vec![first, second], SequenceType::Once);

        assert_eq!(sequence.sequence_type(), SequenceType::Once);
        assert_eq!(sequence.elements().len(), 2);
        assert!(matches!(
            sequence.object().borrow().kind(),
            ObjectKind::Sequence {
                sequence_type: SequenceType::Once,
            }
        ));
        assert_eq!(sequence.to_string(), "Sequence(type=Once)");
    }
}
