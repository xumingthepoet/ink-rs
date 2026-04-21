use std::fmt;

use super::{Identifier, Object, ObjectKind, ObjectRef};

#[derive(Debug, Clone)]
pub struct ConstantDeclaration {
    object: ObjectRef,
}

impl ConstantDeclaration {
    pub fn new(identifier: Identifier, expression: Option<ObjectRef>) -> Self {
        let object = Object::new_ref();
        object
            .borrow_mut()
            .set_constant_declaration_kind(identifier);

        if let Some(expression) = expression {
            Object::add_content(&object, expression);
        }

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn identifier(&self) -> Option<Identifier> {
        match self.object.borrow().kind() {
            ObjectKind::ConstantDeclaration { identifier } => Some(identifier.clone()),
            _ => None,
        }
    }

    pub fn expression(&self) -> Option<ObjectRef> {
        self.object.borrow().content().first().cloned()
    }
}

impl fmt::Display for ConstantDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let identifier = self
            .identifier()
            .map(|identifier| identifier.name)
            .unwrap_or_else(|| "<invalid>".to_string());
        write!(f, "CONST {identifier}")
    }
}

#[cfg(test)]
mod tests {
    use super::ConstantDeclaration;
    use crate::parsed::{Identifier, Number};

    #[test]
    fn constant_declaration_wraps_identifier_and_expression() {
        let declaration =
            ConstantDeclaration::new(Identifier::new("answer"), Some(Number::int(42).object()));

        assert_eq!(
            declaration.identifier().map(|id| id.name),
            Some("answer".to_string())
        );
        assert!(declaration.expression().is_some());
    }
}
