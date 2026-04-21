use std::fmt;

use super::{Identifier, NamedContent, Object, ObjectRef, WeavePoint};

#[derive(Debug, Clone)]
pub struct Gather {
    object: ObjectRef,
    identifier: Option<Identifier>,
    indentation_depth: usize,
}

impl Gather {
    pub fn new(identifier: Option<Identifier>, indentation_depth: usize) -> Self {
        let object = Object::new_ref();
        let kind_identifier = identifier.clone();
        object
            .borrow_mut()
            .set_gather_kind(kind_identifier, indentation_depth);

        Self {
            object,
            identifier,
            indentation_depth,
        }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn identifier(&self) -> Option<Identifier> {
        self.identifier.clone()
    }

    pub fn name(&self) -> Option<&str> {
        self.identifier
            .as_ref()
            .map(|identifier| identifier.name.as_str())
    }

    pub fn indentation_depth(&self) -> usize {
        self.indentation_depth
    }
}

impl NamedContent for Gather {
    fn name(&self) -> Option<&str> {
        self.name()
    }
}

impl WeavePoint for Gather {
    fn identifier(&self) -> Option<&Identifier> {
        self.identifier.as_ref()
    }

    fn indentation_depth(&self) -> usize {
        self.indentation_depth
    }

    fn content(&self) -> Vec<ObjectRef> {
        self.object.borrow().content().to_vec()
    }
}

impl fmt::Display for Gather {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name() {
            Some(name) => write!(f, "- {name}"),
            None => f.write_str("-"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Gather;
    use crate::parsed::{Identifier, Text, WeavePoint};

    #[test]
    fn gather_exposes_weave_point_fields() {
        let gather = Gather::new(Some(Identifier::new("join")), 1);
        gather
            .object()
            .borrow_mut()
            .content_mut()
            .push(Text::new("Hello").object());

        assert_eq!(gather.name(), Some("join"));
        assert_eq!(gather.indentation_depth(), 1);
        assert_eq!(WeavePoint::name(&gather), Some("join"));
        assert_eq!(gather.content().len(), 1);
        assert_eq!(gather.to_string(), "- join");
    }
}
