use crate::support::compiler::compile_fixture;
use crate::support::{
    runtime::{Story, StoryError},
    story_runner as common,
};

#[test]
fn no_choice_test() -> Result<(), StoryError> {
    let mut errors: Vec<String> = Vec::new();

    let text = common::run_story("choices/no-choice-text.ink", None, &mut errors);

    assert_eq!(0, errors.len());
    assert_eq!("Hello world!\nHello back!\n", common::join_text(&text));

    Ok(())
}

#[test]
fn one_test() -> Result<(), StoryError> {
    let mut errors: Vec<String> = Vec::new();

    let text = common::run_story("choices/one.ink", None, &mut errors);

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

    let text = common::run_story("choices/multi-choice.ink", Some(vec![0]), &mut errors);

    assert_eq!(0, errors.len());
    assert_eq!(
        "Hello, world!\nHello back!\nGoodbye\nHello back!\nNice to hear from you\n",
        common::join_text(&text)
    );

    // Select second choice
    let text = common::run_story("choices/multi-choice.ink", Some(vec![1]), &mut errors);

    assert_eq!(0, errors.len());
    assert_eq!(
        "Hello, world!\nHello back!\nGoodbye\nGoodbye\nSee you later\n",
        common::join_text(&text)
    );

    Ok(())
}

#[test]
fn single_choice1_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("choices/single-choice.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("Hello, world!", text[0]);

    Ok(())
}

#[test]
fn single_choic2_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("choices/single-choice.ink");
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
    let mut story = common::story_from_fixture("choices/suppress-choice.ink");
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
    let mut story = common::story_from_fixture("choices/mixed-choice.ink");
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
fn fallback_choice_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("choices/fallback-choice.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    assert_eq!(2, story.get_current_choices().len());

    Ok(())
}

#[test]
fn conditional_choice_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("choices/conditional-choice.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    assert_eq!(4, story.get_current_choices().len());

    Ok(())
}

#[test]
fn square_brackets_in_choice_text_are_literal() {
    let compiled = compile_fixture("choices/literal-choice-brackets.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    assert_eq!(
        story.get_current_choices()[0].text,
        "Display [selected output]"
    );
    story.choose_choice_index(0);
    assert_eq!(story.continue_maximally(), "Branch.\n");
}

#[test]
fn dynamic_choices_expand_from_arrays_and_mix_with_static_choices() {
    let compiled = compile_fixture("choices/dynamic-choice.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 4);
    assert_eq!(choices[0].text, "Fixed first");
    assert_eq!(choices[1].text, "0:Alpha");
    assert_eq!(choices[2].text, "2:Beta");
    assert_eq!(choices[3].text, "Fixed last");
    story.choose_choice_index(2);
    assert_eq!(story.continue_maximally(), "picked 2:Beta.\n");
}

#[test]
fn dynamic_choice_threads_survive_save_load() {
    let compiled = compile_fixture("choices/dynamic-choice.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    let save_string = story.save_state();
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert!(save.get("choiceThreads").is_some());

    let mut reloaded = Story::new(&compiled.json);
    reloaded.load_state(&save_string);
    let choices = reloaded.get_current_choices();
    assert_eq!(choices.len(), 4);
    assert_eq!(choices[2].text, "2:Beta");
    reloaded.choose_choice_index(2);
    assert_eq!(reloaded.continue_maximally(), "picked 2:Beta.\n");
}

#[test]
fn dynamic_choice_leading_braced_expression_is_text_without_colon() {
    let compiled = compile_fixture("choices/dynamic-choice-leading-expression-text.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text, "Alpha");
    assert_eq!(choices[1].text, "Beta");
    story.choose_choice_index(1);
    assert_eq!(story.continue_maximally(), "picked Beta.\n");
}

#[test]
fn dynamic_choices_can_nest_and_capture_outer_bindings() {
    let compiled = compile_fixture("choices/dynamic-choice-nested.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    assert_eq!(story.get_current_choices().len(), 1);
    assert_eq!(story.get_current_choices()[0].text, "Topic menu");
    story.choose_choice_index(0);
    assert_eq!(story.continue_maximally(), "");

    let topic_choices = story.get_current_choices();
    assert_eq!(topic_choices.len(), 2);
    assert_eq!(topic_choices[0].text, "Topic 0:East");
    assert_eq!(topic_choices[1].text, "Topic 1:West");
    story.choose_choice_index(1);
    assert_eq!(story.continue_maximally(), "topic West.\n");

    let detail_choices = story.get_current_choices();
    assert_eq!(detail_choices.len(), 1);
    assert_eq!(detail_choices[0].text, "Detail West-W1");
    story.choose_choice_index(0);
    assert_eq!(story.continue_maximally(), "detail West:W1.\n");
}

#[test]
fn dynamic_choice_bindings_are_visible_inside_for_blocks() {
    let compiled = compile_fixture("choices/dynamic-choice-for-body.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text, "Group 0");
    assert_eq!(choices[1].text, "Group 1");
    story.choose_choice_index(0);
    assert_eq!(story.continue_maximally(), "Item A.\nItem B.\n");
}

#[test]
fn empty_dynamic_choice_allows_invisible_fallback() {
    let compiled = compile_fixture("choices/dynamic-choice-empty-fallback.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "Empty fallback.\n");
    assert!(story.get_current_choices().is_empty());
}

#[test]
fn star_choices_are_repeatable() {
    let compiled = compile_fixture("choices/repeatable-choices.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    assert_eq!(story.get_current_choices().len(), 2);
    story.choose_choice_index(0);
    assert_eq!(story.continue_maximally(), "Star branch.\n");
    assert_eq!(story.get_current_choices().len(), 2);
    story.choose_choice_index(1);
    assert_eq!(story.continue_maximally(), "Plus branch.\n");
    assert_eq!(story.get_current_choices().len(), 2);
}

#[test]
fn selected_choice_text_is_not_echoed() {
    let compiled = compile_fixture("choices/choice-display-only.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    assert_eq!(story.get_current_choices()[0].text, "Display text");
    story.choose_choice_index(0);
    assert_eq!(story.continue_maximally(), "Branch text.\n");
}

#[test]
fn choice_conditions_still_control_visibility() {
    let compiled = compile_fixture("choices/conditional-repeatable-choices.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].text, "Toggle");
    story.choose_choice_index(0);
    assert_eq!(story.continue_maximally(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text, "Open path");
    assert_eq!(choices[1].text, "Toggle");
}

#[test]
fn choice_condition_colon_boundary_allows_dynamic_choice_text() {
    let compiled = compile_fixture("choices/choice-condition-colon-boundary.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].text, "Open path");
    story.choose_choice_index(0);
    assert_eq!(story.continue_maximally(), "Done.\n");
}

#[test]
fn choice_condition_blocks_stay_eager_while_inner_logical_ops_short_circuit() {
    let compiled = compile_fixture("choices/choice-condition-short-circuit-boundaries.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    assert_eq!(
        Some(2),
        story
            .get_variable("game::hits")
            .and_then(|value| value.get::<i32>())
    );
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text, "inner and evaluates right");
    assert_eq!(choices[1].text, "inner or short-circuits");
}

#[test]
fn save_load_preserves_generated_choices_without_regeneration() {
    let compiled = compile_fixture("choices/choice-save-load.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    assert_eq!(story.get_current_choices().len(), 2);
    let save_string = story.save_state();
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert_eq!(save["inkSaveVersion"], serde_json::json!(2));
    assert!(save.get("currentChoices").is_some());
    assert!(save.get("flows").is_none());
    assert!(save.get("evalStack").is_none());
    assert!(save.get("visitCounts").is_none());
    assert!(save.get("turnIndices").is_none());
    assert!(save.get("turnIdx").is_none());

    let mut reloaded = Story::new(&compiled.json);
    reloaded.load_state(&save_string);
    let choices = reloaded.get_current_choices();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text, "First");
    assert_eq!(choices[1].text, "Second");
    reloaded.choose_choice_index(1);
    assert_eq!(reloaded.continue_maximally(), "Picked 2.\n");
}

#[test]
fn save_load_preserves_thread_generated_choices() {
    let compiled = compile_fixture("choices/thread-choice-save-load.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text, "Thread");
    assert_eq!(choices[1].text, "Main");
    let save_string = story.save_state();
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert!(save.get("choiceThreads").is_some());

    let mut reloaded = Story::new(&compiled.json);
    reloaded.load_state(&save_string);
    assert_eq!(reloaded.get_current_choices().len(), 2);
    reloaded.choose_choice_index(0);
    assert_eq!(reloaded.continue_maximally(), "Thread branch.\n");
}

#[test]
fn save_load_preserves_deterministic_random_state() {
    let compiled = compile_fixture("choices/random-save-load.ink");
    let mut uninterrupted = Story::new(&compiled.json);
    let uninterrupted_first_output = uninterrupted.continue_maximally();
    uninterrupted.choose_choice_index(0);
    let uninterrupted_output = uninterrupted.continue_maximally();

    let mut saved = Story::new(&compiled.json);
    assert_eq!(saved.continue_maximally(), uninterrupted_first_output);
    let save_string = saved.save_state();
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert!(save.get("storySeed").is_some());
    assert!(save.get("previousRandom").is_some());
    assert!(save.get("turnIdx").is_none());

    let mut reloaded = Story::new(&compiled.json);
    reloaded.load_state(&save_string);
    reloaded.choose_choice_index(0);
    assert_eq!(reloaded.continue_maximally(), uninterrupted_output);
}
