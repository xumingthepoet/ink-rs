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

fn compile_story(source: &str) -> Story {
    let result = Compiler::new().compile(SourceInput::new(explicit_game_module(source)));
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

fn compile_error_messages(source: &str) -> Vec<String> {
    Compiler::new()
        .compile(SourceInput::new(explicit_game_module(source)))
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

fn choose(story: &mut Story, index: usize) -> String {
    story.choose_choice_index(index);
    story.cont_maximally()
}

fn explicit_game_module(source: &str) -> String {
    let source = source.trim_matches('\n');
    if source.contains("=== module ") {
        return source.to_string();
    }

    let lines = source.lines().collect::<Vec<_>>();
    let mut index = 0;
    let mut declarations = Vec::new();
    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if trimmed.is_empty() {
            declarations.push(lines[index]);
            index += 1;
            continue;
        }
        if trimmed.starts_with("VAR ")
            || trimmed.starts_with("CONST ")
            || trimmed.starts_with("EXTERNAL ")
        {
            declarations.push(lines[index]);
            index += 1;
            continue;
        }
        if trimmed.starts_with("STRUCT ") {
            declarations.push(lines[index]);
            index += 1;
            while index < lines.len() {
                declarations.push(lines[index]);
                let field_line = lines[index].trim();
                index += 1;
                if field_line == "}" {
                    break;
                }
            }
            continue;
        }
        break;
    }

    let body = lines[index..].join("\n");
    let mut module = String::from("=== module game ===\n");
    if !declarations.is_empty() {
        module.push_str(&declarations.join("\n"));
        module.push('\n');
    }
    module.push_str("== main ==\n");
    if let Some(first_flow) = first_flow_name(&body) {
        module.push_str("-> ");
        module.push_str(first_flow);
        module.push('\n');
    }
    module.push_str(&body);
    if !body.contains("-> END") && !body.contains("-> DONE") {
        module.push_str("\n-> END");
    }
    module
}

fn first_flow_name(source: &str) -> Option<&str> {
    let first_content = source.lines().find(|line| !line.trim().is_empty())?.trim();
    if !first_content.starts_with("==") || first_content.starts_with("== function ") {
        return None;
    }

    first_content.trim_matches('=').split_whitespace().next()
}

// The tests below are ports of scenarios from pjohansson/inkling's `tests/`
// directory, whose upstream project is Parity-licensed. Unsupported LIST
// coverage is excluded, and Inkling-specific API assertions are converted into
// compiler/runtime behavior checks.

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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
    assert_eq!(
        Some(7),
        story
            .get_variable("game::a")
            .and_then(|value| value.get::<i32>())
    );
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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
    assert!(story_is_ended(&story));
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
        Some(false),
        story
            .get_variable("game::is_hazardous")
            .and_then(|value| value.get::<bool>())
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
        .set_variable("game::value", &ValueType::Float(15000.0))
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

+   Redo measurement -> root
"#,
    );

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
    let mut story = compile_story(
        r#"
VAR value: float = 3.6
VAR unit: string = "Röntgen"
VAR is_hazardous: bool = false

-> root

== root
The latest measurement is {value} {unit}. {not is_hazardous: Not terrible, not great. | Oh no.}

+   Redo measurement -> root
"#,
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
*   Money bin -> bin

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

*   This {variable} must not fail  Nor this {variable}
*   Diverts should be the same -> knot
*   As should {variable == 0: addresses in conditions}

== knot
Empty knot.
"#,
    );
    assert!(errors.is_empty(), "{errors:#?}");
}
