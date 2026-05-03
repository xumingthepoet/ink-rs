use std::{cell::RefCell, rc::Rc};

use serde_json::Map;

use crate::{
    callstack::{CallStack, Thread},
    choice::Choice,
    container::Container,
    json::{json_read, json_write},
    object::RTObject,
    story_error::StoryError,
};

#[derive(Clone)]
pub(crate) struct Flow {
    pub name: String,
    pub callstack: Rc<RefCell<CallStack>>,
    pub output_stream: Vec<Rc<dyn RTObject>>,
    pub current_choices: Vec<Rc<Choice>>,
}

impl Flow {
    pub fn new(name: &str, main_content_container: Rc<Container>) -> Flow {
        Flow {
            name: name.to_string(),
            callstack: Rc::new(RefCell::new(CallStack::new(main_content_container))),
            output_stream: Vec::new(),
            current_choices: Vec::new(),
        }
    }

    pub fn from_json(
        name: &str,
        main_content_container: Rc<Container>,
        j_obj: &Map<String, serde_json::Value>,
    ) -> Result<Flow, StoryError> {
        let current_choices = j_obj
            .get("currentChoices")
            .ok_or_else(|| StoryError::BadJson("currentChoices not found.".to_owned()))?
            .as_array()
            .ok_or_else(|| StoryError::BadJson("currentChoices must be an array".to_owned()))?;
        let current_choices = json_read::jarray_to_runtime_obj_list(current_choices, false)?
            .into_iter()
            .enumerate()
            .map(|(index, object)| {
                object.into_any().downcast::<Choice>().map_err(|_| {
                    StoryError::BadJson(format!("currentChoices[{index}] must be a choice"))
                })
            })
            .collect::<Result<Vec<Rc<Choice>>, StoryError>>()?;

        let mut flow = Self {
            name: name.to_string(),
            callstack: Rc::new(RefCell::new(CallStack::new(main_content_container.clone()))),
            output_stream: Vec::new(),
            current_choices,
        };

        let callstack = j_obj
            .get("callstack")
            .ok_or_else(|| StoryError::BadJson("loading callstack".to_owned()))?
            .as_object()
            .ok_or_else(|| StoryError::BadJson("callstack must be an object".to_owned()))?;
        flow.callstack
            .borrow_mut()
            .load_json(&main_content_container, callstack)?;
        let j_choice_threads = j_obj.get("choiceThreads");

        flow.load_flow_choice_threads(j_choice_threads, main_content_container)?;

        Ok(flow)
    }

    pub(crate) fn write_json(&self) -> Result<serde_json::Value, StoryError> {
        let mut flow: Map<String, serde_json::Value> = Map::new();

        flow.insert(
            "callstack".to_owned(),
            self.callstack.borrow().write_json()?,
        );

        // choiceThreads: optional
        // Has to come BEFORE the choices themselves are written out
        // since the originalThreadIndex of each choice needs to be set
        let mut has_choice_threads = false;
        let mut jct: Map<String, serde_json::Value> = Map::new();
        for c in self.current_choices.iter() {
            let thread_at_generation = c.get_thread_at_generation().ok_or_else(|| {
                StoryError::InvalidStoryState(
                    "Choice is missing thread state from generation time".to_owned(),
                )
            })?;
            c.original_thread_index
                .replace(thread_at_generation.thread_index);

            if self
                .callstack
                .borrow()
                .get_thread_with_index(*c.original_thread_index.borrow())
                .is_none()
            {
                if !has_choice_threads {
                    has_choice_threads = true;
                }

                jct.insert(
                    c.original_thread_index.borrow().to_string(),
                    thread_at_generation.write_json()?,
                );
            }
        }

        if has_choice_threads {
            flow.insert("choiceThreads".to_owned(), serde_json::Value::Object(jct));
        }

        let mut c_array: Vec<serde_json::Value> = Vec::new();
        for c in self.current_choices.iter() {
            c_array.push(json_write::write_choice(c));
        }

        flow.insert(
            "currentChoices".to_owned(),
            serde_json::Value::Array(c_array),
        );

        Ok(serde_json::Value::Object(flow))
    }

    pub fn load_flow_choice_threads(
        &mut self,
        j_choice_threads: Option<&serde_json::Value>,
        main_content_container: Rc<Container>,
    ) -> Result<(), StoryError> {
        let choice_threads = j_choice_threads
            .map(|value| {
                value.as_object().ok_or_else(|| {
                    StoryError::BadJson("choiceThreads must be an object".to_owned())
                })
            })
            .transpose()?;

        for choice in self.current_choices.iter_mut() {
            let original_thread_index = *choice.original_thread_index.borrow();
            let existing_thread = self
                .callstack
                .borrow()
                .get_thread_with_index(original_thread_index)
                .cloned();

            if let Some(thread) = existing_thread {
                choice.set_thread_at_generation(thread);
                continue;
            }

            let original_thread_key = original_thread_index.to_string();
            let j_saved_choice_thread = choice_threads
                .and_then(|threads| threads.get(&original_thread_key))
                .ok_or_else(|| {
                    StoryError::BadJson(format!(
                        "Missing choiceThreads entry for original thread {original_thread_index}"
                    ))
                })?;
            let j_saved_choice_thread = j_saved_choice_thread.as_object().ok_or_else(|| {
                StoryError::BadJson(format!(
                    "choiceThreads['{original_thread_index}'] must be an object"
                ))
            })?;
            choice.set_thread_at_generation(Thread::from_json(
                &main_content_container,
                j_saved_choice_thread,
            )?);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::Path;
    use std::collections::HashMap;

    #[test]
    fn malformed_json_write_choice_without_generation_thread_returns_error() {
        let root = Container::new(None, 0, Vec::new(), HashMap::new());
        let choice = Rc::new(Choice::new_from_json(
            "0",
            "0".to_string(),
            "Choice",
            0,
            0,
            Vec::new(),
        ));
        let flow = Flow {
            name: "default".to_string(),
            callstack: Rc::new(RefCell::new(CallStack::new(root))),
            output_stream: Vec::new(),
            current_choices: vec![choice],
        };

        let error = match flow.write_json() {
            Ok(_) => panic!("missing choice generation thread should fail"),
            Err(error) => error,
        };

        assert!(matches!(
            error,
            StoryError::InvalidStoryState(message)
                if message.contains("Choice is missing thread state")
        ));
    }

    #[test]
    fn valid_flow_choice_generation_thread_still_writes() {
        let root = Container::new(None, 0, Vec::new(), HashMap::new());
        let callstack = CallStack::new(root.clone());
        let choice = Rc::new(Choice::new(
            Path::new_with_defaults(),
            "0".to_string(),
            false,
            Vec::new(),
            callstack.get_current_thread().clone(),
            "Choice".to_string(),
        ));
        let flow = Flow {
            name: "default".to_string(),
            callstack: Rc::new(RefCell::new(callstack)),
            output_stream: Vec::new(),
            current_choices: vec![choice],
        };

        let json = flow.write_json().expect("valid flow should write");
        assert!(json.get("currentChoices").is_some());
    }
}
