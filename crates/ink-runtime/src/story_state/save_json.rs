use serde_json::{json, Map};

use crate::{
    execution_state::ExecutionState, pointer, story::INK_VERSION_CURRENT, story_error::StoryError,
};

use super::StoryState;

pub const INK_SAVE_STATE_VERSION: u32 = 3;

impl StoryState {
    pub fn to_json(&self) -> Result<String, StoryError> {
        Ok(self.write_json()?.to_string())
    }

    pub fn load_json(&mut self, save_string: &str) -> Result<(), StoryError> {
        match serde_json::from_str(save_string) {
            Ok(value) => self.load_json_obj(value),
            Err(_) => Err(StoryError::BadJson("State not in JSON format.".to_owned())),
        }
    }

    fn write_json(&self) -> Result<serde_json::Value, StoryError> {
        self.ensure_minimal_save_ready()?;

        let mut obj: Map<String, serde_json::Value> = Map::new();

        obj.insert(
            "callstack".to_owned(),
            self.current_execution
                .callstack
                .borrow()
                .write_json_minimal()?,
        );
        obj.insert(
            "variablesState".to_owned(),
            self.variables_state.write_json()?,
        );

        obj.insert("storySeed".to_owned(), json!(self.story_seed));
        obj.insert("previousRandom".to_owned(), json!(self.previous_random));

        obj.insert("inkSaveVersion".to_owned(), json!(INK_SAVE_STATE_VERSION));

        // Not using this right now, but could do in future.
        obj.insert("inkFormatVersion".to_owned(), json!(INK_VERSION_CURRENT));

        Ok(serde_json::Value::Object(obj))
    }

    fn ensure_minimal_save_ready(&self) -> Result<(), StoryError> {
        if !self.evaluation_stack.is_empty() {
            return Err(StoryError::InvalidStoryState(
                "Cannot save while expression evaluation is active.".to_owned(),
            ));
        }

        if !self.diverted_pointer.is_null() {
            return Err(StoryError::InvalidStoryState(
                "Cannot save while a divert target is pending.".to_owned(),
            ));
        }

        if self.in_string_evaluation() {
            return Err(StoryError::InvalidStoryState(
                "Cannot save while string generation is active.".to_owned(),
            ));
        }

        Ok(())
    }

    fn load_json_obj(&mut self, j_object: serde_json::Value) -> Result<(), StoryError> {
        let save_version = j_object
            .get("inkSaveVersion")
            .and_then(|version| version.as_u64())
            .ok_or_else(|| {
                StoryError::BadJson("ink save format incorrect, can't load.".to_owned())
            })?;

        if save_version != INK_SAVE_STATE_VERSION as u64 {
            return Err(StoryError::BadJson(format!(
                "ink-rs save format version mismatch: expected {}, got {}.",
                INK_SAVE_STATE_VERSION, save_version
            )));
        }

        let root_obj = j_object
            .as_object()
            .ok_or_else(|| StoryError::BadJson("Invalid save state object".to_string()))?;
        reject_removed_save_fields(root_obj)?;

        let mut loaded_execution = ExecutionState::new(self.main_content_container.clone());
        let callstack_obj = root_obj
            .get("callstack")
            .ok_or_else(|| StoryError::BadJson("Missing callstack".to_string()))?;
        let callstack_obj = callstack_obj
            .as_object()
            .ok_or_else(|| StoryError::BadJson("Invalid callstack object".to_string()))?;
        loaded_execution
            .callstack
            .borrow_mut()
            .load_json_minimal(&self.main_content_container, callstack_obj)?;

        let variables_state_obj = root_obj
            .get("variablesState")
            .ok_or_else(|| StoryError::BadJson("Missing variables state object".to_string()))?;
        let variables_state_obj = variables_state_obj
            .as_object()
            .ok_or_else(|| StoryError::BadJson("Invalid variables state object".to_string()))?;
        let mut loaded_variables_state = self.variables_state.clone();
        loaded_variables_state.load_json(variables_state_obj)?;
        loaded_variables_state.set_callstack(loaded_execution.callstack.clone());

        let story_seed = root_obj
            .get("storySeed")
            .ok_or_else(|| StoryError::BadJson("Missing story seed".to_string()))?;
        let story_seed = story_seed
            .as_i64()
            .ok_or_else(|| StoryError::BadJson("Invalid story seed".to_string()))
            .and_then(|seed| {
                i32::try_from(seed)
                    .map_err(|_| StoryError::BadJson("Invalid story seed".to_string()))
            })?;

        let previous_random_obj = root_obj
            .get("previousRandom")
            .ok_or_else(|| StoryError::BadJson("Missing previous random value".to_string()))?;
        let previous_random = previous_random_obj
            .as_i64()
            .ok_or_else(|| StoryError::BadJson("Invalid previous random value".to_string()))
            .and_then(|random| {
                i32::try_from(random)
                    .map_err(|_| StoryError::BadJson("Invalid previous random value".to_string()))
            })?;

        loaded_execution.output_stream.clear();

        self.current_execution = loaded_execution;
        self.variables_state = loaded_variables_state;
        self.evaluation_stack.clear();
        self.diverted_pointer = pointer::NULL.clone();
        self.story_seed = story_seed;
        self.previous_random = previous_random;
        self.output_stream_dirty();

        Ok(())
    }
}

fn reject_removed_save_fields(root_obj: &Map<String, serde_json::Value>) -> Result<(), StoryError> {
    for field in [
        "currentChoices",
        "generatedChoices",
        "choiceThreads",
        "flows",
        "currentFlowName",
        "evalStack",
        "currentDivertTarget",
        "visitCounts",
        "turnIndices",
        "turnIdx",
        "resumeMode",
    ] {
        if root_obj.contains_key(field) {
            return Err(StoryError::BadJson(format!(
                "Save-state field '{field}' was removed in inkSaveVersion {INK_SAVE_STATE_VERSION}; load expects minimal execution state."
            )));
        }
    }

    Ok(())
}
