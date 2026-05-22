use std::{collections::HashMap, rc::Rc};

use serde_json::{json, Map};

use crate::{
    container::Container,
    json::{json_read, json_write},
    object::Object,
    path::Path,
    pointer::{self, Pointer},
    push_pop::PushPopType,
    story::Story,
    story_error::StoryError,
    value::Value,
};

#[derive(Clone)]
pub struct Element {
    pub current_pointer: Pointer,
    pub in_expression_evaluation: bool,
    pub temporary_variables: HashMap<String, Rc<Value>>,
    pub push_pop_type: PushPopType,
    pub evaluation_stack_height_when_pushed: usize,
    pub function_start_in_output_stream: i32,
}

impl Element {
    fn new(
        push_pop_type: PushPopType,
        pointer: Pointer,
        in_expression_evaluation: bool,
    ) -> Element {
        Element {
            current_pointer: pointer,
            in_expression_evaluation,
            temporary_variables: HashMap::new(),
            push_pop_type,
            evaluation_stack_height_when_pushed: 0,
            function_start_in_output_stream: 0,
        }
    }
}

#[derive(Clone)]
pub struct Continuation {
    pub callstack: Vec<Element>,
    pub previous_pointer: Pointer,
}

impl Continuation {
    fn new() -> Continuation {
        Continuation {
            callstack: Vec::new(),
            previous_pointer: pointer::NULL.clone(),
        }
    }
}

fn element_from_json(
    main_content_container: &Rc<Container>,
    j_element_obj: &Map<String, serde_json::Value>,
) -> Result<Element, StoryError> {
    let push_pop_type = PushPopType::from_value(
        j_element_obj
            .get("type")
            .and_then(|t| t.as_i64())
            .ok_or(StoryError::BadJson("Invalid push/pop type".to_owned()))? as usize,
    )?;

    let mut pointer = pointer::NULL.clone();

    let current_container_path_str = j_element_obj.get("cPath").and_then(|c| c.as_str());
    if current_container_path_str.is_some() {
        let continuation_pointer_result = main_content_container.content_at_path(
            &Path::new_with_components_string(current_container_path_str),
            0,
            -1,
        );

        pointer.container = continuation_pointer_result.container();
        let pointer_index = j_element_obj
            .get("idx")
            .and_then(|i| i.as_i64())
            .ok_or(StoryError::BadJson("Invalid pointer index".to_owned()))?
            as i32;
        pointer.index = pointer_index;
    }

    let in_expression_evaluation = j_element_obj
        .get("exp")
        .and_then(|exp| exp.as_bool())
        .unwrap_or(false);

    let mut el = Element::new(push_pop_type, pointer, in_expression_evaluation);

    if let Some(temps) = j_element_obj.get("temp").and_then(|temp| temp.as_object()) {
        el.temporary_variables = json_read::jobject_to_hashmap_values(temps)?;
    } else {
        el.temporary_variables.clear();
    }

    Ok(el)
}

fn write_elements_json(elements: &[Element]) -> Result<Vec<serde_json::Value>, StoryError> {
    let mut cs_array: Vec<serde_json::Value> = Vec::new();

    for el in elements.iter() {
        let mut el_map: Map<String, serde_json::Value> = Map::new();

        if !el.current_pointer.is_null() {
            el_map.insert(
                "cPath".to_owned(),
                json!(
                    Object::get_path(el.current_pointer.container.as_ref().unwrap().as_ref())
                        .get_components_string()
                ),
            );
            el_map.insert("idx".to_owned(), json!(el.current_pointer.index));
        }
        el_map.insert("exp".to_owned(), json!(el.in_expression_evaluation));
        el_map.insert("type".to_owned(), json!(el.push_pop_type as u32));

        if !el.temporary_variables.is_empty() {
            el_map.insert(
                "temp".to_owned(),
                json_write::write_dictionary_values(&el.temporary_variables)?,
            );
        }

        cs_array.push(serde_json::Value::Object(el_map));
    }

    Ok(cs_array)
}

#[derive(Clone)]
pub struct CallStack {
    start_of_root: Pointer,
    active_continuation: Continuation,
    legacy_parent_continuation: Option<Continuation>,
}

impl CallStack {
    pub fn new(main_content_container: Rc<Container>) -> CallStack {
        let mut cs = CallStack {
            start_of_root: Pointer::start_of(main_content_container),
            active_continuation: Continuation::new(),
            legacy_parent_continuation: None,
        };

        cs.reset();

        cs
    }

    pub fn get_current_element(&self) -> &Element {
        let cs = &self.active_continuation.callstack;
        cs.last().unwrap()
    }

    pub fn get_current_element_mut(&mut self) -> &mut Element {
        let cs = &mut self.active_continuation.callstack;
        cs.last_mut().unwrap()
    }

    pub fn get_current_element_index(&self) -> i32 {
        self.get_callstack().len() as i32 - 1
    }

    pub fn reset(&mut self) {
        self.active_continuation = Continuation::new();
        self.legacy_parent_continuation = None;
        self.active_continuation.callstack.push(Element::new(
            PushPopType::Tunnel,
            self.start_of_root.clone(),
            false,
        ));
    }

    pub fn can_pop_continuation(&self) -> bool {
        self.legacy_parent_continuation.is_some() && !self.element_is_evaluate_from_game()
    }

    pub fn pop_continuation(&mut self) -> Result<(), StoryError> {
        if self.can_pop_continuation() {
            self.active_continuation = self
                .legacy_parent_continuation
                .take()
                .expect("legacy parent continuation was checked");
            Ok(())
        } else {
            Err(StoryError::InvalidStoryState(
                "Can't pop continuation".to_owned(),
            ))
        }
    }

    pub fn push_continuation(&mut self) {
        self.legacy_parent_continuation = Some(self.active_continuation.clone());
    }

    pub fn can_pop(&self) -> bool {
        self.get_callstack().len() > 1
    }

    pub fn can_pop_type(&self, t: Option<PushPopType>) -> bool {
        if !self.can_pop() {
            return false;
        }

        if t.is_none() {
            return true;
        }

        self.get_current_element().push_pop_type == t.unwrap()
    }

    pub fn pop(&mut self, t: Option<PushPopType>) -> Result<(), StoryError> {
        if self.can_pop_type(t) {
            let l = self.get_callstack().len() - 1;
            self.get_callstack_mut().remove(l);
        } else {
            return Err(StoryError::InvalidStoryState(
                "Mismatched push/pop in Callstack".to_owned(),
            ));
        }

        Ok(())
    }

    pub fn element_is_evaluate_from_game(&self) -> bool {
        self.get_current_element().push_pop_type == PushPopType::FunctionEvaluationFromGame
    }

    pub fn get_elements_mut(&mut self) -> &mut Vec<Element> {
        self.get_callstack_mut()
    }

    pub fn get_callstack(&self) -> &Vec<Element> {
        &self.get_current_continuation().callstack
    }

    pub fn get_callstack_mut(&mut self) -> &mut Vec<Element> {
        &mut self.get_current_continuation_mut().callstack
    }

    pub fn get_current_continuation(&self) -> &Continuation {
        &self.active_continuation
    }

    pub fn get_current_continuation_mut(&mut self) -> &mut Continuation {
        &mut self.active_continuation
    }

    pub fn set_current_continuation(&mut self, value: Continuation) {
        self.active_continuation = value;
        self.legacy_parent_continuation = None;
    }

    pub fn fork_continuation(&mut self) -> Continuation {
        self.get_current_continuation().clone()
    }

    pub fn set_temporary_variable(
        &mut self,
        name: String,
        value: Rc<Value>,
        declare_new: bool,
        context_index: i32,
    ) -> Result<(), StoryError> {
        let callstack_index = self.resolve_temporary_context_index(context_index)?;

        let context_element = self
            .get_callstack_mut()
            .get_mut(callstack_index)
            .expect("temporary context index was checked");

        if !declare_new && !context_element.temporary_variables.contains_key(&name) {
            return Err(StoryError::InvalidStoryState(format!(
                "Could not find temporary variable to set: {}",
                name
            )));
        }

        context_element.temporary_variables.insert(name, value);

        Ok(())
    }

    pub fn context_for_variable_named(&self, name: &str) -> usize {
        // Check if the current temporary context contains the variable.
        if self
            .get_current_element()
            .temporary_variables
            .contains_key(name)
        {
            return (self.get_current_element_index() + 1) as usize;
        }

        // Otherwise, it's a global variable.
        0
    }

    // Get variable value, dereferencing a variable pointer if necessary
    pub fn try_get_temporary_variable_with_name(
        &self,
        name: &str,
        context_index: i32,
    ) -> Result<Option<Rc<Value>>, StoryError> {
        let mut context_index = context_index;
        // contextIndex 0 means global, so index is actually 1-based
        if context_index == -1 {
            context_index = self.get_current_element_index() + 1;
        }

        let context_element = self
            .get_callstack()
            .get(self.resolve_temporary_context_index(context_index)?)
            .expect("temporary context index was checked");
        Ok(context_element.temporary_variables.get(name).cloned())
    }

    fn resolve_temporary_context_index(&self, context_index: i32) -> Result<usize, StoryError> {
        let context_index = if context_index == -1 {
            self.get_current_element_index() + 1
        } else {
            context_index
        };

        if context_index <= 0 {
            return Err(StoryError::InvalidStoryState(format!(
                "Temporary variable context index must be positive, got {context_index}"
            )));
        }

        let callstack_index = usize::try_from(context_index - 1).map_err(|_| {
            StoryError::InvalidStoryState(format!(
                "Temporary variable context index is out of range: {context_index}"
            ))
        })?;
        let callstack_len = self.get_callstack().len();

        if callstack_index >= callstack_len {
            return Err(StoryError::InvalidStoryState(format!(
                "Temporary variable context index {context_index} is outside callstack length {callstack_len}"
            )));
        }

        Ok(callstack_index)
    }

    pub fn push(
        &mut self,
        t: PushPopType,
        external_evaluation_stack_height: usize,
        output_stream_length_with_pushed: i32,
    ) {
        // When pushing to callstack, maintain the current content path, but
        // jump
        // out of expressions by default
        let mut element =
            Element::new(t, self.get_current_element().current_pointer.clone(), false);

        element.evaluation_stack_height_when_pushed = external_evaluation_stack_height;
        element.function_start_in_output_stream = output_stream_length_with_pushed;

        self.get_callstack_mut().push(element);
    }

    pub(crate) fn write_json_minimal(&self) -> Result<serde_json::Value, StoryError> {
        let current = self.get_current_continuation();
        let mut continuation: Map<String, serde_json::Value> = Map::new();

        continuation.insert(
            "frames".to_owned(),
            serde_json::Value::Array(write_elements_json(&current.callstack)?),
        );

        if !current.previous_pointer.is_null() {
            let previous_object = current.previous_pointer.resolve().ok_or_else(|| {
                StoryError::InvalidStoryState(
                    "Continuation previous pointer could not be resolved".to_owned(),
                )
            })?;
            continuation.insert(
                "previousContentObject".to_owned(),
                json!(Object::get_path(previous_object.as_ref()).to_string()),
            );
        }

        Ok(serde_json::Value::Object(continuation))
    }

    pub(crate) fn load_json_minimal(
        &mut self,
        main_content_container: &Rc<Container>,
        j_obj: &Map<String, serde_json::Value>,
    ) -> Result<(), StoryError> {
        let frames = j_obj
            .get("frames")
            .and_then(|frames| frames.as_array())
            .ok_or_else(|| {
                StoryError::BadJson("continuation frames must be an array".to_owned())
            })?;

        let mut continuation = Continuation::new();
        for (index, frame) in frames.iter().enumerate() {
            let frame_obj = frame.as_object().ok_or_else(|| {
                StoryError::BadJson(format!("continuation frames[{index}] must be an object"))
            })?;
            continuation
                .callstack
                .push(element_from_json(main_content_container, frame_obj)?);
        }

        if continuation.callstack.is_empty() {
            return Err(StoryError::BadJson(
                "continuation frames must not be empty".to_owned(),
            ));
        }

        if let Some(prev_content_obj_path) =
            j_obj.get("previousContentObject").and_then(|p| p.as_str())
        {
            let prev_path = Path::new_with_components_string(Some(prev_content_obj_path));
            continuation.previous_pointer =
                Story::pointer_at_path(main_content_container, &prev_path)?;
        }

        self.active_continuation = continuation;
        self.legacy_parent_continuation = None;
        self.start_of_root = Pointer::start_of(main_content_container.clone()).clone();

        Ok(())
    }

    pub fn get_callstack_trace(&self) -> String {
        let mut sb = String::new();

        sb.push_str("=== CONTINUATION (current) ===");

        for element in &self.active_continuation.callstack {
            if element.push_pop_type == PushPopType::Function {
                sb.push_str("  [FUNCTION] ");
            } else {
                sb.push_str("  [TUNNEL] ");
            }

            let pointer = &element.current_pointer;

            if !pointer.is_null() {
                sb.push_str(&format!(
                    "<SOMEWHERE IN {}>\n",
                    pointer.container.as_ref().unwrap().get_path()
                ))
            }
        }

        sb
    }
}
