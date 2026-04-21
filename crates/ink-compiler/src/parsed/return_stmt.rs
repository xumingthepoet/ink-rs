use std::fmt;

use super::{Object, ObjectRef};

#[derive(Debug, Clone)]
pub struct Return {
    object: ObjectRef,
}

impl Return {
    pub fn new(returned_expression: Option<ObjectRef>) -> Self {
        let object = Object::new_ref();
        object.borrow_mut().set_return_kind();

        if let Some(returned_expression) = returned_expression {
            Object::add_content(&object, returned_expression);
        }

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn returned_expression(&self) -> Option<ObjectRef> {
        self.object.borrow().content().first().cloned()
    }
}

impl fmt::Display for Return {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.returned_expression().is_some() {
            f.write_str("return expr")
        } else {
            f.write_str("return")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Return;
    use crate::parsed::Number;

    #[test]
    fn return_wraps_optional_expression() {
        let empty_return = Return::new(None);
        assert!(empty_return.returned_expression().is_none());

        let value_return = Return::new(Some(Number::int(7).object()));
        assert!(value_return.returned_expression().is_some());
    }
}
