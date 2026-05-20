use std::rc::Rc;

use crate::{
    container::Container,
    control_command::ControlCommand,
    pointer::{self, Pointer},
    push_pop::PushPopType,
    story_error::StoryError,
    value::Value,
    value_type::{StringValue, ValueType},
    void::Void,
};

use super::StoryState;

impl StoryState {
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

    pub(super) fn trim_whitespace_from_function_end(&mut self) {
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
                    ValueType::Dict(v) => Value::new_value_type(ValueType::Dict(v.clone())),
                    _ => {
                        return Err(StoryError::InvalidStoryState("ink arguments when calling EvaluateFunction / ChoosePathStringWithParameters must be \
                        int, float, string, bool, array, object or dict.".to_owned()));
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
            let popped_obj = self.pop_evaluation_stack()?;
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
}
