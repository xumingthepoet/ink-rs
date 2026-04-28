mod support;

use support::{runtime::StoryError, story_runner as common};

#[test]
fn tunnel_onwards_divert_override_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("tunnels/tunnel-onwards-divert-override.ink");

    assert_eq!("This is A\nNow in B.\n", story.continue_maximally());

    Ok(())
}
