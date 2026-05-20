use std::{cell::RefCell, rc::Rc};

use crate::state_patch::StatePatch;

use super::StoryState;

impl StoryState {
    pub fn copy_and_start_patching(&self, for_background_save: bool) -> StoryState {
        let mut copy = StoryState::new(self.main_content_container.clone());

        copy.patch = Some(self.patch.clone().unwrap_or_else(StatePatch::new));

        // Hijack the new default flow to become a copy of our current one.
        // If the patch is applied, then this new flow will replace the old one.
        copy.current_flow.name = self.current_flow.name.clone();
        copy.current_flow.callstack = Rc::new(RefCell::new(
            self.current_flow.callstack.as_ref().borrow().clone(),
        ));
        copy.current_flow.output_stream = self.current_flow.output_stream.clone();
        copy.output_stream_dirty();

        // Background saves need choice copies because each choice snapshots the
        // thread at generation time, while internal snapshots can ref-copy.
        if for_background_save {
            copy.current_flow.current_choices =
                Vec::with_capacity(self.current_flow.current_choices.len());

            for choice in self.current_flow.current_choices.iter() {
                let c = choice.as_ref().clone();
                copy.current_flow.current_choices.push(Rc::new(c));
            }
        } else {
            copy.current_flow.current_choices = self.current_flow.current_choices.clone();
        }

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
