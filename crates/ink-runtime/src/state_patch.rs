use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::value::Value;

#[derive(Clone)]
pub struct StatePatch {
    pub globals: HashMap<String, Rc<Value>>,
    pub changed_variables: HashSet<String>,
}

impl StatePatch {
    pub fn new() -> StatePatch {
        StatePatch {
            globals: HashMap::new(),
            changed_variables: HashSet::new(),
        }
    }

    pub fn get_global(&self, name: &str) -> Option<Rc<Value>> {
        self.globals.get(name).cloned()
    }

    pub fn set_global(&mut self, name: &str, value: Rc<Value>) {
        self.globals.insert(name.to_string(), value);
    }

    pub(crate) fn add_changed_variable(&mut self, name: &str) {
        self.changed_variables.insert(name.to_string());
    }
}
