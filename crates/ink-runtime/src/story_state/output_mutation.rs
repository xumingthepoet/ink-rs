use std::rc::Rc;

use crate::{
    control_command::{CommandType, ControlCommand},
    glue::Glue,
    object::RTObject,
    push_pop::PushPopType,
    value::Value,
    value_type::{StringValue, ValueType},
};

use super::StoryState;

impl StoryState {
    pub fn reset_output(&mut self, objs: Option<Vec<Rc<dyn RTObject>>>) {
        self.get_output_stream_mut().clear();
        if let Some(objs) = objs {
            for o in objs {
                self.get_output_stream_mut().push(o.clone());
            }
        }
        self.output_stream_dirty();
    }

    pub fn get_output_stream(&self) -> &Vec<Rc<dyn RTObject>> {
        &self.current_flow.output_stream
    }

    pub(super) fn get_output_stream_mut(&mut self) -> &mut Vec<Rc<dyn RTObject>> {
        &mut self.current_flow.output_stream
    }

    pub(super) fn output_stream_dirty(&mut self) {
        self.output_stream_text_dirty = true;
        self.output_stream_tags_dirty = true;
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

    pub fn pop_from_output_stream(&mut self, count: usize) {
        let len = self.get_output_stream().len();

        if count <= len {
            let start = len - count;
            self.get_output_stream_mut().drain(start..len);
        }

        self.output_stream_dirty();
    }
}
