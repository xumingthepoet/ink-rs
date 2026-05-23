use std::{cell::RefCell, rc::Rc};

use crate::state_patch::StatePatch;

use super::StoryState;

impl StoryState {
    pub fn copy_for_choice_save_snapshot(&self) -> StoryState {
        let mut copy = StoryState::new(self.main_content_container.clone());

        copy.current_execution.callstack = Rc::new(RefCell::new(
            self.current_execution.callstack.as_ref().borrow().clone(),
        ));
        copy.current_execution.output_stream = self.current_execution.output_stream.clone();
        copy.current_execution.current_choices.clear();
        copy.output_stream_dirty();

        if self.has_error() {
            copy.current_errors = self.current_errors.clone();
        }

        if self.has_warning() {
            copy.current_warnings = self.current_warnings.clone();
        }

        copy.variables_state = self.variables_state.clone();
        copy.variables_state
            .set_callstack(copy.get_callstack().clone());
        copy.variables_state.patch = None;

        copy.evaluation_stack = self.evaluation_stack.clone();
        copy.diverted_pointer = self.diverted_pointer.clone();
        copy.set_previous_pointer(self.get_previous_pointer().clone());
        copy.story_seed = self.story_seed;
        copy.previous_random = self.previous_random;
        copy.set_did_safe_exit(self.did_safe_exit);

        copy
    }

    pub fn copy_and_start_patching(&self) -> StoryState {
        let mut copy = StoryState::new(self.main_content_container.clone());

        copy.patch = Some(self.patch.clone().unwrap_or_else(StatePatch::new));

        // If the patch is applied, this copy will replace the current
        // execution state.
        copy.current_execution.callstack = Rc::new(RefCell::new(
            self.current_execution.callstack.as_ref().borrow().clone(),
        ));
        copy.current_execution.output_stream = self.current_execution.output_stream.clone();
        copy.output_stream_dirty();
        copy.current_execution.current_choices = self.current_execution.current_choices.clone();

        if self.has_error() {
            copy.current_errors = self.current_errors.clone();
        }

        if self.has_warning() {
            copy.current_warnings = self.current_warnings.clone();
        }

        // Ref copy: the same variables state is used, with this patch active.
        copy.variables_state = self.variables_state.clone();
        copy.variables_state
            .set_callstack(copy.get_callstack().clone());
        copy.variables_state.patch = copy.patch.clone();

        copy.evaluation_stack = self.evaluation_stack.clone();

        if !self.diverted_pointer.is_null() {
            copy.diverted_pointer = self.diverted_pointer.clone();
        }

        copy.set_previous_pointer(self.get_previous_pointer().clone());

        copy.story_seed = self.story_seed;
        copy.previous_random = self.previous_random;

        copy.set_did_safe_exit(self.did_safe_exit);

        copy
    }

    pub fn restore_after_patch(&mut self) {
        // VariablesState was borrowed by the patched state, so restore it with
        // this state's callstack and patch.
        self.variables_state.callstack = self.get_callstack().clone();
        self.variables_state.patch = self.patch.clone();
    }

    pub fn apply_any_patch(&mut self) {
        if self.patch.is_none() {
            return;
        }

        self.variables_state.apply_patch();

        self.patch = None;
    }
}
