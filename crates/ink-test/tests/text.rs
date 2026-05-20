use std::env;

use crate::support::{
    compiler::{assert_story_output, compile_fixture},
    runtime::StoryError,
    story_runner as common,
};

#[test]
fn oneline_test() -> Result<(), StoryError> {
    println!("{}", env::current_dir().unwrap().to_string_lossy());
    let mut story = common::story_from_fixture("text/oneline.ink");
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
    let mut story = common::story_from_fixture("text/twolines.ink");
    println!("{}", story.build_string_of_hierarchy());

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);
    assert_eq!(2, text.len());
    assert_eq!("Line.", text[0]);
    assert_eq!("Other line.", text[1]);

    Ok(())
}

#[test]
fn smoke_fixture_runs_compiled_story() {
    let compiled = compile_fixture("text/smoke.ink");
    assert_story_output(&compiled, "Hello from the integration test surface.\n");
}

#[test]
fn hello_world_fixture_runs() {
    let compiled = compile_fixture("text/hello-world.ink");

    assert_story_output(&compiled, "Hello world\n");
}
