use std::fmt;

use super::{HasContent, Object, ObjectKind, ObjectRef};

#[derive(Debug, Clone)]
pub struct ContentList {
    object: ObjectRef,
}

impl ContentList {
    pub fn new() -> Self {
        let object = Object::new_ref();
        object.borrow_mut().set_kind(ObjectKind::ContentList {
            dont_flatten: false,
        });

        Self { object }
    }

    pub fn with_objects(objects: Vec<ObjectRef>) -> Self {
        let list = Self::new();
        list.add_content_all(objects);
        list
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn dont_flatten(&self) -> bool {
        matches!(
            self.object.borrow().kind(),
            ObjectKind::ContentList { dont_flatten: true }
        )
    }

    pub fn set_dont_flatten(&self, dont_flatten: bool) {
        if let ObjectKind::ContentList {
            dont_flatten: current,
        } = self.object.borrow_mut().kind_mut()
        {
            *current = dont_flatten;
        }
    }

    pub fn add_content(&self, sub_content: ObjectRef) -> ObjectRef {
        Object::add_content(&self.object, sub_content)
    }

    pub fn add_content_all(&self, objects: Vec<ObjectRef>) {
        for object in objects {
            self.add_content(object);
        }
    }

    pub fn insert_content(&self, index: usize, sub_content: ObjectRef) -> ObjectRef {
        Object::insert_content(&self.object, index, sub_content)
    }

    pub fn content(&self) -> Vec<ObjectRef> {
        self.object.borrow().content().to_vec()
    }

    pub fn trim_trailing_whitespace(&self) {
        let mut object = self.object.borrow_mut();
        let content = object.content_mut();

        for i in (0..content.len()).rev() {
            let mut remove_current = false;
            let mut should_stop = false;

            {
                let child = content[i].clone();
                let mut child_borrow = child.borrow_mut();

                match child_borrow.kind_mut() {
                    ObjectKind::Text { text } => {
                        let trimmed = text.trim_end_matches([' ', '\t']).to_string();
                        if trimmed.is_empty() {
                            remove_current = true;
                        } else {
                            *text = trimmed;
                            should_stop = true;
                        }
                    }
                    _ => break,
                }
            }

            if remove_current {
                content.remove(i);
            }

            if should_stop {
                break;
            }
        }
    }
}

impl Default for ContentList {
    fn default() -> Self {
        Self::new()
    }
}

impl HasContent for ContentList {
    fn content(&self) -> Vec<ObjectRef> {
        ContentList::content(self)
    }
}

impl fmt::Display for ContentList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content = self.content();
        let rendered = content
            .iter()
            .map(|object| {
                let borrowed = object.borrow();
                match borrowed.kind() {
                    ObjectKind::Text { text } => text.clone(),
                    ObjectKind::AuthorWarning { warning_message } => {
                        format!("AuthorWarning({warning_message})")
                    }
                    ObjectKind::Tag {
                        is_start,
                        in_choice: _,
                    } => {
                        if *is_start {
                            "#StartTag".to_string()
                        } else {
                            "#EndTag".to_string()
                        }
                    }
                    _ => "<Object>".to_string(),
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "ContentList({rendered})")
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    object: ObjectRef,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        let object = Object::new_ref();
        object
            .borrow_mut()
            .set_kind(ObjectKind::Text { text: text.into() });

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn text(&self) -> String {
        match self.object.borrow().kind() {
            ObjectKind::Text { text } => text.clone(),
            _ => String::new(),
        }
    }

    pub fn set_text(&self, text: impl Into<String>) {
        if let ObjectKind::Text { text: current } = self.object.borrow_mut().kind_mut() {
            *current = text.into();
        }
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text())
    }
}

#[derive(Debug, Clone)]
pub struct AuthorWarning {
    object: ObjectRef,
}

impl AuthorWarning {
    pub fn new(message: impl Into<String>) -> Self {
        let object = Object::new_ref();
        object.borrow_mut().set_kind(ObjectKind::AuthorWarning {
            warning_message: message.into(),
        });

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn warning_message(&self) -> String {
        match self.object.borrow().kind() {
            ObjectKind::AuthorWarning { warning_message } => warning_message.clone(),
            _ => String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tag {
    object: ObjectRef,
}

impl Tag {
    pub fn new(is_start: bool, in_choice: bool) -> Self {
        let object = Object::new_ref();
        object.borrow_mut().set_kind(ObjectKind::Tag {
            is_start,
            in_choice,
        });

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn is_start(&self) -> bool {
        match self.object.borrow().kind() {
            ObjectKind::Tag { is_start, .. } => *is_start,
            _ => false,
        }
    }

    pub fn in_choice(&self) -> bool {
        match self.object.borrow().kind() {
            ObjectKind::Tag { in_choice, .. } => *in_choice,
            _ => false,
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_start() {
            f.write_str("#StartTag")
        } else {
            f.write_str("#EndTag")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrap<T> {
    wrapped: T,
}

impl<T> Wrap<T> {
    pub fn new(wrapped: T) -> Self {
        Self { wrapped }
    }

    pub fn wrapped(&self) -> &T {
        &self.wrapped
    }

    pub fn wrapped_mut(&mut self) -> &mut T {
        &mut self.wrapped
    }

    pub fn into_inner(self) -> T {
        self.wrapped
    }
}

impl<T: fmt::Display> fmt::Display for Wrap<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.wrapped.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthorWarning, ContentList, Object, ObjectKind, Tag, Text, Wrap};

    #[test]
    fn content_list_trims_trailing_text_whitespace() {
        let list = ContentList::with_objects(vec![
            Text::new("Hello ").object(),
            Text::new("\t ").object(),
        ]);

        list.trim_trailing_whitespace();

        let content = list.content();
        assert_eq!(content.len(), 1);

        let borrowed = content[0].borrow();
        assert!(matches!(
            borrowed.kind(),
            ObjectKind::Text { text } if text == "Hello"
        ));
    }

    #[test]
    fn content_list_keeps_dont_flatten_flag() {
        let list = ContentList::new();

        assert!(!list.dont_flatten());
        list.set_dont_flatten(true);
        assert!(list.dont_flatten());
        assert!(matches!(
            list.object().borrow().kind(),
            ObjectKind::ContentList { dont_flatten: true }
        ));
    }

    #[test]
    fn text_and_author_warning_keep_payloads() {
        let text = Text::new("Alpha");
        let warning = AuthorWarning::new("Be careful");

        assert_eq!(text.to_string(), "Alpha");
        assert_eq!(text.text(), "Alpha");
        assert_eq!(warning.warning_message(), "Be careful");
        assert!(matches!(
            warning.object().borrow().kind(),
            ObjectKind::AuthorWarning { warning_message } if warning_message == "Be careful"
        ));
    }

    #[test]
    fn tag_formats_and_exposes_flags() {
        let start = Tag::new(true, false);
        let end = Tag::new(false, true);

        assert!(start.is_start());
        assert!(!start.in_choice());
        assert_eq!(start.to_string(), "#StartTag");
        assert!(!end.is_start());
        assert!(end.in_choice());
        assert_eq!(end.to_string(), "#EndTag");
    }

    #[test]
    fn wrap_proxies_inner_value() {
        let wrap = Wrap::new(String::from("payload"));

        assert_eq!(wrap.wrapped(), "payload");
        assert_eq!(wrap.to_string(), "payload");
    }

    #[test]
    fn content_list_add_and_insert_keep_tree_links() {
        let list = ContentList::new();
        let first = Object::new_ref();
        let second = Object::new_ref();

        list.add_content(first.clone());
        list.insert_content(0, second.clone());

        let content = list.content();
        assert_eq!(content.len(), 2);
        let list_object = list.object();
        assert!(std::rc::Rc::ptr_eq(&content[0], &second));
        assert!(std::rc::Rc::ptr_eq(&content[1], &first));
        assert!(content[0].borrow().parent().is_some());
        assert!(content[1].borrow().parent().is_some());
        assert!(std::rc::Rc::ptr_eq(
            &content[0].borrow().parent().unwrap(),
            &list_object
        ));
        assert!(std::rc::Rc::ptr_eq(
            &content[1].borrow().parent().unwrap(),
            &list_object
        ));
    }
}
