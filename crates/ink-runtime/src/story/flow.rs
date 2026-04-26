use crate::{story::Story, story_error::StoryError};

/// # Flow
/// Methods to work with flows and the call-stack.
impl Story {
    pub(crate) fn reset_callstack(&mut self) -> Result<(), StoryError> {
        self.if_async_we_cant("ResetCallstack")?;

        self.get_state_mut().force_end();

        Ok(())
    }
}
