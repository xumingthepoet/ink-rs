#![allow(dead_code, unused_imports)]

#[path = "conformance/api.rs"]
mod api;

use api::story::Story;
use api::value_type::ValueType;
use ink_compiler::{Compiler, DiagnosticSeverity, SourceInput};

fn compile_story(source: &str) -> Story {
    let result = Compiler::new().compile(SourceInput::new(source));
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

#[test]
fn inkling_player_style_story_can_be_played() {
    // Covers the same feature mix as Inkling's public player example without
    // vendoring the upstream fixture text: sticky choices, inline choice text,
    // nested choices, knot diverts, and visit-count conditions.
    let mut story = compile_story(
        r#"
Hours pass in the compiler room.

+   [Check the terminal]
    -> terminal
+   "No[!]," you say. One more build.
    + +     [Start coffee]
            More work it is.
            -> next_day
    + +     You close the laptop[]. Sleep wins.
            -> next_day

=== next_day ===
NEXT DAY

+   {terminal} The log is green. -> coffee
+   {not terminal} No news yet.[] Back to work.
    -> END

=== terminal ===
You scan the terminal. The build is still running.
+   You leave it for tomorrow.
    -> next_day
+   [Send a note] -> note

=== note ===
You write, "Still testing."
-> next_day

=== coffee ===
The coffee tastes like victory.
-> END
"#,
    );

    assert_eq!("Hours pass in the compiler room.\n", story.cont_maximally());
    assert_eq!(2, story.get_current_choices_len());
    assert_eq!("Check the terminal", story.get_current_choices()[0].text);
    assert_eq!(r#""No!"#, story.get_current_choices()[1].text);

    story.choose_choice_index(0);
    assert_eq!(
        "You scan the terminal. The build is still running.\n",
        story.cont_maximally()
    );
    assert_eq!(2, story.get_current_choices_len());

    story.choose_choice_index(1);
    assert_eq!(
        "You write, \"Still testing.\"\nNEXT DAY\n",
        story.cont_maximally()
    );
    assert_eq!(1, story.get_current_choices_len());
    assert_eq!("The log is green.", story.get_current_choices()[0].text);

    story.choose_choice_index(0);
    assert_eq!(
        "The log is green. The coffee tastes like victory.\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_unordered_knot_divert_story_runs_to_end() {
    let mut story = compile_story(
        r#"
-> murder

== murder ==
The detective arrives.
-> interview

== evidence ==
The letter is still on the desk.
-> END

== interview ==
The witness points to the desk.
-> evidence
"#,
    );

    assert_eq!(
        "The detective arrives.\nThe witness points to the desk.\nThe letter is still on the desk.\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_flat_story_comments_and_initial_divert_run_in_order() {
    let mut story = compile_story(
        r#"
The recorder starts. // inline notes are not story text
Every switch is labeled.
-> control_room

== control_room ==
The panel is already lit.
-> END
"#,
    );

    assert_eq!(
        "The recorder starts.\nEvery switch is labeled.\nThe panel is already lit.\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_stitch_shorthand_diverts_inside_a_knot() {
    let mut story = compile_story(
        r#"
-> intro

== intro ==
The map is unfolded.
-> chapter.wake

== chapter ==
= breakfast
The kettle whistles.
-> END

= wake
The station clock rings.
-> breakfast
"#,
    );

    assert_eq!(
        "The map is unfolded.\nThe station clock rings.\nThe kettle whistles.\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_one_time_choices_filter_but_sticky_choice_remains() {
    let mut story = compile_story(
        r#"
-> room

== room ==
The storage room smells like dust.

*   [Take the brass key.]
    -> room
*   [Open the crate.]
    -> room
+   [Wait by the door.]
    -> room
"#,
    );

    assert_eq!(
        "The storage room smells like dust.\n",
        story.cont_maximally()
    );
    assert_eq!(3, story.get_current_choices_len());
    assert_eq!("Take the brass key.", story.get_current_choices()[0].text);
    assert_eq!("Open the crate.", story.get_current_choices()[1].text);
    assert_eq!("Wait by the door.", story.get_current_choices()[2].text);

    story.choose_choice_index(1);
    assert_eq!(
        "The storage room smells like dust.\n",
        story.cont_maximally()
    );
    assert_eq!(2, story.get_current_choices_len());
    assert_eq!("Take the brass key.", story.get_current_choices()[0].text);
    assert_eq!("Wait by the door.", story.get_current_choices()[1].text);

    story.choose_choice_index(1);
    assert_eq!(
        "The storage room smells like dust.\n",
        story.cont_maximally()
    );
    assert_eq!(2, story.get_current_choices_len());
    assert_eq!("Take the brass key.", story.get_current_choices()[0].text);
    assert_eq!("Wait by the door.", story.get_current_choices()[1].text);

    story.choose_choice_index(0);
    assert_eq!(
        "The storage room smells like dust.\n",
        story.cont_maximally()
    );
    assert_eq!(1, story.get_current_choices_len());
    assert_eq!("Wait by the door.", story.get_current_choices()[0].text);
}

#[test]
fn inkling_choice_conditions_can_reference_visited_stitches() {
    let mut story = compile_story(
        r#"
-> passage

== passage ==
Two service tunnels meet here.

+   [Left tunnel] -> left_tunnel.torch
+   [Right tunnel] -> dark_room

== dark_room ==
The right tunnel is unlit.

+   {left_tunnel.torch} Use the lamp from the left tunnel.
+   [Head back.]
    -> passage

== left_tunnel ==
= torch
You find a lamp on a hook.
-> passage
"#,
    );

    assert_eq!("Two service tunnels meet here.\n", story.cont_maximally());
    assert_eq!("Left tunnel", story.get_current_choices()[0].text);
    assert_eq!("Right tunnel", story.get_current_choices()[1].text);

    story.choose_choice_index(1);
    assert_eq!("The right tunnel is unlit.\n", story.cont_maximally());
    assert_eq!(1, story.get_current_choices_len());
    assert_eq!("Head back.", story.get_current_choices()[0].text);

    story.choose_choice_index(0);
    assert_eq!("Two service tunnels meet here.\n", story.cont_maximally());
    story.choose_choice_index(0);
    assert_eq!(
        "You find a lamp on a hook.\nTwo service tunnels meet here.\n",
        story.cont_maximally()
    );
    story.choose_choice_index(1);
    assert_eq!("The right tunnel is unlit.\n", story.cont_maximally());
    assert_eq!(2, story.get_current_choices_len());
    assert_eq!(
        "Use the lamp from the left tunnel.",
        story.get_current_choices()[0].text
    );
    assert_eq!("Head back.", story.get_current_choices()[1].text);
}

#[test]
fn inkling_sequences_conditionals_and_variable_assignments_update_text() {
    let mut story = compile_story(
        r#"
VAR count = 0

-> loop

== loop ==
Status {fresh|warming|steady}. {count == 0: First pass.|Count is {count}.}
~ count = count + 1

+   [Again] -> loop
"#,
    );

    assert_eq!("Status fresh. First pass.\n", story.cont_maximally());
    assert_eq!(Some(ValueType::Int(1)), story.get_variable("count"));

    story.choose_choice_index(0);
    assert_eq!("Status warming. Count is 1.\n", story.cont_maximally());
    assert_eq!(Some(ValueType::Int(2)), story.get_variable("count"));

    story.choose_choice_index(0);
    assert_eq!("Status steady. Count is 2.\n", story.cont_maximally());
    assert_eq!(Some(ValueType::Int(3)), story.get_variable("count"));
}

#[test]
fn inkling_math_expressions_in_content_use_current_variables() {
    let mut story = compile_story(
        r#"
VAR a = 3
VAR b = 5

Before: {a} + {b} = {a + b}.
~ a = 7
After: {a} + {b} = {a + b}.
-> END
"#,
    );

    assert_eq!(
        "Before: 3 + 5 = 8.\nAfter: 7 + 5 = 12.\n",
        story.cont_maximally()
    );
    assert_eq!(Some(ValueType::Int(7)), story.get_variable("a"));
}
