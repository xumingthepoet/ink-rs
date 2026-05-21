use crate::support::compiler::{
    assert_json_sequence, assert_story_output, compile_fixture, compile_fixture_to_story,
};
use serde_json::json;

#[test]
fn arithmetic_fixture_runs() {
    let compiled = compile_fixture("expressions/arithmetic.ink");

    assert_story_output(&compiled, "36\n2\n3\n2\n2.3333333\n8\n8\n7\n14\n1\n");
}

#[test]
fn math_expressions_in_content_use_current_variables() {
    let mut story = compile_fixture_to_story(
        "expressions/math-expressions-in-content-use-current-variables.ink",
    );

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
fn mathematical_expressions_can_be_used_in_lines() {
    let compiled = compile_fixture("expressions/mathematical-expressions-can-be-used-in-lines.ink");

    assert_story_output(
        &compiled,
        "Adding is easy: 2 + 3 is 5!\nMultiplication as so: 2 * (3 + 5) is 16!\nLet's nest a bit: -100 is -100!\nStrings can be added, too: string is string!\n",
    );
}

#[test]
fn conditions_may_use_expressions_on_left_or_right_hand_side() {
    let compiled = compile_fixture(
        "expressions/conditions-may-use-expressions-on-left-or-right-hand-side.ink",
    );

    assert_story_output(&compiled, "True\nTrue\nFalse\nTrue\n");
}

#[test]
fn expression_string_inline_tokens_are_literals() {
    let compiled = compile_fixture("expressions/expression-string-inline-tokens-are-literals.ink");
    let json: serde_json::Value =
        serde_json::from_str(&compiled.json).expect("compiled JSON should parse");

    assert_story_output(&compiled, "#####\n# <> -> <-\nvalue ok # <> -> <-\n");
    assert_json_sequence(&json, vec![json!("str"), json!("^#"), json!("/str")]);
    assert_json_sequence(
        &json,
        vec![json!("str"), json!("^# <> -> <-"), json!("/str")],
    );
}
