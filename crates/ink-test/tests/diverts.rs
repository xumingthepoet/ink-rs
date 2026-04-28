mod support;

use support::compiler::{assert_story_output, compile_fixture};
use support::{
    runtime::{Story, StoryError},
    story_runner as common,
};

#[test]
fn simple_divert_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("diverts/simple-divert.ink.json");
    let mut story = Story::new(&json_string);

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);
    assert_eq!(2, text.len());
    assert_eq!("We arrived into London at 9.45pm exactly.", text[0]);
    assert_eq!(
        "We hurried home to Savile Row as fast as we could.",
        text[1]
    );

    Ok(())
}

#[test]
fn invisible_divert_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("diverts/invisible-divert.ink.json");
    let mut story = Story::new(&json_string);

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
fn divert_on_choice_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("diverts/divert-on-choice.ink.json");
    let mut story = Story::new(&json_string);

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);
    story.choose_choice_index(0);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("You open the gate, and step out onto the path.", text[0]);

    Ok(())
}

#[test]
fn complex_branching1_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("diverts/complex-branching.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(0);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    assert_eq!("\"There is not a moment to lose!\" I declared.", text[0]);
    assert_eq!(
        "We hurried home to Savile Row as fast as we could.",
        text[1]
    );

    Ok(())
}

#[test]
fn complex_branching2_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("diverts/complex-branching.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(1);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(3, text.len());
    assert_eq!(
        "\"Monsieur, let us savour this moment!\" I declared.",
        text[0]
    );
    assert_eq!(
        "My master clouted me firmly around the head and dragged me out of the door.",
        text[1]
    );
    assert_eq!(
        "He insisted that we hurried home to Savile Row as fast as we could.",
        text[2]
    );

    Ok(())
}

#[test]
fn explicit_dynamic_diverts_run_at_runtime() {
    let compiled = compile_fixture("diverts/explicit-dynamic-diverts.ink");

    assert_story_output(
        &compiled,
        "First.\nStruct.\nArray.\nConst.\nConst struct.\nConst array.\nFinal.\n",
    );
}

#[test]
fn explicit_dynamic_diverts_support_arguments() {
    let compiled = compile_fixture("diverts/explicit-dynamic-divert-args.ink");

    assert_story_output(&compiled, "Value 5.\n");
}

#[test]
fn explicit_dynamic_tunnels_run_at_runtime() {
    let compiled = compile_fixture("diverts/explicit-dynamic-tunnel.ink");

    assert_story_output(&compiled, "Inside.\nAfter.\n");
}
