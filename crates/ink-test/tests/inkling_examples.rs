#![allow(dead_code, unused_imports)]

#[path = "conformance/api.rs"]
mod api;

use api::story::Story;
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
