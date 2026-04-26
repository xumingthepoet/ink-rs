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

fn compile_error_messages(source: &str) -> Vec<String> {
    Compiler::new()
        .compile(SourceInput::new(source))
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .map(|diagnostic| diagnostic.message)
        .collect()
}

fn assert_compile_errors(source: &str) -> Vec<String> {
    let errors = compile_error_messages(source);
    assert!(!errors.is_empty(), "expected compiler errors");
    errors
}

fn assert_choices(story: &Story, expected: &[&str]) {
    let choices = story.get_current_choices();
    assert_eq!(expected.len(), choices.len());
    for (choice, expected_text) in choices.iter().zip(expected) {
        assert_eq!(*expected_text, choice.text);
    }
}

fn choose(story: &mut Story, index: usize) -> String {
    story.choose_choice_index(index);
    story.cont_maximally()
}

// The tests below are ports of scenarios from pjohansson/inkling's `tests/`
// directory, whose upstream project is Parity-licensed. Unsupported LIST
// coverage is excluded, and Inkling-specific API assertions are converted into
// compiler/runtime behavior checks.

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
VAR count: int = 0

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
VAR a: int = 3
VAR b: int = 5

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

#[test]
fn inkling_story_can_be_read_with_unnamed_knots() {
    let mut story = compile_story(
        r#"
Mont Blanc was a world-renowned mountain guide.
He befriended thousands of climbers and children sightseeing in Switzerland.

-> DONE
"#,
    );

    assert_eq!(
        "Mont Blanc was a world-renowned mountain guide.\nHe befriended thousands of climbers and children sightseeing in Switzerland.\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_story_starts_from_top_even_if_knot_is_unnamed() {
    let mut story = compile_story(
        r#"
Mont Blanc was a world-renowned mountain guide.
He befriended thousands of climbers and children sightseeing in Switzerland.
-> dream

== dream ==
GESICHT'S BEDROOM, MORNING

Gesicht is lying in his bed, eyes wide open and staring at the ceiling.
He just woke from a nightmare.
-> END
"#,
    );

    assert_eq!(
        "Mont Blanc was a world-renowned mountain guide.\nHe befriended thousands of climbers and children sightseeing in Switzerland.\nGESICHT'S BEDROOM, MORNING\nGesicht is lying in his bed, eyes wide open and staring at the ceiling.\nHe just woke from a nightmare.\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_story_can_start_with_named_knot() {
    let mut story = compile_story(
        r#"
-> dream

== dream ==
GESICHT'S BEDROOM (MORNING)

Gesicht is lying in his bed, eyes wide open and staring at the ceiling.
He just woke from a nightmare.
-> END
"#,
    );

    assert_eq!(
        "GESICHT'S BEDROOM (MORNING)\nGesicht is lying in his bed, eyes wide open and staring at the ceiling.\nHe just woke from a nightmare.\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_story_can_divert_at_will_between_unordered_knots() {
    let mut story = compile_story(
        r#"
-> murder

== murder ==
SCENE OF MURDER (TRASHED APARTMENT, DAYTIME)

Gesicht arrives at a grotesque murder scene.

-> cops_hold_him

== investigate_body ==
The body is lying face down in a pool of blood.
A desk lamp and piece of broken wood have been stuck to his head.
They mimic the appearance of antlers.
-> END

== cops_hold_him
The lead detective stop him as he enters the room.
He identifies himself as being from Europol and passes the barrier.

-> investigate_body
"#,
    );

    assert_eq!(
        "SCENE OF MURDER (TRASHED APARTMENT, DAYTIME)\nGesicht arrives at a grotesque murder scene.\nThe lead detective stop him as he enters the room.\nHe identifies himself as being from Europol and passes the barrier.\nThe body is lying face down in a pool of blood.\nA desk lamp and piece of broken wood have been stuck to his head.\nThey mimic the appearance of antlers.\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_story_can_be_structured_using_stitches() {
    let mut story = compile_story(
        r#"
-> introduction

== introduction
Mont Blanc was a world-renowned mountain guide.
He befriended thousands of climbers and children sightseeing in Switzerland.
-> dream.wake

== dream
= interior
GESICHT'S BEDROOM, MORNING

= wake
Gesicht is lying in his bed, eyes wide open and staring at the ceiling.
He just woke from a nightmare.
-> END
"#,
    );

    assert_eq!(
        "Mont Blanc was a world-renowned mountain guide.\nHe befriended thousands of climbers and children sightseeing in Switzerland.\nGesicht is lying in his bed, eyes wide open and staring at the ceiling.\nHe just woke from a nightmare.\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_stitches_can_be_diverted_to_inside_a_knot_without_full_address() {
    let mut story = compile_story(
        r#"
-> introduction

== introduction
Mont Blanc was a world-renowned mountain guide.
He befriended thousands of climbers and children sightseeing in Switzerland.
-> dream.wake

== dream
= interior
GESICHT'S BEDROOM, MORNING

= breakfast
Before he's had the time to eat breakfast a call about a murder comes in.
As Helena probes him about leaving he suggests that they take a vacation.
-> END

= wake
Gesicht is lying in his bed, eyes wide open and staring at the ceiling.
He just woke from a nightmare.
-> breakfast
"#,
    );

    assert_eq!(
        "Mont Blanc was a world-renowned mountain guide.\nHe befriended thousands of climbers and children sightseeing in Switzerland.\nGesicht is lying in his bed, eyes wide open and staring at the ceiling.\nHe just woke from a nightmare.\nBefore he's had the time to eat breakfast a call about a murder comes in.\nAs Helena probes him about leaving he suggests that they take a vacation.\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_story_follows_choices_by_the_user() {
    let mut story = compile_story(
        r#"
Gesicht corners the fugitive and slams him to the ground.
-> cornered

== cornered ==
He points his gun hand in the fugitive's face point blank.
“Now, I can read you your rights and you'll let me arrest you.
“Or, I can shoot you with hypno-gas and you'll lose consciousness.
“Take your pick! Which is it?!”

*   Sirens approach and the police take him in.
*   The fugitive desperately fights back.
    -> fight

== fight ==
The fight barely lasts a moment before Gesicht sedates him with a large dose of gas.
“What was the point in all that?”
-> END
"#,
    );

    assert_eq!(
        "Gesicht corners the fugitive and slams him to the ground.\nHe points his gun hand in the fugitive's face point blank.\n“Now, I can read you your rights and you'll let me arrest you.\n“Or, I can shoot you with hypno-gas and you'll lose consciousness.\n“Take your pick! Which is it?!”\n",
        story.cont_maximally()
    );
    assert_choices(
        &story,
        &[
            "Sirens approach and the police take him in.",
            "The fugitive desperately fights back.",
        ],
    );

    assert_eq!(
        "The fugitive desperately fights back.\nThe fight barely lasts a moment before Gesicht sedates him with a large dose of gas.\n“What was the point in all that?”\n",
        choose(&mut story, 1)
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_following_a_choice_adds_choice_line_to_output() {
    let mut story = compile_story(
        r#"
*   Gesicht took the fugitive in.
"#,
    );

    assert_eq!("", story.cont_maximally());
    assert_choices(&story, &["Gesicht took the fugitive in."]);
    assert_eq!("Gesicht took the fugitive in.\n", choose(&mut story, 0));
    assert!(story.is_ended());
}

#[test]
fn inkling_choices_can_nest_into_multiple_levels() {
    let mut story = compile_story(
        r#"
Gesicht knocks on the door.
A robot in a frilly apron welcomes him in.

*   He steps in and informs the widow about her husband's death<>
    * *     He then offers his condolences.
    * *     He gives her husband's memory chip to her.
            * * *   He helps the widow insert it.
*   He informs about the death and leave the apartment.
"#,
    );

    assert_eq!(
        "Gesicht knocks on the door.\nA robot in a frilly apron welcomes him in.\n",
        story.cont_maximally()
    );
    assert_choices(
        &story,
        &[
            "He steps in and informs the widow about her husband's death",
            "He informs about the death and leave the apartment.",
        ],
    );
    assert_eq!(
        "He steps in and informs the widow about her husband's death",
        choose(&mut story, 0)
    );
    assert_choices(
        &story,
        &[
            "He then offers his condolences.",
            "He gives her husband's memory chip to her.",
        ],
    );
    assert_eq!(
        "He gives her husband's memory chip to her.\n",
        choose(&mut story, 1)
    );
    assert_choices(&story, &["He helps the widow insert it."]);
    assert_eq!("He helps the widow insert it.\n", choose(&mut story, 0));
    assert!(story.is_ended());
}

#[test]
fn inkling_choices_can_divert_in_their_lines() {
    let mut story = compile_story(
        r#"
Gesicht notices that a destroyed patrol bot is being thrown away.

*   Question the garbage worker -> question
*   Ignore it.

== question ==
“Excuse me sir, isn't that a patrol bot?”
-> END
"#,
    );

    assert_eq!(
        "Gesicht notices that a destroyed patrol bot is being thrown away.\n",
        story.cont_maximally()
    );
    assert_choices(&story, &["Question the garbage worker", "Ignore it."]);
    assert_eq!(
        "Question the garbage worker “Excuse me sir, isn't that a patrol bot?”\n",
        choose(&mut story, 0)
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_diverts_are_glue_and_add_single_whitespace_after_story_text() {
    let mut story = compile_story(
        r#"
“We all loved Mont Blanc.
“The memorial service will take place in three days ...
“This place will be filled with tens of thousands of people,-> view_over_stadium

== view_over_stadium ==
probably several hundred thousands.
“all mourning the death of mont blanc.”
-> END
"#,
    );

    assert_eq!(
        "“We all loved Mont Blanc.\n“The memorial service will take place in three days ...\n“This place will be filled with tens of thousands of people, probably several hundred thousands.\n“all mourning the death of mont blanc.”\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_choices_with_bracket_text_have_distinct_choice_and_output_text() {
    let mut story = compile_story(
        r#"
Gesicht descended into the prison.
The elevator doors swung open.
*   [Enter]He entered the storage where Brau 1589 was kept.
    Further in he encountered the robot.
    * *    “Brau 1589[...”],” he said.
            -> END
"#,
    );

    assert_eq!(
        "Gesicht descended into the prison.\nThe elevator doors swung open.\n",
        story.cont_maximally()
    );
    assert_choices(&story, &["Enter"]);
    assert_eq!(
        "He entered the storage where Brau 1589 was kept.\nFurther in he encountered the robot.\n",
        choose(&mut story, 0)
    );
    assert_choices(&story, &["“Brau 1589...”"]);
    assert_eq!("“Brau 1589,” he said.\n", choose(&mut story, 0));
    assert!(story.is_ended());
}

#[test]
fn inkling_gathers_collect_nested_choices_in_story() {
    let mut story = compile_story(
        r#"
Gesicht met with Brando.
They travelled to his apartment by car.
*   “That was an impressive match, Brando.”
    “It's getting tougher and tougher these days,” he replied.
    * *     “Your opponents all wear the same pancreatic suits.
            “But in the ring, you're always strongest.”
            “Any match is determined by who's got the most experience.”
            He stayed silent for a moment.
            “The rest is all luck,” he continued with a wry smile.
    * *     Gesicht thought it best to wait until they were there to have a talk.
    - -     Brando turned the radio on and <>
*   Gesicht said nothing and <>
- they stayed silent during the rest of the ride.
- -> END
"#,
    );

    assert_eq!(
        "Gesicht met with Brando.\nThey travelled to his apartment by car.\n",
        story.cont_maximally()
    );
    assert_choices(
        &story,
        &[
            "“That was an impressive match, Brando.”",
            "Gesicht said nothing and",
        ],
    );
    assert_eq!(
        "“That was an impressive match, Brando.”\n“It's getting tougher and tougher these days,” he replied.\n",
        choose(&mut story, 0)
    );
    assert_choices(
        &story,
        &[
            "“Your opponents all wear the same pancreatic suits.",
            "Gesicht thought it best to wait until they were there to have a talk.",
        ],
    );
    assert_eq!(
        "“Your opponents all wear the same pancreatic suits.\n“But in the ring, you're always strongest.”\n“Any match is determined by who's got the most experience.”\nHe stayed silent for a moment.\n“The rest is all luck,” he continued with a wry smile.\nBrando turned the radio on and they stayed silent during the rest of the ride.\n",
        choose(&mut story, 0)
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_invalid_choice_returns_error_information() {
    let mut story = compile_story(
        r#"
North no. 2 is standing in the bed room as the old man wakes up from his dream.

*   “North no. 2? Is that you?”
*   “I thought I told you to never enter my bedroom.”
    * *     “Sir, your breakfast is ready,” North no. 2 replies.
"#,
    );

    assert_eq!(
        "North no. 2 is standing in the bed room as the old man wakes up from his dream.\n",
        story.cont_maximally()
    );
    assert_choices(
        &story,
        &[
            "“North no. 2? Is that you?”",
            "“I thought I told you to never enter my bedroom.”",
        ],
    );
    assert_eq!(
        "“I thought I told you to never enter my bedroom.”\n",
        choose(&mut story, 1)
    );
    assert_choices(
        &story,
        &["“Sir, your breakfast is ready,” North no. 2 replies."],
    );
    assert!(story.try_choose_choice_index(1).is_err());
}

#[test]
fn inkling_glue_binds_lines_together_without_newline_markers() {
    let mut story = compile_story(
        r#"
“So she abandoned me ... <>
she sent me to a boarding school in England ...
<> and I never heard a thing from her again.”
-> END
"#,
    );

    assert_eq!(
        "“So she abandoned me ... she sent me to a boarding school in England ... and I never heard a thing from her again.”\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_glue_binds_across_diverts() {
    let mut story = compile_story(
        r#"
“So she abandoned me ... <>
-> flashback

== flashback
she sent me to a boarding school in England ...
<> and I never heard a thing from her again.”
-> END
"#,
    );

    assert_eq!(
        "“So she abandoned me ... she sent me to a boarding school in England ... and I never heard a thing from her again.”\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_tags_are_included_with_lines_and_choices() {
    let mut story = compile_story(
        r#"
SCOTLAND, EURO FEDERATION # location card # europe
An old castle heaves in front of you. # description
-> gates

== gates

*   Enter it. # action
"#,
    );

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
fn inkling_choices_can_be_filtered_by_visited_knots() {
    let mut story = compile_story(
        r#"
-> passage

== passage ==
A crossing! Which path do you take?

+   Left -> torch
+   Right -> dark_room

== dark_room ==
You enter a dark room.

+   {torch} Use your torch to light the way forward.
+   Head back.
-> passage

== torch ==
In a small chamber further in you find a torch.
You head back.
-> passage
"#,
    );

    assert_eq!(
        "A crossing! Which path do you take?\n",
        story.cont_maximally()
    );
    assert_choices(&story, &["Left", "Right"]);
    choose(&mut story, 1);
    assert_choices(&story, &["Head back."]);
    choose(&mut story, 0);
    assert_choices(&story, &["Left", "Right"]);
    choose(&mut story, 0);
    assert_choices(&story, &["Left", "Right"]);
    choose(&mut story, 1);
    assert_choices(
        &story,
        &["Use your torch to light the way forward.", "Head back."],
    );
}

#[test]
fn inkling_choices_can_filter_by_internal_stitch_shorthand() {
    let mut story = compile_story(
        r#"
-> exploring_the_tunnel

== exploring_the_tunnel
= passage
A crossing! Which path do you take?

+   Left -> torch
+   Right -> dark_room

= dark_room
You enter a dark room.

+   {torch} Use your torch to light the way forward.
+   Head back.
-> passage

= torch
In a small chamber further in you find a torch.
You head back.
-> passage
"#,
    );

    assert_eq!(
        "A crossing! Which path do you take?\n",
        story.cont_maximally()
    );
    assert_choices(&story, &["Left", "Right"]);
    choose(&mut story, 1);
    assert_choices(&story, &["Head back."]);
    choose(&mut story, 0);
    assert_choices(&story, &["Left", "Right"]);
    choose(&mut story, 0);
    assert_choices(&story, &["Left", "Right"]);
    choose(&mut story, 1);
    assert_choices(
        &story,
        &["Use your torch to light the way forward.", "Head back."],
    );
}

#[test]
fn inkling_choices_can_filter_by_stitches_outside_current_knot() {
    let mut story = compile_story(
        r#"
-> passage

== passage
A crossing! Which path do you take?

+   Left -> left_tunnel
+   Right -> dark_room

== dark_room ==
You enter a dark room.

+   {left_tunnel.torch} Use your torch to light the way forward.
+   Head back.
-> passage

== left_tunnel ==
= torch
In a small chamber further in you find a torch.
You head back.
-> passage
"#,
    );

    assert_eq!(
        "A crossing! Which path do you take?\n",
        story.cont_maximally()
    );
    assert_choices(&story, &["Left", "Right"]);
    choose(&mut story, 1);
    assert_choices(&story, &["Head back."]);
    choose(&mut story, 0);
    assert_choices(&story, &["Left", "Right"]);
    choose(&mut story, 0);
    assert_choices(&story, &["Left", "Right"]);
    choose(&mut story, 1);
    assert_choices(
        &story,
        &["Use your torch to light the way forward.", "Head back."],
    );
}

#[test]
fn inkling_fallback_choices_are_followed_when_no_choices_remain() {
    let mut story = compile_story(
        r#"
-> passage

== passage
A crossing! Which path do you take?

+   [Left] -> left_tunnel

== left_tunnel ==
{In a small chamber further in you find a torch.|This chamber used to hold a torch.}
*   [Pick up torch.] -> torch
*   ->
    <> But there is nothing left so you turn and head back.
    -> passage

== torch
You pick the torch up and head back.
-> passage
"#,
    );

    assert_eq!(
        "A crossing! Which path do you take?\n",
        story.cont_maximally()
    );
    assert_choices(&story, &["Left"]);
    assert_eq!(
        "In a small chamber further in you find a torch.\n",
        choose(&mut story, 0)
    );
    assert_choices(&story, &["Pick up torch."]);
    assert_eq!(
        "You pick the torch up and head back.\nA crossing! Which path do you take?\n",
        choose(&mut story, 0)
    );
    assert_choices(&story, &["Left"]);
    assert_eq!(
        "This chamber used to hold a torch. But there is nothing left so you turn and head back.\nA crossing! Which path do you take?\n",
        choose(&mut story, 0)
    );
}

#[test]
fn inkling_fallback_choices_may_include_text_or_direct_diverts() {
    let mut story = compile_story(
        r#"
-> passage

== passage
A crossing! Which path do you take?

+   [Left] -> left_tunnel

== left_tunnel ==
{In a small chamber further in you find a torch.|This chamber used to hold a torch.}
*   -> torch
+   ->
    But there is nothing left so you turn and head back.
    -> passage

== torch
You pick the torch up and head back.
+   -> passage
"#,
    );

    assert_eq!(
        "A crossing! Which path do you take?\n",
        story.cont_maximally()
    );
    assert_eq!(
        "In a small chamber further in you find a torch.\nYou pick the torch up and head back.\nA crossing! Which path do you take?\n",
        choose(&mut story, 0)
    );
    assert_eq!(
        "This chamber used to hold a torch.\nBut there is nothing left so you turn and head back.\nA crossing! Which path do you take?\n",
        choose(&mut story, 0)
    );
    assert_eq!(
        "This chamber used to hold a torch.\nBut there is nothing left so you turn and head back.\nA crossing! Which path do you take?\n",
        choose(&mut story, 0)
    );
}

#[test]
fn inkling_glue_binds_across_fallback_choices() {
    let mut story = compile_story(
        r#"
-> passage

== passage
A crossing! Which path do you take?

+   [Left] -> left_tunnel

== left_tunnel ==
{In a small chamber further in you find a torch.|This chamber used to hold a torch.} <>
*   -> torch
*   ->
    But there is nothing left so you turn and head back.
    -> passage

== torch
<> You pick the torch up and head back.
-> passage
"#,
    );

    assert_eq!(
        "A crossing! Which path do you take?\n",
        story.cont_maximally()
    );
    assert_eq!(
        "In a small chamber further in you find a torch. You pick the torch up and head back.\nA crossing! Which path do you take?\n",
        choose(&mut story, 0)
    );
    assert_eq!(
        "This chamber used to hold a torch. But there is nothing left so you turn and head back.\nA crossing! Which path do you take?\n",
        choose(&mut story, 0)
    );
}

#[test]
fn inkling_variant_sequences_can_be_nested() {
    let mut story = compile_story(
        r#"
-> start

== start
I {once|twice|have many times} met with a {gentleperson|friend|{&comrade|{&bud|pal}}} from Nantucket. {|||||We're besties.}

+   [Continue] -> start
"#,
    );

    let mut lines = vec![story.cont_maximally()];
    for _ in 0..5 {
        lines.push(choose(&mut story, 0));
    }

    assert_eq!(
        vec![
            "I once met with a gentleperson from Nantucket.\n",
            "I twice met with a friend from Nantucket.\n",
            "I have many times met with a comrade from Nantucket.\n",
            "I have many times met with a bud from Nantucket.\n",
            "I have many times met with a comrade from Nantucket.\n",
            "I have many times met with a pal from Nantucket. We're besties.\n",
        ],
        lines
    );
}

#[test]
fn inkling_choices_can_have_variants_in_selection_text() {
    let mut story = compile_story(
        r#"
-> meeting

== meeting
You meet with Aaron.

+   \ {Hi|Hi again|Hello}! -> meeting
+   \ {Oh, you again|Sorry, I want some me-time right now} -> meeting
"#,
    );

    assert_eq!("You meet with Aaron.\n", story.cont_maximally());
    assert_choices(&story, &["Hi!", "Oh, you again"]);
    choose(&mut story, 0);
    assert_choices(&story, &["Hello!", "Sorry, I want some me-time right now"]);
}

#[test]
fn inkling_lines_can_have_conditional_content() {
    let mut story = compile_story(
        r#"
-> root

== root
I {nantucket: {nantucket > 1: {nantucket > 2: many times | twice } | once } | have never} met {nantucket: with} a comrade from Nantucket.

+   [Go there] -> nantucket

== nantucket
-> root
"#,
    );

    assert_eq!(
        "I have never met a comrade from Nantucket.\n",
        story.cont_maximally()
    );
    assert_eq!(
        "I once met with a comrade from Nantucket.\n",
        choose(&mut story, 0)
    );
    assert_eq!(
        "I twice met with a comrade from Nantucket.\n",
        choose(&mut story, 0)
    );
    assert_eq!(
        "I many times met with a comrade from Nantucket.\n",
        choose(&mut story, 0)
    );
}

#[test]
fn inkling_mathematical_expressions_can_be_used_in_lines() {
    let mut story = compile_story(
        r#"
Adding is easy: <>
{2} + {3} is {2 + 3}!

Multiplication as so: <>
2 * (3 + 5) is {2 * (3 + 5)}!

Let's nest a bit: <>
{(2 + 3 * (4 - 2 * (10 / 2 + (1 + 3 * (((4))))) - 2))} is -100!

Strings can be added, too: <>
{"str" + "ing"} is string!
"#,
    );

    assert_eq!(
        "Adding is easy: 2 + 3 is 5!\nMultiplication as so: 2 * (3 + 5) is 16!\nLet's nest a bit: -100 is -100!\nStrings can be added, too: string is string!\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_variables_can_be_used_in_mathematical_operations() {
    let mut story = compile_story(
        r#"
VAR a: int = 3
VAR b: int = 5
VAR c: int = 13
VAR f: float = 3.0
VAR cf: float = 13.0
VAR bf: float = 5.0

Integer calculation does each step as integers, which may not be what you want.
({a} - {c}) / {a} + {b} = {(a - c) / a + b} which should be 1.66666...!

Float calculation works better:
({f} - {cf}) / {f} + {bf} = {(f - cf) / f + bf}!
"#,
    );

    assert_eq!(
        "Integer calculation does each step as integers, which may not be what you want.\n(3 - 13) / 3 + 5 = 2 which should be 1.66666...!\nFloat calculation works better:\n(3 - 13) / 3 + 5 = 1.6666667!\n",
        story.cont_maximally()
    );
    assert!(story.is_ended());
}

#[test]
fn inkling_variable_expressions_always_use_updated_variables() {
    let mut story = compile_story(
        r#"
VAR a: int = 3
VAR b: int = 5

-> root

== root
{root < 2: Before | After } updating `a`: a = {a}, b = {b}, a + b = {a + b}.

+   [After setting a = 7] -> root
"#,
    );

    assert_eq!(
        "Before updating `a`: a = 3, b = 5, a + b = 8.\n",
        story.cont_maximally()
    );
    story.set_variable("a", &ValueType::Int(7)).unwrap();
    assert_eq!(
        "Before updating `a`: a = 7, b = 5, a + b = 12.\n",
        choose(&mut story, 0)
    );
}

#[test]
fn inkling_conditions_may_use_expressions_on_left_or_right_hand_side() {
    let mut story = compile_story(
        r#"
VAR a: int = 3
VAR b: int = 5

{(a + 2) * 3 >= b * 3: True | False}
{(a + 2) * 3 <= b * 3: True | False}
{(a + 2) * 3 != b * 3: True | False}
{"str" + "ing" == "string": True | False}
"#,
    );

    assert_eq!("True\nTrue\nFalse\nTrue\n", story.cont_maximally());
    assert!(story.is_ended());
}

#[test]
fn inkling_global_variables_are_parsed_when_story_is_read() {
    let mut story = compile_story(
        r#"
VAR value: float = 3.6
VAR unit: string = "Röntgen"
VAR is_hazardous: bool = false

The latest measurement is {value} {unit}.
"#,
    );

    assert_eq!(
        "The latest measurement is 3.6 Röntgen.\n",
        story.cont_maximally()
    );
    assert_eq!(
        Some(ValueType::Bool(false)),
        story.get_variable("is_hazardous")
    );
}

#[test]
fn inkling_global_variables_can_be_changed_from_the_caller() {
    let mut story = compile_story(
        r#"
VAR value: float = 3.6
VAR unit: string = "Röntgen"
VAR is_hazardous: bool = false

The latest measurement is {value} {unit}.
"#,
    );

    story
        .set_variable("value", &ValueType::Float(15000.0))
        .unwrap();
    assert_eq!(
        "The latest measurement is 15000 Röntgen.\n",
        story.cont_maximally()
    );
}

#[test]
fn inkling_variables_can_be_used_in_conditions() {
    let mut story = compile_story(
        r#"
VAR value: float = 3.6
VAR threshold: float = 10.0
VAR unit: string = "Röntgen"

-> root

== root
The latest measurement is {value} {unit}. {value < threshold: Not terrible, not great. | Oh no.}

+   [Redo measurement] -> root
"#,
    );

    assert_eq!(
        "The latest measurement is 3.6 Röntgen. Not terrible, not great.\n",
        story.cont_maximally()
    );
    story
        .set_variable("value", &ValueType::Float(15000.0))
        .unwrap();
    assert_eq!(
        "The latest measurement is 15000 Röntgen. Oh no.\n",
        choose(&mut story, 0)
    );
}

#[test]
fn inkling_variables_can_change_and_influence_story_flow_conditions() {
    let mut story = compile_story(
        r#"
VAR value: float = 3.6
VAR unit: string = "Röntgen"
VAR is_hazardous: bool = false

-> root

== root
The latest measurement is {value} {unit}. {not is_hazardous: Not terrible, not great. | Oh no.}

+   [Redo measurement] -> root
"#,
    );

    assert_eq!(
        "The latest measurement is 3.6 Röntgen. Not terrible, not great.\n",
        story.cont_maximally()
    );
    story
        .set_variable("value", &ValueType::Float(15000.0))
        .unwrap();
    story
        .set_variable("is_hazardous", &ValueType::Bool(true))
        .unwrap();
    assert_eq!(
        "The latest measurement is 15000 Röntgen. Oh no.\n",
        choose(&mut story, 0)
    );
}

#[test]
fn inkling_saved_state_preserves_choice_filtering_context() {
    let mut story = compile_story(
        r#"
-> passage

== passage ==
A crossing! Which path do you take?

+   Left -> torch
+   Right -> dark_room

== dark_room ==
You enter a dark room.

+   {torch} Use your torch to light the way forward.
+   Head back.
-> passage

== torch ==
In a small chamber further in you find a torch.
You head back.
-> passage
"#,
    );

    story.cont_maximally();
    let mut state_without_torch = story.clone();
    choose(&mut state_without_torch, 1);
    assert_choices(&state_without_torch, &["Head back."]);

    choose(&mut story, 0);
    let mut state_with_torch = story.clone();
    choose(&mut state_with_torch, 1);
    assert_choices(
        &state_with_torch,
        &["Use your torch to light the way forward.", "Head back."],
    );
}

#[test]
fn inkling_address_validation_reports_unknown_knot_divert() {
    assert_compile_errors(
        r#"
== duckburg
-> bin

== money_bin
-> END
"#,
    );
}

#[test]
fn inkling_address_validation_reports_unknown_stitch_divert() {
    assert_compile_errors(
        r#"
-> duckburg.bin

== duckburg
= money_bin
-> END
"#,
    );
}

#[test]
fn inkling_address_validation_reports_unknown_relative_stitch() {
    assert_compile_errors(
        r#"
== duckburg
Welcome to Duck Burg!
-> bin

= money_bin
-> END
"#,
    );
}

#[test]
fn inkling_address_validation_reports_choice_text_divert_target_errors() {
    assert_compile_errors(
        r#"
== duckburg
Welcome to Duck Burg!
*   [Money bin] -> bin

= money_bin
-> END
"#,
    );
}

#[test]
fn inkling_address_validation_reports_alternative_sequence_divert_errors() {
    assert_compile_errors(
        r#"
== duckburg
Welcome to Duck Burg! {We live here.|We headed to Uncle Scrooge's money bin. -> bin}

== money_bin
-> END
"#,
    );
}

#[test]
fn inkling_address_validation_reports_nested_branch_divert_errors() {
    assert_compile_errors(
        r#"
== duckburg
*   Money bin
    We headed to the money bin. -> bin

== money_bin
-> END
"#,
    );
}

#[test]
fn inkling_address_validation_reports_condition_address_errors() {
    assert_compile_errors(
        r#"
== duckburg
*   {bin} But we had already visited the money bin.
*   -> END

== money_bin
-> END
"#,
    );
}

#[test]
fn inkling_addresses_in_choices_are_validated_without_raw_parse_nodes() {
    let errors = compile_error_messages(
        r#"
VAR variable: int = 0

*   This {variable} must not fail [] Nor this {variable}
*   Diverts should be the same -> knot
*   As should {variable == 0: addresses in conditions}

== knot
Empty knot.
"#,
    );
    assert!(errors.is_empty(), "{errors:#?}");
}
