use ink_compiler::{Compiler, DiagnosticSeverity, SourceInput};
use ink_runtime::{
    choice::Choice, story::Story as RuntimeStory, story_error::StoryError, value_type::ValueType,
};
use std::rc::Rc;

struct Story {
    inner: RuntimeStory,
}

impl Story {
    fn new(json: &str) -> Self {
        Self {
            inner: RuntimeStory::new(json).expect("compiled JSON should load"),
        }
    }

    fn can_continue(&self) -> bool {
        self.inner.can_continue()
    }

    fn cont(&mut self) -> String {
        self.inner.cont().expect("story should continue")
    }

    fn cont_maximally(&mut self) -> String {
        self.inner
            .continue_maximally()
            .expect("story should continue maximally")
    }

    fn choose_choice_index(&mut self, index: usize) {
        self.inner
            .choose_choice_index(index)
            .expect("choice index should be valid");
    }

    fn get_current_choices(&self) -> Vec<Rc<Choice>> {
        self.inner.get_current_choices()
    }

    fn get_current_tags(&mut self) -> Vec<String> {
        self.inner
            .get_current_tags()
            .expect("current tags should load")
    }

    fn get_variable(&self, name: &str) -> Option<ValueType> {
        self.inner.get_variable(name)
    }

    fn set_variable(&mut self, name: &str, value: &ValueType) -> Result<(), StoryError> {
        self.inner.set_variable(name, value)
    }
}

fn compile_story_fixture(filename: &str) -> Story {
    let result = Compiler::new().compile(SourceInput::named(inkling_fixture(filename), filename));
    let errors = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .map(|diagnostic| {
            format!(
                "line {} column {}: {}",
                diagnostic.line, diagnostic.column, diagnostic.message
            )
        })
        .collect::<Vec<_>>();
    assert!(errors.is_empty(), "{errors:#?}");

    Story::new(&result.artifact.expect("compiled story").json)
}

fn story_is_ended(story: &Story) -> bool {
    !story.can_continue() && story.get_current_choices().is_empty()
}

fn compile_error_messages_fixture(filename: &str) -> Vec<String> {
    Compiler::new()
        .compile(SourceInput::named(inkling_fixture(filename), filename))
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .map(|diagnostic| diagnostic.message)
        .collect()
}

fn assert_compile_errors_fixture(filename: &str) -> Vec<String> {
    let errors = compile_error_messages_fixture(filename);
    assert!(!errors.is_empty(), "expected compiler errors");
    errors
}

fn choose(story: &mut Story, index: usize) -> String {
    story.choose_choice_index(index);
    story.cont_maximally()
}

fn inkling_fixture(filename: &str) -> String {
    ink_test::load_fixture_text(&format!("inkling_examples/{filename}"))
}

// The tests below are ports of scenarios from pjohansson/inkling's `tests/`
// directory, whose upstream project is Parity-licensed. Unsupported LIST
// coverage is excluded, and Inkling-specific API assertions are converted into
// compiler/runtime behavior checks.

#[test]
fn inkling_unordered_knot_divert_story_runs_to_end() {
    let mut story = compile_story_fixture("unordered-knot-divert-story-runs-to-end.ink");

    assert_eq!(
        "The detective arrives.\nThe witness points to the desk.\nThe letter is still on the desk.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_flat_story_comments_and_initial_divert_run_in_order() {
    let mut story =
        compile_story_fixture("flat-story-comments-and-initial-divert-run-in-order.ink");

    assert_eq!(
        "The recorder starts.\nEvery switch is labeled.\nThe panel is already lit.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_stitch_shorthand_diverts_inside_a_knot() {
    let mut story = compile_story_fixture("stitch-shorthand-diverts-inside-a-knot.ink");

    assert_eq!(
        "The map is unfolded.\nThe station clock rings.\nThe kettle whistles.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_math_expressions_in_content_use_current_variables() {
    let mut story = compile_story_fixture("math-expressions-in-content-use-current-variables.ink");

    assert_eq!(
        "Before: 3 + 5 = 8.\nAfter: 7 + 5 = 12.\n",
        story.cont_maximally()
    );
    assert_eq!(
        Some(7),
        story
            .get_variable("game::a")
            .and_then(|value| value.get::<i32>())
    );
}

#[test]
fn inkling_story_can_be_read_with_unnamed_knots() {
    let mut story = compile_story_fixture("story-can-be-read-with-unnamed-knots.ink");

    assert_eq!(
        "Mont Blanc was a world-renowned mountain guide.\nHe befriended thousands of climbers and children sightseeing in Switzerland.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_story_starts_from_top_even_if_knot_is_unnamed() {
    let mut story = compile_story_fixture("story-starts-from-top-even-if-knot-is-unnamed.ink");

    assert_eq!(
        "Mont Blanc was a world-renowned mountain guide.\nHe befriended thousands of climbers and children sightseeing in Switzerland.\nGESICHT'S BEDROOM, MORNING\nGesicht is lying in his bed, eyes wide open and staring at the ceiling.\nHe just woke from a nightmare.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_story_can_start_with_named_knot() {
    let mut story = compile_story_fixture("story-can-start-with-named-knot.ink");

    assert_eq!(
        "GESICHT'S BEDROOM (MORNING)\nGesicht is lying in his bed, eyes wide open and staring at the ceiling.\nHe just woke from a nightmare.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_story_can_divert_at_will_between_unordered_knots() {
    let mut story = compile_story_fixture("story-can-divert-at-will-between-unordered-knots.ink");

    assert_eq!(
        "SCENE OF MURDER (TRASHED APARTMENT, DAYTIME)\nGesicht arrives at a grotesque murder scene.\nThe lead detective stop him as he enters the room.\nHe identifies himself as being from Europol and passes the barrier.\nThe body is lying face down in a pool of blood.\nA desk lamp and piece of broken wood have been stuck to his head.\nThey mimic the appearance of antlers.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_story_can_be_structured_using_stitches() {
    let mut story = compile_story_fixture("story-can-be-structured-using-stitches.ink");

    assert_eq!(
        "Mont Blanc was a world-renowned mountain guide.\nHe befriended thousands of climbers and children sightseeing in Switzerland.\nGesicht is lying in his bed, eyes wide open and staring at the ceiling.\nHe just woke from a nightmare.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_stitches_can_be_diverted_to_inside_a_knot_without_full_address() {
    let mut story =
        compile_story_fixture("stitches-can-be-diverted-to-inside-a-knot-without-full-address.ink");

    assert_eq!(
        "Mont Blanc was a world-renowned mountain guide.\nHe befriended thousands of climbers and children sightseeing in Switzerland.\nGesicht is lying in his bed, eyes wide open and staring at the ceiling.\nHe just woke from a nightmare.\nBefore he's had the time to eat breakfast a call about a murder comes in.\nAs Helena probes him about leaving he suggests that they take a vacation.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_diverts_are_glue_and_add_single_whitespace_after_story_text() {
    let mut story =
        compile_story_fixture("diverts-are-glue-and-add-single-whitespace-after-story-text.ink");

    assert_eq!(
        "“We all loved Mont Blanc.\n“The memorial service will take place in three days ...\n“This place will be filled with tens of thousands of people, probably several hundred thousands.\n“all mourning the death of mont blanc.”\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_glue_binds_lines_together_without_newline_markers() {
    let mut story = compile_story_fixture("glue-binds-lines-together-without-newline-markers.ink");

    assert_eq!(
        "“So she abandoned me ... she sent me to a boarding school in England ... and I never heard a thing from her again.”\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_glue_binds_across_diverts() {
    let mut story = compile_story_fixture("glue-binds-across-diverts.ink");

    assert_eq!(
        "“So she abandoned me ... she sent me to a boarding school in England ... and I never heard a thing from her again.”\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_tags_are_included_with_lines_and_choices() {
    let mut story = compile_story_fixture("tags-are-included-with-lines-and-choices.ink");

    assert_eq!("SCOTLAND, EURO FEDERATION\n", story.cont());
    assert_eq!(
        vec!["location card".to_string(), "europe".to_string()],
        story.get_current_tags()
    );
    assert_eq!("An old castle heaves in front of you.\n", story.cont());
    assert_eq!(vec!["description".to_string()], story.get_current_tags());
    assert_eq!("", story.cont_maximally());
    let choices = story.get_current_choices();
    assert_eq!("Enter it.", choices[0].text);
    assert_eq!(vec!["action".to_string()], choices[0].tags);
}

#[test]
fn inkling_mathematical_expressions_can_be_used_in_lines() {
    let mut story = compile_story_fixture("mathematical-expressions-can-be-used-in-lines.ink");

    assert_eq!(
        "Adding is easy: 2 + 3 is 5!\nMultiplication as so: 2 * (3 + 5) is 16!\nLet's nest a bit: -100 is -100!\nStrings can be added, too: string is string!\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_variables_can_be_used_in_mathematical_operations() {
    let mut story = compile_story_fixture("variables-can-be-used-in-mathematical-operations.ink");

    assert_eq!(
        "Integer calculation does each step as integers, which may not be what you want.\n(3 - 13) / 3 + 5 = 2 which should be 1.66666...!\nFloat calculation works better:\n(3 - 13) / 3 + 5 = 1.6666667!\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_conditions_may_use_expressions_on_left_or_right_hand_side() {
    let mut story =
        compile_story_fixture("conditions-may-use-expressions-on-left-or-right-hand-side.ink");

    assert_eq!("True\nTrue\nFalse\nTrue\n", story.cont_maximally());
    assert!(story_is_ended(&story));
}

#[test]
fn inkling_global_variables_are_parsed_when_story_is_read() {
    let mut story = compile_story_fixture("global-variables.ink");

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
fn inkling_global_variables_can_be_changed_from_the_caller() {
    let mut story = compile_story_fixture("global-variables.ink");

    story
        .set_variable("game::value", &ValueType::Float(15000.0))
        .unwrap();
    assert_eq!(
        "The latest measurement is 15000 Röntgen.\n",
        story.cont_maximally()
    );
}

#[test]
fn inkling_variables_can_be_used_in_conditions() {
    let mut story = compile_story_fixture("variables-can-be-used-in-conditions.ink");

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
fn inkling_variables_can_change_and_influence_story_flow_conditions() {
    let mut story =
        compile_story_fixture("variables-can-change-and-influence-story-flow-conditions.ink");

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

#[test]
fn inkling_address_validation_reports_unknown_knot_divert() {
    assert_compile_errors_fixture("diagnostics/unknown-knot-divert.ink");
}

#[test]
fn inkling_address_validation_reports_unknown_stitch_divert() {
    assert_compile_errors_fixture("diagnostics/unknown-stitch-divert.ink");
}

#[test]
fn inkling_address_validation_reports_unknown_relative_stitch() {
    assert_compile_errors_fixture("diagnostics/unknown-relative-stitch.ink");
}

#[test]
fn inkling_address_validation_reports_choice_text_divert_target_errors() {
    assert_compile_errors_fixture("diagnostics/choice-text-divert-target-errors.ink");
}

#[test]
fn inkling_address_validation_reports_alternative_sequence_divert_errors() {
    assert_compile_errors_fixture("diagnostics/alternative-sequence-divert-errors.ink");
}

#[test]
fn inkling_address_validation_reports_nested_branch_divert_errors() {
    assert_compile_errors_fixture("diagnostics/nested-branch-divert-errors.ink");
}

#[test]
fn inkling_address_validation_reports_condition_address_errors() {
    assert_compile_errors_fixture("diagnostics/condition-address-errors.ink");
}

#[test]
fn inkling_addresses_in_choices_are_validated_without_raw_parse_nodes() {
    let errors = compile_error_messages_fixture(
        "addresses-in-choices-validated-without-raw-parse-nodes.ink",
    );
    assert!(errors.is_empty(), "{errors:#?}");
}
