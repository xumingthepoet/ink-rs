use std::rc::Rc;

use crate::{container::Container, story_error::StoryError};

/// Kept for the existing runtime option name. Compiled story JSON parsing is
/// centralized in `ink-story-json-format`, so the streaming path now shares the
/// same loader instead of maintaining a second token parser.
pub fn load_from_string(s: &str) -> Result<(i32, Rc<Container>), StoryError> {
    super::json_read::load_from_string(s)
}
