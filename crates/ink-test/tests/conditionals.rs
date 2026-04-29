mod support;

use support::{
    compiler::{assert_story_output, compile_fixture},
    runtime::StoryError,
    story_runner as common,
};

#[test]
fn iftrue_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("conditionals/iftrue.ink");
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value is 1.", text[0]);

    Ok(())
}

#[test]
fn iffalse_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("conditionals/iffalse.ink");
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value is 3.", text[0]);

    Ok(())
}

#[test]
fn ifelse_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("conditionals/ifelse.ink");
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value is 1.", text[0]);

    Ok(())
}

#[test]
fn ifelse_ext_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("conditionals/ifelse-ext.ink");
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value is -1.", text[0]);

    Ok(())
}

#[test]
fn ifelse_ext_text1_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("conditionals/ifelse-ext-text1.ink");
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
    let mut story = common::story_from_fixture("conditionals/ifelse-ext-text2.ink");
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
    let mut story = common::story_from_fixture("conditionals/ifelse-ext-text3.ink");
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
    let mut story = common::story_from_fixture("conditionals/condtext.ink");
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
    let mut story = common::story_from_fixture("conditionals/condopt.ink");
    println!("{}", story.build_string_of_hierarchy());
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(1);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(2, story.get_current_choices().len());

    Ok(())
}

#[test]
fn switch_condition_selects_matching_case() {
    let compiled = compile_fixture("conditionals/switch.ink");

    assert_story_output(&compiled, "stage one\n");
}

#[test]
fn bool_switch_with_divert_branches_closes_flow() {
    let compiled = compile_fixture("conditionals/bool-switch-diverts.ink");

    assert_story_output(&compiled, "Finished.\n");
}
