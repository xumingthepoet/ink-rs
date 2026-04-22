use crate::compiler_conformance::api::{story::Story, story_error::StoryError};

use crate::compiler_conformance::common;

#[test]
fn tunnel_onwards_divert_override_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/tunnels/tunnel-onwards-divert-override.ink");

    assert_eq!("This is A\nNow in B.\n", story.continue_maximally());

    Ok(())
}
