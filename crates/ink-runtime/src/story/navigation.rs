use crate::{
    container::Container,
    object::{Object, RTObject},
    path::Path,
    pointer::{self, Pointer},
    push_pop::PushPopType,
    search_result::SearchResult,
    story::Story,
    story_error::StoryError,
    value_type::ValueType,
};
use std::rc::Rc;

/// # Navigation
/// Methods to access specific sections of the story.
impl Story {
    pub(crate) fn get_main_content_container(&self) -> Rc<Container> {
        match self.temporary_evaluation_container.as_ref() {
            Some(c) => c.clone(),
            None => self.main_content_container.clone(),
        }
    }

    /// Change the current position of the story to the given path. From
    /// here you can call [`cont()`](Story::cont) to evaluate the
    /// next line.
    ///
    /// The path string is a dot-separated path as used internally by the
    /// engine. These examples should work:
    ///
    /// ```ink
    ///    myKnot
    ///    myKnot.myStitch
    /// ```
    ///
    /// Note however that this won't necessarily work:
    ///
    /// ```ink
    ///    myKnot.myStitch.myLabelledChoice
    /// ```
    ///
    /// ...because of the way that content is nested within a weave
    /// structure.
    ///
    /// Usually you would reset the callstack beforehand, which means that
    /// any tunnels, continuations or functions you were in at the time of
    /// calling will be discarded. This is different from the
    /// behaviour of
    /// [`choose_choice_index`](Story::choose_choice_index), which
    /// will always keep the callstack, since the choices are known to come
    /// from a correct state, and their source continuation is known.
    ///
    /// You have the option of passing `false` to the `reset_callstack`
    /// parameter if you don't want this behaviour, leaving any active
    /// continuations, tunnels or function calls intact.
    ///
    /// Not reseting the call stack is potentially dangerous! If you're in
    /// the middle of a tunnel, it'll redirect only the inner-most
    /// tunnel, meaning that when you tunnel-return using `->->`,
    /// it'll return to where you were before. This may be what you
    /// want though. However, if you're in the middle of a function,
    /// `choose_path_string` will throw an error.
    pub fn choose_path_string(
        &mut self,
        path: &str,
        reset_call_stack: bool,
        args: Option<&Vec<ValueType>>,
    ) -> Result<(), StoryError> {
        self.if_async_we_cant("call ChoosePathString right now")?;

        if reset_call_stack {
            self.reset_callstack()?;
        } else {
            // ChoosePathString is potentially dangerous since you can call it when the
            // stack is
            // pretty much in any state. Let's catch one of the worst offenders.
            if self
                .get_state()
                .get_callstack()
                .borrow()
                .get_current_element()
                .push_pop_type
                == PushPopType::Function
            {
                let mut func_detail = "".to_owned();
                let container = self
                    .get_state()
                    .get_callstack()
                    .borrow()
                    .get_current_element()
                    .current_pointer
                    .container
                    .clone();
                if let Some(container) = container {
                    func_detail = format!("({})", Object::get_path(container.as_ref()));
                }

                return Err(StoryError::InvalidStoryState(format!("Story was running a function {func_detail} when you called ChoosePathString({}) - this is almost certainly not what you want! Full stack trace: \n{}", path, self.get_state().get_callstack().borrow().get_callstack_trace())));
            }
        }

        self.get_state_mut()
            .pass_arguments_to_evaluation_stack(args)?;
        self.choose_path(&Path::new_with_components_string(Some(path)), true)?;

        Ok(())
    }

    /// Evaluates a function defined in ink, and gathers the (possibly
    /// multi-line) text the function produces while executing. This output
    /// text is any text written as normal content within the function,
    /// as opposed to the ink function's return value, which is specified by
    /// `~ return` in the ink.
    pub fn evaluate_function(
        &mut self,
        func_name: &str,
        args: Option<&Vec<ValueType>>,
        text_output: &mut String,
    ) -> Result<Option<ValueType>, StoryError> {
        self.if_async_we_cant("evaluate a function")?;

        if func_name.trim().is_empty() {
            return Err(StoryError::InvalidStoryState(
                "Function is empty or white space.".to_owned(),
            ));
        }

        // Get the content that we need to run
        let func_container = self.knot_container_with_name(func_name);
        if func_container.is_none() {
            let mut e = "Function doesn't exist: '".to_owned();
            e.push_str(func_name);
            e.push('\'');

            return Err(StoryError::BadArgument(e));
        }

        self.evaluate_function_container(func_name, func_container.unwrap(), args, text_output)
    }

    pub(crate) fn evaluate_function_container(
        &mut self,
        _func_name: &str,
        func_container: Rc<Container>,
        args: Option<&Vec<ValueType>>,
        text_output: &mut String,
    ) -> Result<Option<ValueType>, StoryError> {
        // Snapshot the output stream
        let output_stream_before = self.get_state().get_output_stream().clone();
        self.get_state_mut().reset_output(None);

        // State will temporarily replace the callstack in order to evaluate
        self.get_state_mut()
            .start_function_evaluation_from_game(func_container, args)?;

        // Evaluate the function, and collect the string output
        while self.can_continue() {
            let text = self.cont()?;

            text_output.push_str(&text);
        }

        // Restore the output stream in case this was called
        // during main story evaluation.
        self.get_state_mut()
            .reset_output(Some(output_stream_before));

        // Finish evaluation, and see whether anything was produced
        self.get_state_mut()
            .complete_function_evaluation_from_game()
    }

    pub(crate) fn pointer_at_path(
        main_content_container: &Rc<Container>,
        path: &Path,
    ) -> Result<Pointer, StoryError> {
        if path.len() == 0 {
            return Ok(pointer::NULL.clone());
        }

        let mut p = Pointer::default();
        let mut path_length_to_use = path.len() as i32;

        let result: SearchResult =
            if path.get_last_component().unwrap().is_index() {
                path_length_to_use -= 1;
                let result = SearchResult::from_search_result(
                    &main_content_container.content_at_path(path, 0, path_length_to_use),
                );
                p.container = result.container();
                p.index = path.get_last_component().unwrap().index.unwrap() as i32;

                result
            } else {
                let result = SearchResult::from_search_result(
                    &main_content_container.content_at_path(path, 0, -1),
                );
                p.container = result.container();
                p.index = -1;

                result
            };

        let main_container: Rc<dyn RTObject> = main_content_container.clone();

        if Rc::ptr_eq(&result.obj, &main_container) && path_length_to_use > 0 {
            return Err(StoryError::InvalidStoryState(format!(
                "Failed to find content at path '{}', and no approximation of it was possible.",
                path
            )));
        } else if result.approximate {
            // TODO
            // self.add_error(&format!("Failed to find content at path '{}',
            // so it was approximated to: '{}'.", path
            // , result.obj.unwrap().get_path()), true);
        }

        Ok(p)
    }

    pub(crate) fn knot_container_with_name(&self, name: &str) -> Option<Rc<Container>> {
        let named_container = self.main_content_container.named_content.get(name);

        named_container.cloned()
    }

    pub(crate) fn content_at_path(&self, path: &Path) -> SearchResult {
        self.main_content_container.content_at_path(path, 0, -1)
    }
}
