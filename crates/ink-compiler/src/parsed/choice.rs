use std::fmt;

use super::{Identifier, NamedContent, Object, ObjectKind, ObjectRef, WeavePoint};

#[derive(Debug, Clone)]
pub struct Choice {
    object: ObjectRef,
    identifier: Option<Identifier>,
    condition: Option<ObjectRef>,
    indentation_depth: usize,
    once_only: bool,
    is_invisible_default: bool,
    has_weave_style_inline_brackets: bool,
}

impl Choice {
    pub fn new(identifier: Option<Identifier>, indentation_depth: usize) -> Self {
        let object = Object::new_ref();
        let kind_identifier = identifier.clone();
        object.borrow_mut().set_choice_kind(
            kind_identifier,
            indentation_depth,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        );

        Self {
            object,
            identifier,
            condition: None,
            indentation_depth,
            once_only: true,
            is_invisible_default: false,
            has_weave_style_inline_brackets: false,
        }
    }

    pub fn with_content(
        identifier: Option<Identifier>,
        indentation_depth: usize,
        content: Vec<ObjectRef>,
    ) -> Self {
        let choice = Self::new(identifier, indentation_depth);
        for child in content {
            Object::add_content(&choice.object, child);
        }
        choice
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn identifier(&self) -> Option<Identifier> {
        self.identifier.clone()
    }

    pub fn condition(&self) -> Option<ObjectRef> {
        self.condition.clone()
    }

    pub fn set_condition(&mut self, condition: Option<ObjectRef>) {
        self.condition = condition.clone();
        self.set_has_condition(condition.is_some());
        if let Some(condition) = condition {
            Object::add_content(&self.object, condition);
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.identifier
            .as_ref()
            .map(|identifier| identifier.name.as_str())
    }

    pub fn indentation_depth(&self) -> usize {
        self.indentation_depth
    }

    pub fn once_only(&self) -> bool {
        self.once_only
    }

    pub fn set_once_only(&mut self, once_only: bool) {
        self.once_only = once_only;
        if let ObjectKind::Choice {
            once_only: current, ..
        } = self.object.borrow_mut().kind_mut()
        {
            *current = once_only;
        }
    }

    pub fn is_invisible_default(&self) -> bool {
        self.is_invisible_default
    }

    pub fn set_is_invisible_default(&mut self, is_invisible_default: bool) {
        self.is_invisible_default = is_invisible_default;
        if let ObjectKind::Choice {
            is_invisible_default: current,
            ..
        } = self.object.borrow_mut().kind_mut()
        {
            *current = is_invisible_default;
        }
    }

    pub fn has_weave_style_inline_brackets(&self) -> bool {
        self.has_weave_style_inline_brackets
    }

    pub fn set_has_weave_style_inline_brackets(&mut self, has_weave_style_inline_brackets: bool) {
        self.has_weave_style_inline_brackets = has_weave_style_inline_brackets;
        if let ObjectKind::Choice {
            has_weave_style_inline_brackets: current,
            ..
        } = self.object.borrow_mut().kind_mut()
        {
            *current = has_weave_style_inline_brackets;
        }
    }

    pub fn has_start_content(&self) -> bool {
        matches!(
            self.object.borrow().kind(),
            ObjectKind::Choice {
                has_start_content: true,
                ..
            }
        )
    }

    pub fn set_has_start_content(&mut self, has_start_content: bool) {
        if let ObjectKind::Choice {
            has_start_content: current,
            ..
        } = self.object.borrow_mut().kind_mut()
        {
            *current = has_start_content;
        }
    }

    pub fn has_choice_only_content(&self) -> bool {
        matches!(
            self.object.borrow().kind(),
            ObjectKind::Choice {
                has_choice_only_content: true,
                ..
            }
        )
    }

    pub fn set_has_choice_only_content(&mut self, has_choice_only_content: bool) {
        if let ObjectKind::Choice {
            has_choice_only_content: current,
            ..
        } = self.object.borrow_mut().kind_mut()
        {
            *current = has_choice_only_content;
        }
    }

    pub fn has_condition(&self) -> bool {
        matches!(
            self.object.borrow().kind(),
            ObjectKind::Choice {
                has_condition: true,
                ..
            }
        )
    }

    pub fn set_has_condition(&mut self, has_condition: bool) {
        if let ObjectKind::Choice {
            has_condition: current,
            ..
        } = self.object.borrow_mut().kind_mut()
        {
            *current = has_condition;
        }
    }

    pub fn set_has_inline_inner_content(&mut self, has_inline_inner_content: bool) {
        if let ObjectKind::Choice {
            has_inline_inner_content: current,
            ..
        } = self.object.borrow_mut().kind_mut()
        {
            *current = has_inline_inner_content;
        }
    }
}

impl NamedContent for Choice {
    fn name(&self) -> Option<&str> {
        self.name()
    }
}

impl WeavePoint for Choice {
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

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name() {
            Some(name) => write!(f, "* {name}"),
            None => f.write_str("*"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Choice;
    use crate::parsed::{Identifier, Text, WeavePoint};

    #[test]
    fn choice_exposes_weave_point_fields() {
        let mut choice = Choice::new(Some(Identifier::new("branch")), 2);
        choice.set_once_only(false);
        choice.set_is_invisible_default(true);
        choice.set_has_weave_style_inline_brackets(true);
        choice
            .object()
            .borrow_mut()
            .content_mut()
            .push(Text::new("Hello").object());

        assert_eq!(choice.name(), Some("branch"));
        assert_eq!(choice.indentation_depth(), 2);
        assert!(!choice.once_only());
        assert!(choice.is_invisible_default());
        assert!(choice.has_weave_style_inline_brackets());
        assert!(!choice.has_start_content());
        assert!(!choice.has_choice_only_content());
        assert!(!choice.has_condition());
        assert_eq!(choice.content().len(), 1);
        assert_eq!(WeavePoint::name(&choice), Some("branch"));
        assert_eq!(choice.to_string(), "* branch");
    }
}
