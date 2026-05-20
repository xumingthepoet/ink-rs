use std::{cell::RefCell, rc::Rc};

use crate::{
    callstack::CallStack,
    container::Container,
    flow::Flow,
    object::RTObject,
    pointer::{self, Pointer},
    push_pop::PushPopType,
    state_patch::StatePatch,
    story_error::StoryError,
    variables_state::VariablesState,
};

use rand::Rng;

static DEFAULT_FLOW_NAME: &str = "DEFAULT_FLOW";

mod errors;
mod evaluation_stack;
mod flow_state;
mod function_eval;
mod output;
mod output_mutation;
mod patch;
mod save_json;

pub(crate) struct StoryState {
    pub current_flow: Flow,
    pub did_safe_exit: bool,
    output_stream_text_dirty: bool,
    output_stream_tags_dirty: bool,
    pub variables_state: VariablesState,
    pub evaluation_stack: Vec<Rc<dyn RTObject>>,
    main_content_container: Rc<Container>,
    current_errors: Vec<String>,
    current_warnings: Vec<String>,
    current_text: Option<String>,
    patch: Option<StatePatch>,
    pub diverted_pointer: Pointer,
    pub story_seed: i32,
    pub previous_random: i32,
    current_tags: Vec<String>,
}

impl StoryState {
    pub fn new(main_content_container: Rc<Container>) -> StoryState {
        let current_flow = Flow::new(DEFAULT_FLOW_NAME, main_content_container.clone());
        let callstack = current_flow.callstack.clone();

        let mut rng = rand::rng();
        let story_seed = rng.random_range(0..100);

        let state = StoryState {
            current_flow,
            did_safe_exit: false,
            output_stream_text_dirty: true,
            output_stream_tags_dirty: true,
            variables_state: VariablesState::new(callstack),
            evaluation_stack: Vec::new(),
            main_content_container,
            current_errors: Vec::with_capacity(0),
            current_warnings: Vec::with_capacity(0),
            current_text: None,
            patch: None,
            diverted_pointer: pointer::NULL.clone(),
            story_seed,
            previous_random: 0,
            current_tags: Vec::with_capacity(0),
        };

        state.go_to_start();

        state
    }

    pub fn get_callstack(&self) -> &Rc<RefCell<CallStack>> {
        &self.current_flow.callstack
    }

    pub fn pop_callstack(&mut self, t: Option<PushPopType>) -> Result<(), StoryError> {
        // Add the end of a function call, trim any whitespace from the end.
        if self
            .get_callstack()
            .borrow()
            .get_current_element()
            .push_pop_type
            == PushPopType::Function
        {
            self.trim_whitespace_from_function_end();
        }

        self.get_callstack().borrow_mut().pop(t)
    }

    fn go_to_start(&self) {
        self.get_callstack()
            .as_ref()
            .borrow_mut()
            .get_current_element_mut()
            .current_pointer = Pointer::start_of(self.main_content_container.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use crate::{
        story::Story,
        story_error::StoryError,
        value_type::{DictKey, DictKeyType, DictValue, ValueType},
    };

    const SIMPLE_STORY_JSON: &str = r#"{"inkVersion":1,"root":["done",null]}"#;

    fn simple_save_json() -> serde_json::Value {
        let story = Story::new(SIMPLE_STORY_JSON).expect("valid story");
        serde_json::from_str(&story.save_state().expect("save state")).expect("valid save")
    }

    fn assert_save_load_bad_json(save: serde_json::Value, expected_message: &str) {
        let mut story = Story::new(SIMPLE_STORY_JSON).expect("valid story");
        let error = story
            .load_state(&save.to_string())
            .expect_err("malformed save should be rejected");

        match error {
            StoryError::BadJson(message) => assert!(
                message.contains(expected_message),
                "expected BadJson containing {expected_message:?}, got {message:?}"
            ),
            other => panic!("expected BadJson, got {other:?}"),
        }
    }

    fn choice_save_with_original_thread(
        original_thread_index: usize,
        choice_threads: Option<serde_json::Value>,
    ) -> serde_json::Value {
        let mut save = simple_save_json();
        save["currentChoices"] = json!([{
            "text": "Choice",
            "index": 0,
            "originalChoicePath": "0",
            "originalThreadIndex": original_thread_index,
            "targetPath": "0",
            "tags": []
        }]);
        if let Some(choice_threads) = choice_threads {
            save["choiceThreads"] = choice_threads;
        }
        save
    }

    #[test]
    fn rejects_non_current_save_state_version() {
        let mut story = Story::new(SIMPLE_STORY_JSON).expect("valid story");
        let mut save: serde_json::Value =
            serde_json::from_str(&story.save_state().expect("save state")).expect("valid save");
        save["inkSaveVersion"] = json!(1);

        let error = story
            .load_state(&save.to_string())
            .expect_err("expected save version mismatch");

        assert!(error
            .to_string()
            .contains("Ink save format version mismatch"));
    }

    #[test]
    fn rejects_save_state_without_callstack() {
        let mut story = Story::new(SIMPLE_STORY_JSON).expect("valid story");
        let mut save: serde_json::Value =
            serde_json::from_str(&story.save_state().expect("save state")).expect("valid save");
        save.as_object_mut()
            .expect("save object")
            .remove("callstack");

        let error = story
            .load_state(&save.to_string())
            .expect_err("expected missing callstack error");

        assert!(error.to_string().contains("loading callstack"));
    }

    #[test]
    fn rejects_malformed_flow_save_json_without_panicking() {
        let mut save = simple_save_json();
        save["callstack"] = json!([]);
        assert_save_load_bad_json(save, "callstack must be an object");

        let mut save = simple_save_json();
        save["currentChoices"] = json!({});
        assert_save_load_bad_json(save, "currentChoices must be an array");

        let mut save = simple_save_json();
        save["callstack"]["threads"] = json!({});
        assert_save_load_bad_json(save, "callstack threads must be an array");

        let mut save = simple_save_json();
        save["callstack"]["threadCounter"] = json!("bad");
        assert_save_load_bad_json(save, "threadCounter must be an integer");

        let save = choice_save_with_original_thread(9, Some(json!([])));
        assert_save_load_bad_json(save, "choiceThreads must be an object");

        let save = choice_save_with_original_thread(9, None);
        assert_save_load_bad_json(save, "Missing choiceThreads entry for original thread 9");

        let save = choice_save_with_original_thread(9, Some(json!({ "9": [] })));
        assert_save_load_bad_json(save, "choiceThreads['9'] must be an object");

        let save = choice_save_with_original_thread(9, Some(json!({ "9": { "callstack": [] } })));
        assert_save_load_bad_json(save, "Invalid thread index");
    }

    #[test]
    fn malformed_evaluation_stack_underflow_returns_error() {
        let json = r#"{
            "inkVersion": 1,
            "root": ["ev", {"temp=": "missing"}, "/ev", "done", null]
        }"#;
        let mut story = Story::new(json).expect("malformed story still loads");

        let error = story
            .continue_maximally()
            .expect_err("stack underflow should be reported");

        assert!(matches!(
            error,
            StoryError::InvalidStoryState(message) if message.contains("Evaluation stack underflow")
        ));
    }

    #[test]
    fn malformed_native_call_underflow_returns_error() {
        let json = r#"{
            "inkVersion": 1,
            "root": ["ev", "+", "/ev", "done", null]
        }"#;
        let mut story = Story::new(json).expect("malformed story still loads");

        let error = story
            .continue_maximally()
            .expect_err("native parameter underflow should be reported");

        assert!(matches!(
            error,
            StoryError::InvalidStoryState(message)
                if message.contains("expected 2 value(s), found 0")
        ));
    }

    #[test]
    fn malformed_variable_assignment_non_value_returns_error() {
        let json = r#"{
            "inkVersion": 1,
            "root": ["ev", "void", {"temp=": "missing"}, "/ev", "done", null]
        }"#;
        let mut story = Story::new(json).expect("malformed story still loads");

        let error = story
            .continue_maximally()
            .expect_err("non-value assignment input should be reported");

        assert!(matches!(
            error,
            StoryError::InvalidStoryState(message)
                if message.contains("Variable assignment expected a value")
        ));
    }

    #[test]
    fn save_state_uses_minimal_v2_shape() {
        let story = Story::new(SIMPLE_STORY_JSON).expect("valid story");
        let save: serde_json::Value =
            serde_json::from_str(&story.save_state().expect("save state")).expect("valid save");

        assert_eq!(save["inkSaveVersion"], json!(2));
        assert!(save.get("callstack").is_some());
        assert!(save.get("currentChoices").is_some());
        assert!(save.get("variablesState").is_some());
        assert!(save.get("storySeed").is_some());
        assert!(save.get("previousRandom").is_some());

        for removed_field in [
            "flows",
            "currentFlowName",
            "evalStack",
            "currentDivertTarget",
            "visitCounts",
            "turnIndices",
            "turnIdx",
        ] {
            assert!(
                save.get(removed_field).is_none(),
                "save should not contain removed field {removed_field}"
            );
        }
    }

    #[test]
    fn rejects_save_state_without_previous_random() {
        let mut story = Story::new(SIMPLE_STORY_JSON).expect("valid story");
        let mut save: serde_json::Value =
            serde_json::from_str(&story.save_state().expect("save state")).expect("valid save");
        save.as_object_mut()
            .expect("save object")
            .remove("previousRandom");

        let error = story
            .load_state(&save.to_string())
            .expect_err("expected missing previous random error");

        assert!(error.to_string().contains("Missing previous random value"));
    }

    #[test]
    fn failed_load_state_keeps_existing_story_state() {
        let json = r#"{
            "inkVersion": 1,
            "root": [
                "done",
                {
                    "global decl": [
                        "ev", 0, {"VAR=": "score"}, "/ev",
                        "end",
                        null
                    ]
                }
            ]
        }"#;
        let mut story = Story::new(json).expect("valid story");
        assert!(matches!(
            story.get_variable("score"),
            Some(ValueType::Int(0))
        ));

        let mut save: serde_json::Value =
            serde_json::from_str(&story.save_state().expect("save state")).expect("valid save");
        save["variablesState"]["score"] = json!(7);
        save.as_object_mut()
            .expect("save object")
            .remove("previousRandom");

        let error = story
            .load_state(&save.to_string())
            .expect_err("expected missing previous random error");

        assert!(error.to_string().contains("Missing previous random value"));
        assert!(matches!(
            story.get_variable("score"),
            Some(ValueType::Int(0))
        ));
    }

    #[test]
    fn save_state_roundtrips_array_and_object_variables() {
        let json = r#"{
            "inkVersion": 1,
            "root": [
                "done",
                {
                    "global decl": [
                        "ev", 0, {"VAR=": "items"}, "/ev",
                        "ev", 0, {"VAR=": "player"}, "/ev",
                        "end",
                        null
                    ]
                }
            ]
        }"#;
        let items = ValueType::Array(vec![ValueType::Int(1), ValueType::Bool(true)]);
        let mut player_fields = BTreeMap::new();
        player_fields.insert("items".to_string(), items.clone());
        player_fields.insert("name".to_string(), ValueType::new("Ada"));
        let player = ValueType::Object(player_fields);

        let mut story = Story::new(json).expect("valid story");
        story
            .set_variable("items", &items)
            .expect("array variable should be set");
        story
            .set_variable("player", &player)
            .expect("object variable should be set");

        let save_string = story.save_state().expect("save state");
        let save: serde_json::Value =
            serde_json::from_str(&save_string).expect("save should be JSON");
        assert_eq!(save["variablesState"]["items"], json!([1, true]));
        assert_eq!(
            save["variablesState"]["player"],
            json!({
                "items": [1, true],
                "name": "^Ada"
            })
        );

        let mut reloaded = Story::new(json).expect("valid story");
        reloaded
            .load_state(&save_string)
            .expect("save state should reload");

        assert!(matches!(
            reloaded.get_variable("items"),
            Some(restored) if restored == items
        ));
        assert!(matches!(
            reloaded.get_variable("player"),
            Some(restored) if restored == player
        ));
    }

    #[test]
    fn save_state_roundtrips_dict_variables_and_omits_defaults() {
        let json = r#"{
            "inkVersion": 1,
            "root": [
                "done",
                {
                    "global decl": [
                        "ev", ["dict", "string", []], {"VAR=": "scores"}, "/ev",
                        "end",
                        null
                    ]
                }
            ]
        }"#;
        let scores = ValueType::Dict(
            DictValue::new(
                DictKeyType::String,
                [(
                    DictKey::String("ada".to_string()),
                    ValueType::Array(vec![ValueType::Int(10)]),
                )]
                .into_iter()
                .collect(),
            )
            .expect("valid dict"),
        );

        let mut story = Story::new(json).expect("valid story");
        let fresh_save: serde_json::Value =
            serde_json::from_str(&story.save_state().expect("save state")).expect("valid save");
        assert_eq!(fresh_save["variablesState"], json!({}));

        story
            .set_variable("scores", &scores)
            .expect("dict variable should be set");

        let save_string = story.save_state().expect("save state");
        let save: serde_json::Value =
            serde_json::from_str(&save_string).expect("save should be JSON");
        assert_eq!(
            save["variablesState"]["scores"],
            json!(["dict", "string", [["ada", [10]]]])
        );

        let mut reloaded = Story::new(json).expect("valid story");
        reloaded
            .load_state(&save_string)
            .expect("save state should reload");

        assert!(matches!(
            reloaded.get_variable("scores"),
            Some(restored) if restored == scores
        ));
    }
}
