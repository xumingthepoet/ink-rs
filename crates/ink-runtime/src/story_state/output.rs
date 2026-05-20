use crate::{
    control_command::{CommandType, ControlCommand},
    tag::Tag,
    value::Value,
    value_type::{StringValue, ValueType},
};

use super::StoryState;

impl StoryState {
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
}
