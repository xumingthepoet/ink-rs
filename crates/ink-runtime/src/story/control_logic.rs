use crate::{
    container::Container,
    control_command::{CommandType, ControlCommand},
    divert::Divert,
    dynamic_interface::{
        DynamicInterfaceFunctionCall, DynamicInterfaceMemberKind, DynamicInterfaceTarget,
    },
    native_function_call::NativeFunctionCall,
    object::RTObject,
    path::Path,
    pointer,
    push_pop::PushPopType,
    story::Story,
    story_error::StoryError,
    story_state::StoryState,
    tag::Tag,
    value::Value,
    value_type::{StringValue, ValueType},
    variable_assigment::VariableAssignment,
    variable_reference::VariableReference,
    void::Void,
};
use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

/// # Control and Logic
/// Methods for performing logic and flow control.
impl Story {
    pub(crate) fn perform_logic_and_flow_control(
        &mut self,
        content_obj: &Option<Rc<dyn RTObject>>,
    ) -> Result<bool, StoryError> {
        let content_obj = match content_obj {
            Some(content_obj) => content_obj.clone(),
            None => return Ok(false),
        }; // Divert
        if let Ok(current_divert) = content_obj.clone().into_any().downcast::<Divert>() {
            if current_divert.is_conditional {
                let o = self.get_state_mut().pop_evaluation_stack()?;
                if !self.is_truthy(o)? {
                    return Ok(true);
                }
            }

            if current_divert.has_variable_target() {
                let var_name = &current_divert.variable_divert_name;
                if let Some(var_contents) = self
                    .get_state()
                    .variables_state
                    .get_variable_with_name(var_name.as_ref().unwrap(), -1)
                {
                    if let Some(target) = Value::get_value::<&Path>(var_contents.as_ref()) {
                        let p = Self::pointer_at_path(&self.main_content_container, target)?;
                        self.get_state_mut().set_diverted_pointer(p);
                    } else {
                        let error_message = format!(
                            "Tried to divert to a target from a variable, but the variable ({}) didn't contain a divert target, it ",
                            var_name.as_ref().unwrap()
                        );
                        let error_message = if let ValueType::Int(int_content) = var_contents.value
                        {
                            if int_content == 0 {
                                format!("{}was empty/null (the value 0).", error_message)
                            } else {
                                format!("{}contained '{}'.", error_message, var_contents)
                            }
                        } else {
                            error_message
                        };
                        return Err(StoryError::InvalidStoryState(error_message));
                    }
                } else {
                    return Err(StoryError::InvalidStoryState(format!("Tried to divert using a target from a variable that could not be found ({})", var_name.as_ref().unwrap())));
                }
            } else if current_divert.is_external {
                self.call_external_function(
                    &current_divert.get_target_path_string().unwrap(),
                    current_divert.external_args,
                )?;
                return Ok(true);
            } else {
                self.get_state_mut()
                    .set_diverted_pointer(current_divert.get_target_pointer());
            }

            if current_divert.pushes_to_stack {
                self.get_state().get_callstack().borrow_mut().push(
                    current_divert.stack_push_type,
                    0,
                    self.get_state().get_output_stream().len() as i32,
                );
            }

            if self.get_state().diverted_pointer.is_null() && !current_divert.is_external {
                //     error(format!("Divert resolution failed: {:?}",
                // current_divert));
            }

            return Ok(true);
        }

        if let Some(eval_command) = content_obj
            .as_ref()
            .as_any()
            .downcast_ref::<ControlCommand>()
        {
            match eval_command.command_type {
                CommandType::EvalStart => {
                    if self.get_state().get_in_expression_evaluation() {
                        return Err(StoryError::InvalidStoryState(
                            "Already in expression evaluation?".to_owned(),
                        ));
                    }

                    self.get_state().set_in_expression_evaluation(true);
                }
                CommandType::EvalOutput => {
                    // If the expression turned out to be empty, there may not be
                    // anything on the stack
                    if !self.get_state().evaluation_stack.is_empty() {
                        let output = self.get_state_mut().pop_evaluation_stack()?; // Functions may evaluate to Void, in which case we skip
                                                                                   // output
                        if !output.as_ref().as_any().is::<Void>() {
                            // TODO: Should we really always blanket convert to
                            // string?
                            // It would be okay to have numbers in the output stream
                            // the
                            // only problem is when exporting text for viewing, it
                            // skips over numbers etc.
                            let text: Rc<dyn RTObject> =
                                Rc::new(Value::new::<&str>(&output.to_string()));
                            self.get_state_mut().push_to_output_stream(text);
                        }
                    }
                }
                CommandType::EvalEnd => {
                    if !self.get_state().get_in_expression_evaluation() {
                        return Err(StoryError::InvalidStoryState(
                            "Not in expression evaluation mode".to_owned(),
                        ));
                    }
                    self.get_state().set_in_expression_evaluation(false);
                }
                CommandType::Duplicate => {
                    let obj = self
                        .get_state()
                        .peek_evaluation_stack()
                        .cloned()
                        .ok_or_else(|| {
                            StoryError::InvalidStoryState("Evaluation stack underflow".to_owned())
                        })?;
                    self.get_state_mut().push_evaluation_stack(obj);
                }
                CommandType::PopEvaluatedValue => {
                    let _ = self.get_state_mut().pop_evaluation_stack()?;
                }
                CommandType::PopFunction | CommandType::PopTunnel => {
                    let pop_type = if CommandType::PopFunction == eval_command.command_type {
                        PushPopType::Function
                    } else {
                        PushPopType::Tunnel
                    }; // Tunnel onwards is allowed to specify an optional override
                       // divert to go to immediately after returning: ->-> target
                    let mut override_tunnel_return_target = None;
                    if pop_type == PushPopType::Tunnel {
                        let popped = self.get_state_mut().pop_evaluation_stack()?;
                        if let Some(v) = Value::get_value::<&Path>(popped.as_ref()) {
                            override_tunnel_return_target = Some(v.clone());
                        }

                        if override_tunnel_return_target.is_none()
                            && !popped.as_ref().as_any().is::<Void>()
                        {
                            return Err(StoryError::InvalidStoryState(
                                "Expected void if ->-> doesn't override target".to_owned(),
                            ));
                        }
                    }

                    if self
                        .get_state_mut()
                        .try_exit_function_evaluation_from_game()
                    {
                        return Ok(true);
                    } else if self
                        .get_state()
                        .get_callstack()
                        .borrow()
                        .get_current_element()
                        .push_pop_type
                        != pop_type
                        || !self.get_state().get_callstack().borrow().can_pop()
                    {
                        let mut names: HashMap<PushPopType, String> = HashMap::new();
                        names.insert(
                            PushPopType::Function,
                            "function return statement (~ return)".to_owned(),
                        );
                        names.insert(
                            PushPopType::Tunnel,
                            "tunnel onwards statement (->->)".to_owned(),
                        );
                        let mut expected = names
                            .get(
                                &self
                                    .get_state()
                                    .get_callstack()
                                    .borrow()
                                    .get_current_element()
                                    .push_pop_type,
                            )
                            .cloned();
                        if !self.get_state().get_callstack().borrow().can_pop() {
                            expected =
                                Some("natural end, authored choice, or divert target".to_owned());
                        }

                        return Err(StoryError::InvalidStoryState(format!(
                            "Found {}, when expected {}",
                            names.get(&pop_type).unwrap(),
                            expected.unwrap()
                        )));
                    } else {
                        self.get_state_mut().pop_callstack(None)?; // Does tunnel onwards override by diverting to a new ->->
                                                                   // target?
                        if let Some(override_tunnel_return_target) = override_tunnel_return_target {
                            let p = Self::pointer_at_path(
                                &self.main_content_container,
                                &override_tunnel_return_target,
                            )?;
                            self.get_state_mut().set_diverted_pointer(p);
                        }
                    }
                }
                CommandType::BeginString => {
                    self.get_state_mut()
                        .push_to_output_stream(content_obj.clone());
                    if !self.get_state().get_in_expression_evaluation() {
                        return Err(StoryError::InvalidStoryState(
                            "Expected to be in an expression when evaluating a string".to_owned(),
                        ));
                    }

                    self.get_state().set_in_expression_evaluation(false);
                }
                CommandType::EndString => {
                    // Since we're iterating backward through the content,
                    // build a stack so that when we build the string,
                    // it's in the right order
                    let mut content_stack_for_string: VecDeque<Rc<dyn RTObject>> = VecDeque::new();
                    let mut content_to_retain: VecDeque<Rc<dyn RTObject>> = VecDeque::new();
                    let mut output_count_consumed = 0;
                    for i in (0..self.get_state().get_output_stream().len()).rev() {
                        let obj = &self.get_state().get_output_stream()[i];
                        output_count_consumed += 1;
                        if let Some(command) =
                            obj.as_ref().as_any().downcast_ref::<ControlCommand>()
                        {
                            if command.command_type == CommandType::BeginString {
                                break;
                            }
                        }

                        if obj.as_ref().as_any().downcast_ref::<Tag>().is_some() {
                            content_to_retain.push_back(obj.clone());
                        }

                        if Value::get_value::<&StringValue>(obj.as_ref()).is_some() {
                            content_stack_for_string.push_back(obj.clone());
                        }
                    }

                    // Consume the content that was produced for this string
                    self.get_state_mut()
                        .pop_from_output_stream(output_count_consumed); // Rescue the tags that we want actually to keep on the output stack
                                                                        // rather than consume as part of the string we're building.
                                                                        // At the time of writing, this only applies to Tag objects generated
                                                                        // by choices, which are pushed to the stack during string generation.

                    while let Some(rescue_tag) = content_to_retain.pop_back() {
                        self.get_state_mut().push_to_output_stream(rescue_tag);
                    }

                    // Build string out of the content we collected
                    let mut sb = String::new();
                    while let Some(c) = content_stack_for_string.pop_back() {
                        sb.push_str(&c.to_string());
                    }

                    // Return to expression evaluation (from content mode)
                    self.get_state().set_in_expression_evaluation(true);
                    self.get_state_mut()
                        .push_evaluation_stack(Rc::new(Value::new::<&str>(&sb)));
                }
                CommandType::NoOp => {}
                CommandType::ChoiceCount => {
                    let choice_count = self.get_state().get_generated_choices().len();
                    self.get_state_mut()
                        .push_evaluation_stack(Rc::new(Value::new::<i32>(choice_count as i32)));
                }
                CommandType::Turns => {
                    self.get_state_mut()
                        .push_evaluation_stack(Rc::new(Value::new::<i32>(0)));
                }
                CommandType::TurnsSince | CommandType::ReadCount => {
                    let target = self.get_state_mut().pop_evaluation_stack()?;
                    if Value::get_value::<&Path>(target.as_ref()).is_none() {
                        let mut extra_note = "".to_owned();
                        if Value::get_value::<i32>(target.as_ref()).is_some() {
                            extra_note = format!(". Did you accidentally pass a read count ('knot_name') instead of a target {}",
                                    "('-> knot_name')?").to_owned();
                        }

                        return Err(StoryError::InvalidStoryState(format!("TURNS_SINCE expected a divert target (knot, stitch, label name), but saw {} {}", target
                                , extra_note)));
                    }

                    let target = Value::get_value::<&Path>(target.as_ref()).unwrap();
                    let otmp = self.content_at_path(target).correct_obj();
                    let container = match &otmp {
                        Some(o) => o.clone().into_any().downcast::<Container>().ok(),
                        None => None,
                    };
                    let either_count: i32;
                    match container {
                        Some(_) => {
                            either_count = if eval_command.command_type == CommandType::TurnsSince {
                                -1
                            } else {
                                0
                            };
                        }
                        None => {
                            if eval_command.command_type == CommandType::TurnsSince {
                                either_count = -1; // turn count, default to
                                                   // never/unknown
                            } else {
                                either_count = 0;
                            } // visit count, assume 0 to default to allowing entry

                            self.add_error(
                                &format!(
                                    "Failed to find container for {} lookup at {}",
                                    eval_command, target
                                ),
                                true,
                            );
                        }
                    }

                    self.get_state_mut()
                        .push_evaluation_stack(Rc::new(Value::new::<i32>(either_count)));
                }
                CommandType::Random => {
                    let mut max_int = None;
                    let o = self.get_state_mut().pop_evaluation_stack()?;
                    if let Some(v) = Value::get_value::<i32>(o.as_ref()) {
                        max_int = Some(v);
                    }

                    let o = self.get_state_mut().pop_evaluation_stack()?;
                    let mut min_int = None;
                    if let Some(v) = Value::get_value::<i32>(o.as_ref()) {
                        min_int = Some(v);
                    }

                    if min_int.is_none() {
                        return Err(StoryError::InvalidStoryState(
                            "Invalid value for the minimum parameter of RANDOM(min, max)"
                                .to_owned(),
                        ));
                    }

                    if max_int.is_none() {
                        return Err(StoryError::InvalidStoryState(
                            "Invalid value for the maximum parameter of RANDOM(min, max)"
                                .to_owned(),
                        ));
                    }

                    let min_value = min_int.unwrap();
                    let max_value = max_int.unwrap();
                    let random_range = max_value - min_value + 1;
                    if random_range <= 0 {
                        return Err(StoryError::InvalidStoryState(format!(
                            "RANDOM was called with minimum as {} and maximum as {}. The maximum must be larger",
                            min_value, max_value
                        )));
                    }

                    let result_seed = self
                        .get_state()
                        .story_seed
                        .wrapping_add(self.get_state().previous_random);
                    let next_random = super::compiled_story_random_next(result_seed);
                    let chosen_value = (next_random % random_range) + min_value;
                    self.get_state_mut()
                        .push_evaluation_stack(Rc::new(Value::new::<i32>(chosen_value)));
                    self.get_state_mut().previous_random = next_random;
                }
                CommandType::SeedRandom => {
                    let mut seed: Option<i32> = None;
                    let o = self.get_state_mut().pop_evaluation_stack()?;
                    if let Some(v) = Value::get_value::<i32>(o.as_ref()) {
                        seed = Some(v);
                    }

                    if seed.is_none() {
                        return Err(StoryError::InvalidStoryState(
                            "Invalid value passed to SEED_RANDOM".to_owned(),
                        ));
                    }

                    // Story seed affects both RANDOM and shuffle behaviour
                    self.get_state_mut().story_seed = seed.unwrap();
                    self.get_state_mut().previous_random = 0; // SEED_RANDOM returns nothing.
                    self.get_state_mut()
                        .push_evaluation_stack(Rc::new(Void::new()));
                }
                CommandType::VisitIndex => {
                    // Compatibility fallback for compiled-story sequence JSON.
                    // Source sequences are not part of current ink-rs syntax,
                    // and the runtime no longer tracks visits.
                    let count = -1;
                    self.get_state_mut()
                        .push_evaluation_stack(Rc::new(Value::new::<i32>(count)));
                }
                CommandType::SequenceShuffleIndex => {
                    let shuffle_index = self.next_sequence_shuffle_index()?;
                    let v = Rc::new(Value::new::<i32>(shuffle_index));
                    self.get_state_mut().push_evaluation_stack(v);
                }
                CommandType::LegacyStartThread => {
                    // Historical compiled-story compatibility is handled after
                    // the content pointer advances in the main step function.
                }
                CommandType::Done => {
                    // Historical compiled-story JSON may use "done" to leave a
                    // legacy continuation split, or to mark normal flow exit.
                    if self
                        .get_state()
                        .get_callstack()
                        .borrow()
                        .can_pop_continuation()
                    {
                        self.get_state()
                            .get_callstack()
                            .as_ref()
                            .borrow_mut()
                            .pop_continuation()?;
                    }
                    // In normal flow - allow safe exit without warning
                    else {
                        self.get_state_mut().set_did_safe_exit(true); // Stop flow in current continuation
                        self.get_state().set_current_pointer(pointer::NULL.clone());
                    }
                }
                CommandType::End => self.get_state_mut().force_end(),
                CommandType::BeginTag => self
                    .get_state_mut()
                    .push_to_output_stream(content_obj.clone()),
                CommandType::EndTag => {
                    // EndTag has 2 modes:
                    //  - When in string evaluation (for choices)
                    //  - Normal
                    //
                    // The only way you could have an EndTag in the middle of
                    // string evaluation is if we're currently generating text for a
                    // choice, such as:
                    //
                    //   + choice # tag
                    //
                    // In the above case, the ink will be run twice:
                    //  - First, to generate the choice text. String evaluation will be on, and the
                    //    final string will be pushed to the evaluation stack, ready to be popped to
                    //    make a Choice object.
                    //  - Second, when ink generates text after choosing the choice. On this
                    //    ocassion, it's not in string evaluation mode.
                    //
                    // On the writing side, we disallow manually putting tags within
                    // strings like this:
                    //
                    //   {"hello # world"}
                    //
                    // So we know that the tag must be being generated as part of
                    // choice content. Therefore, when the tag has been generated,
                    // we push it onto the evaluation stack in the exact same way
                    // as the string for the choice content.
                    if self.get_state().in_string_evaluation() {
                        let mut content_stack_for_tag: Vec<String> = Vec::new();
                        let mut output_count_consumed = 0;
                        for i in (0..self.get_state().get_output_stream().len()).rev() {
                            let obj = &self.get_state().get_output_stream()[i];
                            output_count_consumed += 1;
                            if let Some(command) =
                                obj.as_ref().as_any().downcast_ref::<ControlCommand>()
                            {
                                if command.command_type == CommandType::BeginTag {
                                    break;
                                } else {
                                    return Err(StoryError::InvalidStoryState("Unexpected ControlCommand while extracting tag from choice".to_owned()));
                                }
                            }

                            if let Some(sv) = Value::get_value::<&StringValue>(obj.as_ref()) {
                                content_stack_for_tag.push(sv.string.clone());
                            }
                        }

                        // Consume the content that was produced for this string
                        self.get_state_mut()
                            .pop_from_output_stream(output_count_consumed);
                        let mut sb = String::new();
                        for str_val in content_stack_for_tag.iter().rev() {
                            sb.push_str(str_val);
                        }

                        let choice_tag =
                            Rc::new(Tag::new(&StoryState::clean_output_whitespace(&sb)));
                        // Pushing to the evaluation stack means it gets picked up
                        // when a Choice is generated from the next Choice Point.
                        self.get_state_mut().push_evaluation_stack(choice_tag);
                    }
                    // Otherwise! Simply push EndTag, so that in the output stream we
                    // have a structure of: [BeginTag, "the tag content", EndTag]
                    else {
                        self.get_state_mut()
                            .push_to_output_stream(content_obj.clone());
                    }
                }
            }

            return Ok(true);
        }

        // Variable assignment
        if let Some(var_ass) = content_obj
            .as_ref()
            .as_any()
            .downcast_ref::<VariableAssignment>()
        {
            let assigned_val = self.get_state_mut().pop_evaluation_stack()?; // When in temporary evaluation, don't create new variables purely
                                                                             // within
                                                                             // the temporary context, but attempt to create them globally
                                                                             // var prioritiseHigherInCallStack = _temporaryEvaluationContainer
                                                                             // != null;
            let assigned_val = assigned_val.into_any().downcast::<Value>().map_err(|_| {
                StoryError::InvalidStoryState(
                    "Variable assignment expected a value on the evaluation stack".to_owned(),
                )
            })?;
            self.get_state_mut()
                .variables_state
                .assign(var_ass, assigned_val)?;
            return Ok(true);
        }

        // Variable reference
        if let Ok(var_ref) = content_obj
            .clone()
            .into_any()
            .downcast::<VariableReference>()
        {
            let found_value: Rc<Value>; // Explicit read count value
            if var_ref.path_for_count.is_some() {
                found_value = Rc::new(Value::new::<i32>(0));
            }
            // Normal variable reference
            else {
                match self
                    .get_state()
                    .variables_state
                    .get_variable_with_name(&var_ref.name, -1)
                {
                    Some(v) => found_value = v,
                    None => {
                        self.add_error(&format!("Variable not found: '{}'. Using default value of 0 (false). This can happen with temporary variables if the declaration hasn't yet been hit. Globals are always given a default value on load if a value doesn't exist in the save state.", var_ref.name), true);
                        found_value = Rc::new(Value::new::<i32>(0));
                    }
                }
            }

            self.get_state_mut().push_evaluation_stack(found_value);
            return Ok(true);
        }

        // Native function call
        if let Some(func) = content_obj
            .as_ref()
            .as_any()
            .downcast_ref::<NativeFunctionCall>()
        {
            let func_params = self
                .get_state_mut()
                .pop_evaluation_stack_multiple(func.get_number_of_parameters())?;
            let result = func.call(func_params)?;
            self.get_state_mut().push_evaluation_stack(result);
            return Ok(true);
        }

        if let Some(target) = content_obj
            .as_ref()
            .as_any()
            .downcast_ref::<DynamicInterfaceTarget>()
        {
            self.evaluate_dynamic_interface_target(target)?;
            return Ok(true);
        }

        if let Some(call) = content_obj
            .as_ref()
            .as_any()
            .downcast_ref::<DynamicInterfaceFunctionCall>()
        {
            self.evaluate_dynamic_interface_function_call(call)?;
            return Ok(true);
        }

        Ok(false)
    }

    fn evaluate_dynamic_interface_target(
        &mut self,
        target: &DynamicInterfaceTarget,
    ) -> Result<(), StoryError> {
        let module =
            self.pop_dynamic_interface_module("target", target.interface(), target.member(), None)?;

        self.validate_dynamic_interface_module_member(
            target.interface(),
            &module,
            target.member(),
            DynamicInterfaceMemberKind::Knot,
        )?;

        let target_path = format!("{}.{}", module, target.member());
        let path = Path::new_with_components_string(Some(&target_path));
        self.dynamic_interface_container(&path, &target_path, target.interface(), target.member())?;

        self.get_state_mut()
            .push_evaluation_stack(Rc::new(Value::new::<Path>(path)));
        Ok(())
    }

    fn evaluate_dynamic_interface_function_call(
        &mut self,
        call: &DynamicInterfaceFunctionCall,
    ) -> Result<(), StoryError> {
        if self.get_state().evaluation_stack.len() <= call.args() {
            return Err(StoryError::InvalidStoryState(format!(
                "Dynamic interface function {}::{} expected {} argument(s) and a module value on the evaluation stack",
                call.interface(),
                call.member(),
                call.args()
            )));
        }

        let module =
            self.pop_dynamic_interface_module("function", call.interface(), call.member(), None)?;
        self.validate_dynamic_interface_module_member(
            call.interface(),
            &module,
            call.member(),
            DynamicInterfaceMemberKind::Function,
        )?;

        let target_path = format!("{}.{}", module, call.member());
        let path = Path::new_with_components_string(Some(&target_path));
        self.dynamic_interface_container(&path, &target_path, call.interface(), call.member())?;

        let pointer = Self::pointer_at_path(&self.main_content_container, &path)?;
        self.get_state_mut().set_diverted_pointer(pointer);

        let evaluation_stack_height = self.get_state().evaluation_stack.len();
        let output_stream_len = self.get_state().get_output_stream().len() as i32;
        self.get_state().get_callstack().borrow_mut().push(
            PushPopType::Function,
            evaluation_stack_height,
            output_stream_len,
        );

        Ok(())
    }

    fn pop_dynamic_interface_module(
        &mut self,
        instruction: &str,
        interface: &str,
        member: &str,
        args: Option<usize>,
    ) -> Result<String, StoryError> {
        let module_value = self.get_state_mut().pop_evaluation_stack()?;
        Value::get_value::<&StringValue>(module_value.as_ref())
            .map(|value| value.string.clone())
            .ok_or_else(|| {
                let args = args
                    .map(|count| format!(" with {count} argument(s)"))
                    .unwrap_or_default();
                StoryError::InvalidStoryState(format!(
                    "Dynamic interface {instruction} {interface}::{member}{args} expected a string module name, but got {module_value}"
                ))
            })
    }

    fn dynamic_interface_container(
        &self,
        path: &Path,
        path_string: &str,
        interface: &str,
        member: &str,
    ) -> Result<Rc<Container>, StoryError> {
        self.content_at_path(path)
            .correct_obj()
            .and_then(|object| object.into_any().downcast::<Container>().ok())
            .ok_or_else(|| {
                StoryError::InvalidStoryState(format!(
                    "Dynamic interface member {interface}::{member} resolved to missing runtime path {path_string}"
                ))
            })
    }

    fn validate_dynamic_interface_module_member(
        &self,
        interface: &str,
        module: &str,
        member: &str,
        expected_kind: DynamicInterfaceMemberKind,
    ) -> Result<(), StoryError> {
        let definition =
            self.validate_dynamic_interface_member(interface, member, expected_kind)?;
        if !definition.implementations().contains(module) {
            return Err(StoryError::InvalidStoryState(format!(
                "Module {module} does not implement dynamic interface {interface}"
            )));
        }
        Ok(())
    }

    fn validate_dynamic_interface_member(
        &self,
        interface: &str,
        member: &str,
        expected_kind: DynamicInterfaceMemberKind,
    ) -> Result<&crate::dynamic_interface::DynamicInterfaceDefinition, StoryError> {
        let definition = self
            .dynamic_interfaces
            .interface(interface)
            .ok_or_else(|| {
                StoryError::InvalidStoryState(format!(
                    "Dynamic interface {interface} is missing from metadata"
                ))
            })?;
        if definition.implementations().is_empty() {
            return Err(StoryError::InvalidStoryState(format!(
                "Dynamic interface {interface} has no compiled implementations"
            )));
        }

        match definition.member_kind(member) {
            Some(kind) if kind == expected_kind => Ok(definition),
            Some(kind) => Err(StoryError::InvalidStoryState(format!(
                "Dynamic interface member {interface}::{member} has kind {kind:?}, expected {expected_kind:?}"
            ))),
            None => Err(StoryError::InvalidStoryState(format!(
                "Dynamic interface member {interface}::{member} is missing from metadata"
            ))),
        }
    }
}

#[cfg(test)]
mod dynamic_interface_tests {
    use std::collections::BTreeMap;

    use ink_story_json_format as format;

    use crate::{story::Story, value_type::ValueType};

    fn dynamic_interface_story_json() -> String {
        use format::{
            Container as FContainer, ControlCommand as C, InterfaceDefinition, InterfaceMemberKind,
            NamedContainer, Object as O, Program,
        };

        let target = FContainer::named(
            "target",
            vec![O::String("Left.\n".to_string()), O::ControlCommand(C::Done)],
        );
        let score = FContainer::named(
            "score",
            vec![
                O::ControlCommand(C::EvalStart),
                O::Int(7),
                O::ControlCommand(C::EvalEnd),
                O::ControlCommand(C::PopFunction),
            ],
        );
        let mut left = FContainer::named("left", Vec::new());
        left.named_content
            .push(NamedContainer::new("target", target));
        left.named_content.push(NamedContainer::new("score", score));

        let global_decl = FContainer::named(
            "global decl",
            vec![
                O::ControlCommand(C::EvalStart),
                O::String("left".to_string()),
                O::GlobalVariableAssignment("route".to_string()),
                O::ControlCommand(C::EvalEnd),
                O::ControlCommand(C::End),
            ],
        );

        let mut root = FContainer::unnamed(vec![
            O::ControlCommand(C::EvalStart),
            O::VariableReference("route".to_string()),
            O::DynamicInterfaceTarget {
                interface: "IItem".to_string(),
                member: "target".to_string(),
            },
            O::VariableAssignment("$divertTarget".to_string()),
            O::ControlCommand(C::EvalEnd),
            O::Divert {
                target: "$divertTarget".to_string(),
                variable: true,
            },
            O::ControlCommand(C::Done),
        ]);
        root.named_content.push(NamedContainer::new("left", left));
        root.named_content
            .push(NamedContainer::new("global decl", global_decl));

        let mut members = BTreeMap::new();
        members.insert("target".to_string(), InterfaceMemberKind::Knot);
        members.insert("score".to_string(), InterfaceMemberKind::Function);

        let mut program = Program::new(root);
        program.interfaces.insert(
            "IItem".to_string(),
            InterfaceDefinition::new(members, vec!["left".to_string()]),
        );
        program.to_json_string().expect("valid story JSON")
    }

    #[test]
    fn dynamic_interface_target_runs_to_implementation_knot() {
        let json = dynamic_interface_story_json();
        let mut story = Story::new(&json).expect("story should load");

        let output = story
            .continue_maximally()
            .expect("story should continue through dynamic target");

        assert_eq!(output, "Left.\n");
        assert!(story.get_current_errors().is_empty());
    }

    #[test]
    fn dynamic_interface_target_rejects_unknown_module_values() {
        let json = dynamic_interface_story_json();
        let mut story = Story::new(&json).expect("story should load");
        story
            .set_variable("route", &ValueType::new("missing"))
            .expect("route variable should be settable");

        let error = story
            .continue_maximally()
            .expect_err("invalid module should stop with a runtime error");

        assert!(error
            .to_string()
            .contains("Module missing does not implement dynamic interface IItem"));
    }

    #[test]
    fn dynamic_interface_function_call_returns_implementation_value() {
        use format::{ControlCommand as C, Object as O, Program};

        let mut program = Program::from_json_str(&dynamic_interface_story_json())
            .expect("base dynamic interface story JSON should parse");
        program.root.content = vec![
            O::ControlCommand(C::EvalStart),
            O::VariableReference("route".to_string()),
            O::DynamicInterfaceFunctionCall {
                interface: "IItem".to_string(),
                member: "score".to_string(),
                args: 0,
            },
            O::ControlCommand(C::EvalOutput),
            O::ControlCommand(C::EvalEnd),
            O::ControlCommand(C::Done),
        ];
        let json = program.to_json_string().expect("valid story JSON");
        let mut story = Story::new(&json).expect("story should load");

        let output = story
            .continue_maximally()
            .expect("story should continue through dynamic function");

        assert_eq!(output, "7");
        assert!(story.get_current_errors().is_empty());
    }
}
