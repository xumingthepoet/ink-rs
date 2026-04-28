use crate::conformance::api::{Story, StoryError};

use crate::conformance::common;

#[test]
fn iftrue_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/conditional/iftrue.ink.json");
    let mut story = Story::new(&json_string);
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value is 1.", text[0]);

    Ok(())
}

#[test]
fn iffalse_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/conditional/iffalse.ink.json");
    let mut story = Story::new(&json_string);
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value is 3.", text[0]);

    Ok(())
}

#[test]
fn ifelse_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/conditional/ifelse.ink.json");
    let mut story = Story::new(&json_string);
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value is 1.", text[0]);

    Ok(())
}

#[test]
fn ifelse_ext_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/conditional/ifelse-ext.ink.json");
    let mut story = Story::new(&json_string);
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value is -1.", text[0]);

    Ok(())
}

#[test]
fn ifelse_ext_text1_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/conditional/ifelse-ext-text1.ink.json");
    let mut story = Story::new(&json_string);
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("This is text 1.", text[0]);
    assert_eq!(1, story.get_current_choices().len());
    story.choose_choice_index(0);

    common::next_all(&mut story, &mut text);
    assert_eq!(2, text.len());
    assert_eq!("This is the end.", text[1]);

    Ok(())
}

#[test]
fn ifelse_ext_text2_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/conditional/ifelse-ext-text2.ink.json");
    let mut story = Story::new(&json_string);
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("This is text 2.", text[0]);
    assert_eq!(1, story.get_current_choices().len());
    story.choose_choice_index(0);

    common::next_all(&mut story, &mut text);
    assert_eq!(2, text.len());
    assert_eq!("This is the end.", text[1]);

    Ok(())
}

#[test]
fn ifelse_ext_text3_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/conditional/ifelse-ext-text3.ink.json");
    let mut story = Story::new(&json_string);
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("This is text 3.", text[0]);
    assert_eq!(1, story.get_current_choices().len());
    story.choose_choice_index(0);

    common::next_all(&mut story, &mut text);
    assert_eq!(2, text.len());
    assert_eq!("This is the end.", text[1]);

    Ok(())
}

#[test]
fn cond_text2_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/conditional/condtext.ink.json");
    let mut story = Story::new(&json_string);
    println!("{}", story.build_string_of_hierarchy());
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(1);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    assert_eq!(
        "I stared at Monsieur Fogg. \"But there must be a reason for this trip,\" I observed.",
        text[0]
    );

    Ok(())
}

#[test]
fn cond_opt2_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/conditional/condopt.ink.json");
    let mut story = Story::new(&json_string);
    println!("{}", story.build_string_of_hierarchy());
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(1);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(2, story.get_current_choices().len());

    Ok(())
}
