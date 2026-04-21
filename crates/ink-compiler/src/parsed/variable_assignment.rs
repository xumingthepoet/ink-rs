use std::fmt;

use super::{Identifier, Object, ObjectKind, ObjectRef};

#[derive(Debug, Clone)]
pub struct VariableAssignment {
    object: ObjectRef,
}

impl VariableAssignment {
    pub fn new(
        identifier: Identifier,
        expression: Option<ObjectRef>,
        is_global_declaration: bool,
        is_new_temporary_declaration: bool,
    ) -> Self {
        let object = Object::new_ref();
        object.borrow_mut().set_variable_assignment_kind(
            identifier,
            is_global_declaration,
            is_new_temporary_declaration,
        );

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
            ObjectKind::VariableAssignment { identifier, .. } => Some(identifier.clone()),
            _ => None,
        }
    }

    pub fn expression(&self) -> Option<ObjectRef> {
        self.object.borrow().content().first().cloned()
    }

    pub fn is_global_declaration(&self) -> bool {
        matches!(
            self.object.borrow().kind(),
            ObjectKind::VariableAssignment {
                is_global_declaration: true,
                ..
            }
        )
    }

    pub fn is_new_temporary_declaration(&self) -> bool {
        matches!(
            self.object.borrow().kind(),
            ObjectKind::VariableAssignment {
                is_new_temporary_declaration: true,
                ..
            }
        )
    }
}

impl fmt::Display for VariableAssignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let identifier = self
            .identifier()
            .map(|identifier| identifier.name)
            .unwrap_or_else(|| "<invalid>".to_string());

        if self.is_global_declaration() {
            write!(f, "VAR {identifier}")
        } else if self.is_new_temporary_declaration() {
            write!(f, "temp {identifier}")
        } else {
            write!(f, "{identifier}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VariableAssignment;
    use crate::parsed::{Identifier, Number};

    #[test]
    fn variable_assignment_wraps_declaration_flags_and_expression() {
        let assignment = VariableAssignment::new(
            Identifier::new("x"),
            Some(Number::int(5).object()),
            true,
            false,
        );

        assert_eq!(
            assignment.identifier().map(|id| id.name),
            Some("x".to_string())
        );
        assert!(assignment.is_global_declaration());
        assert!(!assignment.is_new_temporary_declaration());
        assert!(assignment.expression().is_some());
    }
}
