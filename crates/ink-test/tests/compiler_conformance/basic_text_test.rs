use crate::compiler_conformance::api::{story::Story, story_error::StoryError};
use std::env;

use crate::compiler_conformance::common;

#[test]
fn oneline_test() -> Result<(), StoryError> {
    println!("{}", env::current_dir().unwrap().to_string_lossy());

    let mut story = common::compile_story("inkfiles/basictext/oneline.ink");
    println!("{}", story.build_string_of_hierarchy());

    assert!(story.can_continue());
    let line = story.cont();
    println!("{}", line);
    assert_eq!("Line.", line.trim());
    assert!(!story.can_continue());

    Ok(())
}

#[test]
fn twolines_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/basictext/twolines.ink");
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);
    assert_eq!(2, text.len());
    assert_eq!("Line.", text[0]);
    assert_eq!("Other line.", text[1]);

    Ok(())
}

#[test]
fn the_intercept_test() -> Result<(), StoryError> {
    let story = common::compile_story("inkfiles/TheIntercept.ink");
    println!("{}", story.build_string_of_hierarchy());
    assert!(story.can_continue() || !story.get_current_choices().is_empty());

    Ok(())
}

#[test]
fn test1_test() -> Result<(), StoryError> {
    let story = common::compile_story("inkfiles/test1.ink");
    println!("{}", story.build_string_of_hierarchy());
    assert!(story.can_continue() || !story.get_current_choices().is_empty());

    Ok(())
}
