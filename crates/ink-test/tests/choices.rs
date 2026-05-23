use crate::support::compiler::compile_fixture;
use crate::support::{
    runtime::{Story, StoryError},
    story_runner as common,
};
use serde_json::Value;

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
fn choice_leading_braced_expression_without_colon_is_text() {
    let compiled = compile_fixture("choices/choice-leading-expression-text.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].text, "true Label");
    story.choose_choice_index(0);
    assert_eq!(story.continue_maximally(), "picked.\n");
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
fn save_after_dynamic_choice_selection_preserves_binding_aftermath() {
    let compiled = compile_fixture("choices/dynamic-choice.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    story.choose_choice_index(2);
    let save_string = story.save_state();
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert_save_json_has_no_choice_or_removed_execution_state(&save);

    let mut reloaded = Story::new(&compiled.json);
    reloaded.load_state(&save_string);
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
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text, "inner and evaluates right");
    assert_eq!(choices[1].text, "inner or short-circuits");
}

#[test]
fn choice_pause_save_is_rejected() {
    let compiled = compile_fixture("choices/choice-save-load.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    assert_eq!(story.get_current_choices().len(), 2);

    assert_pending_choice_save_rejected(&story);
}

#[test]
fn static_choice_pause_save_is_rejected() {
    let compiled = compile_fixture("choices/choice-save-load.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    assert_eq!(story.get_current_choices().len(), 2);

    assert_pending_choice_save_rejected(&story);
}

#[test]
fn dynamic_choice_pause_save_is_rejected() {
    let compiled = compile_fixture("choices/dynamic-choice.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    assert_eq!(story.get_current_choices().len(), 4);

    assert_pending_choice_save_rejected(&story);
}

#[test]
fn selected_choice_aftermath_save_omits_choice_and_removed_execution_state() {
    let compiled = compile_fixture("choices/choice-save-load.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    story.choose_choice_index(1);
    assert_eq!(story.continue_maximally(), "Picked 2.\n");

    let save: Value = serde_json::from_str(&story.save_state()).expect("valid save JSON");
    assert_save_json_has_no_choice_or_removed_execution_state(&save);
}

#[test]
fn save_after_selecting_authored_choice_continues_aftermath() {
    let compiled = compile_fixture("choices/authored-choice-save-load.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text, "Saved");
    assert_eq!(choices[1].text, "Main");
    story.choose_choice_index(0);
    let save_string = story.save_state();
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert_save_json_has_no_choice_or_removed_execution_state(&save);

    let mut reloaded = Story::new(&compiled.json);
    reloaded.load_state(&save_string);
    assert_eq!(reloaded.continue_maximally(), "Saved branch.\n");
}

#[test]
fn save_after_selecting_choice_reaches_gather_after_load() {
    let compiled = compile_fixture("choices/gather-choice-save-load.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    assert_eq!(story.get_current_choices().len(), 2);
    story.choose_choice_index(1);
    let save_string = story.save_state();
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert_save_json_has_no_choice_or_removed_execution_state(&save);

    let mut reloaded = Story::new(&compiled.json);
    reloaded.load_state(&save_string);
    assert_eq!(
        reloaded.continue_maximally(),
        "Right branch.\nregroup\nRegrouped.\n"
    );
}

#[test]
fn weave_fallthrough_works_with_current_choice_structure() {
    let compiled = compile_fixture("choices/weave-fallthrough-current.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "Start.\n");
    assert_eq!(story.get_current_choices().len(), 3);
    story.choose_choice_index(0);
    assert_eq!(story.continue_maximally(), "Hub entry.\n");
    assert_eq!(story.get_current_choices().len(), 2);
    story.choose_choice_index(0);

    assert_eq!(story.continue_maximally(), "Hub branch.\n");
}

#[test]
fn save_after_selecting_dynamic_weave_choice_preserves_aftermath() {
    let compiled = compile_fixture("choices/weave-fallthrough-current.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "Start.\n");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 3);
    assert_eq!(choices[1].text, "Dynamic East");
    story.choose_choice_index(1);

    let save_string = story.save_state();
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert_save_json_has_no_choice_or_removed_execution_state(&save);

    let mut reloaded = Story::new(&compiled.json);
    reloaded.load_state(&save_string);

    assert_eq!(
        reloaded.continue_maximally(),
        "Dynamic branch 0:East.\nregroup\nRegrouped.\n"
    );
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
    saved.choose_choice_index(0);
    let save_string = saved.save_state();
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert!(save.get("storySeed").is_some());
    assert!(save.get("previousRandom").is_some());
    assert!(save.get("turnIdx").is_none());

    let mut reloaded = Story::new(&compiled.json);
    reloaded.load_state(&save_string);
    assert_eq!(reloaded.continue_maximally(), uninterrupted_output);
}

#[test]
fn nested_choice_pause_without_transition_text_rejects_save() {
    let compiled = compile_fixture("choices/dynamic-choice-nested.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "");
    assert_eq!(story.get_current_choices()[0].text, "Topic menu");
    story.choose_choice_index(0);
    assert_eq!(story.continue_maximally(), "");

    let topic_choices = story.get_current_choices();
    assert_eq!(topic_choices.len(), 2);
    assert_eq!(topic_choices[0].text, "Topic 0:East");
    assert_eq!(topic_choices[1].text, "Topic 1:West");
    assert_pending_choice_save_rejected(&story);
}

fn assert_pending_choice_save_rejected(story: &Story) {
    let error = story
        .try_save_state()
        .expect_err("pending choices should not be runtime-saveable");

    assert!(matches!(
        error,
        StoryError::InvalidStoryState(message)
            if message.contains("Cannot save while choices are pending")
    ));
}

fn assert_save_json_has_no_choice_or_removed_execution_state(save: &Value) {
    let forbidden_keys = [
        "currentChoices",
        "generatedChoices",
        "choiceThreads",
        "originalThreadIndex",
        "threadIndex",
        "threads",
        "threadCounter",
        "flows",
        "currentFlowName",
        "evalStack",
        "currentDivertTarget",
        "visitCounts",
        "turnIndices",
        "turnIdx",
        "resumeMode",
    ];
    let mut found = Vec::new();
    collect_forbidden_save_keys(save, "$", &forbidden_keys, &mut found);
    assert!(
        found.is_empty(),
        "save JSON should not contain removed choice/continuation/execution fields: {found:?}\n{save:#}"
    );
}

fn collect_forbidden_save_keys(
    value: &Value,
    path: &str,
    forbidden_keys: &[&str],
    found: &mut Vec<String>,
) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                let nested_path = format!("{path}.{key}");
                if forbidden_keys.contains(&key.as_str()) {
                    found.push(nested_path.clone());
                }
                collect_forbidden_save_keys(nested, &nested_path, forbidden_keys, found);
            }
        }
        Value::Array(items) => {
            for (index, nested) in items.iter().enumerate() {
                collect_forbidden_save_keys(
                    nested,
                    &format!("{path}[{index}]"),
                    forbidden_keys,
                    found,
                );
            }
        }
        _ => {}
    }
}
