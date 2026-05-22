use crate::{story::Story, story_error::StoryError};

/// # Callstack
/// Methods to work with the story callstack.
impl Story {
    pub(crate) fn reset_callstack(&mut self) -> Result<(), StoryError> {
        self.if_async_we_cant("ResetCallstack")?;

        self.get_state_mut().force_end();

        Ok(())
    }
}
