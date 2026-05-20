use std::rc::Rc;

use crate::{object::RTObject, story_error::StoryError};

use super::StoryState;

impl StoryState {
    pub fn get_in_expression_evaluation(&self) -> bool {
        self.get_callstack()
            .borrow()
            .get_current_element()
            .in_expression_evaluation
    }

    pub fn set_in_expression_evaluation(&self, value: bool) {
        self.get_callstack()
            .borrow_mut()
            .get_current_element_mut()
            .in_expression_evaluation = value;
    }

    pub fn push_evaluation_stack(&mut self, obj: Rc<dyn RTObject>) {
        self.evaluation_stack.push(obj);
    }

    pub fn pop_evaluation_stack(&mut self) -> Result<Rc<dyn RTObject>, StoryError> {
        self.evaluation_stack
            .pop()
            .ok_or_else(|| StoryError::InvalidStoryState("Evaluation stack underflow".to_owned()))
    }

    pub fn pop_evaluation_stack_multiple(
        &mut self,
        number_of_objects: usize,
    ) -> Result<Vec<Rc<dyn RTObject>>, StoryError> {
        if self.evaluation_stack.len() < number_of_objects {
            return Err(StoryError::InvalidStoryState(format!(
                "Evaluation stack underflow: expected {number_of_objects} value(s), found {}.",
                self.evaluation_stack.len()
            )));
        }

        let start = self.evaluation_stack.len() - number_of_objects;
        let obj: Vec<Rc<dyn RTObject>> = self.evaluation_stack.drain(start..).collect();

        Ok(obj)
    }

    pub fn peek_evaluation_stack(&self) -> Option<&Rc<dyn RTObject>> {
        self.evaluation_stack.last()
    }
}
