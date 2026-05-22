use std::{cell::RefCell, rc::Rc};

use crate::{callstack::CallStack, choice::Choice, container::Container, object::RTObject};

#[derive(Clone)]
pub(crate) struct ExecutionState {
    pub callstack: Rc<RefCell<CallStack>>,
    pub output_stream: Vec<Rc<dyn RTObject>>,
    pub current_choices: Vec<Rc<Choice>>,
}

impl ExecutionState {
    pub fn new(main_content_container: Rc<Container>) -> ExecutionState {
        ExecutionState {
            callstack: Rc::new(RefCell::new(CallStack::new(main_content_container))),
            output_stream: Vec::new(),
            current_choices: Vec::new(),
        }
    }
}
