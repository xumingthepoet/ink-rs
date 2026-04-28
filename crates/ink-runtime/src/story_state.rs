use std::{cell::RefCell, rc::Rc};

use crate::{
    callstack::CallStack,
    choice::Choice,
    container::Container,
    control_command::{CommandType, ControlCommand},
    flow::Flow,
    glue::Glue,
    object::RTObject,
    path::Path,
    pointer::{self, Pointer},
    push_pop::PushPopType,
    state_patch::StatePatch,
    story::{Story, INK_VERSION_CURRENT},
    story_error::StoryError,
    tag::Tag,
    value::Value,
    value_type::{StringValue, ValueType},
    variables_state::VariablesState,
    void::Void,
};

use rand::Rng;
use serde_json::{json, Map};

pub const INK_SAVE_STATE_VERSION: u32 = 2;

static DEFAULT_FLOW_NAME: &str = "DEFAULT_FLOW";

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

    pub fn can_continue(&self) -> bool {
        !self.get_current_pointer().is_null() && !self.has_error()
    }

    pub fn has_error(&self) -> bool {
        !self.current_errors.is_empty()
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

    pub fn get_callstack(&self) -> &Rc<RefCell<CallStack>> {
        &self.current_flow.callstack
    }

    pub fn set_did_safe_exit(&mut self, did_safe_exit: bool) {
        self.did_safe_exit = did_safe_exit;
    }

    pub fn reset_output(&mut self, objs: Option<Vec<Rc<dyn RTObject>>>) {
        self.get_output_stream_mut().clear();
        if let Some(objs) = objs {
            for o in objs {
                self.get_output_stream_mut().push(o.clone());
            }
        }
        self.output_stream_dirty();
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

    pub fn has_warning(&self) -> bool {
        !self.current_warnings.is_empty()
    }

    pub fn get_current_errors(&self) -> &Vec<String> {
        &self.current_errors
    }

    pub fn get_current_warnings(&self) -> &Vec<String> {
        &self.current_warnings
    }

    pub fn get_output_stream(&self) -> &Vec<Rc<dyn RTObject>> {
        &self.current_flow.output_stream
    }

    fn get_output_stream_mut(&mut self) -> &mut Vec<Rc<dyn RTObject>> {
        &mut self.current_flow.output_stream
    }

    fn output_stream_dirty(&mut self) {
        self.output_stream_text_dirty = true;
        self.output_stream_tags_dirty = true;
    }

    pub fn in_string_evaluation(&self) -> bool {
        for e in self.get_output_stream().iter().rev() {
            if let Some(cmd) = e.as_any().downcast_ref::<ControlCommand>() {
                if cmd.command_type == CommandType::BeginString {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_current_text(&mut self) -> String {
        if self.output_stream_text_dirty {
            let mut sb = String::new();
            let mut in_tag = false;

            for output_obj in self.get_output_stream() {
                let text_content = Value::get_value::<&StringValue>(output_obj.as_ref());

                if let (false, Some(text_content)) = (in_tag, text_content) {
                    sb.push_str(&text_content.string);
                } else if let Some(control_command) = output_obj
                    .as_ref()
                    .as_any()
                    .downcast_ref::<ControlCommand>()
                {
                    if control_command.command_type == CommandType::BeginTag {
                        in_tag = true;
                    } else if control_command.command_type == CommandType::EndTag {
                        in_tag = false;
                    }
                }
            }

            self.current_text = Some(StoryState::clean_output_whitespace(&sb));

            self.output_stream_text_dirty = false;
        }

        self.current_text.as_ref().unwrap().to_string()
    }

    pub fn get_current_tags(&mut self) -> Vec<String> {
        if self.output_stream_tags_dirty {
            self.current_tags.clear();

            let mut in_tag = false;
            let mut sb = String::new();

            for output_obj in self.get_output_stream().clone() {
                if let Some(control_command) = output_obj
                    .as_ref()
                    .as_any()
                    .downcast_ref::<ControlCommand>()
                {
                    match control_command.command_type {
                        CommandType::BeginTag => {
                            if in_tag && !sb.is_empty() {
                                let txt = Self::clean_output_whitespace(&sb);
                                self.current_tags.push(txt);
                                sb.clear();
                            }
                            in_tag = true;
                        }
                        CommandType::EndTag => {
                            if !sb.is_empty() {
                                let txt = Self::clean_output_whitespace(&sb);
                                self.current_tags.push(txt);
                                sb.clear();
                            }
                            in_tag = false;
                        }
                        _ => {}
                    }
                } else if in_tag {
                    if let Some(string_value) =
                        Value::get_value::<&StringValue>(output_obj.as_ref())
                    {
                        sb.push_str(&string_value.string);
                    }
                    if let Some(tag) = output_obj.as_ref().as_any().downcast_ref::<Tag>() {
                        if !tag.get_text().is_empty() {
                            self.current_tags.push(tag.get_text().clone()); // tag.text has whitespace already cleaned
                        }
                    }
                }
            }

            if !sb.is_empty() {
                let txt = Self::clean_output_whitespace(&sb);
                self.current_tags.push(txt);
                sb.clear();
            }

            self.output_stream_tags_dirty = false;
        }

        self.current_tags.clone()
    }

    pub fn clean_output_whitespace(input_str: &str) -> String {
        let mut sb = String::with_capacity(input_str.len());
        let mut current_whitespace_start = -1;
        let mut start_of_line = 0;

        for (i, c) in input_str.chars().enumerate() {
            let is_inline_whitespace = c == ' ' || c == '\t';

            if is_inline_whitespace && current_whitespace_start == -1 {
                current_whitespace_start = i as i32;
            }

            if !is_inline_whitespace {
                if c != '\n'
                    && current_whitespace_start > 0
                    && current_whitespace_start != start_of_line
                {
                    sb.push(' ');
                }
                current_whitespace_start = -1;
            }

            if c == '\n' {
                start_of_line = i as i32 + 1;
            }

            if !is_inline_whitespace {
                sb.push(c);
            }
        }

        sb
    }

    pub fn output_stream_ends_in_newline(&self) -> bool {
        if !self.get_output_stream().is_empty() {
            for e in self.get_output_stream().iter().rev() {
                if e.as_any().is::<ControlCommand>() {
                    break;
                }

                if let Some(val) = e.as_any().downcast_ref::<Value>() {
                    if let ValueType::String(text) = &val.value {
                        if text.is_newline {
                            return true;
                        } else if text.is_non_whitespace() {
                            break;
                        }
                    }
                }
            }
        }

        false
    }

    pub fn set_current_pointer(&self, pointer: Pointer) {
        self.get_callstack()
            .as_ref()
            .borrow_mut()
            .get_current_element_mut()
            .current_pointer = pointer;
    }

    pub fn get_in_expression_evaluation(&self) -> bool {
        self.get_callstack()
            .borrow()
            .get_current_element()
            .in_expression_evaluation
    }

    pub fn set_in_expression_evaluation(&self, value: bool) {
        self.get_callstack()
            .borrow_mut()
            .get_current_element_mut()
            .in_expression_evaluation = value;
    }

    pub fn push_evaluation_stack(&mut self, obj: Rc<dyn RTObject>) {
        self.evaluation_stack.push(obj);
    }

    pub fn push_to_output_stream(&mut self, obj: Rc<dyn RTObject>) {
        let text = Value::get_value::<&StringValue>(obj.as_ref());

        if let Some(s) = text {
            let list_text = StoryState::try_splitting_head_tail_whitespace(&s.string);

            if let Some(list_text) = list_text {
                for text_obj in list_text {
                    self.push_to_output_stream_individual(Rc::new(text_obj));
                }
                self.output_stream_dirty();
                return;
            }
        }

        self.push_to_output_stream_individual(obj);
    }

    pub fn visit_count_for_container(&mut self, _container: &Rc<Container>) -> i32 {
        0
    }

    fn try_splitting_head_tail_whitespace(text: &str) -> Option<Vec<Value>> {
        let mut head_first_newline_idx = -1;
        let mut head_last_newline_idx = -1;
        for (i, c) in text.chars().enumerate() {
            if c == '\n' {
                if head_first_newline_idx == -1 {
                    head_first_newline_idx = i as i32;
                }
                head_last_newline_idx = i as i32;
            } else if c == ' ' || c == '\t' {
                continue;
            } else {
                break;
            }
        }

        let mut tail_last_newline_idx = -1;
        let mut tail_first_newline_idx = -1;
        for (i, c) in text.chars().rev().enumerate() {
            let reversed_i = text.len() as i32 - i as i32 - 1;
            if c == '\n' {
                if tail_last_newline_idx == -1 {
                    tail_last_newline_idx = reversed_i;
                }
                tail_first_newline_idx = reversed_i;
            } else if c == ' ' || c == '\t' {
                continue;
            } else {
                break;
            }
        }

        if head_first_newline_idx == -1 && tail_last_newline_idx == -1 {
            return None;
        }

        let mut list_texts = Vec::new();
        let mut inner_str_start = 0;
        let mut inner_str_end = text.len();

        if head_first_newline_idx != -1 {
            if head_first_newline_idx > 0 {
                let leading_spaces = Value::new::<&str>(&text[0..head_first_newline_idx as usize]);
                list_texts.push(leading_spaces);
            }
            list_texts.push(Value::new::<&str>("\n"));
            inner_str_start = head_last_newline_idx + 1;
        }

        if tail_last_newline_idx != -1 {
            inner_str_end = tail_first_newline_idx as usize;
        }

        if inner_str_end > inner_str_start as usize {
            let inner_str_text = &text[inner_str_start as usize..inner_str_end];
            list_texts.push(Value::new::<&str>(inner_str_text));
        }

        if tail_last_newline_idx != -1 && tail_first_newline_idx > head_last_newline_idx {
            list_texts.push(Value::new::<&str>("\n"));
            if tail_last_newline_idx < text.len() as i32 - 1 {
                let num_spaces = (text.len() as i32 - tail_last_newline_idx) - 1;
                let trailing_spaces = Value::new::<&str>(
                    &text[(tail_last_newline_idx + 1) as usize
                        ..(num_spaces + tail_last_newline_idx + 1) as usize],
                );
                list_texts.push(trailing_spaces);
            }
        }

        Some(list_texts)
    }

    fn push_to_output_stream_individual(&mut self, obj: Rc<dyn RTObject>) {
        let glue = obj.clone().into_any().downcast::<Glue>();
        let text = Value::get_value::<&StringValue>(obj.as_ref());
        let mut include_in_output = true;

        // New glue, so chomp away any whitespace from the end of the stream
        if glue.is_ok() {
            self.trim_newlines_from_output_stream();
            include_in_output = true;
        }
        // New text: do we really want to append it, if it's whitespace?
        // Two different reasons for whitespace to be thrown away:
        // - Function start/end trimming
        // - User-defined glue: <>
        // We also need to know when to stop trimming when there's non-whitespace.
        else if let Some(text) = text {
            let mut function_trim_index = -1;

            {
                // block to release cs borrow
                let cs = self.get_callstack().borrow();
                let curr_el = cs.get_current_element();
                if curr_el.push_pop_type == PushPopType::Function {
                    function_trim_index = curr_el.function_start_in_output_stream;
                }
            }

            let mut glue_trim_index = -1;
            for (i, o) in self.get_output_stream().iter().rev().enumerate() {
                let i = self.get_output_stream().len() - i - 1;
                if let Some(c) = o.as_ref().as_any().downcast_ref::<ControlCommand>() {
                    if c.command_type == CommandType::BeginString {
                        if i as i32 >= function_trim_index {
                            function_trim_index = -1;
                        }

                        break;
                    }
                } else if o.as_ref().as_any().is::<Glue>() {
                    glue_trim_index = i as i32;
                    break;
                }
            }

            let trim_index;
            if glue_trim_index != -1 && function_trim_index != -1 {
                trim_index = function_trim_index.min(glue_trim_index);
            } else if glue_trim_index != -1 {
                trim_index = glue_trim_index;
            } else {
                trim_index = function_trim_index;
            }

            if trim_index != -1 {
                if text.is_newline {
                    include_in_output = false;
                } else if text.is_non_whitespace() {
                    if glue_trim_index > -1 {
                        self.remove_existing_glue();
                    }

                    if function_trim_index > -1 {
                        let mut cs = self.get_callstack().as_ref().borrow_mut();
                        let callstack_elements = cs.get_elements_mut();
                        for i in (0..callstack_elements.len()).rev() {
                            if let Some(el) = callstack_elements.get_mut(i) {
                                if el.push_pop_type == PushPopType::Function {
                                    el.function_start_in_output_stream = -1;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
            } else if text.is_newline
                && (self.output_stream_ends_in_newline() || !self.output_stream_contains_content())
            {
                include_in_output = false;
            }
        }

        if include_in_output {
            self.get_output_stream_mut().push(obj);
            self.output_stream_dirty();
        }
    }

    fn trim_newlines_from_output_stream(&mut self) {
        let mut remove_whitespace_from = -1;
        let output_stream = self.get_output_stream_mut();

        // Work back from the end, and try to find the point where
        // we need to start removing content.
        // - Simply work backwards to find the first newline in a String of
        // whitespace
        // e.g. This is the content \n \n\n
        // ^---------^ whitespace to remove
        // ^--- first while loop stops here
        let mut i = output_stream.len() as i32 - 1;
        while i >= 0 {
            if let Some(obj) = output_stream.get(i as usize) {
                if obj.as_ref().as_any().is::<ControlCommand>() {
                    break;
                } else if let Some(sv) = Value::get_value::<&StringValue>(obj.as_ref()) {
                    if sv.is_non_whitespace() {
                        break;
                    } else if sv.is_newline {
                        remove_whitespace_from = i;
                    }
                }
            }
            i -= 1;
        }

        // Remove the whitespace
        if remove_whitespace_from >= 0 {
            i = remove_whitespace_from;
            while i < output_stream.len() as i32 {
                if Value::get_value::<&StringValue>(output_stream[i as usize].as_ref()).is_some() {
                    output_stream.remove(i as usize);
                } else {
                    i += 1;
                }
            }
        }

        self.output_stream_dirty();
    }

    fn remove_existing_glue(&mut self) {
        let output_stream = self.get_output_stream_mut();

        let mut i = output_stream.len() as i32 - 1;
        while i >= 0 {
            if let Some(c) = output_stream.get(i as usize) {
                if c.as_ref().as_any().is::<Glue>() {
                    output_stream.remove(i as usize);
                } else if c.as_ref().as_any().is::<ControlCommand>() {
                    break;
                }
            }
            i -= 1;
        }

        self.output_stream_dirty();
    }

    fn output_stream_contains_content(&self) -> bool {
        for content in self.get_output_stream() {
            if let Some(v) = content.as_any().downcast_ref::<Value>() {
                if let ValueType::String(_) = v.value {
                    return true;
                }
            }
        }

        false
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

    pub fn try_exit_function_evaluation_from_game(&mut self) -> bool {
        if self
            .get_callstack()
            .borrow()
            .get_current_element()
            .push_pop_type
            == PushPopType::FunctionEvaluationFromGame
        {
            self.set_current_pointer(pointer::NULL.clone());
            self.did_safe_exit = true;
            return true;
        }

        false
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

    pub fn get_current_choices(&self) -> Option<&Vec<Rc<Choice>>> {
        // If we can continue generating text content rather than choices,
        // then we reflect the choice list as being empty, since choices
        // should always come at the end.
        if self.can_continue() {
            return None;
        }

        Some(&self.current_flow.current_choices)
    }

    pub fn copy_and_start_patching(&self, for_background_save: bool) -> StoryState {
        let mut copy = StoryState::new(self.main_content_container.clone());

        copy.patch = Some(self.patch.clone().unwrap_or_else(StatePatch::new));

        // Hijack the new default flow to become a copy of our current one
        // If the patch is applied, then this new flow will replace the old one in
        // _namedFlows
        copy.current_flow.name = self.current_flow.name.clone();
        copy.current_flow.callstack = Rc::new(RefCell::new(
            self.current_flow.callstack.as_ref().borrow().clone(),
        ));
        copy.current_flow.output_stream = self.current_flow.output_stream.clone();
        copy.output_stream_dirty();

        // When background saving we need to make copies of choices since they each have
        // a snapshot of the thread at the time of generation since the game could progress
        // significantly and threads modified during the save process.
        // However, when doing internal saving and restoring of snapshots this isn't an issue,
        // and we can simply ref-copy the choices with their existing threads.
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

        // ref copy - exactly the same variables state!
        // we're expecting not to read it only while in patch mode
        // (though the callstack will be modified)
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
        // VariablesState was being borrowed by the patched
        // state, so restore it with our own callstack.
        // _patch will be null normally, but if you're in the
        // middle of a save, it may contain a _patch for save purpsoes.
        self.variables_state.callstack = self.get_callstack().clone();
        self.variables_state.patch = self.patch.clone(); // usually null
    }

    pub fn apply_any_patch(&mut self) {
        if self.patch.is_none() {
            return;
        }

        self.variables_state.apply_patch();

        self.patch = None;
    }

    pub fn pop_from_output_stream(&mut self, count: usize) {
        let len = self.get_output_stream().len();

        if count <= len {
            let start = len - count;
            self.get_output_stream_mut().drain(start..len);
        }

        self.output_stream_dirty();
    }

    pub fn pop_evaluation_stack(&mut self) -> Rc<dyn RTObject> {
        self.evaluation_stack.pop().unwrap()
    }

    pub fn pop_evaluation_stack_multiple(
        &mut self,
        number_of_objects: usize,
    ) -> Vec<Rc<dyn RTObject>> {
        let start = self.evaluation_stack.len() - number_of_objects;
        let obj: Vec<Rc<dyn RTObject>> = self.evaluation_stack.drain(start..).collect();

        obj
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

    // At the end of a function call, trim any whitespace from the end.
    // We always trim the start and end of the text that a function produces.
    // The start whitespace is discard as it is generated, and the end
    // whitespace is trimmed in one go here when we pop the function.
    fn trim_whitespace_from_function_end(&mut self) {
        assert_eq!(
            self.get_callstack()
                .borrow()
                .get_current_element()
                .push_pop_type,
            PushPopType::Function
        );

        let function_start_point = match self
            .get_callstack()
            .borrow()
            .get_current_element()
            .function_start_in_output_stream
        {
            -1 => 0,
            start_point => start_point,
        };

        // Trim whitespace from END of function call
        let mut i = self.get_output_stream().len() as isize - 1;
        while i >= function_start_point as isize {
            if let Some(obj) = self.get_output_stream().get(i as usize) {
                if obj.as_any().is::<ControlCommand>() {
                    break;
                }

                if let Some(txt) = Value::get_value::<&StringValue>(obj.as_ref()) {
                    if txt.is_newline || txt.is_inline_whitespace {
                        self.get_output_stream_mut().remove(i as usize);
                        self.output_stream_dirty();
                    } else {
                        break;
                    }
                }
            }
            i -= 1;
        }
    }

    pub fn peek_evaluation_stack(&self) -> Option<&Rc<dyn RTObject>> {
        self.evaluation_stack.last()
    }

    pub fn start_function_evaluation_from_game(
        &mut self,
        func_container: Rc<Container>,
        arguments: Option<&Vec<ValueType>>,
    ) -> Result<(), StoryError> {
        self.get_callstack().borrow_mut().push(
            PushPopType::FunctionEvaluationFromGame,
            self.evaluation_stack.len(),
            0,
        );
        self.get_callstack()
            .borrow_mut()
            .get_current_element_mut()
            .current_pointer = Pointer::start_of(func_container);

        self.pass_arguments_to_evaluation_stack(arguments)?;

        Ok(())
    }

    pub fn pass_arguments_to_evaluation_stack(
        &mut self,
        arguments: Option<&Vec<ValueType>>,
    ) -> Result<(), StoryError> {
        // Pass arguments onto the evaluation stack
        if let Some(arguments) = arguments {
            for arg in arguments {
                let value = match arg {
                    ValueType::Bool(v) => Value::new::<bool>(*v),
                    ValueType::Int(v) => Value::new::<i32>(*v),
                    ValueType::Float(v) => Value::new::<f32>(*v),
                    ValueType::String(v) => Value::new::<&str>(&v.string),
                    ValueType::Array(v) => Value::new_value_type(ValueType::Array(v.clone())),
                    ValueType::Object(v) => Value::new_value_type(ValueType::Object(v.clone())),
                    _ => {
                        return Err(StoryError::InvalidStoryState("ink arguments when calling EvaluateFunction / ChoosePathStringWithParameters must be \
                        int, float, string, bool, array or object.".to_owned()));
                    }
                };

                self.push_evaluation_stack(Rc::new(value));
            }
        }

        Ok(())
    }

    pub fn complete_function_evaluation_from_game(
        &mut self,
    ) -> Result<Option<ValueType>, StoryError> {
        if self
            .get_callstack()
            .borrow()
            .get_current_element()
            .push_pop_type
            != PushPopType::FunctionEvaluationFromGame
        {
            return Err(StoryError::InvalidStoryState(format!(
                "Expected external function evaluation to be complete. Stack trace: {}",
                self.get_callstack().borrow().get_callstack_trace()
            )));
        }

        let original_evaluation_stack_height = self
            .get_callstack()
            .borrow()
            .get_current_element()
            .evaluation_stack_height_when_pushed;

        // Do we have a returned value?
        // Potentially pop multiple values off the stack, in case we need
        // to clean up after ourselves (e.g. caller of EvaluateFunction may
        // have passed too many arguments, and we currently have no way to check
        // for that)
        let mut returned_obj = None;
        while self.evaluation_stack.len() > original_evaluation_stack_height {
            let popped_obj = self.pop_evaluation_stack();
            if returned_obj.is_none() {
                returned_obj = Some(popped_obj);
            }
        }

        // Finally, pop the external function evaluation
        self.get_callstack()
            .borrow_mut()
            .pop(Some(PushPopType::FunctionEvaluationFromGame))?;

        // What did we get back?
        if let Some(returned_obj) = returned_obj {
            if returned_obj.as_ref().as_any().is::<Void>() {
                return Ok(None);
            }

            // Some kind of value, if not void
            if let Some(return_val) = returned_obj.as_ref().as_any().downcast_ref::<Value>() {
                // DivertTargets get returned as the string of components
                // (rather than a Path, which isn't public)
                if let ValueType::DivertTarget(p) = &return_val.value {
                    return Ok(Some(ValueType::new::<&str>(&p.to_string())));
                }

                // Other types can just have their exact object type:
                // int, float, string. VariablePointers get returned as strings.
                return Ok(Some(return_val.value.clone()));
            }
        }

        Ok(None)
    }

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

        let flow = self.current_flow.write_json()?;
        let flow = flow
            .as_object()
            .ok_or_else(|| StoryError::BadJson("Invalid flow save data".to_owned()))?;
        obj.insert(
            "callstack".to_owned(),
            flow.get("callstack")
                .ok_or_else(|| StoryError::BadJson("Missing callstack".to_owned()))?
                .clone(),
        );
        obj.insert(
            "currentChoices".to_owned(),
            flow.get("currentChoices")
                .ok_or_else(|| StoryError::BadJson("Missing current choices".to_owned()))?
                .clone(),
        );
        if let Some(choice_threads) = flow.get("choiceThreads") {
            obj.insert("choiceThreads".to_owned(), choice_threads.clone());
        }
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
                "Ink save format version mismatch: expected {}, got {}.",
                INK_SAVE_STATE_VERSION, save_version
            )));
        }

        let root_obj = j_object
            .as_object()
            .ok_or_else(|| StoryError::BadJson("Invalid save state object".to_string()))?;
        self.current_flow = Flow::from_json(
            DEFAULT_FLOW_NAME,
            self.main_content_container.clone(),
            root_obj,
        )?;
        self.output_stream_dirty();

        let variables_state_obj = j_object
            .get("variablesState")
            .ok_or_else(|| StoryError::BadJson("Missing variables state object".to_string()))?;
        self.variables_state.load_json(
            variables_state_obj
                .as_object()
                .ok_or_else(|| StoryError::BadJson("Invalid variables state object".to_string()))?,
        )?;
        self.variables_state
            .set_callstack(self.current_flow.callstack.clone());

        self.evaluation_stack.clear();
        self.diverted_pointer = pointer::NULL.clone();
        self.current_flow.output_stream.clear();
        self.output_stream_dirty();

        let story_seed = j_object
            .get("storySeed")
            .ok_or_else(|| StoryError::BadJson("Missing story seed".to_string()))?;
        self.story_seed = story_seed
            .as_i64()
            .ok_or_else(|| StoryError::BadJson("Invalid story seed".to_string()))?
            as i32;

        let previous_random_obj = j_object
            .get("previousRandom")
            .ok_or_else(|| StoryError::BadJson("Missing previous random value".to_string()))?;
        self.previous_random = previous_random_obj
            .as_i64()
            .ok_or_else(|| StoryError::BadJson("Invalid previous random value".to_string()))?
            as i32;

        Ok(())
    }

    pub(crate) fn add_error(&mut self, message: String, is_warning: bool) {
        if !is_warning {
            self.current_errors.push(message);
        } else {
            self.current_warnings.push(message);
        }
    }

    pub(crate) fn reset_errors(&mut self) {
        self.current_errors.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use crate::{story::Story, story_error::StoryError, value_type::ValueType};

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
}
