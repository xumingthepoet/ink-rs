mod support;

use support::{
    runtime::{Story, StoryError},
    story_runner as common,
};

#[test]
fn tunnel_onwards_divert_override_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("tunnels/tunnel-onwards-divert-override.ink.json");
    let mut story = Story::new(&json_string);

    assert_eq!("This is A\nNow in B.\n", story.continue_maximally());

    Ok(())
}
