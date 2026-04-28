use std::error::Error;

mod support;

use support::{
    runtime::{Story, StoryError, ValueType},
    story_runner as common,
};

#[test]
fn operations_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("misc/operations.ink.json");
    let mut story = Story::new(&json_string);

    assert_eq!(
        "neg:-3\nmod:1\npow:27\nfloor:3\nceiling:4\nint:3\nfloat:1\n",
        &story.continue_maximally()
    );

    Ok(())
}

#[test]
fn issue15_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("misc/issue15.ink.json");
    let mut story = Story::new(&json_string);

    assert_eq!("This is a test\n", story.cont());

    while story.can_continue() {
        // println!(story.buildStringOfHierarchy());
        let line = &story.cont();

        if line.starts_with("SET_X:") {
            let _ = story.set_variable("game::x", &ValueType::String("set".to_string()));
        } else {
            assert_eq!("X is set\n", line);
        }
    }

    Ok(())
}

#[test]
fn newlines_with_string_eval_test() -> Result<(), Box<dyn Error>> {
    let json_string = common::get_json_string("misc/newlines_with_string_eval.ink.json");
    let mut story = Story::new(&json_string);

    assert_eq!("A\nB\nA\n3\nB\n", &story.continue_maximally());

    Ok(())
}

#[test]
fn i18n() -> Result<(), StoryError> {
    let json_string = common::get_json_string("misc/i18n.ink.json");
    let mut story = Story::new(&json_string);

    assert_eq!("áéíóú ñ\n", story.cont());
    assert_eq!("你好\n", story.cont());
    let current_tags = story.get_current_tags();
    assert_eq!(1, current_tags.len());
    assert_eq!("áé", current_tags[0]);
    assert_eq!("你好世界\n", story.cont());

    Ok(())
}
