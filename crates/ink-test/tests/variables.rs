use crate::support::{
    compiler::compile_fixture_to_story,
    runtime::{Story, StoryError, ValueType},
    story_runner as common,
};

fn choose(story: &mut Story, index: usize) -> String {
    story.choose_choice_index(index);
    story.cont_maximally()
}

#[test]
fn variable_declaration_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("variables/variable-declaration.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!(
        "\"My name is Jean Passepartout, but my friend's call me Jackie. I'm 23 years old.\"",
        text[0]
    );

    Ok(())
}

#[test]
fn var_calc_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("variables/varcalc.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The values are true and -1 and -6 and aa.", text[0]);

    Ok(())
}

#[test]
fn var_string_ink_bug_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("variables/varstringinc.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    assert_eq!("ab.", text[1]);

    Ok(())
}

#[test]
fn var_divert_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("variables/var-divert.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(1);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("Everybody dies.", text[0]);

    Ok(())
}

#[test]
fn variables_can_be_used_in_mathematical_operations() {
    let mut story =
        compile_fixture_to_story("variables/variables-can-be-used-in-mathematical-operations.ink");

    assert_eq!(
        "Integer calculation does each step as integers, which may not be what you want.\n(3 - 13) / 3 + 5 = 2 which should be 1.66666...!\nFloat calculation works better:\n(3 - 13) / 3 + 5 = 1.6666667!\n",
        story.cont_maximally()
    );
}

#[test]
fn global_variables_are_parsed_when_story_is_read() {
    let mut story = compile_fixture_to_story("variables/global-variables.ink");

    assert_eq!(
        "The latest measurement is 3.6 Röntgen.\n",
        story.cont_maximally()
    );
    assert_eq!(
        Some(false),
        story
            .get_variable("game::is_hazardous")
            .and_then(|value| value.get::<bool>())
    );
}

#[test]
fn global_variables_can_be_changed_from_the_caller() {
    let mut story = compile_fixture_to_story("variables/global-variables.ink");

    story
        .set_variable("game::value", &ValueType::Float(15000.0))
        .unwrap();
    assert_eq!(
        "The latest measurement is 15000 Röntgen.\n",
        story.cont_maximally()
    );
}

#[test]
fn variables_can_be_used_in_conditions() {
    let mut story = compile_fixture_to_story("variables/variables-can-be-used-in-conditions.ink");

    assert_eq!(
        "The latest measurement is 3.6 Röntgen. Not terrible, not great.\n",
        story.cont_maximally()
    );
    story
        .set_variable("game::value", &ValueType::Float(15000.0))
        .unwrap();
    assert_eq!(
        "The latest measurement is 15000 Röntgen. Oh no.\n",
        choose(&mut story, 0)
    );
}

#[test]
fn variables_can_change_and_influence_story_flow_conditions() {
    let mut story = compile_fixture_to_story(
        "variables/variables-can-change-and-influence-story-flow-conditions.ink",
    );

    assert_eq!(
        "The latest measurement is 3.6 Röntgen. Not terrible, not great.\n",
        story.cont_maximally()
    );
    story
        .set_variable("game::value", &ValueType::Float(15000.0))
        .unwrap();
    story
        .set_variable("game::is_hazardous", &ValueType::Bool(true))
        .unwrap();
    assert_eq!(
        "The latest measurement is 15000 Röntgen. Oh no.\n",
        choose(&mut story, 0)
    );
}
