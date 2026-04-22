use crate::compiler_conformance::api::{story::Story, story_error::StoryError};

use crate::compiler_conformance::common;

#[test]
fn simple_glue_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/glue/simple-glue.ink");

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);
    assert_eq!(1, text.len());
    assert_eq!("Some content with glue.", text[0]);

    Ok(())
}

#[test]
fn glue_with_divert_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/glue/glue-with-divert.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!(
        "We hurried home to Savile Row as fast as we could.",
        text[0]
    );

    Ok(())
}

#[test]
fn has_left_right_glue_matching_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/glue/left-right-glue-matching.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    assert_eq!("A line.", text[0]);
    assert_eq!("Another line.", text[1]);

    Ok(())
}

#[test]
fn bugfix1_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/glue/testbugfix1.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    assert_eq!("A", text[0]);
    assert_eq!("C", text[1]);

    Ok(())
}

#[test]
fn bugfix2_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/glue/testbugfix2.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    //assert_eq!("A", text[0]);
    assert_eq!("X", text[1]);

    Ok(())
}
