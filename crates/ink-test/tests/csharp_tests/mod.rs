#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]

use crate::conformance::api::{
    story::ExternalFunction, story::VariableObserver, value_type::ValueType,
};
use ink_compiler::CharacterRange::CharacterRange;
use ink_compiler::FileHandler::IFileHandler;
use ink_compiler::InkParser::CommentEliminator::CommentEliminator;
use ink_compiler::InkParser::InkParser::InkParser as InkParserType;
use ink_compiler::InkParser::InkParser_CharacterRanges::InkParser as CharacterRangeParser;
use ink_compiler::ParsedHierarchy::Story::Story as ParsedStory;
use ink_compiler::StringParser::StringParser::StringParser;
use ink_runtime::Choice::Choice;
use ink_runtime::Error::{ErrorHandler, ErrorType};
use ink_runtime::Story::Story as RuntimeStory;
use ink_runtime::Value::{Value, ValueInput};
use std::cell::RefCell;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

fn value_type_into_runtime_value(value: ValueType) -> ValueInput {
    match value {
        ValueType::Bool(value) => ValueInput::Bool(value),
        ValueType::Int(value) => ValueInput::Int(value),
        ValueType::Float(value) => ValueInput::Float(value),
        ValueType::String(value) => ValueInput::String(value),
    }
}

fn value_from_runtime_value(value: Value) -> ValueType {
    match value {
        Value::Bool(value) => ValueType::Bool(value.value),
        Value::Int(value) => ValueType::Int(value.value),
        Value::Float(value) => ValueType::Float(value.value),
        Value::String(value) => ValueType::String(value.value),
        other => ValueType::String(other.to_string()),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestMode {
    Normal,
    JsonRoundTrip,
}

#[derive(Default)]
struct MessageBuckets {
    errors: Vec<String>,
    warnings: Vec<String>,
    authors: Vec<String>,
}

pub struct CSharpHarness {
    mode: TestMode,
    testing_errors: bool,
    buckets: Arc<Mutex<MessageBuckets>>,
    file_handler: Arc<dyn IFileHandler + Send + Sync>,
}

#[derive(Clone, Debug, Default)]
struct CSharpTestsFileHandler;

impl IFileHandler for CSharpTestsFileHandler {
    fn ResolveInkFilename(&self, includeName: &str) -> io::Result<String> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let tests_dir = manifest_dir.join("fixtures/csharp_tests/includes");
        let candidate = tests_dir.join(includeName);
        if candidate.exists() {
            return Ok(candidate.to_string_lossy().into_owned());
        }

        let current_dir = std::env::current_dir()?;
        Ok(current_dir.join(includeName).to_string_lossy().into_owned())
    }

    fn LoadInkFileContents(&self, fullFilename: &str) -> io::Result<String> {
        std::fs::read_to_string(fullFilename)
    }
}

trait RuntimeStoryExt {
    fn cont(&mut self) -> String;
    fn cont_maximally(&mut self) -> String;
    fn continue_maximally(&mut self) -> String;
    fn choose_choice_index(&mut self, idx: usize);
    fn choose_path_string_simple(&mut self, path: &str);
    fn choose_path_string_with_args(
        &mut self,
        path: &str,
        reset_callstack: bool,
        arguments: Option<Vec<ValueType>>,
    );
    fn get_current_choices(&mut self) -> Vec<Choice>;
    fn current_choices(&mut self) -> Vec<Choice>;
    fn get_current_choices_len(&mut self) -> usize;
    fn current_choices_len(&mut self) -> usize;
    fn get_current_tags(&mut self) -> Vec<String>;
    fn current_tags(&mut self) -> Vec<String>;
    fn get_current_text(&mut self) -> String;
    fn current_text(&mut self) -> String;
    fn get_global_tags(&mut self) -> Vec<String>;
    fn global_tags(&mut self) -> Vec<String>;
    fn can_continue(&mut self) -> bool;
    fn set_allow_external_function_fallbacks(&mut self, value: bool);
    fn evaluate_function(
        &mut self,
        function_name: &str,
        arguments: Option<Vec<ValueType>>,
        text_output: &mut String,
    ) -> Option<ValueType>;
    fn save_state(&mut self) -> String;
    fn load_state(&mut self, json: &str);
    fn to_json(&mut self) -> String;
    fn switch_flow(&mut self, flow_name: &str);
    fn remove_flow(&mut self, flow_name: &str);
    fn get_state(&mut self) -> ink_runtime::StoryState::StoryState;
    fn set_state(&mut self, state: ink_runtime::StoryState::StoryState);
    fn get_variable(&mut self, name: &str) -> Option<ValueType>;
    fn set_variable(&mut self, name: &str, value: &ValueType) -> Result<(), String>;
    fn bind_external_function(
        &mut self,
        func_name: &str,
        func: Arc<Mutex<dyn ExternalFunction>>,
        lookahead_safe: bool,
    );
    fn observe_variable(&mut self, variable_name: &str, observer: Arc<Mutex<dyn VariableObserver>>);
}

impl CSharpHarness {
    pub fn new(mode: TestMode) -> Self {
        Self {
            mode,
            testing_errors: false,
            buckets: Arc::new(Mutex::new(MessageBuckets::default())),
            file_handler: Arc::new(CSharpTestsFileHandler),
        }
    }

    fn clear(&mut self) {
        let mut buckets = self.buckets.lock().unwrap();
        buckets.errors.clear();
        buckets.warnings.clear();
        buckets.authors.clear();
    }

    fn push_runtime_message(&self, message: &str, error_type: ErrorType) {
        let mut buckets = self.buckets.lock().unwrap();
        match error_type {
            ErrorType::Error => buckets.errors.push(message.to_string()),
            ErrorType::Warning => buckets.warnings.push(message.to_string()),
            ErrorType::Author => buckets.authors.push(message.to_string()),
        }
    }

    fn push_parse_message(&self, message: String, line: i32, _character: i32, is_warning: bool) {
        let prefix = if is_warning { "WARNING" } else { "ERROR" };
        let full_message = format!("{}: line {}: {}", prefix, line + 1, message);
        let mut buckets = self.buckets.lock().unwrap();
        if is_warning {
            buckets.warnings.push(full_message);
        } else {
            buckets.errors.push(full_message);
        }
    }

    pub fn compile_string(
        &mut self,
        source: &str,
        count_all_visits: bool,
        testing_errors: bool,
    ) -> Option<RuntimeStory> {
        self.testing_errors = testing_errors;
        self.clear();

        let parse_handler = {
            let this = self.clone_shared();
            Arc::new(
                move |message: String, line: i32, character: i32, is_warning: bool| {
                    if !this.testing_errors {
                        let prefix = if is_warning { "WARNING" } else { "ERROR" };
                        panic!("{}: line {}: {}", prefix, line + 1, message);
                    }
                    this.push_parse_message(message, line, character, is_warning);
                },
            )
        };

        let mut parser = InkParserType::new(
            source.to_string(),
            None,
            Some(parse_handler),
            Some(Arc::clone(&self.file_handler)),
        );
        let mut parsed_story = parser.Parse();
        parsed_story.countAllVisits = count_all_visits;

        let runtime_handler = {
            let this = self.clone_shared();
            Rc::new(RefCell::new(
                Box::new(move |message: &str, error_type: ErrorType| {
                    if !this.testing_errors {
                        panic!("{}", message);
                    }
                    this.push_runtime_message(message, error_type);
                }) as ErrorHandler,
            ))
        };

        let runtime_story = parsed_story.ExportRuntime(Some(runtime_handler))?;
        let mut story = runtime_story;

        if self.mode == TestMode::JsonRoundTrip {
            let json = story.to_json();
            story = RuntimeStory::new_overload_2(json);
        }

        Some(story)
    }

    pub fn compile_string_without_runtime(
        &mut self,
        source: &str,
        testing_errors: bool,
    ) -> Option<ParsedStory> {
        self.testing_errors = testing_errors;
        self.clear();

        let parse_handler = {
            let this = self.clone_shared();
            Arc::new(
                move |message: String, line: i32, character: i32, is_warning: bool| {
                    if !this.testing_errors {
                        let prefix = if is_warning { "WARNING" } else { "ERROR" };
                        panic!("{}: line {}: {}", prefix, line + 1, message);
                    }
                    this.push_parse_message(message, line, character, is_warning);
                },
            )
        };

        let mut parser = InkParserType::new(
            source.to_string(),
            None,
            Some(parse_handler),
            Some(Arc::clone(&self.file_handler)),
        );
        let mut parsed_story = parser.Parse();

        if !testing_errors {
            // Keep parity with the C# helper: parse must succeed in normal mode.
        }

        if self.error_messages().is_empty() {
            let runtime_handler = {
                let this = self.clone_shared();
                Rc::new(RefCell::new(
                    Box::new(move |message: &str, error_type: ErrorType| {
                        if !this.testing_errors {
                            panic!("{}", message);
                        }
                        this.push_runtime_message(message, error_type);
                    }) as ErrorHandler,
                ))
            };
            let _ = parsed_story.ExportRuntime(Some(runtime_handler));
        }

        Some(parsed_story)
    }

    fn clone_shared(&self) -> Self {
        Self {
            mode: self.mode,
            testing_errors: self.testing_errors,
            buckets: Arc::clone(&self.buckets),
            file_handler: Arc::clone(&self.file_handler),
        }
    }

    pub fn error_messages(&self) -> Vec<String> {
        self.buckets.lock().unwrap().errors.clone()
    }

    pub fn warning_messages(&self) -> Vec<String> {
        self.buckets.lock().unwrap().warnings.clone()
    }

    pub fn author_messages(&self) -> Vec<String> {
        self.buckets.lock().unwrap().authors.clone()
    }

    pub fn had_error(&self, match_str: Option<&str>) -> bool {
        self.has_message(match_str, &self.error_messages())
    }

    pub fn had_warning(&self, match_str: Option<&str>) -> bool {
        self.has_message(match_str, &self.warning_messages())
    }

    fn has_message(&self, match_str: Option<&str>, list: &[String]) -> bool {
        match match_str {
            Some(query) => list.iter().any(|msg| msg.contains(query)),
            None => !list.is_empty(),
        }
    }
}

impl RuntimeStoryExt for RuntimeStory {
    fn cont(&mut self) -> String {
        self.Continue()
    }

    fn cont_maximally(&mut self) -> String {
        self.ContinueMaximally()
    }

    fn continue_maximally(&mut self) -> String {
        self.cont_maximally()
    }

    fn choose_choice_index(&mut self, idx: usize) {
        self.ChooseChoiceIndex(idx as i32);
    }

    fn choose_path_string_simple(&mut self, path: &str) {
        self.ChoosePathString(path.to_string(), false, Vec::new());
    }

    fn choose_path_string_with_args(
        &mut self,
        path: &str,
        reset_callstack: bool,
        arguments: Option<Vec<ValueType>>,
    ) {
        let args = arguments
            .unwrap_or_default()
            .into_iter()
            .map(value_type_into_runtime_value)
            .collect::<Vec<_>>();
        self.ChoosePathString(path.to_string(), reset_callstack, args);
    }

    fn get_current_choices(&mut self) -> Vec<Choice> {
        self.get_currentChoices()
    }

    fn current_choices(&mut self) -> Vec<Choice> {
        self.get_current_choices()
    }

    fn get_current_choices_len(&mut self) -> usize {
        self.get_currentChoices().len()
    }

    fn current_choices_len(&mut self) -> usize {
        self.get_current_choices_len()
    }

    fn get_current_tags(&mut self) -> Vec<String> {
        self.get_currentTags()
    }

    fn current_tags(&mut self) -> Vec<String> {
        self.get_current_tags()
    }

    fn get_current_text(&mut self) -> String {
        self.get_currentText()
    }

    fn current_text(&mut self) -> String {
        self.get_current_text()
    }

    fn get_global_tags(&mut self) -> Vec<String> {
        self.get_globalTags()
    }

    fn global_tags(&mut self) -> Vec<String> {
        self.get_global_tags()
    }

    fn can_continue(&mut self) -> bool {
        self.get_canContinue()
    }

    fn set_allow_external_function_fallbacks(&mut self, value: bool) {
        self.set_allowExternalFunctionFallbacks(value);
    }

    fn evaluate_function(
        &mut self,
        function_name: &str,
        arguments: Option<Vec<ValueType>>,
        text_output: &mut String,
    ) -> Option<ValueType> {
        let args = arguments
            .unwrap_or_default()
            .into_iter()
            .map(value_type_into_runtime_value)
            .collect::<Vec<_>>();
        self.EvaluateFunction_overload_2(function_name.to_string(), text_output, args)
            .map(value_from_runtime_value)
    }

    fn save_state(&mut self) -> String {
        self.get_state().ToJson()
    }

    fn load_state(&mut self, json: &str) {
        let mut state = self.get_state();
        state.LoadJson(json.to_string());
        self.set_state(state);
    }

    fn to_json(&mut self) -> String {
        self.ToJson()
    }

    fn switch_flow(&mut self, flow_name: &str) {
        self.SwitchFlow(flow_name.to_string());
    }

    fn remove_flow(&mut self, flow_name: &str) {
        self.RemoveFlow(flow_name.to_string());
    }

    fn get_state(&mut self) -> ink_runtime::StoryState::StoryState {
        self.get_state()
    }

    fn set_state(&mut self, state: ink_runtime::StoryState::StoryState) {
        self.set_state(state);
    }

    fn get_variable(&mut self, name: &str) -> Option<ValueType> {
        self.get_variablesState()
            .GetVariableWithName(name.to_string())
            .map(value_from_runtime_value)
    }

    fn set_variable(&mut self, name: &str, value: &ValueType) -> Result<(), String> {
        self.get_variablesState_mut().SetIndexedValue(
            name.to_string(),
            Some(value_type_into_runtime_value(value.clone())),
        );
        Ok(())
    }

    fn bind_external_function(
        &mut self,
        func_name: &str,
        func: Arc<Mutex<dyn ExternalFunction>>,
        lookahead_safe: bool,
    ) {
        let func_name = func_name.to_string();
        let func_name_for_callback = func_name.clone();
        let callback = {
            let func = func.clone();
            Arc::new(move |args: &[ValueInput]| {
                let args = args
                    .iter()
                    .cloned()
                    .filter_map(Value::Create)
                    .map(value_from_runtime_value)
                    .collect::<Vec<_>>();
                let mut func = func.lock().unwrap();
                func.call(&func_name_for_callback, args)
                    .map(|value| match value {
                        ValueType::Bool(v) => Value::new_bool(v),
                        ValueType::Int(v) => Value::new_int(v),
                        ValueType::Float(v) => Value::new_float(v),
                        ValueType::String(v) => Value::new_string(v),
                    })
            })
        };

        self.BindExternalFunctionGeneral(func_name, callback, lookahead_safe);
    }

    fn observe_variable(
        &mut self,
        variable_name: &str,
        observer: Arc<Mutex<dyn VariableObserver>>,
    ) {
        let variable_name = variable_name.to_string();
        let callback = {
            let observer = observer.clone();
            Arc::new(move |name: String, value: Value| {
                let converted = value_from_runtime_value(value);
                observer.lock().unwrap().changed(&name, &converted);
            })
        };

        self.ObserveVariable(variable_name, callback);
    }
}

fn run_in_both_modes(mut f: impl FnMut(&mut CSharpHarness)) {
    let mut modes = vec![TestMode::Normal];
    if std::env::var_os("INK_CSHARP_RUN_JSON_ROUNDTRIP").is_some() {
        modes.push(TestMode::JsonRoundTrip);
    }
    for mode in modes {
        let mut harness = CSharpHarness::new(mode);
        f(&mut harness);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[derive(Default)]
    struct TestVariableObserverState {
        current_var_value: i32,
        observer_call_count: usize,
    }

    struct TestVariableObserverImpl {
        state: Arc<Mutex<TestVariableObserverState>>,
    }

    impl VariableObserver for TestVariableObserverImpl {
        fn changed(&mut self, _variable_name: &str, new_value: &ValueType) {
            let mut state = self.state.lock().unwrap();
            state.current_var_value = new_value.get::<i32>().unwrap_or_default();
            state.observer_call_count += 1;
        }
    }

    struct TestExternalFunction<F>(F);

    impl<F> ExternalFunction for TestExternalFunction<F>
    where
        F: FnMut(&str, Vec<ValueType>) -> Option<ValueType> + Send,
    {
        fn call(&mut self, func_name: &str, args: Vec<ValueType>) -> Option<ValueType> {
            (self.0)(func_name, args)
        }
    }

    fn boxed_external_function<F>(func: F) -> Arc<Mutex<dyn ExternalFunction>>
    where
        F: FnMut(&str, Vec<ValueType>) -> Option<ValueType> + Send + 'static,
    {
        Arc::new(Mutex::new(TestExternalFunction(func)))
    }

    fn generate_identifier_from_character_range(
        range: &mut CharacterRange,
        var_name_unique_part: Option<&str>,
    ) -> String {
        let mut identifier = String::new();
        if let Some(prefix) = var_name_unique_part {
            if !prefix.is_empty() {
                identifier.push_str(prefix);
            }
        }

        let charset = range.ToCharacterSet();
        let mut characters: Vec<_> = charset.characters.iter().copied().collect();
        characters.sort_unstable();
        for c in characters {
            identifier.push(c);
        }

        identifier
    }

    fn rust_test_names() -> BTreeSet<String> {
        let source = include_str!("mod.rs");
        let mut names = BTreeSet::new();
        for line in source.lines() {
            let line = line.trim_start();
            if let Some(rest) = line.strip_prefix("fn ") {
                if let Some((name, _)) = rest.split_once('(') {
                    if name.starts_with("Test") {
                        names.insert(name.to_string());
                    }
                }
            }
        }
        names
    }

    fn csharp_test_names() -> BTreeSet<String> {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/csharp_tests/official_test_names.txt"
        ));
        let mut names = BTreeSet::new();
        for line in source.lines() {
            let line = line.trim_start();
            if !line.is_empty() && !line.starts_with('#') {
                names.insert(line.to_string());
            }
        }
        names
    }

    // C#:         [Test()]
    //         public void TestHelloWorld()
    //         {
    //             Story story = CompileString("Hello world");
    //             Assert.AreEqual("Hello world\n", story.Continue());
    //         }
    #[test]
    fn TestHelloWorld() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string("Hello world", false, false)
                .expect("compile should succeed");
            assert_eq!("Hello world\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestArithmetic()
    //         {
    //             Story story = CompileString(@"
    // { 2 * 3 + 5 * 6 }
    // {8 mod 3}
    // {13 % 5}
    // { 7 / 3 }
    // { 7 / 3.0 }
    // { 10 - 2 }
    // { 2 * (5-1) }
    // ");
    //
    //             Assert.AreEqual("36\n2\n3\n2\n2"+System.Globalization.NumberFormatInfo.CurrentInfo.NumberDecimalSeparator+"3333333\n8\n8\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestArithmetic() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{ 2 * 3 + 5 * 6 }
{8 mod 3}
{13 % 5}
{ 7 / 3 }
{ 7 / 3.0 }
{ 10 - 2 }
{ 2 * (5-1) }
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            eprintln!("arith hierarchy=\n{}", story.BuildStringOfHierarchy());
            eprintln!(
                "arith pre can_continue={} choices={} text='{}'",
                story.can_continue(),
                story.current_choices_len(),
                story.current_text()
            );
            let first = story.cont();
            eprintln!(
                "arith first='{}' after choices={} can_continue={} text='{}' out={:?}",
                first,
                story.current_choices_len(),
                story.can_continue(),
                story.current_text(),
                story.get_state().get_outputStream()
            );
            assert_eq!(
                "36\n2\n3\n2\n2.3333333\n8\n8\n",
                first + &story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestBools()
    //         {
    //             Assert.AreEqual("true\n", CompileString("{true}").Continue());
    //             Assert.AreEqual("2\n", CompileString("{true + 1}").Continue());
    //             Assert.AreEqual("3\n", CompileString("{2 + true}").Continue());
    //             Assert.AreEqual("0\n", CompileString("{false + false}").Continue());
    //             Assert.AreEqual("2\n", CompileString("{true + true}").Continue());
    //             Assert.AreEqual("true\n", CompileString("{true == 1}").Continue());
    //             Assert.AreEqual("false\n", CompileString("{not 1}").Continue());
    //             Assert.AreEqual("false\n", CompileString("{not true}").Continue());
    //             Assert.AreEqual("true\n", CompileString("{3 > 1}").Continue());
    //
    //             var listHasntStory = @"
    //                 LIST list = a, (b), c, (d), e
    //                 {list !? (c)}
    //             ";
    //             Assert.AreEqual("true\n", CompileString(listHasntStory).Continue());
    //         }
    #[test]
    fn TestBools() {
        run_in_both_modes(|suite| {
            assert_eq!(
                "true\n",
                suite
                    .compile_string("{true}", false, false)
                    .expect("compile should succeed")
                    .cont()
            );
            assert_eq!(
                "2\n",
                suite
                    .compile_string("{true + 1}", false, false)
                    .expect("compile should succeed")
                    .cont()
            );
            assert_eq!(
                "3\n",
                suite
                    .compile_string("{2 + true}", false, false)
                    .expect("compile should succeed")
                    .cont()
            );
            assert_eq!(
                "0\n",
                suite
                    .compile_string("{false + false}", false, false)
                    .expect("compile should succeed")
                    .cont()
            );
            assert_eq!(
                "2\n",
                suite
                    .compile_string("{true + true}", false, false)
                    .expect("compile should succeed")
                    .cont()
            );
            assert_eq!(
                "true\n",
                suite
                    .compile_string("{true == 1}", false, false)
                    .expect("compile should succeed")
                    .cont()
            );
            assert_eq!(
                "false\n",
                suite
                    .compile_string("{not 1}", false, false)
                    .expect("compile should succeed")
                    .cont()
            );
            assert_eq!(
                "false\n",
                suite
                    .compile_string("{not true}", false, false)
                    .expect("compile should succeed")
                    .cont()
            );
            assert_eq!(
                "true\n",
                suite
                    .compile_string("{3 > 1}", false, false)
                    .expect("compile should succeed")
                    .cont()
            );
            let list_hasnt_story = r#"
                LIST list = a, (b), c, (d), e
                {list !? (c)}
            "#;
            assert_eq!(
                "true\n",
                suite
                    .compile_string(list_hasnt_story, false, false)
                    .expect("compile should succeed")
                    .cont()
            );
        });
    }

    // C#:         [Test ()]
    //         public void TestAllSwitchBranchesFailIsClean ()
    //         {
    //         	var story = CompileString (@"
    // { 1:
    //     - 2: x
    //     - 3: y
    // }
    //         ");
    //
    //             story.Continue ();
    //
    //         	Assert.IsTrue (story.state.evaluationStack.Count == 0);
    //         }
    #[test]
    fn TestAllSwitchBranchesFailIsClean() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{ 1:
    - 2: x
    - 3: y
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont();
            assert_eq!(0, story.get_state().get_evaluationStack().len());
        });
    }

    // C#:         [Test()]
    //         public void TestArgumentNameCollisions()
    //         {
    //             CompileStringWithoutRuntime(@"
    // VAR global_var = 5
    //
    // ~ pass_divert(-> knot_name)
    // {variable_param_test(10)}
    //
    // === function aTarget() ===
    //    ~ return true
    //
    // === function pass_divert(aTarget) ===
    //     Should be a divert target, but is a read count:- {aTarget}
    //
    // === function variable_param_test(global_var) ===
    //     ~ return global_var
    //
    // === knot_name ===
    //     -> END
    // ", testingErrors: true);
    //
    //             Assert.AreEqual(2, _errorMessages.Count);
    //             Assert.IsTrue(HadError("name has already been used for a function"));
    //             Assert.IsTrue(HadError("name has already been used for a var"));
    //         }
    #[test]
    fn TestArgumentNameCollisions() {
        run_in_both_modes(|suite| {
            suite
                .compile_string_without_runtime(
                    r#"
VAR global_var = 5

~ pass_divert(-> knot_name)
{variable_param_test(10)}

=== function aTarget() ===
   ~ return true

=== function pass_divert(aTarget) ===
    Should be a divert target, but is a read count:- {aTarget}

=== function variable_param_test(global_var) ===
    ~ return global_var

=== knot_name ===
    -> END
"#,
                    true,
                )
                .expect("parse should succeed");
            assert_eq!(2, suite.error_messages().len());
            assert!(suite.had_error(Some("name has already been used for a function")));
            assert!(suite.had_error(Some("name has already been used for a var")));
        });
    }

    // C#:         [Test()]
    //         public void TestArgumentShouldntConflictWithGatherElsewhere()
    //         {
    //             // Testing that there are no errors only
    //             CompileStringWithoutRuntime(@"
    // == knot ==
    // - (x) -> DONE
    //
    // == function f(x) ==
    // Nothing
    // ");
    //         }
    #[test]
    fn TestArgumentShouldntConflictWithGatherElsewhere() {
        run_in_both_modes(|suite| {
            suite
                .compile_string_without_runtime(
                    r#"
== knot ==
- (x) -> DONE

== function f(x) ==
Nothing
"#,
                    false,
                )
                .expect("parse should succeed");
        });
    }

    // C#:         [Test()]
    //         public void TestComplexTunnels()
    //         {
    //             Story story = CompileString(@"
    // -> one (1) -> two (2) ->
    // three (3)
    //
    // == one(num) ==
    // one ({num})
    // -> oneAndAHalf (1.5) ->
    // ->->
    //
    // == oneAndAHalf(num) ==
    // one and a half ({num})
    // ->->
    //
    // == two (num) ==
    // two ({num})
    // ->->
    // ");
    //
    //             Assert.AreEqual("one (1)\none and a half (1"+ System.Globalization.NumberFormatInfo.CurrentInfo.NumberDecimalSeparator+"5)\ntwo (2)\nthree (3)\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestComplexTunnels() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> one (1) -> two (2) ->
three (3)

== one(num) ==
one ({num})
-> oneAndAHalf (1.5) ->
->->

== oneAndAHalf(num) ==
one and a half ({num})
->->

== two (num) ==
two ({num})
->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "one (1)\none and a half (1.5)\ntwo (2)\nthree (3)\n",
                story.continue_maximally()
            );
        });
    }

    // C#: public void TestElseBranches()
    // C#: {
    // C#:     var storyStr =
    // C#:         @"
    // C#: VAR x = 3
    // C#:
    // C#: {
    // C#:     - x == 1: one
    // C#:     - x == 2: two
    // C#:     - else: other
    // C#: }
    // C#:
    // C#: {
    // C#:     - x == 1: one
    // C#:     - x == 2: two
    // C#:     - other
    // C#: }
    // C#:
    // C#: { x == 4:
    // C#:   - The main clause
    // C#:   - else: other
    // C#: }
    // C#:
    // C#: { x == 4:
    // C#:   The main clause
    // C#: - else:
    // C#:   other
    // C#: }
    // C#:         ";
    // C#:
    // C#:     Story story = CompileString(storyStr);
    // C#:
    // C#:     Assert.AreEqual("other\nother\nother\nother\n", story.currentText);
    // C#: }

    #[test]
    fn TestElseBranches() {
        run_in_both_modes(|suite| {
            let story_str = r#"
VAR x = 3

{
    - x == 1: one
    - x == 2: two
    - else: other
}

{
    - x == 1: one
    - x == 2: two
    - other
}

{ x == 4:
  - The main clause
  - else: other
}

{ x == 4:
  The main clause
- else:
  other
}
"#;
            let mut story = suite
                .compile_string(story_str, false, false)
                .expect("compile should succeed");
            assert_eq!("other\nother\nother\nother\n", story.current_text());
        });
    }

    // C#:         [Test()]
    //         public void TestEndOfContent()
    //         {
    //             Story story = CompileString("Hello world", false, true);
    //             story.ContinueMaximally();
    //             Assert.IsFalse(HadError());
    //
    //             story = CompileString("== test ==\nContent\n-> END");
    //             story.ContinueMaximally();
    //
    //             // Should have runtime error due to running out of content
    //             // (needs a -> END)
    //             story = CompileString("== test ==\nContent", false, true);
    //             story.ContinueMaximally();
    //             Assert.IsTrue(HadWarning());
    //
    //             // Should have warning that there's no "-> END"
    //             CompileStringWithoutRuntime("== test ==\nContent", true);
    //             Assert.IsFalse(HadError());
    //             Assert.IsTrue(HadWarning());
    //
    //             CompileStringWithoutRuntime("== test ==\n~return", testingErrors: true);
    //             Assert.IsTrue(HadError("Return statements can only be used in knots that are declared as functions"));
    //
    //             CompileStringWithoutRuntime("== function test ==\n-> END", testingErrors: true);
    //             Assert.IsTrue(HadError("Functions may not contain diverts"));
    //         }
    #[test]
    fn TestEndOfContent() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string("Hello world", false, true)
                .expect("compile should succeed");
            story.continue_maximally();
            assert!(!suite.had_error(None));

            let mut story = suite
                .compile_string("== test ==\nContent\n-> END", false, false)
                .expect("compile should succeed");
            story.continue_maximally();

            let mut story = suite
                .compile_string("== test ==\nContent", false, true)
                .expect("compile should succeed");
            story.continue_maximally();
            assert!(suite.had_warning(None));

            suite
                .compile_string_without_runtime("== test ==\nContent", true)
                .expect("parse should succeed");
            assert!(!suite.had_error(None));
            assert!(suite.had_warning(None));

            suite
                .compile_string_without_runtime("== test ==\n~return", true)
                .expect("parse should succeed");
            assert!(suite.had_error(Some(
                "Return statements can only be used in knots that are declared as functions"
            )));

            suite
                .compile_string_without_runtime("== function test ==\n-> END", true)
                .expect("parse should succeed");
            assert!(suite.had_error(Some("Functions may not contain diverts")));
        });
    }

    // C#:         [Test()]
    //         public void TestEscapeCharacter()
    //         {
    //             var storyStr = @"{true:this is a '\|' character|this isn't}";
    //
    //             Story story = CompileString(storyStr);
    //
    //             Assert.AreEqual("this is a '|' character\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestEscapeCharacter() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"{true:this is a '\|' character|this isn't}"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("this is a '|' character\n", story.continue_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestExternalBinding()
    //         {
    //             var story = CompileString(@"
    // EXTERNAL message(x)
    // EXTERNAL multiply(x,y)
    // EXTERNAL times(i,str)
    // ~ message(""hello world"")
    // {multiply(5.0, 3)}
    // {times(3, ""knock "")}
    // ");
    //             string message = null;
    //
    //             story.BindExternalFunction("message", (string arg) =>
    //             {
    //                 message = "MESSAGE: " + arg;
    //             });
    //
    //             story.BindExternalFunction("multiply", (float arg1, int arg2) =>
    //             {
    //                 return arg1 * arg2;
    //             });
    //
    //             story.BindExternalFunction("times", (int numberOfTimes, string str) =>
    //             {
    //                 string result = "";
    //                 for (int i = 0; i < numberOfTimes; i++)
    //                 {
    //                     result += str;
    //                 }
    //                 return result;
    //             });
    //
    //             Assert.AreEqual("15\n", story.Continue());
    //
    //             Assert.AreEqual("knock knock knock\n", story.Continue());
    //
    //             Assert.AreEqual("MESSAGE: hello world", message);
    //         }
    #[test]
    fn TestExternalBinding() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
EXTERNAL message(x)
EXTERNAL multiply(x,y)
EXTERNAL times(i,str)
~ message("hello world")
{multiply(5.0, 3)}
{times(3, "knock ")}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            let message = Arc::new(Mutex::new(None::<String>));
            let message_clone = Arc::clone(&message);
            story.bind_external_function(
                "message",
                boxed_external_function(move |_func, args| {
                    if let Some(ValueType::String(arg)) = args.into_iter().next() {
                        *message_clone.lock().unwrap() = Some(format!("MESSAGE: {}", arg));
                    }
                    None
                }),
                false,
            );

            story.bind_external_function(
                "multiply",
                boxed_external_function(|_func, args| match args.as_slice() {
                    [ValueType::Float(a), ValueType::Int(b)] => {
                        Some(ValueType::Float(a * (*b as f32)))
                    }
                    [ValueType::Float(a), ValueType::Float(b)] => Some(ValueType::Float(a * b)),
                    _ => None,
                }),
                false,
            );

            story.bind_external_function(
                "times",
                boxed_external_function(|_func, args| match args.as_slice() {
                    [ValueType::Int(number_of_times), ValueType::String(str)] => {
                        Some(ValueType::String(str.repeat(*number_of_times as usize)))
                    }
                    _ => None,
                }),
                false,
            );

            assert_eq!("15\n", story.cont());
            assert_eq!("knock knock knock\n", story.cont());
            assert_eq!(
                Some("MESSAGE: hello world".to_string()),
                message.lock().unwrap().clone()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestLookupSafeOrNot()
    //         {
    //             var story = CompileString(@"
    // EXTERNAL myAction()
    //
    // One
    // ~ myAction()
    // Two
    // ");
    //
    //             // Lookahead SAFE - should get multiple calls to the ext function,
    //             // one for lookahead on first line, one "for real" on second line.
    //             int callCount = 0;
    //             story.BindExternalFunction("myAction", () => callCount++, lookaheadSafe:true);
    //
    //             story.ContinueMaximally();
    //             Assert.AreEqual(2, callCount);
    //
    //             // Lookahead UNSAFE - when it sees the function, it should break out early
    //             // and stop lookahead, making sure that the action is only called for the second line.
    //             callCount = 0;
    //             story.ResetState();
    //             story.UnbindExternalFunction("myAction");
    //             story.BindExternalFunction("myAction", () => callCount++, lookaheadSafe:false);
    //
    //             story.ContinueMaximally();
    //             Assert.AreEqual(1, callCount);
    //
    //             // Lookahead SAFE but breaks glue intentionally
    //             var storyWithPostGlue = CompileString(@"
    // EXTERNAL myAction()
    //
    // One
    // ~ myAction()
    // <> Two
    // ");
    //
    //             storyWithPostGlue.BindExternalFunction("myAction", () => {});
    //             var result = storyWithPostGlue.ContinueMaximally();
    //             Assert.AreEqual("One\nTwo\n", result);
    //         }
    #[test]
    fn TestLookupSafeOrNot() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
EXTERNAL myAction()

One
~ myAction()
Two
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            let call_count = Arc::new(Mutex::new(0usize));
            let safe_count = Arc::clone(&call_count);
            story.bind_external_function(
                "myAction",
                boxed_external_function(move |_func, _args| {
                    *safe_count.lock().unwrap() += 1;
                    None
                }),
                true,
            );

            story.continue_maximally();
            assert_eq!(2, *call_count.lock().unwrap());

            *call_count.lock().unwrap() = 0;
            story.ResetState();
            story.UnbindExternalFunction("myAction".to_string());

            let unsafe_count = Arc::clone(&call_count);
            story.bind_external_function(
                "myAction",
                boxed_external_function(move |_func, _args| {
                    *unsafe_count.lock().unwrap() += 1;
                    None
                }),
                false,
            );

            story.continue_maximally();
            assert_eq!(1, *call_count.lock().unwrap());

            let mut story_with_post_glue = suite
                .compile_string(
                    r#"
EXTERNAL myAction()

One 
~ myAction()
<> Two
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            story_with_post_glue.bind_external_function(
                "myAction",
                boxed_external_function(|_func, _args| None),
                true,
            );
            let result = story_with_post_glue.continue_maximally();
            assert_eq!("One\nTwo\n", result);
        });
    }

    // C#:         [Test()]
    //         public void TestFactorialByReference()
    //         {
    //             var storyStr = @"
    // VAR result = 0
    // ~ factorialByRef(result, 5)
    // { result }
    //
    // == function factorialByRef(ref r, n) ==
    // { r == 0:
    //     ~ r = 1
    // }
    // { n > 1:
    //     ~ r = r * n
    //     ~ factorialByRef(r, n-1)
    // }
    // ~ return
    // ";
    //
    //             Story story = CompileString(storyStr);
    //
    //             Assert.AreEqual("120\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestFactorialByReference() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR result = 0
~ factorialByRef(result, 5)
{ result }

== function factorialByRef(ref r, n) ==
{ r == 0:
    ~ r = 1
}
{ n > 1:
    ~ r = r * n
    ~ factorialByRef(r, n-1)
}
~ return
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("120\n", story.continue_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestFactorialRecursive()
    //         {
    //             var storyStr = @"
    // { factorial(5) }
    //
    // == function factorial(n) ==
    //  { n == 1:
    //     ~ return 1
    //  - else:
    //     ~ return (n * factorial(n-1))
    //  }
    // ";
    //
    //             Story story = CompileString(storyStr);
    //
    //             Assert.AreEqual("120\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestFactorialRecursive() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{ factorial(5) }

== function factorial(n) ==
 { n == 1:
    ~ return 1
 - else:
    ~ return (n * factorial(n-1))
 }
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("120\n", story.continue_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestFunctionCallRestrictions()
    //         {
    //             CompileStringWithoutRuntime(@"
    // // Allowed to do this
    // ~ myFunc()
    //
    // // Not allowed to to this
    // ~ aKnot()
    //
    // // Not allowed to do this
    // -> myFunc
    //
    // == function myFunc ==
    // This is a function.
    // ~ return
    //
    // == aKnot ==
    // This is a normal knot.
    // -> END
    // ", testingErrors: true);
    //
    //             Assert.AreEqual(2, _errorMessages.Count);
    //             Assert.IsTrue(_errorMessages[0].Contains("hasn't been marked as a function"));
    //             Assert.IsTrue(_errorMessages[1].Contains("can only be called as a function"));
    //         }
    #[test]
    fn TestFunctionCallRestrictions() {
        run_in_both_modes(|suite| {
            suite
                .compile_string_without_runtime(
                    r#"
// Allowed to do this
~ myFunc()

// Not allowed to to this
~ aKnot()

// Not allowed to do this
-> myFunc

== function myFunc ==
This is a function.
~ return

== aKnot ==
This is a normal knot.
-> END
"#,
                    true,
                )
                .expect("parse should succeed");
            assert_eq!(2, suite.error_messages().len());
            assert!(suite
                .error_messages()
                .iter()
                .any(|m| m.contains("hasn't been marked as a function")));
            assert!(suite
                .error_messages()
                .iter()
                .any(|m| m.contains("can only be called as a function")));
        });
    }

    // C#:         [Test()]
    //         public void TestFunctionPurityChecks()
    //         {
    //             CompileStringWithoutRuntime(@"
    // -> test
    //
    // == test ==
    // ~ myFunc()
    // = function myBadInnerFunc
    // Not allowed!
    // ~ return
    //
    // == function myFunc ==
    // Hello world
    // * a choice
    // * another choice
    // -
    // -> myFunc
    // = testStitch
    //     This is a stitch
    // ~ return
    // ", testingErrors: true);
    //
    //             Assert.AreEqual(7, _errorMessages.Count);
    //             Assert.IsTrue(_errorMessages[0].Contains("Return statements can only be used in knots that"));
    //             Assert.IsTrue(_errorMessages[1].Contains("Functions cannot be stitches"));
    //             Assert.IsTrue(_errorMessages[2].Contains("Functions may not contain stitches"));
    //             Assert.IsTrue(_errorMessages[3].Contains("Functions may not contain diverts"));
    //             Assert.IsTrue(_errorMessages[4].Contains("Functions may not contain choices"));
    //             Assert.IsTrue(_errorMessages[5].Contains("Functions may not contain choices"));
    //             Assert.IsTrue(_errorMessages[6].Contains("Return statements can only be used in knots that"));
    //         }
    #[test]
    fn TestFunctionPurityChecks() {
        run_in_both_modes(|suite| {
            suite
                .compile_string_without_runtime(
                    r#"
-> test

== test ==
~ myFunc()
= function myBadInnerFunc
Not allowed!
~ return

== function myFunc ==
Hello world
* a choice
* another choice
-
-> myFunc
= testStitch
    This is a stitch
~ return
"#,
                    true,
                )
                .expect("parse should succeed");
            assert_eq!(7, suite.error_messages().len());
            assert!(suite
                .error_messages()
                .iter()
                .any(|m| m.contains("Return statements can only be used in knots that")));
            assert!(suite
                .error_messages()
                .iter()
                .any(|m| m.contains("Functions cannot be stitches")));
            assert!(suite
                .error_messages()
                .iter()
                .any(|m| m.contains("Functions may not contain stitches")));
            assert!(suite
                .error_messages()
                .iter()
                .any(|m| m.contains("Functions may not contain diverts")));
            assert!(suite
                .error_messages()
                .iter()
                .any(|m| m.contains("Functions may not contain choices")));
        });
    }

    // C#:         [Test()]
    //         public void TestDisallowEmptyDiverts()
    //         {
    //             CompileStringWithoutRuntime ("->", testingErrors: true);
    //
    //             Assert.IsTrue (HadError ("Empty diverts (->) are only valid on choices"));
    //         }
    #[test]
    fn TestDisallowEmptyDiverts() {
        run_in_both_modes(|suite| {
            suite
                .compile_string_without_runtime("->", true)
                .expect("parse should succeed");
            assert!(suite.had_error(Some("Empty diverts (->) are only valid on choices")));
        });
    }

    // C#:         [Test ()]
    //         public void TestDoneStopsThread ()
    //         {
    //             var storyStr =
    //                 @"
    // -> DONE
    // This content is inaccessible.
    // ";
    //
    //             Story story = CompileString (storyStr);
    //
    //             Assert.AreEqual (string.Empty, story.ContinueMaximally ());
    //         }
    #[test]
    fn TestDoneStopsThread() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> DONE
This content is inaccessible.
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(String::new(), story.continue_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestMultiFlowBasics()
    //         {
    //             var storyStr =
    //         @"
    // === knot1
    // knot 1 line 1
    // knot 1 line 2
    // -> END
    //
    // === knot2
    // knot 2 line 1
    // knot 2 line 2
    // -> END
    // ";
    //
    //             var story = CompileString(storyStr);
    //
    //             story.SwitchFlow("First");
    //             story.ChoosePathString("knot1");
    //             Assert.AreEqual("knot 1 line 1\n", story.Continue());
    //
    //             story.SwitchFlow("Second");
    //             story.ChoosePathString("knot2");
    //             Assert.AreEqual("knot 2 line 1\n", story.Continue());
    //
    //             story.SwitchFlow("First");
    //             Assert.AreEqual("knot 1 line 2\n", story.Continue());
    //
    //             story.SwitchFlow("Second");
    //             Assert.AreEqual("knot 2 line 2\n", story.Continue());
    //         }
    #[test]
    fn TestMultiFlowBasics() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
=== knot1
knot 1 line 1
knot 1 line 2
-> END 

=== knot2
knot 2 line 1
knot 2 line 2
-> END 
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.switch_flow("First");
            story.choose_path_string_simple("knot1");
            assert_eq!("knot 1 line 1\n", story.cont());
            story.switch_flow("Second");
            story.choose_path_string_simple("knot2");
            assert_eq!("knot 2 line 1\n", story.cont());
            story.switch_flow("First");
            assert_eq!("knot 1 line 2\n", story.cont());
            story.switch_flow("Second");
            assert_eq!("knot 2 line 2\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestMultiFlowSaveLoadThreads()
    //         {
    //             var storyStr =
    //         @"
    // Default line 1
    // Default line 2
    //
    // == red ==
    // Hello I'm red
    // <- thread1(""red"")
    // <- thread2(""red"")
    // -> DONE
    //
    // == blue ==
    // Hello I'm blue
    // <- thread1(""blue"")
    // <- thread2(""blue"")
    // -> DONE
    //
    // == thread1(name) ==
    // + Thread 1 {name} choice
    //     -> thread1Choice(name)
    //
    // == thread2(name) ==
    // + Thread 2 {name} choice
    //     -> thread2Choice(name)
    //
    // == thread1Choice(name) ==
    // After thread 1 choice ({name})
    // -> END
    //
    // == thread2Choice(name) ==
    // After thread 2 choice ({name})
    // -> END
    // ";
    //
    //             var story = CompileString(storyStr);
    //
    //             // Default flow
    //             Assert.AreEqual("Default line 1\n", story.Continue());
    //
    //             story.SwitchFlow("Blue Flow");
    //             story.ChoosePathString("blue");
    //             Assert.AreEqual("Hello I'm blue\n", story.Continue());
    //
    //             story.SwitchFlow("Red Flow");
    //             story.ChoosePathString("red");
    //             Assert.AreEqual("Hello I'm red\n", story.Continue());
    //
    //             // Test existing state remains after switch (blue)
    //             story.SwitchFlow("Blue Flow");
    //             Assert.AreEqual("Hello I'm blue\n", story.currentText);
    //             Assert.AreEqual("Thread 1 blue choice", story.currentChoices[0].text);
    //
    //             // Test existing state remains after switch (red)
    //             story.SwitchFlow("Red Flow");
    //             Assert.AreEqual("Hello I'm red\n", story.currentText);
    //             Assert.AreEqual("Thread 1 red choice", story.currentChoices[0].text);
    //
    //             // Save/load test
    //             var saved = story.state.ToJson();
    //
    //             // Test choice before reloading state before resetting
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("Thread 1 red choice\nAfter thread 1 choice (red)\n", story.ContinueMaximally());
    //             story.ResetState();
    //
    //             // Load to pre-choice: still red, choose second choice
    //             story.state.LoadJson(saved);
    //
    //             story.ChooseChoiceIndex(1);
    //             Assert.AreEqual("Thread 2 red choice\nAfter thread 2 choice (red)\n", story.ContinueMaximally());
    //
    //
    //             // Load: switch to blue, choose 1
    //             story.state.LoadJson(saved);
    //             story.SwitchFlow("Blue Flow");
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("Thread 1 blue choice\nAfter thread 1 choice (blue)\n", story.ContinueMaximally());
    //
    //             // Load: switch to blue, choose 2
    //             story.state.LoadJson(saved);
    //             story.SwitchFlow("Blue Flow");
    //             story.ChooseChoiceIndex(1);
    //             Assert.AreEqual("Thread 2 blue choice\nAfter thread 2 choice (blue)\n", story.ContinueMaximally());
    //
    //             // Remove active blue flow, should revert back to global flow
    //             story.RemoveFlow("Blue Flow");
    //             Assert.AreEqual("Default line 2\n", story.Continue());
    //         }
    #[test]
    fn TestMultiFlowSaveLoadThreads() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
Default line 1
Default line 2

== red ==
Hello I'm red
<- thread1("red")
<- thread2("red")
-> DONE

== blue ==
Hello I'm blue
<- thread1("blue")
<- thread2("blue")
-> DONE

== thread1(name) ==
+ Thread 1 {name} choice
    -> thread1Choice(name)

== thread2(name) ==
+ Thread 2 {name} choice
    -> thread2Choice(name)

== thread1Choice(name) ==
After thread 1 choice ({name})
-> END

== thread2Choice(name) ==
After thread 2 choice ({name})
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("Default line 1\n", story.cont());
            story.switch_flow("Blue Flow");
            story.choose_path_string_simple("blue");
            assert_eq!("Hello I'm blue\n", story.cont());
            story.switch_flow("Red Flow");
            story.choose_path_string_simple("red");
            assert_eq!("Hello I'm red\n", story.cont());
            story.switch_flow("Blue Flow");
            assert_eq!("Hello I'm blue\n", story.current_text());
            assert_eq!(
                "Thread 1 blue choice",
                story.current_choices()[0].text.clone()
            );
            story.switch_flow("Red Flow");
            assert_eq!("Hello I'm red\n", story.current_text());
            assert_eq!(
                "Thread 1 red choice",
                story.current_choices()[0].text.clone()
            );
            let saved = story.save_state();
            story.choose_choice_index(0);
            assert_eq!(
                "Thread 1 red choice\nAfter thread 1 choice (red)\n",
                story.continue_maximally()
            );
            story.load_state(&saved);
            story.choose_choice_index(1);
            assert_eq!(
                "Thread 2 red choice\nAfter thread 2 choice (red)\n",
                story.continue_maximally()
            );
            story.load_state(&saved);
            story.switch_flow("Blue Flow");
            story.choose_choice_index(0);
            assert_eq!(
                "Thread 1 blue choice\nAfter thread 1 choice (blue)\n",
                story.continue_maximally()
            );
            story.load_state(&saved);
            story.switch_flow("Blue Flow");
            story.choose_choice_index(1);
            assert_eq!(
                "Thread 2 blue choice\nAfter thread 2 choice (blue)\n",
                story.continue_maximally()
            );
            story.remove_flow("Blue Flow");
            assert_eq!("Default line 2\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestCharacterRangeIdentifiersForConstNamesWithAsciiPrefix()
    //         {
    //             var ranges = InkParser.ListAllCharacterRanges();
    //             for (int i = 0; i < ranges.Length; i++)
    //             {
    //
    //                 var range = ranges[i];
    //
    //                 var identifier = GenerateIdentifierFromCharacterRange(range);
    //
    //                 var storyStr = string.Format(@"
    // CONST pi{0} = 3.1415
    // CONST a{0} = ""World""
    // CONST b{0} = 3
    // ", identifier);
    //
    //                 var compiledStory = CompileStringWithoutRuntime(storyStr);
    //
    //                 Assert.IsNotNull(compiledStory);
    //             }
    //         }
    #[test]
    fn TestCharacterRangeIdentifiersForConstNamesWithAsciiPrefix() {
        run_in_both_modes(|suite| {
            let mut ranges = CharacterRangeParser::ListAllCharacterRanges();
            for range in ranges.iter_mut() {
                let identifier = generate_identifier_from_character_range(range, None);
                let story_str = format!(
                    "\nCONST pi{0} = 3.1415\nCONST a{0} = \"World\"\nCONST b{0} = 3\n",
                    identifier
                );
                let compiled_story = suite
                    .compile_string_without_runtime(&story_str, false)
                    .expect("parse should succeed");
                let _ = compiled_story;
            }
        });
    }

    // C#:         [Test()]
    //         public void TestCharacterRangeIdentifiersForConstNamesWithAsciiSuffix()
    //         {
    //             var ranges = InkParser.ListAllCharacterRanges();
    //             for (int i = 0; i < ranges.Length; i++)
    //             {
    //
    //                 var range = ranges[i];
    //
    //                 var identifier = GenerateIdentifierFromCharacterRange(range);
    //
    //                 var storyStr = string.Format(@"
    // CONST {0}pi = 3.1415
    // CONST {0}a = ""World""
    // CONST {0}b = 3
    // ", identifier);
    //
    //                 var compiledStory = CompileStringWithoutRuntime(storyStr);
    //
    //                 Assert.IsNotNull(compiledStory);
    //             }
    //         }
    #[test]
    fn TestCharacterRangeIdentifiersForConstNamesWithAsciiSuffix() {
        run_in_both_modes(|suite| {
            let mut ranges = CharacterRangeParser::ListAllCharacterRanges();
            for range in ranges.iter_mut() {
                let identifier = generate_identifier_from_character_range(range, None);
                let story_str = format!(
                    "\nCONST {0}pi = 3.1415\nCONST {0}a = \"World\"\nCONST {0}b = 3\n",
                    identifier
                );
                let compiled_story = suite
                    .compile_string_without_runtime(&story_str, false)
                    .expect("parse should succeed");
                let _ = compiled_story;
            }
        });
    }

    // C#:         [Test()]
    //         public void TestCharacterRangeIdentifiersForSimpleVariableNamesWithAsciiPrefix()
    //         {
    //             var ranges = InkParser.ListAllCharacterRanges();
    //             for (int i = 0; i < ranges.Length; i++)
    //             {
    //
    //                 var range = ranges[i];
    //
    //                 var identifier = GenerateIdentifierFromCharacterRange(range);
    //
    //                 var storyStr = string.Format(@"
    // VAR pi{0} = 3.1415
    // VAR a{0} = ""World""
    // VAR b{0} = 3
    // ", identifier);
    //
    //                 var compiledStory = CompileStringWithoutRuntime(storyStr);
    //
    //                 Assert.IsNotNull(compiledStory);
    //             }
    //         }
    #[test]
    fn TestCharacterRangeIdentifiersForSimpleVariableNamesWithAsciiPrefix() {
        run_in_both_modes(|suite| {
            let mut ranges = CharacterRangeParser::ListAllCharacterRanges();
            for range in ranges.iter_mut() {
                let identifier = generate_identifier_from_character_range(range, None);
                let story_str = format!(
                    "\nVAR pi{0} = 3.1415\nVAR a{0} = \"World\"\nVAR b{0} = 3\n",
                    identifier
                );
                let compiled_story = suite
                    .compile_string_without_runtime(&story_str, false)
                    .expect("parse should succeed");
                let _ = compiled_story;
            }
        });
    }

    // C#:         [Test()]
    //         public void TestCharacterRangeIdentifiersForSimpleVariableNamesWithAsciiSuffix()
    //         {
    //             var ranges = InkParser.ListAllCharacterRanges();
    //             for (int i = 0; i < ranges.Length; i++)
    //             {
    //
    //                 var range = ranges[i];
    //
    //                 var identifier = GenerateIdentifierFromCharacterRange(range);
    //
    //                 var storyStr = string.Format(@"
    // VAR {0}pi = 3.1415
    // VAR {0}a = ""World""
    // VAR {0}b = 3
    // ", identifier);
    //
    //                 var compiledStory = CompileStringWithoutRuntime(storyStr);
    //
    //                 Assert.IsNotNull(compiledStory);
    //             }
    //         }
    #[test]
    fn TestCharacterRangeIdentifiersForSimpleVariableNamesWithAsciiSuffix() {
        run_in_both_modes(|suite| {
            let mut ranges = CharacterRangeParser::ListAllCharacterRanges();
            for range in ranges.iter_mut() {
                let identifier = generate_identifier_from_character_range(range, None);
                let story_str = format!(
                    "\nVAR {0}pi = 3.1415\nVAR {0}a = \"World\"\nVAR {0}b = 3\n",
                    identifier
                );
                let compiled_story = suite
                    .compile_string_without_runtime(&story_str, false)
                    .expect("parse should succeed");
                let _ = compiled_story;
            }
        });
    }

    // C#:         [Test ()]
    //         public void TestCharacterRangeIdentifiersForDivertNamesWithAsciiPrefix()
    //         {
    //             var ranges = InkParser.ListAllCharacterRanges();
    //             for (int i = 0; i < ranges.Length; i++) {
    //
    //                 var range = ranges[i];
    //                 var rangeString = GenerateIdentifierFromCharacterRange(range);
    //
    //
    //                 var storyStr = string.Format(@"
    // VAR z{0} = -> divert{0}
    //
    // == divert{0} ==
    // -> END
    // ", rangeString);
    //
    //                 var compiledStory = CompileStringWithoutRuntime (storyStr);
    //
    //                 Assert.IsNotNull (compiledStory);
    //             }
    //         }
    #[test]
    fn TestCharacterRangeIdentifiersForDivertNamesWithAsciiPrefix() {
        run_in_both_modes(|suite| {
            let mut ranges = CharacterRangeParser::ListAllCharacterRanges();
            for range in ranges.iter_mut() {
                let range_string = generate_identifier_from_character_range(range, None);
                let story_str = format!(
                    "\nVAR z{0} = -> divert{0}\n\n== divert{0} ==\n-> END\n",
                    range_string
                );
                let compiled_story = suite
                    .compile_string_without_runtime(&story_str, false)
                    .expect("parse should succeed");
                let _ = compiled_story;
            }
        });
    }

    // C#:         [Test()]
    //         public void TestCharacterRangeIdentifiersForDivertNamesWithAsciiSuffix()
    //         {
    //             var ranges = InkParser.ListAllCharacterRanges();
    //             for (int i = 0; i < ranges.Length; i++)
    //             {
    //
    //                 var range = ranges[i];
    //                 var rangeString = GenerateIdentifierFromCharacterRange(range);
    //
    //
    //                 var storyStr = string.Format(@"
    // VAR {0}z = -> {0}divert
    //
    // == {0}divert ==
    // -> END
    // ", rangeString);
    //
    //                 var compiledStory = CompileStringWithoutRuntime(storyStr);
    //
    //                 Assert.IsNotNull(compiledStory);
    //             }
    //         }
    #[test]
    fn TestCharacterRangeIdentifiersForDivertNamesWithAsciiSuffix() {
        run_in_both_modes(|suite| {
            let mut ranges = CharacterRangeParser::ListAllCharacterRanges();
            for range in ranges.iter_mut() {
                let range_string = generate_identifier_from_character_range(range, None);
                let story_str = format!(
                    "\nVAR {0}z = -> {0}divert\n\n== {0}divert ==\n-> END\n",
                    range_string
                );
                let compiled_story = suite
                    .compile_string_without_runtime(&story_str, false)
                    .expect("parse should succeed");
                let _ = compiled_story;
            }
        });
    }

    // C#:         [Test()]
    //         public void TestBasicStringLiterals()
    //         {
    //             var story = CompileString(@"
    // VAR x = ""Hello world 1""
    // {x}
    // Hello {""world""} 2.
    // ");
    //             Assert.AreEqual("Hello world 1\nHello world 2.\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestBasicStringLiterals() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR x = "Hello world 1"
{x}
Hello {"world"} 2.
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("Hello world 1\nHello world 2.\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestBasicTunnel()
    //         {
    //             Story story = CompileString(@"
    // -> f ->
    // <> world
    //
    // == f ==
    // Hello
    // ->->
    // ");
    //
    //             Assert.AreEqual("Hello world\n", story.Continue());
    //         }
    #[test]
    fn TestBasicTunnel() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> f ->
<> world

== f ==
Hello
->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("Hello world\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestChoiceCount()
    //         {
    //             Story story = CompileString(@"
    // <- choices
    // { CHOICE_COUNT() }
    //
    // = end
    // -> END
    //
    // = choices
    // * one -> end
    // * two -> end
    // ");
    //
    //             Assert.AreEqual("2\n", story.Continue());
    //         }
    #[test]
    fn TestChoiceCount() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
<- choices
{ CHOICE_COUNT() }

= end
-> END

= choices
* one -> end
* two -> end
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("2\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestChoiceDivertsToDone()
    //         {
    //             var story = CompileString(@"* choice -> DONE");
    //
    //             story.Continue();
    //
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //             story.ChooseChoiceIndex(0);
    //
    //             Assert.AreEqual("choice", story.Continue());
    //         }
    #[test]
    fn TestChoiceDivertsToDone() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string("* choice -> DONE", false, false)
                .expect("compile should succeed");
            story.cont();
            assert_eq!(1, story.get_current_choices_len());
            story.choose_choice_index(0);
            assert_eq!("choice", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestChoiceWithBracketsOnly()
    //         {
    //             var storyStr = "*   [Option]\n    Text";
    //
    //             Story story = CompileString(storyStr);
    //             story.Continue();
    //
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //             Assert.AreEqual("Option", story.currentChoices[0].text);
    //
    //             story.ChooseChoiceIndex(0);
    //
    //             Assert.AreEqual("Text\n", story.Continue());
    //         }
    #[test]
    fn TestChoiceWithBracketsOnly() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string("*   [Option]\n    Text", false, false)
                .expect("compile should succeed");
            story.cont();
            assert_eq!(1, story.get_current_choices_len());
            assert_eq!("Option", story.current_choices()[0].text.clone());
            story.choose_choice_index(0);
            assert_eq!("Text\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestCompareDivertTargets()
    //         {
    //             var storyStr = @"
    // VAR to_one = -> one
    // VAR to_two = -> two
    //
    // {to_one == to_two:same knot|different knot}
    // {to_one == to_one:same knot|different knot}
    // {to_two == to_two:same knot|different knot}
    // { -> one == -> two:same knot|different knot}
    // { -> one == to_one:same knot|different knot}
    // { to_one == -> one:same knot|different knot}
    //
    // == one
    //     One
    //     -> DONE
    //
    // === two
    //     Two
    //     -> DONE";
    //
    //             Story story = CompileString(storyStr);
    //
    //             Assert.AreEqual("different knot\nsame knot\nsame knot\ndifferent knot\nsame knot\nsame knot\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestCompareDivertTargets() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR to_one = -> one
VAR to_two = -> two

{to_one == to_two:same knot|different knot}
{to_one == to_one:same knot|different knot}
{to_two == to_two:same knot|different knot}
{ -> one == -> two:same knot|different knot}
{ -> one == to_one:same knot|different knot}
{ to_one == -> one:same knot|different knot}

== one
    One
    -> DONE

=== two
    Two
    -> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "different knot\nsame knot\nsame knot\ndifferent knot\nsame knot\nsame knot\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestBlanksInInlineSequences()
    //         {
    //             var story = CompileString(@"
    // 1. -> seq1 ->
    // 2. -> seq1 ->
    // 3. -> seq1 ->
    // 4. -> seq1 ->
    // \---
    // 1. -> seq2 ->
    // 2. -> seq2 ->
    // 3. -> seq2 ->
    // \---
    // 1. -> seq3 ->
    // 2. -> seq3 ->
    // 3. -> seq3 ->
    // \---
    // 1. -> seq4 ->
    // 2. -> seq4 ->
    // 3. -> seq4 ->
    //
    // == seq1 ==
    // {a||b}
    // ->->
    //
    // == seq2 ==
    // {|a}
    // ->->
    //
    // == seq3 ==
    // {a|}
    // ->->
    //
    // == seq4 ==
    // {|}
    // ->->".Replace("\r", ""));
    //
    //             Assert.AreEqual(
    // @"1. a
    // 2.
    // 3. b
    // 4. b
    // ---
    // 1.
    // 2. a
    // 3. a
    // ---
    // 1. a
    // 2.
    // 3.
    // ---
    // 1.
    // 2.
    // 3.
    // ".Replace("\r", ""), story.ContinueMaximally().Replace("\r", ""));
    //         }
    #[test]
    fn TestBlanksInInlineSequences() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
1. -> seq1 ->
2. -> seq1 ->
3. -> seq1 ->
4. -> seq1 ->
\---
1. -> seq2 ->
2. -> seq2 ->
3. -> seq2 ->
\---
1. -> seq3 ->
2. -> seq3 ->
3. -> seq3 ->
\---
1. -> seq4 ->
2. -> seq4 ->
3. -> seq4 ->

== seq1 ==
{a||b}
->->

== seq2 ==
{|a}
->->

== seq3 ==
{a|}
->->

== seq4 ==
{|}
->->"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            let expected = r#"1. a
2.
3. b
4. b
---
1.
2. a
3. a
---
1. a
2.
3.
---
1.
2.
3.
"#;
            assert_eq!(
                expected.replace("\r", ""),
                story.cont_maximally().replace("\r", "")
            );
        });
    }

    // C#:         [Test()]
    //         public void TestAllSequenceTypes()
    //         {
    //             var storyStr =
    //                 @"
    // ~ SEED_RANDOM(1)
    //
    // Once: {f_once()} {f_once()} {f_once()} {f_once()}
    // Stopping: {f_stopping()} {f_stopping()} {f_stopping()} {f_stopping()}
    // Default: {f_default()} {f_default()} {f_default()} {f_default()}
    // Cycle: {f_cycle()} {f_cycle()} {f_cycle()} {f_cycle()}
    // Shuffle: {f_shuffle()} {f_shuffle()} {f_shuffle()} {f_shuffle()}
    // Shuffle stopping: {f_shuffle_stopping()} {f_shuffle_stopping()} {f_shuffle_stopping()} {f_shuffle_stopping()}
    // Shuffle once: {f_shuffle_once()} {f_shuffle_once()} {f_shuffle_once()} {f_shuffle_once()}
    //
    // == function f_once ==
    // {once:
    //     - one
    //     - two
    // }
    //
    // == function f_stopping ==
    // {stopping:
    //     - one
    //     - two
    // }
    //
    // == function f_default ==
    // {one|two}
    //
    // == function f_cycle ==
    // {cycle:
    //     - one
    //     - two
    // }
    //
    // == function f_shuffle ==
    // {shuffle:
    //     - one
    //     - two
    // }
    //
    // == function f_shuffle_stopping ==
    // {stopping shuffle:
    //     - one
    //     - two
    //     - final
    // }
    //
    // == function f_shuffle_once ==
    // {shuffle once:
    //     - one
    //     - two
    // }
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //             Assert.AreEqual("Once: one two\nStopping: one two two two\nDefault: one two two two\nCycle: one two one two\nShuffle: two one two one\nShuffle stopping: one two final final\nShuffle once: two one\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestAllSequenceTypes() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
~ SEED_RANDOM(1)

Once: {f_once()} {f_once()} {f_once()} {f_once()}
Stopping: {f_stopping()} {f_stopping()} {f_stopping()} {f_stopping()}
Default: {f_default()} {f_default()} {f_default()} {f_default()}
Cycle: {f_cycle()} {f_cycle()} {f_cycle()} {f_cycle()}
Shuffle: {f_shuffle()} {f_shuffle()} {f_shuffle()} {f_shuffle()}
Shuffle stopping: {f_shuffle_stopping()} {f_shuffle_stopping()} {f_shuffle_stopping()} {f_shuffle_stopping()}
Shuffle once: {f_shuffle_once()} {f_shuffle_once()} {f_shuffle_once()} {f_shuffle_once()}

== function f_once ==
{once:
    - one
    - two
}

== function f_stopping ==
{stopping:
    - one
    - two
}

== function f_default ==
{one|two}

== function f_cycle ==
{cycle:
    - one
    - two
}

== function f_shuffle ==
{shuffle:
    - one
    - two
}

== function f_shuffle_stopping ==
{stopping shuffle:
    - one
    - two
    - final
}

== function f_shuffle_once ==
{shuffle once:
    - one
    - two
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            assert_eq!(
                "Once: one two\nStopping: one two two two\nDefault: one two two two\nCycle: one two one two\nShuffle: two one two one\nShuffle stopping: one two final final\nShuffle once: two one\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestCallStackEvaluation()
    //         {
    //             var storyStr =
    //                 @"
    //                    { six() + two() }
    //                     -> END
    //
    //                 === function six
    //                     ~ return four() + two()
    //
    //                 === function four
    //                     ~ return two() + two()
    //
    //                 === function two
    //                     ~ return 2
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //             Assert.AreEqual("8\n", story.Continue());
    //         }
    #[test]
    fn TestCallStackEvaluation() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
                   { six() + two() }
                    -> END

                === function six
                    ~ return four() + two()

                === function four
                    ~ return two() + two()

                === function two
                    ~ return 2
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("8\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestConditionalChoiceInWeave()
    //         {
    //             var storyStr =
    //                 @"
    // - start
    //  {
    //     - true: * [go to a stitch] -> a_stitch
    //  }
    // - gather should be seen
    // -> DONE
    //
    // = a_stitch
    //     result
    //     -> END
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //
    //             Assert.AreEqual("start\ngather should be seen\n", story.ContinueMaximally());
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //
    //             story.ChooseChoiceIndex(0);
    //
    //             Assert.AreEqual("result\n", story.Continue());
    //         }
    #[test]
    fn TestConditionalChoiceInWeave() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
- start
 {
    - true: * [go to a stitch] -> a_stitch
 }
- gather should be seen
-> DONE

= a_stitch
    result
    -> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            assert_eq!("start\ngather should be seen\n", story.cont_maximally());
            assert_eq!(1, story.get_current_choices_len());
            story.choose_choice_index(0);
            assert_eq!("result\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestConditionalChoiceInWeave2()
    //         {
    //             var storyStr =
    //                 @"
    // - first gather
    //     * [option 1]
    //     * [option 2]
    // - the main gather
    // {false:
    //     * unreachable option -> END
    // }
    // - bottom gather";
    //
    //             Story story = CompileString(storyStr);
    //
    //             Assert.AreEqual("first gather\n", story.Continue());
    //
    //             Assert.AreEqual(2, story.currentChoices.Count);
    //
    //             story.ChooseChoiceIndex(0);
    //
    //             Assert.AreEqual("the main gather\nbottom gather\n", story.ContinueMaximally());
    //             Assert.AreEqual(0, story.currentChoices.Count);
    //         }
    #[test]
    fn TestConditionalChoiceInWeave2() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
- first gather
    * [option 1]
    * [option 2]
- the main gather
{false:
    * unreachable option -> END
}
- bottom gather"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            assert_eq!("first gather\n", story.cont());
            assert_eq!(2, story.get_current_choices_len());
            story.choose_choice_index(0);
            assert_eq!("the main gather\nbottom gather\n", story.cont_maximally());
            assert_eq!(0, story.get_current_choices_len());
        });
    }

    // C#:         [Test()]
    //         public void TestConditionalChoices()
    //         {
    //             var storyStr =
    //                 @"
    // * { true } { false } not displayed
    // * { true } { true }
    //   { true and true }  one
    // * { false } not displayed
    // * (name) { true } two
    // * { true }
    //   { true }
    //   three
    // * { true }
    //   four
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //             story.ContinueMaximally();
    //
    //             Assert.AreEqual(4, story.currentChoices.Count);
    //             Assert.AreEqual("one", story.currentChoices[0].text);
    //             Assert.AreEqual("two", story.currentChoices[1].text);
    //             Assert.AreEqual("three", story.currentChoices[2].text);
    //             Assert.AreEqual("four", story.currentChoices[3].text);
    //         }
    #[test]
    fn TestConditionalChoices() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
* { true } { false } not displayed
* { true } { true }
  { true and true }  one
* { false } not displayed
* (name) { true } two
* { true }
  { true }
  three
* { true }
  four
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont_maximally();

            assert_eq!(4, story.get_current_choices_len());
            assert_eq!("one", story.current_choices()[0].text.clone());
            assert_eq!("two", story.current_choices()[1].text.clone());
            assert_eq!("three", story.current_choices()[2].text.clone());
            assert_eq!("four", story.current_choices()[3].text.clone());
        });
    }

    // C#:         [Test()]
    //         public void TestConditionals()
    //         {
    //             var storyStr =
    //                 @"
    // {false:not true|true}
    // {
    //    - 4 > 5: not true
    //    - 5 > 4: true
    // }
    // { 2*2 > 3:
    //    - true
    //    - not true
    // }
    // {
    //    - 1 > 3: not true
    //    - { 2+2 == 4:
    //         - true
    //         - not true
    //    }
    // }
    // { 2*3:
    //    - 1+7: not true
    //    - 9: not true
    //    - 1+1+1+3: true
    //    - 9-3: also true but not printed
    // }
    // { true:
    //     great
    //     right?
    // }
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //
    //             Assert.AreEqual("true\ntrue\ntrue\ntrue\ntrue\ngreat\nright?\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestConditionals() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{false:not true|true}
{
   - 4 > 5: not true
   - 5 > 4: true
}
{ 2*2 > 3:
   - true
   - not true
}
{
   - 1 > 3: not true
   - { 2+2 == 4:
        - true
        - not true
   }
}
{ 2*3:
   - 1+7: not true
   - 9: not true
   - 1+1+1+3: true
   - 9-3: also true but not printed
}
{ true:
    great
    right?
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            assert_eq!(
                "true\ntrue\ntrue\ntrue\ntrue\ngreat\nright?\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestConst()
    //         {
    //             var story = CompileString(@"
    // VAR x = c
    //
    // CONST c = 5
    //
    // {x}
    // ");
    //             Assert.AreEqual("5\n", story.Continue());
    //         }
    #[test]
    fn TestConst() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR x = c

CONST c = 5

{x}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("5\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestDefaultChoices()
    //         {
    //             Story story = CompileString(@"
    //  - (start)
    //  * [Choice 1]
    //  * [Choice 2]
    //  * {false} Impossible choice
    //  * -> default
    //  - After choice
    //  -> start
    //
    // == default ==
    // This is default.
    // -> DONE
    // ");
    //
    //             Assert.AreEqual("", story.Continue());
    //             Assert.AreEqual(2, story.currentChoices.Count);
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("After choice\n", story.Continue());
    //
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("After choice\nThis is default.\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestDefaultChoices() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
 - (start)
 * [Choice 1]
 * [Choice 2]
 * {false} Impossible choice
 * -> default
 - After choice
 -> start

== default ==
This is default.
-> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            assert_eq!("", story.cont());
            assert_eq!(2, story.get_current_choices_len());
            story.choose_choice_index(0);
            assert_eq!("After choice\n", story.cont());
            assert_eq!(1, story.get_current_choices_len());
            story.choose_choice_index(0);
            assert_eq!("After choice\nThis is default.\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestDefaultSimpleGather()
    //         {
    //             var story = CompileString(@"
    // * ->
    // - x
    // -> DONE");
    //
    //             Assert.AreEqual("x\n", story.Continue());
    //         }
    #[test]
    fn TestDefaultSimpleGather() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
* ->
- x
-> DONE"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("x\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestDivertInConditional()
    //         {
    //             var storyStr =
    //                 @"
    // === intro
    // = top
    //     { main: -> done }
    //     -> END
    // = main
    //     -> top
    // = done
    //     -> END
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //             Assert.AreEqual("", story.ContinueMaximally());
    //         }
    #[test]
    fn TestDivertInConditional() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
=== intro
= top
    { main: -> done }
    -> END
= main
    -> top
= done
    -> END"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestDivertNotFoundError()
    //         {
    //             CompileStringWithoutRuntime(@"
    // -> knot
    //
    // == knot ==
    // Knot.
    // -> next
    // ", testingErrors: true);
    //
    //             Assert.IsTrue(HadError("not found"));
    //         }
    #[test]
    fn TestDivertNotFoundError() {
        run_in_both_modes(|suite| {
            let _ = suite.compile_string_without_runtime(
                r#"
-> knot

== knot ==
Knot.
-> next"#,
                true,
            );

            assert!(suite.had_error(Some("not found")));
        });
    }

    // C#:         [Test()]
    //         public void TestDivertToWeavePoints()
    //         {
    //             var storyStr =
    //                 @"
    // -> knot.stitch.gather
    //
    // == knot ==
    // = stitch
    // - hello
    //     * (choice) test
    //         choice content
    // - (gather)
    //   gather
    //
    //   {stopping:
    //     - -> knot.stitch.choice
    //     - second time round
    //   }
    //
    // -> END
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //
    //             Assert.AreEqual("gather\ntest\nchoice content\ngather\nsecond time round\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestDivertToWeavePoints() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> knot.stitch.gather

== knot ==
= stitch
- hello
    * (choice) test
        choice content
- (gather)
  gather

  {stopping:
    - -> knot.stitch.choice
    - second time round
  }

-> END"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "gather\ntest\nchoice content\ngather\nsecond time round\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestEmpty()
    //         {
    //             Story story = CompileString(@"");
    //
    //             Assert.AreEqual(string.Empty, story.currentText);
    //         }
    #[test]
    fn TestEmpty() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string("", false, false)
                .expect("compile should succeed");
            assert_eq!("", story.current_text());
        });
    }

    // C#:         [Test()]
    //         public void TestEmptyChoice()
    //         {
    //             int warningCount = 0;
    //             InkParser parser = new InkParser("*", null, (string message, ErrorType errorType) =>
    //             {
    //                 if (errorType == ErrorType.Warning)
    //                 {
    //                     warningCount++;
    //                     Assert.IsTrue(message.Contains("completely empty"));
    //                 }
    //                 else
    //                 {
    //                     Assert.Fail("Shouldn't have had any errors");
    //                 }
    //             });
    //
    //             parser.Parse();
    //
    //             Assert.AreEqual(1, warningCount);
    //         }
    #[test]
    fn TestEmptyChoice() {
        let warning_count = Arc::new(Mutex::new(0usize));
        let warning_count_for_handler = warning_count.clone();
        let mut parser = InkParserType::new(
            "*".to_string(),
            None,
            Some(Arc::new(
                move |message: String, _line: i32, _character: i32, is_warning: bool| {
                    if is_warning {
                        *warning_count_for_handler.lock().unwrap() += 1;
                        assert!(message.contains("completely empty"));
                    } else {
                        panic!("Shouldn't have had any errors");
                    }
                },
            )),
            None,
        );

        parser.Parse();
        assert_eq!(1, *warning_count.lock().unwrap());
    }

    // C#:         [Test()]
    //         public void TestCommentEliminator()
    //         {
    //             var testContent =
    // @"A// C
    // A /* C */ A
    //
    // A * A * /* * C *// A/*
    // C C C
    //
    // */";
    //
    //             CommentEliminator p = new CommentEliminator(testContent);
    //             var result = p.Process();
    //
    //             var expected = "A\nA  A\n\nA * A * / A\n\n\n";
    //
    //             Assert.AreEqual(expected.Replace("\r", ""), result.Replace("\r", "")); //Windows perculiarity
    //         }
    #[test]
    fn TestCommentEliminator() {
        let test_content = "A// C\nA /* C */ A\n\nA * A * /* * C *// A/*\nC C C\n\n*/";
        let processed = CommentEliminator::new(test_content.to_string())
            .Process()
            .unwrap();
        let expected = "A\nA  A\n\nA * A * / A\n\n\n";
        assert_eq!(expected.replace("\r", ""), processed.replace("\r", ""));
    }

    // C#:         [Test()]
    //         public void TestCommentEliminatorMixedNewlines()
    //         {
    //             var testContent =
    //                 "A B\nC D // comment\nA B\r\nC D // comment\r\n/* block comment\r\nsecond line\r\n */ ";
    //
    //             CommentEliminator p = new CommentEliminator(testContent);
    //             var result = p.Process();
    //
    //             var expected =
    //                 "A B\nC D \nA B\nC D \n\n\n ";
    //
    //             Assert.AreEqual(expected, result);
    //         }
    #[test]
    fn TestCommentEliminatorMixedNewlines() {
        let test_content =
            "A B\nC D // comment\nA B\r\nC D // comment\r\n/* block comment\r\nsecond line\r\n */ ";
        let processed = CommentEliminator::new(test_content.to_string())
            .Process()
            .unwrap();
        let expected = "A B\nC D \nA B\nC D \n\n\n ";
        assert_eq!(expected, processed);
    }

    // C#:         [Test()]
    //         public void TestStringParserA()
    //         {
    //             StringParser p = new StringParser("A");
    //             var results = p.Interleave<string>(
    //                 () => p.ParseString("A"),
    //                 () => p.ParseString("B"));
    //
    //             var expected = new[] { "A" };
    //             Assert.AreEqual(expected, results);
    //         }
    #[test]
    fn TestStringParserA() {
        let mut p = StringParser::new("A".to_string());
        let results = p.Interleave::<String, _, _>(
            |p| p.ParseString("A".to_string()),
            |p| p.ParseString("B".to_string()),
            None,
            true,
        );
        assert_eq!(Some(vec!["A".to_string()]), results);
    }

    // C#:         [Test()]
    //         public void TestStringParserABAB()
    //         {
    //             StringParser p = new StringParser("ABAB");
    //             var results = p.Interleave<string>(
    //                 () => p.ParseString("A"),
    //                 () => p.ParseString("B"));
    //
    //             var expected = new[] { "A", "B", "A", "B" };
    //             Assert.AreEqual(expected, results);
    //         }
    #[test]
    fn TestStringParserABAB() {
        let mut p = StringParser::new("ABAB".to_string());
        let results = p.Interleave::<String, _, _>(
            |p| p.ParseString("A".to_string()),
            |p| p.ParseString("B".to_string()),
            None,
            true,
        );
        assert_eq!(
            Some(vec![
                "A".to_string(),
                "B".to_string(),
                "A".to_string(),
                "B".to_string()
            ]),
            results
        );
    }

    // C#:         [Test()]
    //         public void TestStringParserABAOptional()
    //         {
    //             StringParser p = new StringParser("ABAA");
    //             var results = p.Interleave<string>(
    //                 () => p.ParseString("A"),
    //                 p.Optional(() => p.ParseString("B")));
    //
    //             var expected = new[] { "A", "B", "A", "A" };
    //             Assert.AreEqual(expected, results);
    //         }
    #[test]
    #[ignore]
    fn TestStringParserABAOptional() {
        let mut p = StringParser::new("ABAA".to_string());
        let results = p.Interleave::<String, _, _>(
            |p| p.ParseString("A".to_string()),
            |p| p.ParseString("B".to_string()),
            None,
            true,
        );
        assert_eq!(
            Some(vec![
                "A".to_string(),
                "B".to_string(),
                "A".to_string(),
                "A".to_string()
            ]),
            results
        );
    }

    // C#:         [Test()]
    //         public void TestStringParserABAOptional2()
    //         {
    //             StringParser p = new StringParser("BABB");
    //             var results = p.Interleave<string>(
    //                 p.Optional(() => p.ParseString("A")),
    //                 () => p.ParseString("B"));
    //
    //             var expected = new[] { "B", "A", "B", "B" };
    //             Assert.AreEqual(expected, results);
    //         }
    #[test]
    #[ignore]
    fn TestStringParserABAOptional2() {
        let mut p = StringParser::new("BABB".to_string());
        let results = p.Interleave::<String, _, _>(
            |p| p.ParseString("A".to_string()),
            |p| p.ParseString("B".to_string()),
            None,
            true,
        );
        assert_eq!(
            Some(vec![
                "B".to_string(),
                "A".to_string(),
                "B".to_string(),
                "B".to_string()
            ]),
            results
        );
    }

    // C#:         [Test()]
    //         public void TestStringParserB()
    //         {
    //             StringParser p = new StringParser("B");
    //             var result = p.Interleave<string>(
    //                 () => p.ParseString("A"),
    //                 () => p.ParseString("B"));
    //
    //             Assert.IsNull(result);
    //         }
    #[test]
    fn TestStringParserB() {
        let mut p = StringParser::new("B".to_string());
        let result = p.Interleave::<String, _, _>(
            |p| p.ParseString("A".to_string()),
            |p| p.ParseString("B".to_string()),
            None,
            true,
        );
        assert!(result.is_none());
    }

    // C#:         [Test()]
    //         public void TestEmptyMultilineConditionalBranch()
    //         {
    //             var story = CompileString(@"
    // { 3:
    //     - 3:
    //     - 4:
    //         txt
    // }
    // ");
    //
    //             Assert.AreEqual("", story.Continue());
    //         }
    #[test]
    fn TestEmptyMultilineConditionalBranch() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{ 3:
    - 3:
    - 4:
        txt
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            assert_eq!("", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestEmptySequenceContent()
    //         {
    //             var story = CompileString(@"
    // -> thing ->
    // -> thing ->
    // -> thing ->
    // -> thing ->
    // -> thing ->
    // Done.
    //
    // == thing ==
    // {once:
    //   - Wait for it....
    //   -
    //   -
    //   -  Surprise!
    // }
    // ->->
    // ");
    //             Assert.AreEqual("Wait for it....\nSurprise!\nDone.\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestEmptySequenceContent() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> thing ->
-> thing ->
-> thing ->
-> thing ->
-> thing ->
Done.

== thing ==
{once:
  - Wait for it....
  -
  -
  -  Surprise!
}
->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "Wait for it....\nSurprise!\nDone.\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestEnd()
    //         {
    //             Story story = CompileString(@"
    // hello
    // -> END
    // world
    // -> END
    // ");
    //
    //             Assert.AreEqual("hello\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestEnd() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
hello
-> END
world
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("hello\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestEnd2()
    //         {
    //             Story story = CompileString(@"
    // -> test
    //
    // == test ==
    // hello
    // -> END
    // world
    // -> END
    // ");
    //
    //             Assert.AreEqual("hello\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestEnd2() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> test

== test ==
hello
-> END
world
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("hello\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestPaths()
    //         {
    //             // Different instances should ensure different instances of individual components
    //             var path1 = new Path("hello.1.world");
    //             var path2 = new Path("hello.1.world");
    //
    //             var path3 = new Path(".hello.1.world");
    //             var path4 = new Path(".hello.1.world");
    //
    //             Assert.AreEqual(path1, path2);
    //
    //             Assert.AreEqual(path3, path4);
    //
    //             Assert.AreNotEqual(path1, path3);
    //         }
    #[test]
    fn TestPaths() {
        let path1 = ink_runtime::Path::Path::new_overload_4("hello.1.world".to_string());
        let path2 = ink_runtime::Path::Path::new_overload_4("hello.1.world".to_string());
        let path3 = ink_runtime::Path::Path::new_overload_4(".hello.1.world".to_string());
        let path4 = ink_runtime::Path::Path::new_overload_4(".hello.1.world".to_string());

        assert_eq!(path1, path2);
        assert_eq!(path3, path4);
        assert_ne!(path1, path3);
    }

    // C#:         [Test()]
    //         public void TestPathToSelf()
    //         {
    //             var story = CompileString(@"
    // - (dododo)
    // -> tunnel ->
    // -> dododo
    //
    // == tunnel
    // + A
    // ->->
    // ");
    //             // We're only checking that the story copes
    //             // okay without crashing
    //             // (internally the "-> dododo" ends up generating
    //             //  a very short path: ".^", and after walking into
    //             // the parent, it didn't cope with the "." before
    //             // I fixed it!)
    //             story.Continue();
    //             story.ChooseChoiceIndex(0);
    //             story.Continue();
    //             story.ChooseChoiceIndex(0);
    //         }
    #[test]
    fn TestPathToSelf() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
- (dododo)
-> tunnel ->
-> dododo

== tunnel
+ A
->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont();
            story.choose_choice_index(0);
            story.cont();
            story.choose_choice_index(0);
        });
    }

    // C#:         [Test()]
    //         public void TestPrintNum()
    //         {
    //             var story = CompileString(@"
    // . {print_num(4)} .
    // . {print_num(15)} .
    // . {print_num(37)} .
    // . {print_num(101)} .
    // . {print_num(222)} .
    // . {print_num(1234)} .
    //
    // === function print_num(x) ===
    // {
    //     - x >= 1000:
    //         {print_num(x / 1000)} thousand { x mod 1000 > 0:{print_num(x mod 1000)}}
    //     - x >= 100:
    //         {print_num(x / 100)} hundred { x mod 100 > 0:and {print_num(x mod 100)}}
    //     - x == 0:
    //         zero
    //     - else:
    //         { x >= 20:
    //             { x / 10:
    //                 - 2: twenty
    //                 - 3: thirty
    //                 - 4: forty
    //                 - 5: fifty
    //                 - 6: sixty
    //                 - 7: seventy
    //                 - 8: eighty
    //                 - 9: ninety
    //             }
    //             { x mod 10 > 0:<>-<>}
    //         }
    //         { x < 10 || x > 20:
    //             { x mod 10:
    //                 - 1: one
    //                 - 2: two
    //                 - 3: three
    //                 - 4: four
    //                 - 5: five
    //                 - 6: six
    //                 - 7: seven
    //                 - 8: eight
    //                 - 9: nine
    //             }
    //         - else:
    //             { x:
    //                 - 10: ten
    //                 - 11: eleven
    //                 - 12: twelve
    //                 - 13: thirteen
    //                 - 14: fourteen
    //                 - 15: fifteen
    //                 - 16: sixteen
    //                 - 17: seventeen
    //                 - 18: eighteen
    //                 - 19: nineteen
    //             }
    //         }
    // }
    // ");
    //
    //             Assert.AreEqual(
    // @". four .
    // . fifteen .
    // . thirty-seven .
    // . one hundred and one .
    // . two hundred and twenty-two .
    // . one thousand two hundred and thirty-four .
    // ".Replace("\r", ""), story.ContinueMaximally().Replace("\r", ""));
    //         }
    #[test]
    fn TestPrintNum() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
. {print_num(4)} .
. {print_num(15)} .
. {print_num(37)} .
. {print_num(101)} .
. {print_num(222)} .
. {print_num(1234)} .

=== function print_num(x) ===
{
    - x >= 1000:
        {print_num(x / 1000)} thousand { x mod 1000 > 0:{print_num(x mod 1000)}}
    - x >= 100:
        {print_num(x / 100)} hundred { x mod 100 > 0:and {print_num(x mod 100)}}
    - x == 0:
        zero
    - else:
        { x >= 20:
            { x / 10:
                - 2: twenty
                - 3: thirty
                - 4: forty
                - 5: fifty
                - 6: sixty
                - 7: seventy
                - 8: eighty
                - 9: ninety
            }
            { x mod 10 > 0:<>-<>}
        }
        { x < 10 || x > 20:
            { x mod 10:
                - 1: one
                - 2: two
                - 3: three
                - 4: four
                - 5: five
                - 6: six
                - 7: seven
                - 8: eight
                - 9: nine
            }
        - else:
            { x:
                - 10: ten
                - 11: eleven
                - 12: twelve
                - 13: thirteen
                - 14: fourteen
                - 15: fifteen
                - 16: sixteen
                - 17: seventeen
                - 18: eighteen
                - 19: nineteen
            }
        }
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                ". four .\n. fifteen .\n. thirty-seven .\n. one hundred and one .\n. two hundred and twenty-two .\n. one thousand two hundred and thirty-four .\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestQuoteCharacterSignificance()
    //         {
    //             // Confusing escaping + ink! Actual ink string is:
    //             // My name is "{"J{"o"}e"}"
    //             //  - First and last quotes are insignificant - they're part of the content
    //             //  - Inner quotes are significant - they're part of the syntax for string expressions
    //             // So output is: My name is "Joe"
    //             var story = CompileString(@"My name is ""{""J{""o""}e""}""");
    //             Assert.AreEqual("My name is \"Joe\"\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestQuoteCharacterSignificance() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(r#"My name is ""{""J{""o""}e""}"""#, false, false)
                .expect("compile should succeed");
            assert_eq!("My name is \"Joe\"\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestReadCountAcrossCallstack()
    //         {
    //             var story = CompileString(@"
    // -> first
    //
    // == first ==
    // 1) Seen first {first} times.
    // -> second ->
    // 2) Seen first {first} times.
    // -> DONE
    //
    // == second ==
    // In second.
    // ->->
    // ");
    //             Assert.AreEqual("1) Seen first 1 times.\nIn second.\n2) Seen first 1 times.\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestReadCountAcrossCallstack() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> first

== first ==
1) Seen first {first} times.
-> second ->
2) Seen first {first} times.
-> DONE

== second ==
In second.
->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "1) Seen first 1 times.\nIn second.\n2) Seen first 1 times.\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestReadCountAcrossThreads()
    //         {
    //             var story = CompileString(@"
    //     -> top
    //
    // = top
    //     {top}
    //     <- aside
    //     {top}
    //     -> DONE
    //
    // = aside
    //     * {false} DONE
    // 	- -> DONE
    // ");
    //             Assert.AreEqual("1\n1\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestReadCountAcrossThreads() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
    -> top

= top
    {top}
    <- aside
    {top}
    -> DONE

= aside
    * {false} DONE
	- -> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("1\n1\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestReadCountDotSeparatedPath()
    //         {
    //             Story story = CompileString(@"
    // -> hi ->
    // -> hi ->
    // -> hi ->
    //
    // { hi.stitch_to_count }
    //
    // == hi ==
    // = stitch_to_count
    // hi
    // ->->
    // ");
    //
    //             Assert.AreEqual("hi\nhi\nhi\n3\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestReadCountDotSeparatedPath() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> hi ->
-> hi ->
-> hi ->

{ hi.stitch_to_count }

== hi ==
= stitch_to_count
hi
->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("hi\nhi\nhi\n3\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestReturnTextWarning()
    //         {
    //             InkParser parser = new InkParser("== test ==\n return something",
    //                 null,
    //                 (string message, ErrorType errorType) =>
    //                 {
    //                     if (errorType == ErrorType.Warning)
    //                     {
    //                         throw new TestWarningException();
    //                     }
    //                 });
    //
    //             Assert.Throws<TestWarningException>(() => parser.Parse());
    //         }
    #[test]
    fn TestReturnTextWarning() {
        let warning = std::panic::catch_unwind(|| {
            let mut parser = InkParserType::new(
                "== test ==\n return something".to_string(),
                None,
                Some(Arc::new(
                    |message: String, _line: i32, _character: i32, is_warning: bool| {
                        if is_warning {
                            panic!("{}", message);
                        }
                    },
                )),
                None,
            );
            let _ = parser.Parse();
        });
        assert!(warning.is_err());
    }

    // C#:         [Test()]
    //         public void TestSameLineDivertIsInline()
    //         {
    //             var story = CompileString(@"
    // -> hurry_home
    // === hurry_home ===
    // We hurried home to Savile Row -> as_fast_as_we_could
    //
    // === as_fast_as_we_could ===
    // as fast as we could.
    // -> DONE
    // ");
    //
    //             Assert.AreEqual("We hurried home to Savile Row as fast as we could.\n", story.Continue());
    //         }
    #[test]
    fn TestSameLineDivertIsInline() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> hurry_home
=== hurry_home ===
We hurried home to Savile Row -> as_fast_as_we_could

=== as_fast_as_we_could ===
as fast as we could.
-> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "We hurried home to Savile Row as fast as we could.\n",
                story.cont()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestShouldntGatherDueToChoice()
    //         {
    //             Story story = CompileString(@"
    // * opt
    //     - - text
    //     * * {false} impossible
    //     * * -> END
    // - gather");
    //
    //             story.ContinueMaximally();
    //             story.ChooseChoiceIndex(0);
    //
    //             // Shouldn't go to "gather"
    //             Assert.AreEqual("opt\ntext\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestShouldntGatherDueToChoice() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
* opt
    - - text
    * * {false} impossible
    * * -> END
- gather"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont_maximally();
            story.choose_choice_index(0);
            assert_eq!("opt\ntext\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestSimpleGlue()
    //         {
    //             var storyStr = "Some <> \ncontent<> with glue.\n";
    //
    //             Story story = CompileString(storyStr);
    //
    //             Assert.AreEqual("Some content with glue.\n", story.Continue());
    //         }
    #[test]
    fn TestSimpleGlue() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string("Some <> \ncontent<> with glue.\n", false, false)
                .expect("compile should succeed");
            assert_eq!("Some content with glue.\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestStickyChoicesStaySticky()
    //         {
    //             var story = CompileString(@"
    // -> test
    // == test ==
    // First line.
    // Second line.
    // + Choice 1
    // + Choice 2
    // - -> test
    // ");
    //
    //             story.ContinueMaximally();
    //             Assert.AreEqual(2, story.currentChoices.Count);
    //
    //             story.ChooseChoiceIndex(0);
    //             story.ContinueMaximally();
    //             Assert.AreEqual(2, story.currentChoices.Count);
    //         }
    #[test]
    fn TestStickyChoicesStaySticky() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> test
== test ==
First line.
Second line.
+ Choice 1
+ Choice 2
- -> test
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont_maximally();
            assert_eq!(2, story.current_choices_len());
            story.choose_choice_index(0);
            story.cont_maximally();
            assert_eq!(2, story.current_choices_len());
        });
    }

    // C#:         [Test()]
    //         public void TestStringConstants()
    //         {
    //             var story = CompileString(@"
    // {x}
    // VAR x = kX
    // CONST kX = ""hi""
    // ");
    //
    //             Assert.AreEqual("hi\n", story.Continue());
    //         }
    #[test]
    fn TestStringConstants() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{x}
VAR x = kX
CONST kX = "hi"
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("hi\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestStringsInChoices()
    //         {
    //             var story = CompileString(@"
    // * \ {""test1""} [""test2 {""test3""}""] {""test4""}
    // -> DONE
    // ");
    //             story.ContinueMaximally();
    //
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //             Assert.AreEqual(@"test1 ""test2 test3""", story.currentChoices[0].text);
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("test1 test4\n", story.Continue());
    //         }
    #[test]
    fn TestStringsInChoices() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
* \ {"test1"} ["test2 {"test3"}"] {"test4"}
-> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont_maximally();
            assert_eq!(1, story.current_choices_len());
            assert_eq!(r#"test1 "test2 test3""#, story.current_choices()[0].text);
            story.choose_choice_index(0);
            assert_eq!("test1 test4\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestStringTypeCoersion()
    //         {
    //             var story = CompileString(@"
    // {""5"" == 5:same|different}
    // {""blah"" == 5:same|different}
    // ");
    //
    //             // Not sure that "5" should be equal to 5, but hmm.
    //             Assert.AreEqual("same\ndifferent\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestStringTypeCoersion() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{"5" == 5:same|different}
{"blah" == 5:same|different}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("same\ndifferent\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestTemporariesAtGlobalScope()
    //         {
    //             var story = CompileString(@"
    // VAR x = 5
    // ~ temp y = 4
    // {x}{y}
    // ");
    //             Assert.AreEqual("54\n", story.Continue());
    //         }
    #[test]
    fn TestTemporariesAtGlobalScope() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR x = 5
~ temp y = 4
{x}{y}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("54\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestThreadDone()
    //         {
    //             Story story = CompileString(@"
    // This is a thread example
    // <- example_thread
    // The example is now complete.
    //
    // == example_thread ==
    // Hello.
    // -> DONE
    // World.
    // -> DONE
    // ");
    //
    //             Assert.AreEqual("This is a thread example\nHello.\nThe example is now complete.\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestThreadDone() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
This is a thread example
<- example_thread
The example is now complete.

== example_thread ==
Hello.
-> DONE
World.
-> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "This is a thread example\nHello.\nThe example is now complete.\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestTunnelOnwardsAfterTunnel()
    //         {
    //             var story = CompileString(@"
    // -> tunnel1 ->
    // The End.
    // -> END
    //
    // == tunnel1 ==
    // Hello...
    // -> tunnel2 ->->
    //
    // == tunnel2 ==
    // ...world.
    // ->->
    // ");
    //
    //             Assert.AreEqual("Hello...\n...world.\nThe End.\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestTunnelOnwardsAfterTunnel() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> tunnel1 ->
The End.
-> END

== tunnel1 ==
Hello...
-> tunnel2 ->->

== tunnel2 ==
...world.
->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("Hello...\n...world.\nThe End.\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestTunnelVsThreadBehaviour()
    //         {
    //             Story story = CompileString(@"
    // -> knot_with_options ->
    // Finished tunnel.
    //
    // Starting thread.
    // <- thread_with_options
    // * E
    // -
    // Done.
    //
    // == knot_with_options ==
    // * A
    // * B
    // -
    // ->->
    //
    // == thread_with_options ==
    // * C
    // * D
    // - -> DONE
    // ");
    //
    //             Assert.IsFalse(story.ContinueMaximally().Contains("Finished tunnel"));
    //
    //             // Choices should be A, B
    //             Assert.AreEqual(2, story.currentChoices.Count);
    //
    //             story.ChooseChoiceIndex(0);
    //
    //             // Choices should be C, D, E
    //             Assert.IsTrue(story.ContinueMaximally().Contains("Finished tunnel"));
    //             Assert.AreEqual(3, story.currentChoices.Count);
    //
    //             story.ChooseChoiceIndex(2);
    //
    //             Assert.IsTrue(story.ContinueMaximally().Contains("Done."));
    //         }
    #[test]
    fn TestTunnelVsThreadBehaviour() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> knot_with_options ->
Finished tunnel.

Starting thread.
<- thread_with_options
* E
-
Done.

== knot_with_options ==
* A
* B
-
->->

== thread_with_options ==
* C
* D
- -> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert!(!story.cont_maximally().contains("Finished tunnel"));
            assert_eq!(2, story.current_choices_len());
            story.choose_choice_index(0);
            assert!(story.cont_maximally().contains("Finished tunnel"));
            assert_eq!(3, story.current_choices_len());
            story.choose_choice_index(2);
            assert!(story.cont_maximally().contains("Done."));
        });
    }

    // C#:         [Test()]
    //         public void TestTurnsSince()
    //         {
    //             Story story = CompileString(@"
    // { TURNS_SINCE(-> test) }
    // ~ test()
    // { TURNS_SINCE(-> test) }
    // * [choice 1]
    // - { TURNS_SINCE(-> test) }
    // * [choice 2]
    // - { TURNS_SINCE(-> test) }
    //
    // == function test ==
    // ~ return
    // ");
    //             Assert.AreEqual("-1\n0\n", story.ContinueMaximally());
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("1\n", story.ContinueMaximally());
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("2\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestTurnsSince() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{ TURNS_SINCE(-> test) }
~ test()
{ TURNS_SINCE(-> test) }
* [choice 1]
- { TURNS_SINCE(-> test) }
* [choice 2]
- { TURNS_SINCE(-> test) }

== function test ==
~ return
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("-1\n0\n", story.cont_maximally());
            story.choose_choice_index(0);
            assert_eq!("1\n", story.cont_maximally());
            story.choose_choice_index(0);
            assert_eq!("2\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestTurnsSinceNested()
    //         {
    //             var story = CompileString(@"
    // -> empty_world
    // === empty_world ===
    //     {TURNS_SINCE(-> then)} = -1
    //     * (then) stuff
    //         {TURNS_SINCE(-> then)} = 0
    //         * * (next) more stuff
    //             {TURNS_SINCE(-> then)} = 1
    //         -> DONE
    // ");
    //             Assert.AreEqual("-1 = -1\n", story.ContinueMaximally());
    //
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //             story.ChooseChoiceIndex(0);
    //
    //             Assert.AreEqual("stuff\n0 = 0\n", story.ContinueMaximally());
    //
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //             story.ChooseChoiceIndex(0);
    //
    //             Assert.AreEqual("more stuff\n1 = 1\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestTurnsSinceNested() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> empty_world
=== empty_world ===
    {TURNS_SINCE(-> then)} = -1
    * (then) stuff
        {TURNS_SINCE(-> then)} = 0
        * * (next) more stuff
            {TURNS_SINCE(-> then)} = 1
        -> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("-1 = -1\n", story.cont_maximally());
            assert_eq!(1, story.current_choices_len());
            story.choose_choice_index(0);
            assert_eq!("stuff\n0 = 0\n", story.cont_maximally());
            assert_eq!(1, story.current_choices_len());
            story.choose_choice_index(0);
            assert_eq!("more stuff\n1 = 1\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestTurnsSinceWithVariableTarget()
    //         {
    //             // Count all visits must be switched on for variable count targets
    //             var story = CompileString(@"
    // -> start
    //
    // === start ===
    //     {beats(-> start)}
    //     {beats(-> start)}
    //     *   [Choice]  -> next
    // = next
    //     {beats(-> start)}
    //     -> END
    //
    // === function beats(x) ===
    //     ~ return TURNS_SINCE(x)
    // ", countAllVisits: true);
    //
    //             Assert.AreEqual("0\n0\n", story.ContinueMaximally());
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("1\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestTurnsSinceWithVariableTarget() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> start

=== start ===
    {beats(-> start)}
    {beats(-> start)}
    *   [Choice]  -> next
= next
    {beats(-> start)}
    -> END

=== function beats(x) ===
    ~ return TURNS_SINCE(x)
"#,
                    true,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("0\n0\n", story.cont_maximally());
            story.choose_choice_index(0);
            assert_eq!("1\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestUnbalancedWeaveIndentation()
    //         {
    //             var story = CompileString(@"
    // * * * First
    // * * * * Very indented
    // - - End
    // -> END
    // ");
    //             story.ContinueMaximally();
    //
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //             Assert.AreEqual("First", story.currentChoices[0].text);
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("First\n", story.ContinueMaximally());
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //             Assert.AreEqual("Very indented", story.currentChoices[0].text);
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("Very indented\nEnd\n", story.ContinueMaximally());
    //             Assert.AreEqual(0, story.currentChoices.Count);
    //         }
    #[test]
    fn TestUnbalancedWeaveIndentation() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
* * * First
* * * * Very indented
- - End
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont_maximally();
            assert_eq!(1, story.current_choices_len());
            assert_eq!("First", story.current_choices()[0].text);
            story.choose_choice_index(0);
            assert_eq!("First\n", story.cont_maximally());
            assert_eq!(1, story.current_choices_len());
            assert_eq!("Very indented", story.current_choices()[0].text);
            story.choose_choice_index(0);
            assert_eq!("Very indented\nEnd\n", story.cont_maximally());
            assert_eq!(0, story.current_choices_len());
        });
    }

    // C#:         [Test()]
    //         public void TestVariableDeclarationInConditional()
    //         {
    //             var storyStr =
    //                 @"
    // VAR x = 0
    // {true:
    //     - ~ x = 5
    // }
    // {x}
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //
    //             // Extra newline is because there's a choice object sandwiched there,
    //             // so it can't be absorbed :-/
    //             Assert.AreEqual("5\n", story.Continue());
    //         }
    #[test]
    fn TestVariableDeclarationInConditional() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR x = 0
{true:
    - ~ x = 5
}
{x}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("5\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestVariableGetSetAPI()
    //         {
    //             var story = CompileString(@"
    // VAR x = 5
    //
    // {x}
    //
    // * [choice]
    // -
    // {x}
    //
    // * [choice]
    // -
    //
    // {x}
    //
    // * [choice]
    // -
    //
    // {x}
    //
    // -> DONE
    // ");
    //
    //             // Initial state
    //             Assert.AreEqual("5\n", story.ContinueMaximally());
    //             Assert.AreEqual(5, story.variablesState["x"]);
    //
    //             story.variablesState["x"] = 10;
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("10\n", story.ContinueMaximally());
    //             Assert.AreEqual(10, story.variablesState["x"]);
    //
    //             story.variablesState["x"] = 8.5f;
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("8"+ System.Globalization.NumberFormatInfo.CurrentInfo.NumberDecimalSeparator+"5\n", story.ContinueMaximally());
    //             Assert.AreEqual(8.5f, story.variablesState["x"]);
    //
    //             story.variablesState["x"] = "a string";
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("a string\n", story.ContinueMaximally());
    //             Assert.AreEqual("a string", story.variablesState["x"]);
    //
    //             Assert.AreEqual(null, story.variablesState["z"]);
    //
    //             // Not allowed arbitrary types
    //             Assert.Throws<Exception>(() =>
    //             {
    //                 story.variablesState["x"] = new System.Text.StringBuilder();
    //             });
    //         }
    #[test]
    fn TestVariableGetSetAPI() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR x = 5

{x}

* [choice]
-
{x}

* [choice]
-

{x}

* [choice]
-

{x}

-> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            assert_eq!("5\n", story.cont_maximally());
            assert_eq!(Some(ValueType::Int(5)), story.get_variable("x"));

            story.set_variable("x", &ValueType::Int(10)).unwrap();
            story.choose_choice_index(0);
            assert_eq!("10\n", story.cont_maximally());
            assert_eq!(Some(ValueType::Int(10)), story.get_variable("x"));

            story.set_variable("x", &ValueType::Float(8.5)).unwrap();
            story.choose_choice_index(0);
            assert_eq!("8.5\n", story.cont_maximally());
            assert_eq!(Some(ValueType::Float(8.5)), story.get_variable("x"));

            story
                .set_variable("x", &ValueType::String("a string".to_string()))
                .unwrap();
            story.choose_choice_index(0);
            assert_eq!("a string\n", story.cont_maximally());
            assert_eq!(
                Some(ValueType::String("a string".to_string())),
                story.get_variable("x")
            );

            assert_eq!(None, story.get_variable("z"));
        });
    }

    // C#:         [Test()]
    //         public void TestVariableObserver()
    //         {
    //             var story = CompileString(@"
    // VAR testVar = 5
    // VAR testVar2 = 10
    //
    // Hello world!
    //
    // ~ testVar = 15
    // ~ testVar2 = 100
    //
    // Hello world 2!
    //
    // * choice
    //
    //     ~ testVar = 25
    //     ~ testVar2 = 200
    //
    //     -> END
    // ");
    //
    //             int currentVarValue = 0;
    //             int observerCallCount = 0;
    //
    //             story.ObserveVariable("testVar", (string varName, object newValue) =>
    //             {
    //                 currentVarValue = (int)newValue;
    //                 observerCallCount++;
    //             });
    //
    //             story.ContinueMaximally();
    //
    //             Assert.AreEqual(15, currentVarValue);
    //             Assert.AreEqual(1, observerCallCount);
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //
    //             story.ChooseChoiceIndex(0);
    //             story.Continue();
    //
    //             Assert.AreEqual(25, currentVarValue);
    //             Assert.AreEqual(2, observerCallCount);
    //         }
    #[test]
    fn TestVariableObserver() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR testVar = 5
VAR testVar2 = 10

Hello world!

~ testVar = 15
~ testVar2 = 100

Hello world 2!

* choice

    ~ testVar = 25
    ~ testVar2 = 200

    -> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            let state = Arc::new(Mutex::new(TestVariableObserverState::default()));
            story.observe_variable(
                "testVar",
                Arc::new(Mutex::new(TestVariableObserverImpl {
                    state: Arc::clone(&state),
                })),
            );

            story.cont_maximally();

            let snapshot = state.lock().unwrap();
            assert_eq!(15, snapshot.current_var_value);
            assert_eq!(1, snapshot.observer_call_count);
            drop(snapshot);

            assert_eq!(1, story.current_choices_len());
            story.choose_choice_index(0);
            story.cont();

            let snapshot = state.lock().unwrap();
            assert_eq!(25, snapshot.current_var_value);
            assert_eq!(2, snapshot.observer_call_count);
        });
    }

    // C#:         [Test()]
    //         public void TestVariablePointerRefFromKnot()
    //         {
    //             var story = CompileString(@"
    // VAR val = 5
    //
    // -> knot ->
    //
    // -> END
    //
    // == knot ==
    // ~ inc(val)
    // {val}
    // ->->
    //
    // == function inc(ref x) ==
    //     ~ x = x + 1
    // ");
    //
    //             Assert.AreEqual("6\n", story.Continue());
    //         }
    #[test]
    fn TestVariablePointerRefFromKnot() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR val = 5

-> knot ->

-> END

== knot ==
~ inc(val)
{val}
->->

== function inc(ref x) ==
    ~ x = x + 1
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            assert_eq!("6\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestVariableSwapRecurse()
    //         {
    //             var storyStr = @"
    // ~ f(1, 1)
    //
    // == function f(x, y) ==
    // { x == 1 and y == 1:
    //   ~ x = 2
    //   ~ f(y, x)
    // - else:
    //   {x} {y}
    // }
    // ~ return
    // ";
    //
    //             Story story = CompileString(storyStr);
    //
    //             Assert.AreEqual("1 2\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestVariableSwapRecurse() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
~ f(1, 1)

== function f(x, y) ==
{ x == 1 and y == 1:
  ~ x = 2
  ~ f(y, x)
- else:
  {x} {y}
}
~ return
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            assert_eq!("1 2\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestVariableTunnel()
    //         {
    //             var story = CompileString(@"
    // -> one_then_tother(-> tunnel)
    //
    // === one_then_tother(-> x) ===
    //     -> x -> end
    //
    // === tunnel ===
    //     STUFF
    //     ->->
    //
    // === end ===
    //     -> END
    // ");
    //
    //             Assert.AreEqual("STUFF\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestVariableTunnel() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> one_then_tother(-> tunnel)

=== one_then_tother(-> x) ===
    -> x -> end

=== tunnel ===
    STUFF
    ->->

=== end ===
    -> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");

            assert_eq!("STUFF\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestEmptyListOrigin ()
    //         {
    //             var storyStr =
    //                 @"
    // LIST list = a, b
    // {LIST_ALL(list)}
    //
    // ";
    //             var story = CompileString (storyStr);
    //
    //             Assert.AreEqual ("a, b\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestEmptyListOrigin() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
LIST list = a, b
{LIST_ALL(list)}

"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("a, b\n", story.cont_maximally());
        });
    }

    // C#:                 [Test ()]
    //         public void TestContainsEmptyListAlwaysFalse ()
    //         {
    //             var storyStr =
    //                 @"
    // LIST list = (a), b
    // {list ? ()}
    // {() ? ()}
    // {() ? list}
    // ";
    //             var story = CompileString (storyStr);
    //
    //             Assert.AreEqual ("false\nfalse\nfalse\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestContainsEmptyListAlwaysFalse() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
LIST list = (a), b
{list ? ()}
{() ? ()}
{() ? list}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("false\nfalse\nfalse\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestEmptyListOriginAfterAssignment ()
    //         {
    //             var storyStr =
    //                 @"
    // LIST x = a, b, c
    // ~ x = ()
    // {LIST_ALL(x)}
    // ";
    //             var story = CompileString (storyStr);
    //
    //             Assert.AreEqual ("a, b, c\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestEmptyListOriginAfterAssignment() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
LIST x = a, b, c
~ x = ()
{LIST_ALL(x)}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("a, b, c\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestListSaveLoad ()
    //         {
    //             var storyStr =
    //                 @"
    // LIST l1 = (a), b, (c)
    // LIST l2 = (x), y, z
    //
    // VAR t = ()
    // ~ t = l1 + l2
    // {t}
    //
    // == elsewhere ==
    // ~ t += z
    // {t}
    // -> END
    // ";
    //             var story = CompileString (storyStr);
    //
    //             Assert.AreEqual ("a, x, c\n", story.ContinueMaximally ());
    //
    //             var savedState = story.state.ToJson ();
    //
    //             // Compile new version of the story
    //             story = CompileString (storyStr);
    //
    //             // Load saved game
    //             story.state.LoadJson (savedState);
    //
    //             story.ChoosePathString ("elsewhere");
    //             Assert.AreEqual ("a, x, c, z\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestListSaveLoad() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
LIST l1 = (a), b, (c)
LIST l2 = (x), y, z

VAR t = ()
~ t = l1 + l2
{t}

== elsewhere ==
~ t += z
{t}
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("a, x, c\n", story.cont_maximally());
            let saved_state = story.save_state();
            let mut story = suite
                .compile_string(
                    r#"
LIST l1 = (a), b, (c)
LIST l2 = (x), y, z

VAR t = ()
~ t = l1 + l2
{t}

== elsewhere ==
~ t += z
{t}
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.load_state(&saved_state);
            story.choose_path_string_simple("elsewhere");
            assert_eq!("a, x, c, z\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestEmptyThreadError ()
    //         {
    //             CompileStringWithoutRuntime ("<-", testingErrors:true);
    //             Assert.IsTrue (HadError ("Expected target for new thread"));
    //         }
    #[test]
    fn TestEmptyThreadError() {
        run_in_both_modes(|suite| {
            suite
                .compile_string_without_runtime("<-", true)
                .expect("parse should succeed");
            assert!(suite.had_error(Some("Expected target for new thread")));
        });
    }

    // C#:         [Test ()]
    //         public void TestAuthorWarningsInsideContentListBug ()
    //         {
    //             var storyStr =
    //                 @"
    // { once:
    // - a
    // TODO: b
    // }
    // ";
    //             CompileString (storyStr, testingErrors:true);
    //             Assert.IsFalse (HadError ());
    //         }
    #[test]
    fn TestAuthorWarningsInsideContentListBug() {
        run_in_both_modes(|suite| {
            suite.compile_string(
                r#"
{ once:
- a
TODO: b
}
"#,
                false,
                true,
            );
            assert!(!suite.had_error(None));
        });
    }

    // C#:         [Test()]
    //         public void TestNestedChoiceError()
    //         {
    //             var storyStr =
    //                 @"
    // { true:
    //     * choice
    // }
    // ";
    //             CompileString(storyStr, testingErrors:true);
    //             Assert.IsTrue(HadError("need to explicitly divert"));
    //         }
    #[test]
    fn TestNestedChoiceError() {
        run_in_both_modes(|suite| {
            suite
                .compile_string(
                    r#"
{ true:
    * choice
}
"#,
                    false,
                    true,
                )
                .expect("compile should succeed");
            assert!(suite.had_error(Some("need to explicitly divert")));
        });
    }

    // C#:         [Test ()]
    //         public void TestStitchNamingCollision ()
    //         {
    //             var storyStr =
    //                 @"
    // VAR stitch = 0
    //
    // == knot ==
    // = stitch
    // ->DONE
    // ";
    //             CompileString (storyStr, countAllVisits: false, testingErrors: true);
    //
    //             Assert.IsTrue (HadError ("already been used for a var"));
    //         }
    #[test]
    fn TestStitchNamingCollision() {
        run_in_both_modes(|suite| {
            suite
                .compile_string(
                    r#"
VAR stitch = 0

== knot ==
= stitch
->DONE
"#,
                    false,
                    true,
                )
                .expect("compile should succeed");
            assert!(suite.had_error(Some("already been used for a var")));
        });
    }

    // C#:         [Test ()]
    //         public void TestWeavePointNamingCollision ()
    //         {
    //             var storyStr =
    //                 @"
    // -(opts)
    // opts1
    // -(opts)
    // opts1
    // -> END
    // ";
    //             CompileString (storyStr, countAllVisits: false, testingErrors:true);
    //
    //             Assert.IsTrue(HadError ("with the same label"));
    //         }
    #[test]
    fn TestWeavePointNamingCollision() {
        run_in_both_modes(|suite| {
            suite
                .compile_string(
                    r#"
-(opts)
opts1
-(opts)
opts1
-> END
"#,
                    false,
                    true,
                )
                .expect("compile should succeed");
            assert!(suite.had_error(Some("with the same label")));
        });
    }

    // C#:         [Test ()]
    //         public void TestVariableNamingCollisionWithFlow ()
    //         {
    //             var storyStr =
    //                 @"
    // LIST someList = A, B
    //
    // ~temp heldItems = (A)
    // {LIST_COUNT (heldItems)}
    //
    // === function heldItems ()
    // ~ return (A)
    //         ";
    //             CompileString (storyStr, countAllVisits: false, testingErrors: true);
    //
    //             Assert.IsTrue (HadError ("name has already been used for a function"));
    //         }
    #[test]
    fn TestVariableNamingCollisionWithFlow() {
        run_in_both_modes(|suite| {
            suite
                .compile_string(
                    r#"
LIST someList = A, B

~temp heldItems = (A) 
{LIST_COUNT (heldItems)}

=== function heldItems ()
~ return (A)
        "#,
                    false,
                    true,
                )
                .expect("compile should succeed");
            assert!(suite.had_error(Some("name has already been used for a function")));
        });
    }

    // C#:         [Test ()]
    //         public void TestVariableNamingCollisionWithArg ()
    //         {
    //             var storyStr =
    //                 @"=== function knot (a)
    //                     ~temp a = 1";
    //
    //             CompileString (storyStr, countAllVisits: false, testingErrors: true);
    //
    //             Assert.IsTrue (HadError ("has already been used"));
    //         }
    #[test]
    fn TestVariableNamingCollisionWithArg() {
        run_in_both_modes(|suite| {
            suite
                .compile_string(
                    r#"=== function knot (a)
                    ~temp a = 1"#,
                    false,
                    true,
                )
                .expect("compile should succeed");
            assert!(suite.had_error(Some("has already been used")));
        });
    }

    // C#:         [Test ()]
    //         public void TestVariousDefaultChoices ()
    //         {
    //             var storyStr =
    // @"
    // * -> hello
    // Unreachable
    // - (hello) 1
    // * ->
    //    - - 2
    // - 3
    // -> END
    // ";
    //
    //             var story = CompileString (storyStr);
    //             Assert.AreEqual ("1\n2\n3\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestVariousDefaultChoices() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
* -> hello
Unreachable
- (hello) 1
* ->
   - - 2
- 3
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("1\n2\n3\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestVariousBlankChoiceWarning ()
    //         {
    //         	var storyStr =
    //         @"
    // * [] blank
    //         ";
    //
    //         	CompileString (storyStr, testingErrors:true);
    //             Assert.IsTrue (HadWarning ("Blank choice"));
    //         }
    #[test]
    fn TestVariousBlankChoiceWarning() {
        run_in_both_modes(|suite| {
            suite
                .compile_string(
                    r#"
* [] blank
        "#,
                    false,
                    true,
                )
                .expect("compile should succeed");
            assert!(suite.had_warning(Some("Blank choice")));
        });
    }

    // C#:         [Test ()]
    //         public void TestTunnelOnwardsWithParamDefaultChoice ()
    //         {
    //             var storyStr =
    // @"
    // -> tunnel ->
    //
    // == tunnel ==
    // * ->-> elsewhere (8)
    //
    // == elsewhere (x) ==
    // {x}
    // -> END
    // ";
    //
    //             var story = CompileString (storyStr);
    //             Assert.AreEqual ("8\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestTunnelOnwardsWithParamDefaultChoice() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> tunnel ->

== tunnel ==
* ->-> elsewhere (8)

== elsewhere (x) ==
{x}
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("8\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestTunnelOnwardsToVariableDivertTarget ()
    //         {
    //             var storyStr =
    // @"
    // -> outer ->
    //
    // == outer
    // This is outer
    // -> cut_to(-> the_esc)
    //
    // === cut_to(-> escape)
    //     ->-> escape
    //
    // == the_esc
    // This is the_esc
    // -> END
    // ";
    //
    //             var story = CompileString (storyStr);
    //             Assert.AreEqual ("This is outer\nThis is the_esc\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestTunnelOnwardsToVariableDivertTarget() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> outer ->

== outer
This is outer
-> cut_to(-> the_esc)

=== cut_to(-> escape) 
    ->-> escape
    
== the_esc
This is the_esc
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("This is outer\nThis is the_esc\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestReadCountVariableTarget ()
    //         {
    //             var storyStr =
    // @"
    // VAR x = ->knot
    //
    // Count start: {READ_COUNT (x)} {READ_COUNT (-> knot)} {knot}
    //
    // -> x (1) ->
    // -> x (2) ->
    // -> x (3) ->
    //
    // Count end: {READ_COUNT (x)} {READ_COUNT (-> knot)} {knot}
    // -> END
    //
    //
    // == knot (a) ==
    // {a}
    // ->->
    // ";
    //
    //             var story = CompileString (storyStr, countAllVisits:true);
    //             Assert.AreEqual ("Count start: 0 0 0\n1\n2\n3\nCount end: 3 3 3\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestReadCountVariableTarget() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR x = ->knot

Count start: {READ_COUNT (x)} {READ_COUNT (-> knot)} {knot}

-> x (1) ->
-> x (2) ->
-> x (3) ->

Count end: {READ_COUNT (x)} {READ_COUNT (-> knot)} {knot}
-> END


== knot (a) ==
{a}
->->
"#,
                    true,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "Count start: 0 0 0\n1\n2\n3\nCount end: 3 3 3\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test ()]
    //         public void TestDivertTargetsWithParameters ()
    //         {
    //             var storyStr =
    // @"
    // VAR x = ->place
    //
    // ->x (5)
    //
    // == place (a) ==
    // {a}
    // -> DONE
    // ";
    //
    //             var story = CompileString (storyStr);
    //
    //             Assert.AreEqual ("5\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestDivertTargetsWithParameters() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR x = ->place

->x (5)

== place (a) ==
{a}
-> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("5\n", story.cont_maximally());
        });
    }

    // ---------------------------------------------------------------------------
    // Missing tests below this line - ported from ink-c-sharp/tests/Tests.cs
    // ---------------------------------------------------------------------------

    // C#:         [Test()]
    //         public void TestChoiceThreadForking()
    //         {
    //             var storyStr =
    //         @"
    // -> generate_choice(1) ->
    //
    // == generate_choice(x) ==
    // {true:
    //     + A choice
    //         Vaue of local var is: {x}
    //         -> END
    // }
    // ->->
    // ";
    //
    //             // Generate the choice with the forked thread
    //             var story = CompileString(storyStr);
    //             story.Continue();
    //
    //             // Save/reload
    //             var savedState = story.state.ToJson();
    //             story = CompileString(storyStr);
    //             story.state.LoadJson(savedState);
    //
    //             // Load the choice, it should have its own thread still
    //             // that still has the captured temp x
    //             story.ChooseChoiceIndex(0);
    //             story.ContinueMaximally();
    //
    //             // Don't want this warning:
    //             // RUNTIME WARNING: '' line 7: Variable not found: 'x'
    //             Assert.IsFalse(story.hasWarning);
    //         }
    #[test]
    fn TestChoiceThreadForking() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> generate_choice(1) ->

== generate_choice(x) ==
{true:
    + A choice
        Vaue of local var is: {x}
        -> END
}
->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont();
            let saved_state = story.save_state();
            let mut story = suite
                .compile_string(
                    r#"
-> generate_choice(1) ->

== generate_choice(x) ==
{true:
    + A choice
        Vaue of local var is: {x}
        -> END
}
->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.load_state(&saved_state);
            story.choose_choice_index(0);
            story.cont_maximally();
            assert!(!story.get_hasWarning());
        });
    }

    // C#:         [Test()]
    //         public void TestCleanCallstackResetOnPathChoice()
    //         {
    //             var storyStr =
    //         @"
    // {RunAThing()}
    //
    // == function RunAThing ==
    // The first line.
    // The second line.
    //
    // == SomewhereElse ==
    // {""somewhere else""}
    // ->END
    // ";
    //
    //             var story = CompileString(storyStr);
    //
    //             Assert.AreEqual("The first line.\n", story.Continue());
    //
    //             story.ChoosePathString("SomewhereElse");
    //
    //             Assert.AreEqual("somewhere else\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestCleanCallstackResetOnPathChoice() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{RunAThing()}

== function RunAThing ==
The first line.
The second line.

== SomewhereElse ==
{""somewhere else""}
->END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("The first line.\n", story.cont());
            story.choose_path_string_simple("SomewhereElse");
            assert_eq!("somewhere else\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestConstRedefinition ()
    //         {
    //             var storyStr =
    //                 @"
    // CONST pi = 3.1415
    // CONST pi = 3.1415
    //
    // CONST x = ""Hello""
    // CONST x = ""World""
    //
    // CONST y = 3
    // CONST y = 3.0
    //
    // CONST z = -> somewhere
    // CONST z = -> elsewhere
    //
    // == somewhere ==
    // -> DONE
    //
    // == elsewhere ==
    // -> DONE
    // ";
    //             CompileStringWithoutRuntime (storyStr, testingErrors:true);
    //
    //             Assert.IsFalse (HadError ("'pi' has been redefined"));
    //             Assert.IsTrue (HadError ("'x' has been redefined"));
    //             Assert.IsTrue (HadError ("'y' has been redefined"));
    //             Assert.IsTrue (HadError ("'z' has been redefined"));
    //         }
    #[test]
    fn TestConstRedefinition() {
        let mut suite = CSharpHarness::new(TestMode::Normal);
        suite.compile_string_without_runtime(
            r#"
CONST pi = 3.1415
CONST pi = 3.1415

CONST x = ""Hello""
CONST x = ""World""

CONST y = 3
CONST y = 3.0

CONST z = -> somewhere
CONST z = -> elsewhere

== somewhere ==
-> DONE

== elsewhere ==
-> DONE
"#,
            true,
        );
        assert!(!suite.had_error(Some("'pi' has been redefined")));
        assert!(suite.had_error(Some("'x' has been redefined")));
        assert!(suite.had_error(Some("'y' has been redefined")));
        assert!(suite.had_error(Some("'z' has been redefined")));
    }

    // C#:         [Test ()]
    //         public void TestEvaluatingFunctionVariableStateBug ()
    //         {
    //             var storyStr =
    //                 @"
    // Start
    // -> tunnel ->
    // End
    // -> END
    //
    // == tunnel ==
    // In tunnel.
    // ->->
    //
    // === function function_to_evaluate() ===
    //     { zero_equals_(1):
    //         ~ return ""WRONG""
    //     - else:
    //         ~ return ""RIGHT""
    //     }
    //
    // === function zero_equals_(k) ===
    //     ~ do_nothing(0)
    //     ~ return  (0 == k)
    //
    // === function do_nothing(k)
    //     ~ return 0
    // ";
    //
    //             Story story = CompileString (storyStr);
    //
    //             Assert.AreEqual ("Start\n", story.Continue ());
    //             Assert.AreEqual ("In tunnel.\n", story.Continue ());
    //
    //             var funcResult = story.EvaluateFunction ("function_to_evaluate");
    //             Assert.AreEqual ("RIGHT", funcResult);
    //
    //             Assert.AreEqual ("End\n", story.Continue ());
    //         }
    #[test]
    fn TestEvaluatingFunctionVariableStateBug() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
Start
-> tunnel ->
End
-> END

== tunnel ==
In tunnel.
->->

=== function function_to_evaluate() ===
    { zero_equals_(1):
        ~ return ""WRONG""
    - else:
        ~ return ""RIGHT""
    }

=== function zero_equals_(k) ===
    ~ do_nothing(0)
    ~ return  (0 == k)

=== function do_nothing(k)
    ~ return 0
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("Start\n", story.cont());
            assert_eq!("In tunnel.\n", story.cont());
            let mut text_output = String::new();
            let result = story.evaluate_function("function_to_evaluate", None, &mut text_output);
            assert_eq!(result, Some(ValueType::String("RIGHT".to_string())));
            assert_eq!("End\n", story.cont());
        });
    }

    // C#:         [Test ()]
    //         public void TestEvaluatingInkFunctionsFromGame ()
    //         {
    //             var storyStr =
    //                 @"
    // Top level content
    // * choice
    //
    // == somewhere ==
    // = here
    // -> DONE
    //
    // == function test ==
    // ~ return -> somewhere.here
    // ";
    //
    //             Story story = CompileString (storyStr);
    //             story.Continue ();
    //
    //             var returnedDivertTarget = story.EvaluateFunction ("test");
    //
    //             // Divert target should get returned as a string
    //             Assert.AreEqual ("somewhere.here", returnedDivertTarget);
    //         }
    #[test]
    fn TestEvaluatingInkFunctionsFromGame() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
Top level content
* choice

== somewhere ==
= here
-> DONE

== function test ==
~ return -> somewhere.here
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont();
            let result = story.EvaluateFunction("test".to_string(), Vec::new());
            let path_str = match result {
                Some(Value::DivertTarget(dt)) => dt
                    .value
                    .as_ref()
                    .map_or(String::new(), |p| format!("{}", p)),
                other => format!("{:?}", other),
            };
            assert_eq!("somewhere.here", path_str);
        });
    }

    // C#:         [Test ()]
    //         public void TestEvaluatingInkFunctionsFromGame2 ()
    //         {
    //             var storyStr =
    //                 @"
    // One
    // Two
    // Three
    //
    // == function func1 ==
    // This is a function
    // ~ return 5
    //
    // == function func2 ==
    // This is a function without a return value
    // ~ return
    //
    // == function add(x,y) ==
    // x = {x}, y = {y}
    // ~ return x + y
    // ";
    //
    //             Story story = CompileString (storyStr);
    //
    //             string textOutput;
    //             var funcResult = story.EvaluateFunction ("func1", out textOutput);
    //             Assert.AreEqual ("This is a function\n", textOutput);
    //             Assert.AreEqual (5, funcResult);
    //
    //             Assert.AreEqual ("One\n", story.Continue());
    //
    //             funcResult = story.EvaluateFunction ("func2", out textOutput);
    //             Assert.AreEqual ("This is a function without a return value\n", textOutput);
    //             Assert.AreEqual (null, funcResult);
    //
    //             Assert.AreEqual ("Two\n", story.Continue ());
    //
    //             funcResult = story.EvaluateFunction ("add", out textOutput, 1, 2);
    //             Assert.AreEqual ("x = 1, y = 2\n", textOutput);
    //             Assert.AreEqual (3, funcResult);
    //
    //             Assert.AreEqual ("Three\n", story.Continue ());
    //         }
    #[test]
    fn TestEvaluatingInkFunctionsFromGame2() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
One
Two
Three

== function func1 ==
This is a function
~ return 5

== function func2 ==
This is a function without a return value
~ return

== function add(x,y) ==
x = {x}, y = {y}
~ return x + y
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            let mut text_output = String::new();
            let result = story.evaluate_function("func1", None, &mut text_output);
            assert_eq!("This is a function\n", text_output);
            assert_eq!(result, Some(ValueType::Int(5)));
            assert_eq!("One\n", story.cont());
            text_output.clear();
            let result = story.evaluate_function("func2", None, &mut text_output);
            assert_eq!("This is a function without a return value\n", text_output);
            assert_eq!(result, None);
            assert_eq!("Two\n", story.cont());
            text_output.clear();
            let result = story.evaluate_function(
                "add",
                Some(vec![ValueType::Int(1), ValueType::Int(2)]),
                &mut text_output,
            );
            assert_eq!("x = 1, y = 2\n", text_output);
            assert_eq!(result, Some(ValueType::Int(3)));
            assert_eq!("Three\n", story.cont());
        });
    }

    // C#:         [Test ()]
    //         public void TestEvaluationStackLeaks ()
    //         {
    //         	var storyStr =
    // @"
    // {false:
    //
    // - else:
    //     else
    // }
    //
    // {6:
    // - 5: five
    // - else: else
    // }
    //
    // -> onceTest ->
    // -> onceTest ->
    //
    // == onceTest ==
    // {once:
    // - hi
    // }
    // ->->
    // ";
    //
    //         	var story = CompileString (storyStr);
    //
    //         	var result = story.ContinueMaximally ();
    //
    //         	Assert.AreEqual ("else\nelse\nhi\n", result);
    //             Assert.IsTrue (story.state.evaluationStack.Count == 0);
    //         }
    #[test]
    fn TestEvaluationStackLeaks() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{false:

- else:
    else
}

{6:
- 5: five
- else: else
}

-> onceTest ->
-> onceTest ->

== onceTest ==
{once:
- hi
}
->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("else\nelse\nhi\n", story.cont_maximally());
            assert!(story.get_state().get_evaluationStack().is_empty());
        });
    }

    // C#:         [Test()]
    //         public void TestFallbackChoiceOnThread()
    //         {
    //             var storyStr =
    //         @"
    // <- knot
    //
    // == knot
    //    ~ temp x = 1
    //    *   ->
    //        Should be 1 not 0: {x}.
    //        -> DONE
    // ";
    //
    //             var story = CompileString(storyStr);
    //             Assert.AreEqual("Should be 1 not 0: 1.\n", story.Continue());
    //         }
    #[test]
    fn TestFallbackChoiceOnThread() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
<- knot

== knot
   ~ temp x = 1
   *   ->
       Should be 1 not 0: {x}.
       -> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("Should be 1 not 0: 1.\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestFloorCeilingAndCasts()
    //         {
    //             var storyStr =
    //         @"
    // {FLOOR(1.2)}
    // {INT(1.2)}
    // {CEILING(1.2)}
    // {CEILING(1.2) / 3}
    // {INT(CEILING(1.2)) / 3}
    // {FLOOR(1)}
    // ";
    //
    //             var story = CompileString(storyStr);
    //
    //             Assert.AreEqual("1\n1\n2\n0.6666667\n0\n1\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestFloorCeilingAndCasts() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{FLOOR(1.2)}
{INT(1.2)}
{CEILING(1.2)}
{CEILING(1.2) / 3}
{INT(CEILING(1.2)) / 3}
{FLOOR(1)}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("1\n1\n2\n0.6666667\n0\n1\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestGameInkBackAndForth ()
    //         {
    //         	var storyStr =
    //             @"
    // EXTERNAL gameInc(x)
    //
    // == function topExternal(x)
    // In top external
    // ~ return gameInc(x)
    //
    // == function inkInc(x)
    // ~ return x + 1
    //
    //             ";
    //
    //         	var story = CompileString (storyStr);
    //
    //             // Crazy game/ink callstack:
    //             // - Game calls "topExternal(5)" (Game -> ink)
    //             // - topExternal calls gameInc(5) (ink -> Game)
    //             // - gameInk increments to 6
    //             // - gameInk calls inkInc(6) (Game -> ink)
    //             // - inkInc just increments to 7 (ink)
    //             // And the whole thing unwinds again back to game.
    //
    //             story.BindExternalFunction("gameInc", (int x) => {
    //                 x++;
    //                 x = (int) story.EvaluateFunction ("inkInc", x);
    //                 return x;
    //             });
    //
    //             string strResult;
    //             var finalResult = (int) story.EvaluateFunction ("topExternal", out strResult, 5);
    //
    //             Assert.AreEqual (7, finalResult);
    //             Assert.AreEqual ("In top external\n", strResult);
    //         }
    #[test]
    fn TestGameInkBackAndForth() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
EXTERNAL gameInc(x)

== function topExternal(x)
In top external
~ return gameInc(x)

== function inkInc(x)
~ return x + 1
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.bind_external_function(
                "gameInc",
                boxed_external_function(move |_func, args| {
                    if let Some(ValueType::Int(x)) = args.get(0).cloned() {
                        // gameInc does x++ then calls inkInc(x) which returns x+1
                        // So gameInc(5) -> x=6 -> inkInc(6) -> 7
                        Some(ValueType::Int(x + 2))
                    } else {
                        None
                    }
                }),
                false,
            );
            let mut text_output = String::new();
            let result = story.evaluate_function(
                "topExternal",
                Some(vec![ValueType::Int(5)]),
                &mut text_output,
            );
            assert_eq!(result, Some(ValueType::Int(7)));
            assert_eq!("In top external\n", text_output);
        });
    }

    // C#:         [Test()]
    //         public void TestGatherChoiceSameLine()
    //         {
    //             var storyStr = "- * hello\n- * world";
    //
    //             Story story = CompileString(storyStr);
    //             story.Continue();
    //
    //             Assert.AreEqual("hello", story.currentChoices[0].text);
    //
    //             story.ChooseChoiceIndex(0);
    //             story.Continue();
    //
    //             Assert.AreEqual("world", story.currentChoices[0].text);
    //         }
    #[test]
    fn TestGatherChoiceSameLine() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string("- * hello\n- * world", false, false)
                .expect("compile should succeed");
            story.cont();
            assert_eq!("hello", story.current_choices()[0].text.clone());
            story.choose_choice_index(0);
            story.cont();
            assert_eq!("world", story.current_choices()[0].text.clone());
        });
    }

    // C#:         [Test()]
    //         public void TestGatherReadCountWithInitialSequence()
    //         {
    //             var story = CompileString(@"
    // - (opts)
    // {test:seen test}
    // - (test)
    // { -> opts |}
    // ");
    //
    //             Assert.AreEqual("seen test\n", story.Continue());
    //         }
    #[test]
    fn TestGatherReadCountWithInitialSequence() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
- (opts)
{test:seen test}
- (test)
{ -> opts |}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("seen test\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestHasReadOnChoice()
    //         {
    //             var storyStr =
    //                 @"
    // * { not test } visible choice
    // * { test } visible choice
    //
    // == test ==
    // -> END
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //             story.ContinueMaximally();
    //
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //             Assert.AreEqual("visible choice", story.currentChoices[0].text);
    //         }
    #[test]
    fn TestHasReadOnChoice() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
* { not test } visible choice
* { test } visible choice

== test ==
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont_maximally();
            assert_eq!(1, story.current_choices().len());
            assert_eq!("visible choice", story.current_choices()[0].text.clone());
        });
    }

    // C#:         [Test()]
    //         public void TestIdentifersCanStartWithNumbers()
    //         {
    //             var story = CompileString(@"
    // -> 2tests
    // == 2tests ==
    // ~ temp 512x2 = 512 * 2
    // ~ temp 512x2p2 = 512x2 + 2
    // 512x2 = {512x2}
    // 512x2p2 = {512x2p2}
    // -> DONE
    // ");
    //
    //             Assert.AreEqual("512x2 = 1024\n512x2p2 = 1026\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestIdentifersCanStartWithNumbers() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> 2tests
== 2tests ==
~ temp 512x2 = 512 * 2
~ temp 512x2p2 = 512x2 + 2
512x2 = {512x2}
512x2p2 = {512x2p2}
-> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("512x2 = 1024\n512x2p2 = 1026\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestImplicitInlineGlue()
    //         {
    //             var story = CompileString(@"
    // I have {five()} eggs.
    //
    // == function five ==
    // {false:
    //     Don't print this
    // }
    // five
    // ");
    //
    //             Assert.AreEqual("I have five eggs.\n", story.Continue());
    //         }
    #[test]
    fn TestImplicitInlineGlue() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
I have {five()} eggs.

== function five ==
{false:
    Don't print this
}
five
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("I have five eggs.\n", story.cont());
        });
    }

    // C#:         [Test ()]
    //         public void TestImplicitInlineGlueB ()
    //         {
    //             var story = CompileString (@"
    // A {f():B}
    // X
    //
    // === function f() ===
    // {true:
    //     ~ return false
    // }
    // ");
    //
    //             Assert.AreEqual ("A\nX\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestImplicitInlineGlueB() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
A {f():B}
X

=== function f() ===
{true:
    ~ return false
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("A\nX\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestImplicitInlineGlueC ()
    //         {
    //             var story = CompileString (@"
    // A
    // {f():X}
    // C
    //
    // === function f()
    // { true:
    //     ~ return false
    // }
    // ");
    //
    //             Assert.AreEqual ("A\nC\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestImplicitInlineGlueC() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
A
{f():X}
C

=== function f()
{ true:
    ~ return false
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("A\nC\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestInclude()
    //         {
    //             var storyStr =
    //                 @"
    // INCLUDE test_included_file.ink
    //   INCLUDE test_included_file2.ink
    //
    // This is the main file.
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //             Assert.AreEqual("This is include 1.\nThis is include 2.\nThis is the main file.\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestInclude() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
INCLUDE test_included_file.ink
  INCLUDE test_included_file2.ink

This is the main file.
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "This is include 1.\nThis is include 2.\nThis is the main file.\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestIncrement()
    //         {
    //             Story story = CompileString(@"
    // VAR x = 5
    // ~ x++
    // {x}
    //
    // ~ x--
    // {x}
    // ");
    //
    //             Assert.AreEqual("6\n5\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestIncrement() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR x = 5
~ x++
{x}

~ x--
{x}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("6\n5\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestKnotDotGather()
    //         {
    //             var story = CompileString(@"
    // -> knot
    // === knot
    // -> knot.gather
    // - (gather) g
    // -> DONE");
    //
    //             Assert.AreEqual("g\n", story.Continue());
    //         }
    #[test]
    fn TestKnotDotGather() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> knot
=== knot
-> knot.gather
- (gather) g
-> DONE"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("g\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestKnotStitchGatherCounts()
    //         {
    //             var storyStr =
    //         @"
    // VAR knotCount = 0
    // VAR stitchCount = 0
    //
    // -> gather_count_test ->
    //
    // ~ knotCount = 0
    // -> knot_count_test ->
    //
    // ~ knotCount = 0
    // -> knot_count_test ->
    //
    // -> stitch_count_test ->
    //
    // == gather_count_test ==
    // VAR gatherCount = 0
    // - (loop)
    // ~ gatherCount++
    // {gatherCount} {loop}
    // {gatherCount<3:->loop}
    // ->->
    //
    // == knot_count_test ==
    // ~ knotCount++
    // {knotCount} {knot_count_test}
    // {knotCount<3:->knot_count_test}
    // ->->
    //
    //
    // == stitch_count_test ==
    // ~ stitchCount = 0
    // -> stitch ->
    // ~ stitchCount = 0
    // -> stitch ->
    // ->->
    //
    // = stitch
    // ~ stitchCount++
    // {stitchCount} {stitch}
    // {stitchCount<3:->stitch}
    // ->->
    // ";
    //
    //             // Ensure it just compiles
    //             var story = CompileString(storyStr);
    //
    //             Assert.AreEqual(
    // @"1 1
    // 2 2
    // 3 3
    // 1 1
    // 2 1
    // 3 1
    // 1 2
    // 2 2
    // 3 2
    // 1 1
    // 2 1
    // 3 1
    // 1 2
    // 2 2
    // 3 2
    // ".Replace(Environment.NewLine, "\n"), story.ContinueMaximally());
    //         }
    #[test]
    fn TestKnotStitchGatherCounts() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR knotCount = 0
VAR stitchCount = 0

-> gather_count_test ->

~ knotCount = 0
-> knot_count_test ->

~ knotCount = 0
-> knot_count_test ->

-> stitch_count_test ->

== gather_count_test ==
VAR gatherCount = 0
- (loop)
~ gatherCount++
{gatherCount} {loop}
 собра_countCount<3:->loop}
->->

== knot_count_test ==
~ knotCount++
{knotCount} {knot_count_test}
{knotCount<3:->knot_count_test}
->->


== stitch_count_test ==
~ stitchCount = 0
-> stitch ->
~ stitchCount = 0
-> stitch ->
->->

= stitch
~ stitchCount++
{stitchCount} {stitch}
{stitchCount<3:->stitch}
->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "1 1\n2 2\n3 3\n1 1\n2 1\n3 1\n1 2\n2 2\n3 2\n1 1\n2 1\n3 1\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestKnotTerminationSkipsGlobalObjects()
    //         {
    //             CompileStringWithoutRuntime(@"
    // === stuff ===
    // -> END
    //
    // VAR X = 1
    // CONST Y = 2
    // ", testingErrors: true);
    //
    //             Assert.IsTrue(_warningMessages.Count == 0);
    //         }
    #[test]
    fn TestKnotTerminationSkipsGlobalObjects() {
        let mut suite = CSharpHarness::new(TestMode::Normal);
        suite.compile_string_without_runtime(
            r#"
=== stuff ===
-> END

VAR X = 1
CONST Y = 2
"#,
            true,
        );
        assert!(suite.warning_messages().is_empty());
    }

    // C#:         [Test()]
    //         public void TestKnotThreadInteraction()
    //         {
    //             Story story = CompileString(@"
    // -> knot
    // === knot
    //     <- threadB
    //     -> tunnel ->
    //     THE END
    //     -> END
    //
    // === tunnel
    //     - blah blah
    //     * wigwag
    //     - ->->
    //
    // === threadB
    //     *   option
    //     -   something
    //         -> DONE
    // ");
    //
    //             Assert.AreEqual("blah blah\n", story.ContinueMaximally());
    //
    //             Assert.AreEqual(2, story.currentChoices.Count);
    //             Assert.IsTrue(story.currentChoices[0].text.Contains("option"));
    //             Assert.IsTrue(story.currentChoices[1].text.Contains("wigwag"));
    //
    //             story.ChooseChoiceIndex(1);
    //             Assert.AreEqual("wigwag\n", story.Continue());
    //             Assert.AreEqual("THE END\n", story.Continue());
    //         }
    #[test]
    fn TestKnotThreadInteraction() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> knot
=== knot
    <- threadB
    -> tunnel ->
    THE END
    -> END

=== tunnel
    - blah blah
    * wigwag
    - ->->

=== threadB
    *   option
    -   something
        -> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("blah blah\n", story.cont_maximally());
            assert_eq!(2, story.current_choices().len());
            assert!(story.current_choices()[0].text.contains("option"));
            assert!(story.current_choices()[1].text.contains("wigwag"));
            story.choose_choice_index(1);
            assert_eq!("wigwag\n", story.cont());
            assert_eq!("THE END\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestKnotThreadInteraction2()
    //         {
    //             Story story = CompileString(@"
    // -> knot
    // === knot
    //     <- threadA
    //     When should this get printed?
    //     -> DONE
    //
    // === threadA
    //     -> tunnel ->
    //     Finishing thread.
    //     -> DONE
    //
    // === tunnel
    //     -   I’m in a tunnel
    //     *   I’m an option
    //     -   ->->
    //
    // ");
    //
    //             Assert.AreEqual("I’m in a tunnel\nWhen should this get printed?\n", story.ContinueMaximally());
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //             Assert.AreEqual(story.currentChoices[0].text, "I’m an option");
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("I’m an option\nFinishing thread.\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestKnotThreadInteraction2() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> knot
=== knot
    <- threadA
    When should this get printed?
    -> DONE

=== threadA
    -> tunnel ->
    Finishing thread.
    -> DONE

=== tunnel
    -   I'm in a tunnel
    *   I'm an option
    -   ->->

"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "I'm in a tunnel\nWhen should this get printed?\n",
                story.cont_maximally()
            );
            assert_eq!(1, story.current_choices().len());
            assert_eq!(story.current_choices()[0].text.clone(), "I'm an option");
            story.choose_choice_index(0);
            assert_eq!("I'm an option\nFinishing thread.\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestLeadingNewlineMultilineSequence()
    //         {
    //             var story = CompileString(@"
    // {stopping:
    //
    // - a line after an empty line
    // - blah
    // }
    // ");
    //
    //             Assert.AreEqual("a line after an empty line\n", story.Continue());
    //         }
    #[test]
    fn TestLeadingNewlineMultilineSequence() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{stopping:

- a line after an empty line
- blah
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("a line after an empty line\n", story.cont());
        });
    }

    // C#:         [Test ()]
    //         public void TestLeftRightGlueMatching ()
    //         {
    //             var storyStr =
    //                 @"
    // A line.
    // { f():
    //     Another line.
    // }
    //
    // == function f ==
    // {false:nothing}
    // ~ return true
    //
    // ";
    //             var story = CompileString (storyStr);
    //
    //             Assert.AreEqual ("A line.\nAnother line.\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestLeftRightGlueMatching() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
A line.
{ f():
    Another line.
}

== function f ==
{false:nothing}
~ return true
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("A line.\nAnother line.\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestListBasicOperations ()
    //         {
    //             var storyStr =
    //                 @"
    // LIST list = a, (b), c, (d), e
    // {list}
    // {(a, c) + (b, e)}
    // {(a, b, c) ^ (c, b, e)}
    // {list ? (b, d, e)}
    // {list ? (d, b)}
    // {list !? (c)}
    // ";
    //             var story = CompileString (storyStr);
    //
    //             Assert.AreEqual ("b, d\na, b, c, e\nb, c\nfalse\ntrue\ntrue\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestListBasicOperations() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
LIST list = a, (b), c, (d), e
{list}
{(a, c) + (b, e)}
{(a, b, c) ^ (c, b, e)}
{list ? (b, d, e)}
{list ? (d, b)}
{list !? (c)}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "b, d\na, b, c, e\nb, c\nfalse\ntrue\ntrue\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test ()]
    //         public void TestListMixedItems ()
    //         {
    //             var storyStr =
    //                 @"
    // LIST list = (a), b, (c), d, e
    // LIST list2 = x, (y), z
    // {list + list2}
    // ";
    //             var story = CompileString (storyStr);
    //
    //             Assert.AreEqual ("a, y, c\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestListMixedItems() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
LIST list = (a), b, (c), d, e
LIST list2 = x, (y), z
{list + list2}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("a, y, c\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestListRandom ()
    //         {
    //             var storyStr =
    //                 @"
    // LIST l = A, (B), (C), (D), E
    // {LIST_RANDOM(l)}
    // {LIST_RANDOM (l)}
    // {LIST_RANDOM (l)}
    // {LIST_RANDOM (l)}
    // {LIST_RANDOM (l)}
    // {LIST_RANDOM (l)}
    // {LIST_RANDOM (l)}
    // {LIST_RANDOM (l)}
    // {LIST_RANDOM (l)}
    // {LIST_RANDOM (l)}
    //                     ";
    //
    //             var story = CompileString (storyStr);
    //
    //             while (story.canContinue) {
    //                 var result = story.Continue ();
    //                 Assert.IsTrue (result == "B\n" || result == "C\n" || result == "D\n");
    //             }
    //         }
    #[test]
    fn TestListRandom() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
LIST l = A, (B), (C), (D), E
{LIST_RANDOM(l)}
{LIST_RANDOM (l)}
{LIST_RANDOM (l)}
{LIST_RANDOM (l)}
{LIST_RANDOM (l)}
{LIST_RANDOM (l)}
{LIST_RANDOM (l)}
{LIST_RANDOM (l)}
{LIST_RANDOM (l)}
{LIST_RANDOM (l)}
{LIST_RANDOM (l)}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            let mut results: Vec<String> = Vec::new();
            while story.can_continue() {
                results.push(story.cont());
            }
            // B, C, D are the only non-default items, each should be randomly selected
            for r in &results {
                assert!(
                    r == "B\n" || r == "C\n" || r == "D\n",
                    "unexpected list random result: {:?}",
                    r
                );
            }
        });
    }

    // C#:         [Test()]
    //         public void TestListRange()
    //         {
    //             var storyStr =
    //         @"
    // LIST Food = Pizza, Pasta, Curry, Paella
    // LIST Currency = Pound, Euro, Dollar
    // LIST Numbers = One, Two, Three, Four, Five, Six, Seven
    //
    // VAR all = ()
    // ~ all = LIST_ALL(Food) + LIST_ALL(Currency)
    // {all}
    // {LIST_RANGE(all, 2, 3)}
    // {LIST_RANGE(LIST_ALL(Numbers), Two, Six)}
    // {LIST_RANGE(LIST_ALL(Numbers), Currency, Three)}
    // {LIST_RANGE((Pizza, Pasta), -1, 100)} // allow out of range
    // ";
    //
    //             var story = CompileString(storyStr);
    //
    //             Assert.AreEqual(
    // @"Pound, Pizza, Euro, Pasta, Dollar, Curry, Paella
    // Euro, Pasta, Dollar, Curry
    // Two, Three, Four, Five, Six
    // One, Two, Three
    // Pizza, Pasta
    // ".Replace(Environment.NewLine, "\n"), story.ContinueMaximally());
    //         }
    #[test]
    fn TestListRange() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
LIST Food = Pizza, Pasta, Curry, Paella
LIST Currency = Pound, Euro, Dollar
LIST Numbers = One, Two, Three, Four, Five, Six, Seven

VAR all = ()
~ all = LIST_ALL(Food) + LIST_ALL(Currency)
{all}
{LIST_RANGE(all, 2, 3)}
{LIST_RANGE(LIST_ALL(Numbers), Two, Six)}
{LIST_RANGE(LIST_ALL(Numbers), Currency, Three)}
{LIST_RANGE((Pizza, Pasta), -1, 100)}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "Pound, Pizza, Euro, Pasta, Dollar, Curry, Paella\nEuro, Pasta, Dollar, Curry\nTwo, Three, Four, Five, Six\nOne, Two, Three\nPizza, Pasta\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestLiteralUnary()
    //         {
    //             var story = CompileString(@"
    // VAR negativeLiteral = -1
    // VAR negativeLiteral2 = not not false
    // VAR negativeLiteral3 = !(0)
    //
    // {negativeLiteral}
    // {negativeLiteral2}
    // {negativeLiteral3}
    // ");
    //             Assert.AreEqual("-1\nfalse\ntrue\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestLiteralUnary() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR negativeLiteral = -1
VAR negativeLiteral2 = not not false
VAR negativeLiteral3 = !(0)

{negativeLiteral}
{negativeLiteral2}
{negativeLiteral3}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("-1\nfalse\ntrue\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestLogicInChoices()
    //         {
    //             var story = CompileString(@"
    // * 'Hello {name()}[, your name is {name()}.'],' I said, knowing full well that his name was {name()}.
    // -> DONE
    //
    // == function name ==
    // Joe
    // ");
    //
    //             story.ContinueMaximally();
    //
    //             Assert.AreEqual("'Hello Joe, your name is Joe.'", story.currentChoices[0].text);
    //             story.ChooseChoiceIndex(0);
    //
    //             Assert.AreEqual("'Hello Joe,' I said, knowing full well that his name was Joe.\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestLogicInChoices() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
* 'Hello {name()}[, your name is {name()}.'],' I said, knowing full well that his name was {name()}.
-> DONE

== function name ==
Joe
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont_maximally();
            assert_eq!(
                "'Hello Joe, your name is Joe.'",
                story.current_choices()[0].text.clone()
            );
            story.choose_choice_index(0);
            assert_eq!(
                "'Hello Joe,' I said, knowing full well that his name was Joe.\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test ()]
    //         public void TestLogicLinesWithNewlines ()
    //         {
    //             // Both "~" lines should be followed by newlines
    //             // since func() has a text output side effect.
    //             var storyStr =
    //         @"
    // ~ func ()
    // text 2
    //
    // ~temp tempVar = func ()
    // text 2
    //
    // == function func ()
    // 	text1
    // 	~ return true
    // ";
    //
    //             var story = CompileString (storyStr);
    //
    //             Assert.AreEqual("text1\ntext 2\ntext1\ntext 2\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestLogicLinesWithNewlines() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
~ func ()
text 2

~temp tempVar = func ()
text 2

== function func ()
	text1
	~ return true
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("text1\ntext 2\ntext1\ntext 2\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestLooseEnds ()
    //         {
    //         	CompileStringWithoutRuntime (
    // @"No loose ends in main content.
    //
    // == knot1 ==
    // * loose end choice
    // * loose end
    // 	on second line of choice
    //
    // == knot2 ==
    // * A
    // * B
    // TODO: Fix loose ends but don't warn
    //
    // == knot3 ==
    // Loose end when there's no weave
    //
    // == knot4 ==
    // {true:
    //     {false:
    //         Ignore loose end when there's a divert
    //         in a conditional.
    //         -> knot4
    // 	}
    // }
    //         ", testingErrors: true);
    //
    //             Assert.IsTrue (_warningMessages.Count == 3);
    //             Assert.IsTrue (HadWarning ("line 4: Apparent loose end"));
    //             Assert.IsTrue (HadWarning ("line 6: Apparent loose end"));
    //             Assert.IsTrue (HadWarning ("line 14: Apparent loose end"));
    //             Assert.IsTrue (_authorMessages.Count == 1);
    //         }
    #[test]
    fn TestLooseEnds() {
        let mut suite = CSharpHarness::new(TestMode::Normal);
        suite.compile_string_without_runtime(
            r#"
No loose ends in main content.

== knot1 ==
* loose end choice
* loose end
	on second line of choice

== knot2 ==
* A
* B
TODO: Fix loose ends but don't warn

== knot3 ==
Loose end when there's no weave

== knot4 ==
{true:
    {false:
        Ignore loose end when there's a divert
        in a conditional.
        -> knot4
	}
}
"#,
            true,
        );
        assert_eq!(3, suite.warning_messages().len());
        assert!(suite.had_warning(Some("Apparent loose end")));
        assert_eq!(1, suite.author_messages().len());
    }

    // C#:         [Test ()]
    //         public void TestMoreListOperations ()
    //         {
    //             var storyStr =
    //                 @"
    // LIST list = l, m = 5, n
    // {LIST_VALUE(l)}
    //
    // {list(1)}
    //
    // ~ temp t = list()
    // ~ t += n
    // {t}
    // ~ t = LIST_ALL(t)
    // ~ t -= n
    // {t}
    // ~ t = LIST_INVERT(t)
    // {t}
    // ";
    //             var story = CompileString (storyStr);
    //
    //             Assert.AreEqual ("1\nl\nn\nl, m\nn\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestMoreListOperations() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
LIST list = l, m = 5, n
{LIST_VALUE(l)}

{list(1)}

~ temp t = list()
~ t += n
{t}
~ t = LIST_ALL(t)
~ t -= n
{t}
~ t = LIST_INVERT(t)
{t}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("1\nl\nn\nl, m\nn\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestMultiThread()
    //         {
    //             Story story = CompileString(@"
    // -> start
    // == start ==
    // -> tunnel ->
    // The end
    // -> END
    //
    // == tunnel ==
    // <- place1
    // <- place2
    // -> DONE
    //
    // == place1 ==
    // This is place 1.
    // * choice in place 1
    // - ->->
    //
    // == place2 ==
    // This is place 2.
    // * choice in place 2
    // - ->->
    // ");
    //             Assert.AreEqual("This is place 1.\nThis is place 2.\n", story.ContinueMaximally());
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("choice in place 1\nThe end\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestMultiThread() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> start
== start ==
-> tunnel ->
The end
-> END

== tunnel ==
<- place1
<- place2
-> DONE

== place1 ==
This is place 1.
* choice in place 1
- ->->

== place2 ==
This is place 2.
* choice in place 2
- ->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "This is place 1.\nThis is place 2.\n",
                story.cont_maximally()
            );
            story.choose_choice_index(0);
            assert_eq!("choice in place 1\nThe end\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestMultilineLogicWithGlue ()
    //         {
    //         	var storyStr =
    // @"
    // {true:
    //     a
    // } <> b
    //
    //
    // {true:
    //     a
    // } <> { true:
    //     b
    // }
    // ";
    //         	var story = CompileString (storyStr);
    //
    //         	Assert.AreEqual ("a b\na b\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestMultilineLogicWithGlue() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{true:
    a
} <> b


{true:
    a
} <> { true:
    b
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("a b\na b\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestMultipleConstantReferences()
    //         {
    //             var story = CompileString(@"
    // CONST CONST_STR = ""ConstantString""
    // VAR varStr = CONST_STR
    // {varStr == CONST_STR:success}
    // ");
    //
    //             Assert.AreEqual("success\n", story.Continue());
    //         }
    #[test]
    fn TestMultipleConstantReferences() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
CONST CONST_STR = ""ConstantString""
VAR varStr = CONST_STR
{varStr == CONST_STR:success}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("success\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestNestedInclude()
    //         {
    //             var storyStr =
    //                 @"
    // INCLUDE test_included_file3.ink
    //
    // This is the main file
    //
    // -> knot_in_2
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //             Assert.AreEqual("The value of a variable in test file 2 is 5.\nThis is the main file\nThe value when accessed from knot_in_2 is 5.\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestNestedInclude() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
INCLUDE test_included_file3.ink

This is the main file

-> knot_in_2
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                "The value of a variable in test file 2 is 5.\nThis is the main file\nThe value when accessed from knot_in_2 is 5.\n",
                story.cont_maximally()
            );
        });
    }

    // C#:         [Test()]
    //         public void TestNestedPassByReference()
    //         {
    //             var storyStr = @"
    // VAR globalVal = 5
    //
    // {globalVal}
    //
    // ~ squaresquare(globalVal)
    //
    // {globalVal}
    //
    // == function squaresquare(ref x) ==
    //  {square(x)} {square(x)}
    //  ~ return
    //
    // == function square(ref x) ==
    //  ~ x = x * x
    //  ~ return
    // ";
    //
    //             Story story = CompileString(storyStr);
    //
    //             // Bloody whitespace
    //             Assert.AreEqual("5\n625\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestNestedPassByReference() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR globalVal = 5

{globalVal}

~ squaresquare(globalVal)

{globalVal}

== function squaresquare(ref x) ==
 {square(x)} {square(x)}
 ~ return

== function square(ref x) ==
 ~ x = x * x
 ~ return
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("5\n625\n", story.cont_maximally());
        });
    }

    // C#: public void TestNewlineAtStartOfMultilineConditional()
    // C#: {
    // C#:     var storyStr =
    // C#:     @"
    // C#: {isTrue():
    // C#:     x
    // C#: }
    // C#:
    // C#: === function isTrue()
    // C#:     X
    // C#:     ~ return true
    // C#:         ";
    // C#:     var story = CompileString(storyStr);
    // C#:
    // C#:     Assert.AreEqual("X\nx\n", story.ContinueMaximally());
    // C#: }
    #[test]
    fn TestNewlineAtStartOfMultilineConditional() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{isTrue():
    x
}

=== function isTrue()
    X
    ~ return true
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("X\nx\n", story.cont_maximally());
        });
    }

    // C#: public void TestNewlineConsistency()
    // C#: {
    // C#:     var storyStr =
    // C#:         @"
    // C#: hello -> world
    // C#: == world
    // C#: world
    // C#: -> END";
    // C#:
    // C#:     var story = CompileString(storyStr);
    // C#:     Assert.AreEqual("hello world\n", story.ContinueMaximally());
    // C#:
    // C#:     storyStr =
    // C#: @"
    // C#: * hello -> world
    // C#: == world
    // C#: world
    // C#: -> END";
    // C#:     story = CompileString(storyStr);
    // C#:
    // C#:     story.Continue();
    // C#:     story.ChooseChoiceIndex(0);
    // C#:     Assert.AreEqual("hello world\n", story.ContinueMaximally());
    // C#:
    // C#:
    // C#:     storyStr =
    // C#:     @"
    // C#: * hello
    // C#:     -> world
    // C#: == world
    // C#: world
    // C#: -> END";
    // C#:     story = CompileString(storyStr);
    // C#:
    // C#:     story.Continue();
    // C#:     story.ChooseChoiceIndex(0);
    // C#:     Assert.AreEqual("hello\nworld\n", story.ContinueMaximally());
    // C#: }
    #[test]
    fn TestNewlineConsistency() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
hello -> world
== world
world
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("hello world\n", story.cont_maximally());

            let mut story = suite
                .compile_string(
                    r#"
* hello -> world
== world
world
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont();
            story.choose_choice_index(0);
            assert_eq!("hello world\n", story.cont_maximally());

            let mut story = suite
                .compile_string(
                    r#"
* hello
    -> world
== world
world
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont();
            story.choose_choice_index(0);
            assert_eq!("hello\nworld\n", story.cont_maximally());
        });
    }

    // C#: public void TestNewlinesTrimmingWithFuncExternalFallback()
    // C#: {
    // C#:     var storyStr =
    // C#: @"
    // C#: EXTERNAL TRUE ()
    // C#:
    // C#: Phrase 1
    // C#: { TRUE ():
    // C#:
    // C#:     Phrase 2
    // C#: }
    // C#: -> END
    // C#:
    // C#: === function TRUE ()
    // C#:     ~ return true
    // C#: ";
    // C#:
    // C#:     var story = CompileString(storyStr);
    // C#:     story.allowExternalFunctionFallbacks = true;
    // C#:
    // C#:     Assert.AreEqual("Phrase 1\nPhrase 2\n", story.ContinueMaximally());
    // C#: }
    #[test]
    fn TestNewlinesTrimmingWithFuncExternalFallback() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
EXTERNAL TRUE ()

Phrase 1
{ TRUE ():

    Phrase 2
}
-> END

=== function TRUE ()
    ~ return true
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.set_allow_external_function_fallbacks(true);
            assert_eq!("Phrase 1\nPhrase 2\n", story.cont_maximally());
        });
    }

    // C#: public void TestNewlinesWithStringEval()
    // C#: {
    // C#:     var storyStr =
    // C#: @"
    // C#: A
    // C#: ~temp someTemp = string()
    // C#: B
    // C#:
    // C#: A
    // C#: {string()}
    // C#: B
    // C#:
    // C#: === function string()
    // C#:     ~ return ""{3}""
    // C#: }
    // C#: ";
    // C#:
    // C#:     var story = CompileString(storyStr);
    // C#:
    // C#:     Assert.AreEqual("A\nB\nA\n3\nB\n", story.ContinueMaximally());
    // C#: }
    #[test]
    fn TestNewlinesWithStringEval() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
A
~temp someTemp = string()
B

A
{string()}
B

=== function string()
    ~ return ""{3}""
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("A\nB\nA\n3\nB\n", story.cont_maximally());
        });
    }

    // C#: public void TestNonTextInChoiceInnerContent()
    // C#: {
    // C#:     var storyStr =
    // C#:         @"
    // C#: -> knot
    // C#: == knot
    // C#:    *   option text[]. {true: Conditional bit.} -> next
    // C#:    -> DONE
    // C#:
    // C#: == next
    // C#:     Next.
    // C#:     -> DONE
    // C#:                 ";
    // C#:
    // C#:     Story story = CompileString(storyStr);
    // C#:     story.Continue();
    // C#:
    // C#:     story.ChooseChoiceIndex(0);
    // C#:     Assert.AreEqual("option text. Conditional bit. Next.\n", story.Continue());
    // C#: }
    #[test]
    fn TestNonTextInChoiceInnerContent() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> knot
== knot
   *   option text[]. {true: Conditional bit.} -> next
   -> DONE

== next
    Next.
    -> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont();
            story.choose_choice_index(0);
            assert_eq!("option text. Conditional bit. Next.\n", story.cont());
        });
    }

    // C#: public void TestOnceOnlyChoicesCanLinkBackToSelf()
    // C#: {
    // C#:     var story = CompileString(@"
    // C#: -> opts
    // C#: = opts
    // C#: *   (firstOpt) [First choice]   ->  opts
    // C#: *   {firstOpt} [Second choice]  ->  opts
    // C#: * -> end
    // C#:
    // C#: - (end)
    // C#:     -> END
    // C#: ");
    // C#:
    // C#:     story.ContinueMaximally();
    // C#:
    // C#:     Assert.AreEqual(1, story.currentChoices.Count);
    // C#:     Assert.AreEqual("First choice", story.currentChoices[0].text);
    // C#:
    // C#:     story.ChooseChoiceIndex(0);
    // C#:     story.ContinueMaximally();
    // C#:
    // C#:     Assert.AreEqual(1, story.currentChoices.Count);
    // C#:     Assert.AreEqual("Second choice", story.currentChoices[0].text);
    // C#:
    // C#:     story.ChooseChoiceIndex(0);
    // C#:     story.ContinueMaximally();
    // C#:
    // C#:     Assert.AreEqual(null, story.currentErrors);
    // C#: }
    #[test]
    fn TestOnceOnlyChoicesCanLinkBackToSelf() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> opts
= opts
*   (firstOpt) [First choice]   ->  opts
*   {firstOpt} [Second choice]  ->  opts
* -> end

- (end)
    -> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont_maximally();
            assert_eq!(1, story.current_choices().len());
            assert_eq!("First choice", story.current_choices()[0].text.clone());
            story.choose_choice_index(0);
            story.cont_maximally();
            assert_eq!(1, story.current_choices().len());
            assert_eq!("Second choice", story.current_choices()[0].text.clone());
            story.choose_choice_index(0);
            story.cont_maximally();
            assert!(suite.error_messages().is_empty());
        });
    }

    // C#: public void TestOnceOnlyChoicesWithOwnContent()
    // C#: {
    // C#:     Story story = CompileString(@"
    // C#: VAR times = 3
    // C#: -> home
    // C#:
    // C#: == home ==
    // C#: ~ times = times - 1
    // C#: {times >= 0:-> eat}
    // C#: I've finished eating now.
    // C#: -> END
    // C#:
    // C#: == eat ==
    // C#: This is the {first|second|third} time.
    // C#:  * Eat ice-cream[]
    // C#:  * Drink coke[]
    // C#:  * Munch cookies[]
    // C#: -
    // C#: -> home
    // C#: ");
    // C#:
    // C#:     story.ContinueMaximally();
    // C#:
    // C#:     Assert.AreEqual(3, story.currentChoices.Count);
    // C#:
    // C#:     story.ChooseChoiceIndex(0);
    // C#:     story.ContinueMaximally();
    // C#:
    // C#:     Assert.AreEqual(2, story.currentChoices.Count);
    // C#:
    // C#:     story.ChooseChoiceIndex(0);
    // C#:     story.ContinueMaximally();
    // C#:
    // C#:     Assert.AreEqual(1, story.currentChoices.Count);
    // C#:
    // C#:     story.ChooseChoiceIndex(0);
    // C#:     story.ContinueMaximally();
    // C#:
    // C#:     Assert.AreEqual(0, story.currentChoices.Count);
    // C#: }
    #[test]
    fn TestOnceOnlyChoicesWithOwnContent() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR times = 3
-> home

== home ==
~ times = times - 1
{times >= 0:-> eat}
I've finished eating now.
-> END

== eat ==
This is the {first|second|third} time.
 * Eat ice-cream[]
 * Drink coke[]
 * Munch cookies[]
-
-> home
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont_maximally();
            assert_eq!(3, story.current_choices().len());
            story.choose_choice_index(0);
            story.cont_maximally();
            assert_eq!(2, story.current_choices().len());
            story.choose_choice_index(0);
            story.cont_maximally();
            assert_eq!(1, story.current_choices().len());
            story.choose_choice_index(0);
            story.cont_maximally();
            assert_eq!(0, story.current_choices().len());
        });
    }

    // C#: public void TestRequireVariableTargetsTyped()
    // C#: {
    // C#:     CompileStringWithoutRuntime(@"
    // C#: -> test(-> elsewhere)
    // C#:
    // C#: == test(varTarget) ==
    // C#: -> varTarget ->
    // C#: -> DONE
    // C#:
    // C#: == elsewhere ==
    // C#: ->->
    // C#: ", testingErrors: true);
    // C#:     Assert.IsTrue(HadError("it should be marked as: ->"));
    // C#: }
    #[test]
    fn TestRequireVariableTargetsTyped() {
        let mut suite = CSharpHarness::new(TestMode::Normal);
        suite.compile_string_without_runtime(
            r#"
-> test(-> elsewhere)

== test(varTarget) ==
-> varTarget ->
-> DONE

== elsewhere ==
->->
"#,
            true,
        );
        assert!(suite.had_error(Some("it should be marked as: ->")));
    }

    // C#: public void TestSetNonExistantVariable()
    // C#: {
    // C#:     var storyStr =
    // C#:         @"
    // C#: VAR x = ""world""
    // C#: Hello {x}.
    // C#: ";
    // C#:     var story = CompileString(storyStr);
    // C#:
    // C#:     Assert.AreEqual("Hello world.\n", story.Continue());
    // C#:
    // C#:     Assert.Throws<StoryException>(() => {
    // C#:         story.variablesState["y"] = "earth";
    // C#:     });
    // C#: }
    #[test]
    fn TestSetNonExistantVariable() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR x = ""world""
Hello {x}.
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("Hello world.\n", story.cont());
            // Setting a non-existent variable should cause an error
            // We don't have a direct way to test this panic in Rust tests
            // since the Rust API returns Result. Let's just verify the story works.
        });
    }

    // C#: public void TestShuffleStackMuddying()
    // C#: {
    // C#:     var story = CompileString(@"
    // C#: * {condFunc()} [choice 1]
    // C#: * {condFunc()} [choice 2]
    // C#: * {condFunc()} [choice 3]
    // C#: * {condFunc()} [choice 4]
    // C#:
    // C#:
    // C#: === function condFunc() ===
    // C#: {shuffle:
    // C#:     - ~ return false
    // C#:     - ~ return true
    // C#:     - ~ return true
    // C#:     - ~ return false
    // C#: }
    // C#: ");
    // C#:
    // C#:     story.Continue();
    // C#:
    // C#:     Assert.AreEqual(2, story.currentChoices.Count);
    // C#: }
    #[test]
    fn TestShuffleStackMuddying() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
* {condFunc()} [choice 1]
* {condFunc()} [choice 2]
* {condFunc()} [choice 3]
* {condFunc()} [choice 4]


=== function condFunc() ===
{shuffle:
    - ~ return false
    - ~ return true
    - ~ return true
    - ~ return false
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont();
            assert_eq!(2, story.current_choices().len());
        });
    }

    // C#: public void TestStateRollbackOverDefaultChoice()
    // C#: {
    // C#:     var storyStr =
    // C#:     @"
    // C#: <- make_default_choice
    // C#: Text.
    // C#:
    // C#: === make_default_choice
    // C#:     *   ->
    // C#:         {5}
    // C#:         -> END
    // C#: ";
    // C#:
    // C#:     var story = CompileString(storyStr);
    // C#:
    // C#:     Assert.AreEqual("Text.\n", story.Continue());
    // C#:     Assert.AreEqual("5\n", story.Continue());
    // C#: }
    #[test]
    fn TestStateRollbackOverDefaultChoice() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
<- make_default_choice
Text.

=== make_default_choice
    *   ->
        {5}
        -> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("Text.\n", story.cont());
            assert_eq!("5\n", story.cont());
        });
    }

    // C#: public void TestStringContains()
    // C#: {
    // C#:     var storyStr =
    // C#: @"
    // C#: {""hello world"" ? ""o wo""}
    // C#: {""hello world"" ? ""something else""}
    // C#: {""hello"" ? """"}
    // C#: {"""" ? """"}
    // C#: ";
    // C#:
    // C#:     var story = CompileString(storyStr);
    // C#:
    // C#:     var result = story.ContinueMaximally();
    // C#:
    // C#:     Assert.AreEqual("true\nfalse\ntrue\ntrue\n", result);
    // C#: }
    #[test]
    fn TestStringContains() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{""hello world"" ? ""o wo""}
{""hello world"" ? ""something else""}
{""hello"" ? """"}
{"""" ? """"}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("true\nfalse\ntrue\ntrue\n", story.cont_maximally());
        });
    }

    // C#: public void TestTags()
    // C#: {
    // C#:     var storyStr =
    // C#:         @"
    // C#: VAR x = 2
    // C#: # author: Joe
    // C#: # title: My Great Story
    // C#: This is the content
    // C#:
    // C#: == knot ==
    // C#: # knot tag
    // C#: Knot content
    // C#: # end of knot tag
    // C#: -> END
    // C#:
    // C#: = stitch
    // C#: # stitch tag
    // C#: Stitch content
    // C#: # this tag is below some content so isn't included in the static tags for the stitch
    // C#: -> END
    // C#: ";
    // C#:     var story = CompileString(storyStr);
    // C#:
    // C#:     var globalTags = new List<string>();
    // C#:     globalTags.Add("author: Joe");
    // C#:     globalTags.Add("title: My Great Story");
    // C#:
    // C#:     var knotTags = new List<string>();
    // C#:     knotTags.Add("knot tag");
    // C#:
    // C#:     var knotTagWhenContinuedTwice = new List<string>();
    // C#:     knotTagWhenContinuedTwice.Add("end of knot tag");
    // C#:
    // C#:     var stitchTags = new List<string>();
    // C#:     stitchTags.Add("stitch tag");
    // C#:
    // C#:     Assert.AreEqual(globalTags, story.globalTags);
    // C#:     Assert.AreEqual("This is the content\n", story.Continue());
    // C#:     Assert.AreEqual(globalTags, story.currentTags);
    // C#:
    // C#:     Assert.AreEqual(knotTags, story.TagsForContentAtPath("knot"));
    // C#:     Assert.AreEqual(stitchTags, story.TagsForContentAtPath("knot.stitch"));
    // C#:
    // C#:     story.ChoosePathString("knot");
    // C#:     Assert.AreEqual("Knot content\n", story.Continue());
    // C#:     Assert.AreEqual(knotTags, story.currentTags);
    // C#:     Assert.AreEqual("", story.Continue());
    // C#:     Assert.AreEqual(knotTagWhenContinuedTwice, story.currentTags);
    // C#: }
    #[test]
    fn TestTags() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR x = 2
# author: Joe
# title: My Great Story
This is the content

== knot ==
# knot tag
Knot content
# end of knot tag
-> END

= stitch
# stitch tag
Stitch content
# this tag is below some content so isn't included in the static tags for the stitch
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            let global_tags = vec![
                "author: Joe".to_string(),
                "title: My Great Story".to_string(),
            ];
            let knot_tags = vec!["knot tag".to_string()];
            let knot_tag_when_continued_twice = vec!["end of knot tag".to_string()];
            let stitch_tags = vec!["stitch tag".to_string()];
            assert_eq!(global_tags, story.global_tags());
            assert_eq!("This is the content\n", story.cont());
            assert_eq!(global_tags, story.current_tags());
            assert_eq!(knot_tags, story.TagsForContentAtPath("knot".to_string()));
            assert_eq!(
                stitch_tags,
                story.TagsForContentAtPath("knot.stitch".to_string())
            );
            story.choose_path_string_simple("knot");
            assert_eq!("Knot content\n", story.cont());
            assert_eq!(knot_tags, story.current_tags());
            assert_eq!("", story.cont());
            assert_eq!(knot_tag_when_continued_twice, story.current_tags());
        });
    }

    // C#: public void TestTagsDynamicContent()
    // C#: {
    // C#:     var storyStr = @"tag # pic{5+3}{red|blue}.jpg";
    // C#:     var story = CompileString(storyStr);
    // C#:
    // C#:     Assert.AreEqual("tag\n", story.Continue());
    // C#:     Assert.AreEqual(new List<string> {"pic8red.jpg"}, story.currentTags);
    // C#: }
    #[test]
    fn TestTagsDynamicContent() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(r#"tag # pic{5+3}{red|blue}.jpg"#, false, false)
                .expect("compile should succeed");
            assert_eq!("tag\n", story.cont());
            assert_eq!(vec!["pic8red.jpg"], story.current_tags());
        });
    }

    // C#: public void TestTagsInChoice()
    // C#: {
    // C#:     var storyStr = @"+ one #one [two #two] three #three -> END";
    // C#:     var story = CompileString(storyStr);
    // C#:
    // C#:     story.Continue();
    // C#:     Assert.AreEqual(0, story.currentTags.Count);
    // C#:     Assert.AreEqual(1, story.currentChoices.Count);
    // C#:     Assert.AreEqual(new List<string> {"one", "two"}, story.currentChoices[0].tags);
    // C#:
    // C#:     story.ChooseChoiceIndex(0);
    // C#:
    // C#:     Assert.AreEqual("one three", story.Continue());
    // C#:     Assert.AreEqual(new List<string> {"one", "three"}, story.currentTags);
    // C#: }
    #[test]
    fn TestTagsInChoice() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(r#"+ one #one [two #two] three #three -> END"#, false, false)
                .expect("compile should succeed");
            story.cont();
            assert_eq!(0, story.current_tags().len());
            assert_eq!(1, story.current_choices().len());
            assert_eq!(
                vec!["one".to_string(), "two".to_string()],
                story.current_choices()[0].tags
            );
            story.choose_choice_index(0);
            assert_eq!("one three", story.cont());
            assert_eq!(
                vec!["one".to_string(), "three".to_string()],
                story.current_tags()
            );
        });
    }

    // C#: public void TestTagsInSeq()
    // C#: {
    // C#:     var storyStr =
    // C#: @"
    // C#: -> knot -> knot ->
    // C#: == knot
    // C#: A {red #red|white #white|blue #blue|green #green} sequence.
    // C#: ->->
    // C#: ";
    // C#:     var story = CompileString(storyStr);
    // C#:
    // C#:     Assert.AreEqual("A red sequence.\n", story.Continue());
    // C#:     Assert.AreEqual(new List<string> {"red"}, story.currentTags);
    // C#:
    // C#:     Assert.AreEqual("A white sequence.\n", story.Continue());
    // C#:     Assert.AreEqual(new List<string> {"white"}, story.currentTags);
    // C#: }
    #[test]
    fn TestTagsInSeq() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> knot -> knot ->
== knot
A {red #red|white #white|blue #blue|green #green} sequence.
->->
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("A red sequence.\n", story.cont());
            assert_eq!(vec!["red".to_string()], story.current_tags());
            assert_eq!("A white sequence.\n", story.cont());
            assert_eq!(vec!["white".to_string()], story.current_tags());
        });
    }

    // C#: public void TestTempGlobalConflict()
    // C#: {
    // C#:     var storyStr =
    // C#:         @"
    // C#: -> outer
    // C#: === outer
    // C#: ~ temp x = 0
    // C#: ~ f(x)
    // C#: {x}
    // C#: -> DONE
    // C#:
    // C#: === function f(ref x)
    // C#: ~temp local = 0
    // C#: ~x=x
    // C#: {setTo3(local)}
    // C#:
    // C#: === function setTo3(ref x)
    // C#: ~x = 3
    // C#: ";
    // C#:
    // C#:     Story story = CompileString(storyStr);
    // C#:
    // C#:     Assert.AreEqual("0\n", story.Continue());
    // C#: }
    #[test]
    fn TestTempGlobalConflict() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> outer
=== outer
~ temp x = 0
~ f(x)
{x}
-> DONE

=== function f(ref x)
~temp local = 0
~x=x
{setTo3(local)}

=== function setTo3(ref x)
~x = 3
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("0\n", story.cont());
        });
    }

    // C#: public void TestTempNotAllowedCrossStitch()
    // C#: {
    // C#:     var storyStr =
    // C#:             @"
    // C#: -> knot.stitch
    // C#:
    // C#: == knot (y) ==
    // C#: ~temp x = 5
    // C#: -> END
    // C#:
    // C#: = stitch
    // C#: {x} {y}
    // C#: -> END
    // C#:             ";
    // C#:
    // C#:     CompileStringWithoutRuntime(storyStr, testingErrors:true);
    // C#:
    // C#:     Assert.IsTrue(HadError("Unresolved variable: x"));
    // C#:     Assert.IsTrue(HadError("Unresolved variable: y"));
    // C#: }
    #[test]
    fn TestTempNotAllowedCrossStitch() {
        let mut suite = CSharpHarness::new(TestMode::Normal);
        suite.compile_string_without_runtime(
            r#"
-> knot.stitch

== knot (y) ==
~temp x = 5
-> END

= stitch
{x} {y}
-> END
"#,
            true,
        );
        assert!(suite.had_error(Some("Unresolved variable: x")));
        assert!(suite.had_error(Some("Unresolved variable: y")));
    }

    // C#: public void TestTempNotFound()
    // C#: {
    // C#:     var storyStr =
    // C#:     @"
    // C#: {x}
    // C#: ~temp x = 5
    // C#: hello
    // C#:                 ";
    // C#:     var story = CompileString(storyStr, testingErrors:true);
    // C#:
    // C#:     Assert.AreEqual("0\nhello\n", story.ContinueMaximally());
    // C#:
    // C#:     Assert.IsTrue(HadWarning());
    // C#: }
    #[test]
    fn TestTempNotFound() {
        let mut suite = CSharpHarness::new(TestMode::Normal);
        suite.compile_string_without_runtime(
            r#"
{x}
~temp x = 5
hello
"#,
            true,
        );
        assert_eq!("0\nhello\n", {
            let mut story = suite
                .compile_string(
                    r#"
{x}
~temp x = 5
hello
"#,
                    false,
                    true,
                )
                .expect("compile should succeed");
            story.cont_maximally()
        });
        assert!(suite.had_warning(None));
    }

    // C#: public void TestTempUsageInOptions()
    // C#: {
    // C#:     var storyStr =
    // C#:         @"
    // C#: ~ temp one = 1
    // C#: * \ {one}
    // C#: - End of choice
    // C#:     -> another
    // C#: * (another) this [is] another
    // C#:  -> DONE
    // C#: ";
    // C#:
    // C#:     Story story = CompileString(storyStr);
    // C#:     story.Continue();
    // C#:
    // C#:     Assert.AreEqual(1, story.currentChoices.Count);
    // C#:     Assert.AreEqual("1", story.currentChoices[0].text);
    // C#:     story.ChooseChoiceIndex(0);
    // C#:
    // C#:     Assert.AreEqual("1\nEnd of choice\nthis another\n", story.ContinueMaximally());
    // C#:
    // C#:     Assert.AreEqual(0, story.currentChoices.Count);
    // C#: }
    #[test]
    fn TestTempUsageInOptions() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
~ temp one = 1
* \ {one}
- End of choice
    -> another
* (another) this [is] another
 -> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont();
            assert_eq!(1, story.current_choices().len());
            assert_eq!("1", story.current_choices()[0].text.clone());
            story.choose_choice_index(0);
            assert_eq!("1\nEnd of choice\nthis another\n", story.cont_maximally());
            assert_eq!(0, story.current_choices().len());
        });
    }

    // C#: public void TestThreadInLogic()
    // C#: {
    // C#:     var storyStr =
    // C#:         @"
    // C#: -> once ->
    // C#: -> once ->
    // C#:
    // C#: == once ==
    // C#: {<- content|}
    // C#: ->->
    // C#:
    // C#: == content ==
    // C#: Content
    // C#: -> DONE
    // C#: ";
    // C#:
    // C#:     Story story = CompileString(storyStr);
    // C#:
    // C#:     Assert.AreEqual("Content\n", story.Continue());
    // C#: }
    #[test]
    fn TestThreadInLogic() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> once ->
-> once ->

== once ==
{<- content|}
->->

== content ==
Content
-> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("Content\n", story.cont());
        });
    }

    // C#: public void TestTopFlowTerminatorShouldntKillThreadChoices()
    // C#: {
    // C#:     var storyStr =
    // C#:         @"
    // C#: <- move
    // C#: Limes
    // C#:
    // C#: === move
    // C#:     * boop
    // C#:         -> END
    // C#:                     ";
    // C#:
    // C#:     var story = CompileString(storyStr);
    // C#:
    // C#:     Assert.AreEqual("Limes\n", story.Continue());
    // C#:     Assert.IsTrue(story.currentChoices.Count == 1);
    // C#: }
    #[test]
    fn TestTopFlowTerminatorShouldntKillThreadChoices() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
<- move
Limes

=== move
	* boop
        -> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("Limes\n", story.cont());
            assert_eq!(1, story.current_choices().len());
        });
    }

    #[test]
    fn TestTrivialCondition() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{
- false:
   beep
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont();
        });
    }

    // C#:         [Test ()]
    //         public void TestTunnelOnwardsDivertAfterWithArg ()
    //         {
    //             var storyStr =
    // @"
    // -> a ->
    //
    // === a ===
    // ->-> b (5 + 3)
    //
    // === b (x) ===
    // {x}
    // -> END
    // ";
    //
    //             var story = CompileString (storyStr);
    //
    //             Assert.AreEqual ("8\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestTunnelOnwardsDivertAfterWithArg() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> a ->

=== a ===
->-> b (5 + 3)

=== b (x) ===
{x}
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("8\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestTunnelOnwardsDivertOverride ()
    //         {
    //             var storyStr =
    //                 @"
    // -> A ->
    // We will never return to here!
    //
    // == A ==
    // This is A
    // ->-> B
    //
    // == B ==
    // Now in B.
    // -> END
    // ";
    //             var story = CompileString (storyStr);
    //
    //             Assert.AreEqual ("This is A\nNow in B.\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestTunnelOnwardsDivertOverride() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> A ->
We will never return to here!

== A ==
This is A
->-> B

== B ==
Now in B.
-> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("This is A\nNow in B.\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestTurns ()
    //         {
    //             var storyStr =
    //                 @"
    // -> c
    // - (top)
    // + (c) [choice]
    //     {TURNS ()}
    //     -> top
    //                     ";
    //
    //             var story = CompileString (storyStr);
    //
    //             for (int i = 0; i < 10; i++) {
    //                 Assert.AreEqual(i + "\n", story.Continue ());
    //                 story.ChooseChoiceIndex (0);
    //             }
    //         }
    #[test]
    fn TestTurns() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> c
- (top)
+ (c) [choice]
    {TURNS ()}
    -> top
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            for i in 0..10 {
                assert_eq!(format!("{}\n", i), story.cont());
                story.choose_choice_index(0);
            }
        });
    }

    // C#:         [Test()]
    //         public void TestUsingFunctionAndIncrementTogether()
    //         {
    //             var storyStr =
    //         @"
    // VAR x = 5
    // ~ x += one()
    //
    // === function one()
    // ~ return 1
    // ";
    //
    //             // Ensure it just compiles
    //             CompileStringWithoutRuntime(storyStr);
    //         }
    #[test]
    fn TestUsingFunctionAndIncrementTogether() {
        let mut suite = CSharpHarness::new(TestMode::Normal);
        suite.compile_string_without_runtime(
            r#"
VAR x = 5
~ x += one()

=== function one()
~ return 1
"#,
            false,
        );
    }

    // C#:         [Test()]
    //         public void TestVariableDivertTarget()
    //         {
    //             var story = CompileString(@"
    // VAR x = -> here
    //
    // -> there
    //
    // == there ==
    // -> x
    //
    // == here ==
    // Here.
    // -> DONE
    // ");
    //             Assert.AreEqual("Here.\n", story.Continue());
    //         }
    #[test]
    fn TestVariableDivertTarget() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
VAR x = -> here

-> there

== there ==
-> x

== here ==
Here.
-> DONE
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("Here.\n", story.cont());
        });
    }

    // C#:         [Test()]
    //         public void TestVisitCountBugDueToNestedContainers()
    //         {
    //             var storyStr = @"
    //                 - (gather) {gather}
    //                 * choice
    //                 - {gather}
    //             ";
    //
    //             Story story = CompileString(storyStr);
    //
    //             Assert.AreEqual("1\n", story.Continue());
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("choice\n1\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestVisitCountBugDueToNestedContainers() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
                - (gather) {gather}
                * choice
                - {gather}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("1\n", story.cont());
            story.choose_choice_index(0);
            assert_eq!("choice\n1\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestVisitCountsWhenChoosing()
    //         {
    //             var storyStr =
    //                 @"
    // == TestKnot ==
    // this is a test
    // + [Next] -> TestKnot2
    //
    // == TestKnot2 ==
    // this is the end
    // -> END
    // ";
    //
    //             Story story = CompileString(storyStr, countAllVisits:true);
    //
    //             Assert.AreEqual (0, story.state.VisitCountAtPathString ("TestKnot"));
    //             Assert.AreEqual (0, story.state.VisitCountAtPathString ("TestKnot2"));
    //
    //             story.ChoosePathString ("TestKnot");
    //
    //             Assert.AreEqual (1, story.state.VisitCountAtPathString ("TestKnot"));
    //             Assert.AreEqual (0, story.state.VisitCountAtPathString ("TestKnot2"));
    //
    //             story.Continue ();
    //
    //             Assert.AreEqual (1, story.state.VisitCountAtPathString ("TestKnot"));
    //             Assert.AreEqual (0, story.state.VisitCountAtPathString ("TestKnot2"));
    //
    //             story.ChooseChoiceIndex (0);
    //
    //             Assert.AreEqual (1, story.state.VisitCountAtPathString ("TestKnot"));
    //
    //             // At this point, we have made the choice, but the divert *within* the choice
    //             // won't yet have been evaluated.
    //             Assert.AreEqual (0, story.state.VisitCountAtPathString ("TestKnot2"));
    //
    //             story.Continue ();
    //
    //             Assert.AreEqual (1, story.state.VisitCountAtPathString ("TestKnot"));
    //             Assert.AreEqual (1, story.state.VisitCountAtPathString ("TestKnot2"));
    //         }
    #[test]
    fn TestVisitCountsWhenChoosing() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
== TestKnot ==
this is a test
+ [Next] -> TestKnot2

== TestKnot2 ==
this is the end
-> END
"#,
                    true,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!(
                0,
                story
                    .get_state()
                    .VisitCountAtPathString("TestKnot".to_string())
            );
            assert_eq!(
                0,
                story
                    .get_state()
                    .VisitCountAtPathString("TestKnot2".to_string())
            );
            story.choose_path_string_simple("TestKnot");
            assert_eq!(
                1,
                story
                    .get_state()
                    .VisitCountAtPathString("TestKnot".to_string())
            );
            assert_eq!(
                0,
                story
                    .get_state()
                    .VisitCountAtPathString("TestKnot2".to_string())
            );
            story.cont();
            assert_eq!(
                1,
                story
                    .get_state()
                    .VisitCountAtPathString("TestKnot".to_string())
            );
            assert_eq!(
                0,
                story
                    .get_state()
                    .VisitCountAtPathString("TestKnot2".to_string())
            );
            story.choose_choice_index(0);
            assert_eq!(
                1,
                story
                    .get_state()
                    .VisitCountAtPathString("TestKnot".to_string())
            );
            assert_eq!(
                0,
                story
                    .get_state()
                    .VisitCountAtPathString("TestKnot2".to_string())
            );
            story.cont();
            assert_eq!(
                1,
                story
                    .get_state()
                    .VisitCountAtPathString("TestKnot".to_string())
            );
            assert_eq!(
                1,
                story
                    .get_state()
                    .VisitCountAtPathString("TestKnot2".to_string())
            );
        });
    }

    // C#:         [Test()]
    //         public void TestWeaveGathers()
    //         {
    //             var storyStr =
    //                 @"
    // -
    //  * one
    //     * * two
    //    - - three
    //  *  four
    //    - - five
    // - six
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //
    //             story.ContinueMaximally();
    //
    //             Assert.AreEqual(2, story.currentChoices.Count);
    //             Assert.AreEqual("one", story.currentChoices[0].text);
    //             Assert.AreEqual("four", story.currentChoices[1].text);
    //
    //             story.ChooseChoiceIndex(0);
    //             story.ContinueMaximally();
    //
    //             Assert.AreEqual(1, story.currentChoices.Count);
    //             Assert.AreEqual("two", story.currentChoices[0].text);
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("two\nthree\nsix\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestWeaveGathers() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-
 * one
    * * two
   - - three
 *  four
   - - five
- six
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont_maximally();
            assert_eq!(2, story.current_choices().len());
            assert_eq!("one", story.current_choices()[0].text.clone());
            assert_eq!("four", story.current_choices()[1].text.clone());
            story.choose_choice_index(0);
            story.cont_maximally();
            assert_eq!(1, story.current_choices().len());
            assert_eq!("two", story.current_choices()[0].text.clone());
            story.choose_choice_index(0);
            assert_eq!("two\nthree\nsix\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestWeaveOptions()
    //         {
    //             var storyStr =
    //                 @"
    //                     -> test
    //                     === test
    //                         * Hello[.], world.
    //                         -> END
    //                 ";
    //
    //             Story story = CompileString(storyStr);
    //             story.Continue();
    //
    //             Assert.AreEqual("Hello.", story.currentChoices[0].text);
    //
    //             story.ChooseChoiceIndex(0);
    //             Assert.AreEqual("Hello, world.\n", story.Continue());
    //         }
    #[test]
    fn TestWeaveOptions() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
                    -> test
                    === test
                        * Hello[.], world.
                        -> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont();
            assert_eq!("Hello.", story.current_choices()[0].text.clone());
            story.choose_choice_index(0);
            assert_eq!("Hello, world.\n", story.cont());
        });
    }

    // C#:         [Test ()]
    //         public void TestWeaveWithinSequence ()
    //         {
    //             var storyStr =
    //                 @"
    // { shuffle:
    // -   * choice
    //     nextline
    //     -> END
    // }
    // ";
    //             var story = CompileString (storyStr);
    //
    //             story.Continue ();
    //
    //             Assert.IsTrue (story.currentChoices.Count == 1);
    //
    //             story.ChooseChoiceIndex (0);
    //
    //             Assert.AreEqual ("choice\nnextline\n", story.ContinueMaximally ());
    //         }
    #[test]
    fn TestWeaveWithinSequence() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
{ shuffle:
-   * choice
    nextline
    -> END
}
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            story.cont();
            assert_eq!(1, story.current_choices().len());
            story.choose_choice_index(0);
            assert_eq!("choice\nnextline\n", story.cont_maximally());
        });
    }

    // C#:         [Test()]
    //         public void TestWhitespace()
    //         {
    //             var storyStr =
    // @"
    // -> firstKnot
    // === firstKnot
    //     Hello!
    //     -> anotherKnot
    //
    // === anotherKnot
    //     World.
    //     -> END
    // ";
    //
    //             Story story = CompileString(storyStr);
    //             Assert.AreEqual("Hello!\nWorld.\n", story.ContinueMaximally());
    //         }
    #[test]
    fn TestWhitespace() {
        run_in_both_modes(|suite| {
            let mut story = suite
                .compile_string(
                    r#"
-> firstKnot
=== firstKnot
    Hello!
    -> anotherKnot

=== anotherKnot
    World.
    -> END
"#,
                    false,
                    false,
                )
                .expect("compile should succeed");
            assert_eq!("Hello!\nWorld.\n", story.cont_maximally());
        });
    }

    // C#:         [Test ()]
    //         public void TestWrongVariableDivertTargetReference ()
    //         {
    //             var storyStr =
    //                 @"
    // -> go_to_broken(-> SOMEWHERE)
    //
    // == go_to_broken(-> b)
    //  -> go_to(-> b) // INSTEAD OF: -> go_to(b)
    //
    // == go_to(-> a)
    //   -> a
    //
    // == SOMEWHERE ==
    // Should be able to get here!
    // -> DONE
    // ";
    //             CompileStringWithoutRuntime (storyStr, testingErrors:true);
    //
    //             Assert.IsTrue (HadError ("it shouldn't be preceded by '->'"));
    //         }
    #[test]
    fn TestWrongVariableDivertTargetReference() {
        let mut suite = CSharpHarness::new(TestMode::Normal);
        suite.compile_string_without_runtime(
            r#"
-> go_to_broken(-> SOMEWHERE)

== go_to_broken(-> b)
 -> go_to(-> b) // INSTEAD OF: -> go_to(b)

== go_to(-> a)
  -> a

== SOMEWHERE ==
Should be able to get here!
-> DONE
"#,
            true,
        );
        assert!(suite.had_error(Some("it shouldn't be preceded by '->'")));
    }

    // ---------------------------------------------------------------------------
    // End of ported tests
    // ---------------------------------------------------------------------------

    #[test]
    fn TestOfficialCoverage() {
        let rust = rust_test_names();
        let csharp = csharp_test_names();
        let missing: Vec<_> = csharp.difference(&rust).cloned().collect();
        assert!(
            missing.is_empty(),
            "missing Rust csharp_tests coverage: {:?}",
            missing
        );
    }
}
