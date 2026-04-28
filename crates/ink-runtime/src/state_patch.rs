use std::{collections::HashMap, rc::Rc};

use crate::value::Value;

#[derive(Clone)]
pub struct StatePatch {
    pub globals: HashMap<String, Rc<Value>>,
}

impl StatePatch {
    pub fn new() -> StatePatch {
        StatePatch {
            globals: HashMap::new(),
        }
    }

    pub fn get_global(&self, name: &str) -> Option<Rc<Value>> {
        self.globals.get(name).cloned()
    }

    pub fn set_global(&mut self, name: &str, value: Rc<Value>) {
        self.globals.insert(name.to_string(), value);
    }
}
