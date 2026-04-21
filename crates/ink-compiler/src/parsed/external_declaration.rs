use std::fmt;

use super::{Identifier, Object, ObjectKind, ObjectRef};

#[derive(Debug, Clone)]
pub struct ExternalDeclaration {
    object: ObjectRef,
}

impl ExternalDeclaration {
    pub fn new(identifier: Identifier, argument_names: Vec<String>) -> Self {
        let object = Object::new_ref();
        object
            .borrow_mut()
            .set_external_declaration_kind(identifier, argument_names);

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn identifier(&self) -> Option<Identifier> {
        match self.object.borrow().kind() {
            ObjectKind::ExternalDeclaration { identifier, .. } => Some(identifier.clone()),
            _ => None,
        }
    }

    pub fn argument_names(&self) -> Vec<String> {
        match self.object.borrow().kind() {
            ObjectKind::ExternalDeclaration { argument_names, .. } => argument_names.clone(),
            _ => Vec::new(),
        }
    }
}

impl fmt::Display for ExternalDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let identifier = self
            .identifier()
            .map(|identifier| identifier.name)
            .unwrap_or_else(|| "<invalid>".to_string());
        write!(f, "EXTERNAL {identifier}")
    }
}

#[cfg(test)]
mod tests {
    use super::ExternalDeclaration;
    use crate::parsed::Identifier;

    #[test]
    fn external_declaration_wraps_identifier_and_arguments() {
        let declaration = ExternalDeclaration::new(
            Identifier::new("print"),
            vec!["message".to_string(), "count".to_string()],
        );

        assert_eq!(
            declaration.identifier().map(|id| id.name),
            Some("print".to_string())
        );
        assert_eq!(declaration.argument_names().len(), 2);
    }
}
