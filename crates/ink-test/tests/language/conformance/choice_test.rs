use crate::conformance::api::{Story, StoryError};

use crate::conformance::common;

#[test]
fn no_choice_test() -> Result<(), StoryError> {
    let mut errors: Vec<String> = Vec::new();

    let text = common::run_story(
        "inkfiles/choices/no-choice-text.ink.json",
        None,
        &mut errors,
    );

    assert_eq!(0, errors.len());
    assert_eq!("Hello world!\nHello back!\n", common::join_text(&text));

    Ok(())
}

#[test]
fn one_test() -> Result<(), StoryError> {
    let mut errors: Vec<String> = Vec::new();

    let text = common::run_story("inkfiles/choices/one.ink.json", None, &mut errors);

    assert_eq!(0, errors.len());
    assert_eq!(
        "Hello world!\nHello back!\nHello back!\n",
        common::join_text(&text)
    );

    Ok(())
}

#[test]
fn multi_choice_test() -> Result<(), StoryError> {
    let mut errors: Vec<String> = Vec::new();

    let text = common::run_story(
        "inkfiles/choices/multi-choice.ink.json",
        Some(vec![0]),
        &mut errors,
    );

    assert_eq!(0, errors.len());
    assert_eq!(
        "Hello, world!\nHello back!\nGoodbye\nHello back!\nNice to hear from you\n",
        common::join_text(&text)
    );

    // Select second choice
    let text = common::run_story(
        "inkfiles/choices/multi-choice.ink.json",
        Some(vec![1]),
        &mut errors,
    );

    assert_eq!(0, errors.len());
    assert_eq!(
        "Hello, world!\nHello back!\nGoodbye\nGoodbye\nSee you later\n",
        common::join_text(&text)
    );

    Ok(())
}

#[test]
fn single_choice1_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/choices/single-choice.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("Hello, world!", text[0]);

    Ok(())
}

#[test]
fn single_choic2_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/choices/single-choice.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);
    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    assert_eq!("Hello back!", text[0]);
    assert_eq!("Nice to hear from you", text[1]);

    Ok(())
}

#[test]
fn suppress_choice_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/choices/suppress-choice.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    assert_eq!(
        "Hello back!",
        story.get_current_choices().first().unwrap().text
    );
    story.choose_choice_index(0);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("Nice to hear from you.", text[0]);

    Ok(())
}

#[test]
fn mixed_choice_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/choices/mixed-choice.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    assert_eq!(
        "Hello back!",
        story.get_current_choices().first().unwrap().text
    );
    story.choose_choice_index(0);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    assert_eq!("Hello right back to you!", text[0]);
    assert_eq!("Nice to hear from you.", text[1]);

    Ok(())
}

#[test]
fn sticky_choice_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/choices/sticky-choice.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    assert_eq!(2, story.get_current_choices().len());
    story.choose_choice_index(0);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(2, story.get_current_choices().len());

    Ok(())
}

#[test]
fn fallback_choice_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/choices/fallback-choice.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    assert_eq!(2, story.get_current_choices().len());

    Ok(())
}

#[test]
fn conditional_choice_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/choices/conditional-choice.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    assert_eq!(4, story.get_current_choices().len());

    Ok(())
}
