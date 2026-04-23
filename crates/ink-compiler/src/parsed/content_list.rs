use crate::source::SourceSpan;

use super::{Object, Text};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentList {
    objects: Vec<Object>,
}

impl ContentList {
    pub fn new(objects: Vec<Object>) -> Self {
        Self { objects }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn from_text(text: impl Into<String>, span: SourceSpan) -> Self {
        Self::new(vec![Object::Text(Text::new(text, span))])
    }

    pub fn objects(&self) -> &[Object] {
        &self.objects
    }

    pub fn into_objects(self) -> Vec<Object> {
        self.objects
    }

    pub fn push(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, item_indent: usize) {
        for object in &self.objects {
            object.write_parse_snapshot(out, item_indent);
        }
    }
}
