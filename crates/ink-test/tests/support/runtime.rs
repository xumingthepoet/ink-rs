use std::{cell::RefCell, rc::Rc};

use ink_runtime::{choice::Choice, story::Story as RuntimeStory};

pub use ink_runtime::{
    story::external_functions::ExternalFunction, story_error::StoryError, value_type::ValueType,
};

pub struct Story {
    inner: RuntimeStory,
    source_json: String,
}

impl Clone for Story {
    fn clone(&self) -> Self {
        let mut cloned = Self::new(&self.source_json);
        let saved_state = self.save_state();
        cloned.load_state(&saved_state);
        cloned
    }
}

impl Story {
    pub fn new(json: &str) -> Self {
        Self {
            inner: RuntimeStory::new(json).expect("expected runtime story to load"),
            source_json: json.to_string(),
        }
    }

    pub fn can_continue(&self) -> bool {
        self.inner.can_continue()
    }

    pub fn cont(&mut self) -> String {
        self.inner.cont().expect("expected story to continue")
    }

    pub fn cont_maximally(&mut self) -> String {
        self.inner
            .continue_maximally()
            .expect("expected story to continue maximally")
    }

    pub fn continue_maximally(&mut self) -> String {
        self.cont_maximally()
    }

    pub fn choose_choice_index(&mut self, idx: usize) {
        self.inner
            .choose_choice_index(idx)
            .expect("expected choice index to be valid");
    }

    pub fn choose_path_string(
        &mut self,
        path: &str,
        reset_callstack: bool,
        arguments: Option<Vec<ValueType>>,
    ) {
        self.inner
            .choose_path_string(path, reset_callstack, arguments.as_ref())
            .expect("expected path to resolve");
    }

    pub fn get_current_choices(&self) -> Vec<Rc<Choice>> {
        self.inner.get_current_choices()
    }

    pub fn get_current_tags(&mut self) -> Vec<String> {
        self.inner
            .get_current_tags()
            .expect("expected current tags to load")
    }

    pub fn get_current_errors(&self) -> Vec<String> {
        self.inner.get_current_errors().clone()
    }

    pub fn get_global_tags(&self) -> Vec<String> {
        self.inner
            .get_global_tags()
            .expect("expected global tags to load")
    }

    pub fn get_variable(&self, name: &str) -> Option<ValueType> {
        self.inner.get_variable(name)
    }

    pub fn set_variable(&mut self, name: &str, value: &ValueType) -> Result<(), StoryError> {
        self.inner.set_variable(name, value)
    }

    pub fn bind_external_function(
        &mut self,
        func_name: &str,
        func: Rc<RefCell<dyn ExternalFunction>>,
        lookahead_safe: bool,
    ) {
        self.inner
            .bind_external_function(func_name, func, lookahead_safe)
            .expect("expected external function binding to succeed");
    }

    pub fn call_internal(
        &mut self,
        func_name: &str,
        arguments: Option<Vec<ValueType>>,
    ) -> Result<Option<ValueType>, StoryError> {
        self.inner.call_internal(func_name, arguments.as_ref())
    }

    pub fn save_state(&self) -> String {
        self.inner.save_state().expect("expected state to save")
    }

    pub fn try_save_state(&self) -> Result<String, StoryError> {
        self.inner.save_state()
    }

    pub fn load_state(&mut self, json: &str) {
        self.inner.load_state(json).expect("expected state to load")
    }

    pub fn build_string_of_hierarchy(&self) -> String {
        self.inner.build_string_of_hierarchy()
    }
}
