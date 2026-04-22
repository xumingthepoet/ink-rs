use std::{
    cell::RefCell,
    error::Error,
    fmt,
    rc::Rc,
    sync::{Arc, Mutex},
};

use ink_compiler::{Compiler, CompilerOptions};
use ink_runtime::{
    choice::Choice, story::errors::ErrorHandler as RuntimeErrorHandler,
    story::external_functions::ExternalFunction as RuntimeExternalFunction,
    story::variable_observer::VariableObserver as RuntimeVariableObserver,
    story::Story as RuntimeStory, story_error::StoryError as RuntimeStoryError,
    value_type::ValueType as RuntimeValueType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryError(pub String);

impl fmt::Display for StoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for StoryError {}

impl From<RuntimeStoryError> for StoryError {
    fn from(value: RuntimeStoryError) -> Self {
        Self(value.to_string())
    }
}

fn panic_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "story panicked".to_string()
    }
}

fn catch_story_error<T>(f: impl FnOnce() -> T) -> Result<T, StoryError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .map_err(|payload| StoryError(panic_to_string(payload)))
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueType {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
}

pub trait FromValueType: Sized {
    fn from_value(value: &ValueType) -> Option<Self>;
}

impl FromValueType for i32 {
    fn from_value(value: &ValueType) -> Option<Self> {
        value.coerce_to_int()
    }
}

impl FromValueType for bool {
    fn from_value(value: &ValueType) -> Option<Self> {
        value.coerce_to_bool()
    }
}

impl FromValueType for String {
    fn from_value(value: &ValueType) -> Option<Self> {
        match value {
            ValueType::String(text) => Some(text.clone()),
            ValueType::Int(value) => Some(value.to_string()),
            ValueType::Float(value) => Some(value.to_string()),
            ValueType::Bool(value) => Some(value.to_string()),
        }
    }
}

impl ValueType {
    pub fn new<T: Into<ValueType>>(value: T) -> Self {
        value.into()
    }

    pub fn coerce_to_int(&self) -> Option<i32> {
        match self {
            ValueType::Int(value) => Some(*value),
            ValueType::Bool(value) => Some(if *value { 1 } else { 0 }),
            ValueType::Float(value) => Some(*value as i32),
            ValueType::String(text) => text.parse::<i32>().ok(),
        }
    }

    pub fn coerce_to_bool(&self) -> Option<bool> {
        match self {
            ValueType::Bool(value) => Some(*value),
            ValueType::Int(value) => Some(*value != 0),
            ValueType::Float(value) => Some(*value != 0.0),
            ValueType::String(text) => Some(!text.is_empty()),
        }
    }

    pub fn get<T: FromValueType>(&self) -> Option<T> {
        T::from_value(self)
    }

    fn into_runtime_value(self) -> RuntimeValueType {
        match self {
            ValueType::Bool(value) => RuntimeValueType::Bool(value),
            ValueType::Int(value) => RuntimeValueType::Int(value),
            ValueType::Float(value) => RuntimeValueType::Float(value),
            ValueType::String(value) => RuntimeValueType::from(value.as_str()),
        }
    }

    fn from_runtime_value(value: RuntimeValueType) -> Self {
        match value {
            RuntimeValueType::Bool(value) => ValueType::Bool(value),
            RuntimeValueType::Int(value) => ValueType::Int(value),
            RuntimeValueType::Float(value) => ValueType::Float(value),
            RuntimeValueType::String(value) => ValueType::String(value.string),
            RuntimeValueType::DivertTarget(path) => ValueType::String(path.to_string()),
            _ => ValueType::String("<unsupported runtime value>".to_string()),
        }
    }
}

impl From<&str> for ValueType {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for ValueType {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<bool> for ValueType {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i32> for ValueType {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<f32> for ValueType {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

pub trait ExternalFunction: Send {
    fn call(&mut self, func_name: &str, args: Vec<ValueType>) -> Option<ValueType>;
}

pub trait VariableObserver: Send {
    fn changed(&mut self, variable_name: &str, new_value: &ValueType);
}

struct ExternalFunctionAdapter {
    function: Arc<Mutex<dyn ExternalFunction>>,
}

impl RuntimeExternalFunction for ExternalFunctionAdapter {
    fn call(&mut self, func_name: &str, args: Vec<RuntimeValueType>) -> Option<RuntimeValueType> {
        let args = args
            .into_iter()
            .map(ValueType::from_runtime_value)
            .collect::<Vec<_>>();

        let mut function = self.function.lock().expect("external function lock");
        function
            .call(func_name, args)
            .map(ValueType::into_runtime_value)
    }
}

struct VariableObserverAdapter {
    observer: Arc<Mutex<dyn VariableObserver>>,
}

impl RuntimeVariableObserver for VariableObserverAdapter {
    fn changed(&mut self, variable_name: &str, value: &RuntimeValueType) {
        let converted = ValueType::from_runtime_value(value.clone());
        self.observer
            .lock()
            .expect("variable observer lock")
            .changed(variable_name, &converted);
    }
}

pub struct Story {
    inner: RuntimeStory,
    compiled_json: String,
    source_ink: String,
    last_current_text: String,
}

impl Clone for Story {
    fn clone(&self) -> Self {
        let mut cloned = Self::new(&self.source_ink);
        let saved_state = self.save_state();
        cloned.load_state(&saved_state);
        cloned
    }
}

#[allow(dead_code)]
impl Story {
    pub fn new(source: &str) -> Self {
        Self::from_source(source, None)
    }

    pub fn from_source(source: &str, source_filename: Option<&str>) -> Self {
        let options = CompilerOptions {
            source_filename: source_filename.map(ToString::to_string),
            ..Default::default()
        };
        let mut compiler = Compiler::new(source.to_string(), Some(options));
        let compile_json_result = compiler.compile_json();

        if compile_json_result.json.is_none() || compile_json_result.has_errors() {
            panic!(
                "source compilation should succeed: {:?}",
                compile_json_result.diagnostics
            );
        }

        let compiled_json = compile_json_result
            .json
            .expect("json checked to exist above");
        let inner = RuntimeStory::new(&compiled_json).expect("expected runtime story to load");

        Self {
            inner,
            compiled_json,
            source_ink: source.to_string(),
            last_current_text: String::new(),
        }
    }

    pub fn to_json(&self) -> String {
        self.compiled_json.clone()
    }

    pub fn can_continue(&self) -> bool {
        self.inner.can_continue()
    }

    pub fn cont(&mut self) -> String {
        self.inner.cont().expect("expected story to continue")
    }

    pub fn cont_async(&mut self, millisecs_limit: f32) -> String {
        let _ = millisecs_limit;
        let current_text = self.inner.cont().expect("expected story to continue");
        let delta = current_text
            .strip_prefix(&self.last_current_text)
            .unwrap_or(current_text.as_str())
            .to_string();
        self.last_current_text = current_text;
        delta
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
        let runtime_args = arguments.map(|values| {
            values
                .into_iter()
                .map(ValueType::into_runtime_value)
                .collect::<Vec<_>>()
        });

        self.inner
            .choose_path_string(path, reset_callstack, runtime_args.as_ref())
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

    pub fn get_current_errors_ref(&self) -> Vec<String> {
        self.get_current_errors()
    }

    pub fn get_current_warnings(&self) -> Vec<String> {
        self.inner.get_current_warnings().clone()
    }

    pub fn get_current_text(&mut self) -> String {
        let current_text = self
            .inner
            .get_current_text()
            .expect("expected current text");
        self.last_current_text = current_text.clone();
        current_text
    }

    pub fn get_current_text_ref(&mut self) -> String {
        self.get_current_text()
    }

    pub fn switch_flow(&mut self, flow_name: &str) {
        self.inner
            .switch_flow(flow_name)
            .expect("expected flow switch to succeed");
    }

    pub fn remove_flow(&mut self, flow_name: &str) {
        self.inner
            .remove_flow(flow_name)
            .expect("expected flow removal to succeed");
    }

    pub fn get_global_tags(&self) -> Vec<String> {
        self.inner
            .get_global_tags()
            .expect("expected global tags to load")
    }

    pub fn tags_for_content_at_path(&self, path: &str) -> Vec<String> {
        self.inner
            .tags_for_content_at_path(path)
            .expect("expected tags to load for path")
    }

    pub fn get_variable(&self, name: &str) -> Option<ValueType> {
        self.inner
            .get_variable(name)
            .map(ValueType::from_runtime_value)
    }

    pub fn set_variable(&mut self, name: &str, value: &ValueType) -> Result<(), StoryError> {
        self.inner
            .set_variable(name, &value.clone().into_runtime_value())
            .map_err(StoryError::from)
    }

    pub fn set_allow_external_function_fallbacks(&mut self, value: bool) {
        self.inner.set_allow_external_function_fallbacks(value);
    }

    pub fn set_error_handler(&mut self, err_handler: Rc<RefCell<dyn RuntimeErrorHandler>>) {
        self.inner.set_error_handler(err_handler);
    }

    pub fn evaluate_function(
        &mut self,
        function_name: &str,
        arguments: Option<Vec<ValueType>>,
        text_output: &mut String,
    ) -> Option<ValueType> {
        let runtime_args = arguments.map(|values| {
            values
                .into_iter()
                .map(ValueType::into_runtime_value)
                .collect::<Vec<_>>()
        });

        self.inner
            .evaluate_function(function_name, runtime_args.as_ref(), text_output)
            .ok()
            .flatten()
            .map(ValueType::from_runtime_value)
    }

    pub fn bind_external_function(
        &mut self,
        func_name: &str,
        func: Arc<Mutex<dyn ExternalFunction>>,
        lookahead_safe: bool,
    ) {
        let adapter: Rc<RefCell<dyn RuntimeExternalFunction>> =
            Rc::new(RefCell::new(ExternalFunctionAdapter { function: func }));
        self.inner
            .bind_external_function(func_name, adapter, lookahead_safe)
            .expect("expected external function binding to succeed");
    }

    pub fn observe_variable(
        &mut self,
        variable_name: &str,
        observer: Arc<Mutex<dyn VariableObserver>>,
    ) {
        let adapter: Rc<RefCell<dyn RuntimeVariableObserver>> =
            Rc::new(RefCell::new(VariableObserverAdapter { observer }));
        self.inner
            .observe_variable(variable_name, adapter)
            .expect("expected variable observer registration to succeed");
    }

    pub fn save_state(&self) -> String {
        self.inner.save_state().expect("expected state to save")
    }

    pub fn load_state(&mut self, json: &str) {
        self.inner.load_state(json).expect("expected state to load")
    }

    pub fn get_visit_count_at_path_string(&self, path: &str) -> Result<i32, StoryError> {
        self.inner
            .get_visit_count_at_path_string(path)
            .map_err(StoryError::from)
    }

    pub fn build_string_of_hierarchy(&self) -> String {
        self.inner.build_string_of_hierarchy()
    }

    pub fn get_can_continue(&self) -> bool {
        self.can_continue()
    }

    pub fn get_current_choices_len(&self) -> usize {
        self.get_current_choices().len()
    }

    pub fn get_current_choices_ref(&self) -> Vec<Rc<Choice>> {
        self.get_current_choices()
    }

    pub fn is_ended(&self) -> bool {
        !self.can_continue() && self.get_current_choices().is_empty()
    }
}

pub mod story_error {
    pub use super::StoryError;
}

pub mod value_type {
    #[allow(unused_imports)]
    pub use super::{FromValueType, ValueType};
}

pub mod story {
    #[allow(unused_imports)]
    pub use super::{ExternalFunction, Story, VariableObserver};

    pub mod external_functions {
        #[allow(unused_imports)]
        pub use super::super::ExternalFunction;
    }

    pub mod variable_observer {
        #[allow(unused_imports)]
        pub use super::super::VariableObserver;
    }
}
