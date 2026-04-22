use crate::compiler_conformance::api::{story::Story, story_error::StoryError};

use crate::compiler_conformance::common;

#[test]
fn single_line_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/knot/single-line.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("Hello, world!", text[0]);

    Ok(())
}

#[test]
fn multi_line_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/knot/multi-line.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(3, text.len());
    assert_eq!("Hello, world!", text[0]);
    assert_eq!("Hello?", text[1]);
    assert_eq!("Hello, are you there?", text[2]);

    Ok(())
}

#[test]
fn strip_empty_lines_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/knot/strip-empty-lines.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(3, text.len());
    assert_eq!("Hello, world!", text[0]);
    assert_eq!("Hello?", text[1]);
    assert_eq!("Hello, are you there?", text[2]);

    Ok(())
}

#[test]
fn param_strings_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/knot/param-strings.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(2);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("\"I accuse myself!\" Poirot declared.", text[0]);

    Ok(())
}

#[test]
fn param_ints_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/knot/param-ints.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(1);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("You give 2 dollars.", text[0]);

    Ok(())
}

#[test]
fn param_floats_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/knot/param-floats.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(1);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("You give 2.5 dollars.", text[0]);

    Ok(())
}

#[test]
fn param_vars_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/knot/param-vars.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(1);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("You give 2 dollars.", text[0]);

    Ok(())
}

#[test]
fn param_multi_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/knot/param-multi.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(0);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("You give 1 or 2 dollars. Hmm.", text[0]);

    Ok(())
}

#[test]
fn param_recurse_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/knot/param-recurse.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    assert_eq!("\"The result is 120!\" you announce.", text[0]);

    Ok(())
}
