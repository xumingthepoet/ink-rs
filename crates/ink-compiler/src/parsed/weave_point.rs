use super::{Identifier, NamedContent, ObjectRef};

pub trait WeavePoint: NamedContent {
    fn identifier(&self) -> Option<&Identifier>;

    fn indentation_depth(&self) -> usize;

    fn content(&self) -> Vec<ObjectRef>;

    fn name(&self) -> Option<&str> {
        self.identifier().map(|identifier| identifier.name.as_str())
    }
}
