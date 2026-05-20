use std::rc::Rc;

use crate::{
    choice::Choice,
    path::Path,
    pointer::{self, Pointer},
    story::Story,
    story_error::StoryError,
};

use super::StoryState;

impl StoryState {
    pub fn can_continue(&self) -> bool {
        !self.get_current_pointer().is_null() && !self.has_error()
    }

    /// String representation of the location where the story currently is.
    pub fn current_path_string(&self) -> Option<String> {
        let pointer = self.get_current_pointer();
        pointer.get_path().map(|path| path.to_string())
    }

    pub fn get_current_pointer(&self) -> Pointer {
        self.get_callstack()
            .borrow()
            .get_current_element()
            .current_pointer
            .clone()
    }

    pub fn set_did_safe_exit(&mut self, did_safe_exit: bool) {
        self.did_safe_exit = did_safe_exit;
    }

    pub fn get_generated_choices_mut(&mut self) -> &mut Vec<Rc<Choice>> {
        &mut self.current_flow.current_choices
    }

    pub fn get_generated_choices(&self) -> &Vec<Rc<Choice>> {
        &self.current_flow.current_choices
    }

    pub fn is_did_safe_exit(&self) -> bool {
        self.did_safe_exit
    }

    pub fn set_current_pointer(&self, pointer: Pointer) {
        self.get_callstack()
            .as_ref()
            .borrow_mut()
            .get_current_element_mut()
            .current_pointer = pointer;
    }

    pub fn set_previous_pointer(&self, p: Pointer) {
        self.get_callstack()
            .as_ref()
            .borrow_mut()
            .get_current_thread_mut()
            .previous_pointer = p.clone();
    }

    pub fn get_previous_pointer(&self) -> Pointer {
        self.get_callstack()
            .as_ref()
            .borrow_mut()
            .get_current_thread_mut()
            .previous_pointer
            .clone()
    }

    pub fn get_current_choices(&self) -> Option<&Vec<Rc<Choice>>> {
        // If we can continue generating text content rather than choices,
        // then we reflect the choice list as being empty, since choices
        // should always come at the end.
        if self.can_continue() {
            return None;
        }

        Some(&self.current_flow.current_choices)
    }

    pub fn set_diverted_pointer(&mut self, p: Pointer) {
        self.diverted_pointer = p;
    }

    pub fn set_chosen_path(
        &mut self,
        path: &Path,
        incrementing_turn_index: bool,
    ) -> Result<(), StoryError> {
        // Changing direction, assume we need to clear current set of choices
        self.current_flow.current_choices.clear();

        let mut new_pointer = Story::pointer_at_path(&self.main_content_container, path)?;
        if !new_pointer.is_null() && new_pointer.index == -1 {
            new_pointer.index = 0;
        }

        self.set_current_pointer(new_pointer);

        let _ = incrementing_turn_index;

        Ok(())
    }

    pub(crate) fn force_end(&mut self) {
        self.get_callstack().borrow_mut().reset();

        self.current_flow.current_choices.clear();

        self.set_current_pointer(pointer::NULL.clone());
        self.set_previous_pointer(pointer::NULL.clone());

        self.set_did_safe_exit(true);
    }
}
