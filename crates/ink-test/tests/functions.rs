use ink_runtime::story::Story as RuntimeStory;

use crate::support::{
    compiler::{assert_story_output, compile_fixture},
    runtime::{Story, StoryError, ValueType},
    story_runner as common,
};

#[test]
fn fun_basic_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/func-basic.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value of x is 4.4.", text[0]);

    Ok(())
}

#[test]
fn fun_none_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/func-none.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value of x is 3.8.", text[0]);

    Ok(())
}

#[test]
fn fun_inline_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/func-inline.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value of x is 4.4.", text[0]);

    Ok(())
}

#[test]
fn setvar_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/setvar-func.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value is 6.", text[0]);

    Ok(())
}

#[test]
fn interface_dynamic_function_calls_run() {
    let compiled = compile_fixture("functions/interface-dynamic-functions.ink");

    assert_story_output(&compiled, "Score 3.\n");

    let mut story = Story::new(&compiled.json);
    story
        .set_variable("game::route", &ValueType::from("right"))
        .expect("route should accept another implementing module");
    assert_eq!(story.continue_maximally(), "Score 4.\n");
    assert!(
        story.get_current_errors().is_empty(),
        "story should not emit runtime errors: {:#?}",
        story.get_current_errors()
    );

    let mut invalid = RuntimeStory::new(&compiled.json).expect("story should load");
    invalid
        .set_variable("game::route", &ValueType::from("missing"))
        .expect("route is stored as a runtime string");
    let error = invalid
        .continue_maximally()
        .expect_err("invalid route should stop with a runtime error");
    assert!(error
        .to_string()
        .contains("Module missing does not implement dynamic interface IItem"));
}

#[test]
fn complex_func1_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/complex-func1.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The values are 6 and 10.", text[0]);

    Ok(())
}

#[test]
fn complex_func2_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/complex-func2.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The values are -1 and 0 and 1.", text[0]);

    Ok(())
}

#[test]
fn complex_func3_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/complex-func3.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("\"I will pay you 120 reales if you get the goods to their destination. The goods will take up 20 cargo spaces.\"",
    text[0]);

    Ok(())
}

#[test]
fn rnd() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/rnd-func.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(4, text.len());
    assert_eq!("Rolling dice 1: 6.", text[0]);
    assert_eq!("Rolling dice 2: 6.", text[1]);
    assert_eq!("Rolling dice 3: 4.", text[2]);
    assert_eq!("Rolling dice 4: 2.", text[3]);

    Ok(())
}
