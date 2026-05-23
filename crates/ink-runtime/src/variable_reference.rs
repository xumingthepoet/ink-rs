use std::fmt;

use crate::object::{Object, RTObject};

pub struct VariableReference {
    obj: Object,
    pub name: String,
}

impl VariableReference {
    pub fn new(name: &str) -> Self {
        VariableReference {
            obj: Object::new(),
            name: name.to_string(),
        }
    }
}

impl RTObject for VariableReference {
    fn get_object(&self) -> &Object {
        &self.obj
    }
}

impl fmt::Display for VariableReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "var({})", self.name)
    }
}
