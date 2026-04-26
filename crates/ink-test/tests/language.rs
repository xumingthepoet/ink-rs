use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use ink_compiler::{Compiler, Diagnostic, DiagnosticSeverity, SourceInput};
use ink_runtime::{
    story::{external_functions::ExternalFunction, Story},
    value_type::ValueType,
};
use serde_json::{json, Value};

fn language_fixture_text(filename: &str) -> String {
    ink_test::load_fixture_text(&format!("language/{filename}"))
}

fn compile_language_fixture(filename: &str) -> ink_compiler::CompiledStory {
    compile_language_source(filename, language_fixture_text(filename))
}

fn compile_language_source(name: &str, source: impl Into<String>) -> ink_compiler::CompiledStory {
    let output = Compiler::default().compile(SourceInput::named(source, name));
    assert!(
        output.diagnostics.is_empty(),
        "compile for {name} should not emit diagnostics: {:#?}",
        output.diagnostics
    );
    output.artifact.expect("expected compiled story")
}

fn diagnostics_for_language_source(name: &str, source: impl Into<String>) -> Vec<Diagnostic> {
    let output = Compiler::default().compile(SourceInput::named(source, name));
    assert!(
        output.artifact.is_none(),
        "compile for {name} should fail when asserting diagnostics"
    );
    output.diagnostics
}

fn assert_diagnostic(
    diagnostics: &[Diagnostic],
    severity: DiagnosticSeverity,
    message_fragment: &str,
) {
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == severity
                && diagnostic.message.contains(message_fragment)),
        "expected {severity:?} diagnostic containing {message_fragment:?}, got {diagnostics:#?}"
    );
}

fn assert_story_output(compiled: &ink_compiler::CompiledStory, expected: &str) {
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");
    let output = story
        .continue_maximally()
        .expect("compiled story should run");
    assert_eq!(output, expected);
    assert!(
        story.get_current_errors().is_empty(),
        "story should not emit runtime errors: {:#?}",
        story.get_current_errors()
    );
}

fn assert_json_sequence(json: &Value, sequence: Vec<Value>) {
    assert!(
        json_contains_sequence(json, &sequence),
        "expected compiled JSON to contain sequence {sequence:?}, got {json:#}"
    );
}

fn json_contains_sequence(value: &Value, sequence: &[Value]) -> bool {
    match value {
        Value::Array(items) => {
            items
                .windows(sequence.len())
                .any(|window| window == sequence)
                || items
                    .iter()
                    .any(|item| json_contains_sequence(item, sequence))
        }
        Value::Object(fields) => fields
            .values()
            .any(|field| json_contains_sequence(field, sequence)),
        _ => false,
    }
}

fn json_contains_divert_target(value: &Value, predicate: &impl Fn(&str) -> bool) -> bool {
    match value {
        Value::Array(items) => items
            .iter()
            .any(|item| json_contains_divert_target(item, predicate)),
        Value::Object(fields) => {
            fields
                .get("->")
                .and_then(Value::as_str)
                .is_some_and(predicate)
                || fields
                    .values()
                    .any(|field| json_contains_divert_target(field, predicate))
        }
        _ => false,
    }
}

#[test]
fn language_fixture_smoke_test_runs_compiled_story() {
    let compiled = compile_language_fixture("smoke.ink");
    assert_story_output(&compiled, "Hello from the language test surface.\n");
}

#[test]
fn typed_primitives_fixture_runs() {
    let compiled = compile_language_fixture("typed/primitives.ink");
    assert_story_output(&compiled, "0|1.5|true|Ada Lovelace|2|true\n");
}

#[test]
fn typed_struct_fixture_runs() {
    let compiled = compile_language_fixture("typed/structs.ink");
    assert_story_output(&compiled, "10|3|false|Ada\n");
}

#[test]
fn typed_array_fixture_runs() {
    let compiled = compile_language_fixture("typed/arrays.ink");
    assert_story_output(&compiled, "3|1|10|3\n2|10|3\n");
}

#[test]
fn typed_divert_target_fixture_runs() {
    let compiled = compile_language_fixture("typed/divert-targets.ink");
    assert_story_output(&compiled, "Here.\nStruct.\nArray.\nFallback.\n");
}

#[test]
fn explicit_dynamic_diverts_run_at_runtime() {
    let compiled = compile_language_source(
        "explicit-dynamic-diverts.ink",
        concat!(
            "VAR next: -> = -> first\n",
            "CONST fallback: -> = -> const_target\n",
            "CONST const_targets: ->[] = [-> const_array_target]\n",
            "VAR targets: ->[] = [-> array_target]\n",
            "STRUCT Route {\n",
            "next: ->\n",
            "}\n",
            "VAR route: Route = { next: -> struct_target }\n",
            "CONST const_route: Route = { next: -> const_struct_target }\n",
            "-> {next}\n",
            "== first ==\n",
            "First.\n",
            "-> {route.next}\n",
            "== struct_target ==\n",
            "Struct.\n",
            "-> {targets[0]}\n",
            "== array_target ==\n",
            "Array.\n",
            "-> {fallback}\n",
            "== const_target ==\n",
            "Const.\n",
            "-> {const_route.next}\n",
            "== const_struct_target ==\n",
            "Const struct.\n",
            "-> {const_targets[0]}\n",
            "== const_array_target ==\n",
            "Const array.\n",
            "-> {pick(true)}\n",
            "== function pick(flag: bool) => -> ==\n",
            "{ flag:\n",
            "    ~ return -> final\n",
            "- else:\n",
            "    ~ return -> first\n",
            "}\n",
            "== final ==\n",
            "Final.\n",
            "-> END",
        ),
    );

    assert_story_output(
        &compiled,
        "First.\nStruct.\nArray.\nConst.\nConst struct.\nConst array.\nFinal.\n",
    );
}

#[test]
fn explicit_dynamic_diverts_support_arguments() {
    let compiled = compile_language_source(
        "explicit-dynamic-divert-args.ink",
        concat!(
            "VAR next: -> = -> target\n",
            "VAR value: int = 5\n",
            "-> {next}(value)\n",
            "== target(x: int) ==\n",
            "Value {x}.\n",
            "-> END",
        ),
    );

    assert_story_output(&compiled, "Value 5.\n");
}

#[test]
fn explicit_dynamic_tunnels_run_at_runtime() {
    let compiled = compile_language_source(
        "explicit-dynamic-tunnel.ink",
        concat!(
            "VAR next: -> = -> tunnel\n",
            "-> {next} ->\n",
            "After.\n",
            "-> DONE\n",
            "== tunnel ==\n",
            "Inside.\n",
            "->->",
        ),
    );

    assert_story_output(&compiled, "Inside.\nAfter.\n");
}

#[test]
fn static_diverts_to_variables_report_current_syntax_error() {
    let cases = [
        (
            "old-global-divert.ink",
            concat!(
                "VAR next: -> = -> target\n",
                "-> next\n",
                "== target ==\n",
                "-> DONE"
            ),
        ),
        (
            "old-const-divert.ink",
            concat!(
                "CONST next: -> = -> target\n",
                "-> next\n",
                "== target ==\n",
                "-> DONE"
            ),
        ),
        (
            "old-param-divert.ink",
            concat!(
                "-> start(-> target)\n",
                "== start(next: ->) ==\n",
                "-> next\n",
                "== target ==\n",
                "-> DONE"
            ),
        ),
        (
            "old-temp-divert.ink",
            concat!(
                "~ temp next: -> = -> target\n",
                "-> next\n",
                "== target ==\n",
                "-> DONE"
            ),
        ),
    ];

    for (name, source) in cases {
        let diagnostics = diagnostics_for_language_source(name, source);
        assert_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Static divert targets must be knot or stitch paths",
        );
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, "-> {next}");
    }
}

#[test]
fn dynamic_divert_target_type_is_checked() {
    let diagnostics = diagnostics_for_language_source(
        "dynamic-divert-wrong-type.ink",
        concat!("VAR value: int = 1\n", "-> {value}\n"),
    );

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Dynamic divert target has type int but expected ->",
    );

    let diagnostics = diagnostics_for_language_source(
        "dynamic-divert-wrong-field-type.ink",
        concat!(
            "STRUCT Player {\n",
            "hp: int\n",
            "}\n",
            "VAR player: Player = { hp: 10 }\n",
            "-> {player.hp}\n",
            "== player ==\n",
            "= hp\n",
            "-> DONE",
        ),
    );

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Dynamic divert target has type int but expected ->",
    );
}

#[test]
fn square_brackets_in_choice_text_are_literal() {
    let compiled = compile_language_source(
        "literal-choice-brackets.ink",
        concat!(
            "* Display [selected output]\n",
            "    Branch.\n",
            "    -> DONE"
        ),
    );
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");

    assert_eq!(story.continue_maximally().unwrap(), "");
    assert_eq!(
        story.get_current_choices()[0].text,
        "Display [selected output]"
    );
    story.choose_choice_index(0).unwrap();
    assert_eq!(story.continue_maximally().unwrap(), "Branch.\n");
}

#[test]
fn star_and_plus_choices_are_repeatable() {
    let compiled = compile_language_source(
        "repeatable-choices.ink",
        concat!(
            "-> menu\n",
            "== menu ==\n",
            "* Star choice\n",
            "    Star branch.\n",
            "    -> menu\n",
            "+ Plus choice\n",
            "    Plus branch.\n",
            "    -> menu",
        ),
    );
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");

    assert_eq!(story.continue_maximally().unwrap(), "");
    assert_eq!(story.get_current_choices().len(), 2);
    story.choose_choice_index(0).unwrap();
    assert_eq!(story.continue_maximally().unwrap(), "Star branch.\n");
    assert_eq!(story.get_current_choices().len(), 2);
    story.choose_choice_index(1).unwrap();
    assert_eq!(story.continue_maximally().unwrap(), "Plus branch.\n");
    assert_eq!(story.get_current_choices().len(), 2);
}

#[test]
fn selected_choice_text_is_not_echoed() {
    let compiled = compile_language_source(
        "choice-display-only.ink",
        concat!("* Display text\n", "    Branch text.\n", "    -> DONE",),
    );
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");

    assert_eq!(story.continue_maximally().unwrap(), "");
    assert_eq!(story.get_current_choices()[0].text, "Display text");
    story.choose_choice_index(0).unwrap();
    assert_eq!(story.continue_maximally().unwrap(), "Branch text.\n");
}

#[test]
fn choice_conditions_still_control_visibility() {
    let compiled = compile_language_source(
        "conditional-repeatable-choices.ink",
        concat!(
            "VAR open: bool = false\n",
            "-> menu\n",
            "== menu ==\n",
            "* {open} Open path\n",
            "    Done.\n",
            "    -> DONE\n",
            "* Toggle\n",
            "    ~ open = true\n",
            "    -> menu",
        ),
    );
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");

    assert_eq!(story.continue_maximally().unwrap(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].text, "Toggle");
    story.choose_choice_index(0).unwrap();
    assert_eq!(story.continue_maximally().unwrap(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text, "Open path");
    assert_eq!(choices[1].text, "Toggle");
}

#[test]
fn save_load_preserves_generated_choices_without_regeneration() {
    let compiled = compile_language_source(
        "choice-save-load.ink",
        concat!(
            "VAR picked: int = 0\n",
            "* First\n",
            "    ~ picked = 1\n",
            "    Picked {picked}.\n",
            "    -> DONE\n",
            "* Second\n",
            "    ~ picked = 2\n",
            "    Picked {picked}.\n",
            "    -> DONE",
        ),
    );
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");

    assert_eq!(story.continue_maximally().unwrap(), "");
    assert_eq!(story.get_current_choices().len(), 2);
    let save_string = story.save_state().expect("choice state should save");
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert_eq!(save["inkSaveVersion"], json!(2));
    assert!(save.get("currentChoices").is_some());
    assert!(save.get("flows").is_none());
    assert!(save.get("evalStack").is_none());
    assert!(save.get("visitCounts").is_none());
    assert!(save.get("turnIndices").is_none());
    assert!(save.get("turnIdx").is_none());

    let mut reloaded = Story::new(&compiled.json).expect("compiled JSON should load");
    reloaded
        .load_state(&save_string)
        .expect("choice state should reload");
    let choices = reloaded.get_current_choices();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text, "First");
    assert_eq!(choices[1].text, "Second");
    reloaded.choose_choice_index(1).unwrap();
    assert_eq!(reloaded.continue_maximally().unwrap(), "Picked 2.\n");
}

#[test]
fn save_load_preserves_thread_generated_choices() {
    let compiled = compile_language_source(
        "thread-choice-save-load.ink",
        concat!(
            "<- side\n",
            "* Main\n",
            "    Main branch.\n",
            "    -> DONE\n",
            "== side ==\n",
            "* Thread\n",
            "    Thread branch.\n",
            "    -> DONE",
        ),
    );
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");

    assert_eq!(story.continue_maximally().unwrap(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text, "Thread");
    assert_eq!(choices[1].text, "Main");
    let save_string = story.save_state().expect("thread choice state should save");
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert!(save.get("choiceThreads").is_some());

    let mut reloaded = Story::new(&compiled.json).expect("compiled JSON should load");
    reloaded
        .load_state(&save_string)
        .expect("thread choice state should reload");
    assert_eq!(reloaded.get_current_choices().len(), 2);
    reloaded.choose_choice_index(0).unwrap();
    assert_eq!(reloaded.continue_maximally().unwrap(), "Thread branch.\n");
}

#[test]
fn save_load_preserves_deterministic_random_state() {
    let compiled = compile_language_source(
        "random-save-load.ink",
        concat!(
            "~ SEED_RANDOM(12)\n",
            "{RANDOM(1, 100)}\n",
            "* Continue\n",
            "    {RANDOM(1, 100)}\n",
            "    -> DONE",
        ),
    );
    let mut uninterrupted = Story::new(&compiled.json).expect("compiled JSON should load");
    let uninterrupted_first_output = uninterrupted.continue_maximally().unwrap();
    uninterrupted.choose_choice_index(0).unwrap();
    let uninterrupted_output = uninterrupted.continue_maximally().unwrap();

    let mut saved = Story::new(&compiled.json).expect("compiled JSON should load");
    assert_eq!(
        saved.continue_maximally().unwrap(),
        uninterrupted_first_output
    );
    let save_string = saved.save_state().expect("random state should save");
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert!(save.get("storySeed").is_some());
    assert!(save.get("previousRandom").is_some());
    assert!(save.get("turnIdx").is_none());

    let mut reloaded = Story::new(&compiled.json).expect("compiled JSON should load");
    reloaded
        .load_state(&save_string)
        .expect("random state should reload");
    reloaded.choose_choice_index(0).unwrap();
    assert_eq!(reloaded.continue_maximally().unwrap(), uninterrupted_output);
}

#[test]
fn typed_nested_fixture_runs() {
    let compiled = compile_language_fixture("typed/nested.ink");
    assert_story_output(&compiled, "2|0|4|1|false\n");
}

#[test]
fn typed_function_fixture_runs() {
    let compiled = compile_language_fixture("typed/functions.ink");
    assert_story_output(&compiled, "5|Ada!|7\n2|3\n");
}

#[test]
fn typed_tco_fixture_runs() {
    let compiled = compile_language_fixture("typed/tail-recursion.ink");
    let json = compiled.program.to_json_value();
    assert!(
        json_contains_divert_target(&json, &|target| target.ends_with(".2")),
        "tail recursion should lower to a direct jump to the function body after parameter reassignment, got {json:#}"
    );
    assert_story_output(&compiled, "1500|15\n");
}

#[test]
fn typed_default_initializers_run_at_runtime() {
    let compiled = compile_language_source(
        "typed-defaults.ink",
        concat!(
            "STRUCT Player {\n",
            "hp: int\n",
            "name: string\n",
            "ready: bool\n",
            "inventory: int[]\n",
            "}\n",
            "VAR global_score: int\n",
            "VAR global_ready: bool\n",
            "VAR global_label: string\n",
            "VAR global_ratio: float\n",
            "VAR global_values: int[]\n",
            "VAR global_player: Player\n",
            "~ temp temp_score: int\n",
            "~ temp temp_values: int[]\n",
            "~ temp temp_player: Player\n",
            "{global_score}|{global_ready}|{global_label}|{global_ratio}|{global_values}|{global_player}|{temp_score}|{temp_values}|{temp_player}\n",
            "-> DONE",
        ),
    );

    assert_story_output(
        &compiled,
        "0|false||0|[]|{hp: 0, inventory: [], name: , ready: false}|0|[]|{hp: 0, inventory: [], name: , ready: false}\n",
    );
}

#[test]
fn typed_default_initializers_are_lowered_to_json() {
    let compiled = compile_language_source(
        "typed-defaults-json.ink",
        concat!(
            "STRUCT Player {\n",
            "hp: int\n",
            "name: string\n",
            "ready: bool\n",
            "inventory: int[]\n",
            "}\n",
            "VAR global_score: int\n",
            "VAR global_values: int[]\n",
            "VAR global_player: Player\n",
            "~ temp temp_score: int\n",
            "~ temp temp_values: int[]\n",
            "~ temp temp_player: Player\n",
            "-> DONE",
        ),
    );
    let json = compiled.program.to_json_value();
    let default_player = json!({
        "hp": 0,
        "inventory": [],
        "name": "^",
        "ready": false,
    });

    assert_json_sequence(&json, vec![json!(0), json!({"VAR=": "global_score"})]);
    assert_json_sequence(&json, vec![json!([]), json!({"VAR=": "global_values"})]);
    assert_json_sequence(
        &json,
        vec![default_player.clone(), json!({"VAR=": "global_player"})],
    );
    assert_json_sequence(
        &json,
        vec![
            json!("ev"),
            json!(0),
            json!("/ev"),
            json!({"temp=": "temp_score"}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![
            json!("ev"),
            json!([]),
            json!("/ev"),
            json!({"temp=": "temp_values"}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![
            json!("ev"),
            default_player,
            json!("/ev"),
            json!({"temp=": "temp_player"}),
        ],
    );
}

#[test]
fn array_literals_run_at_runtime() {
    let compiled = compile_language_source(
        "array-literals.ink",
        concat!(
            "STRUCT Player {\n",
            "hp: int\n",
            "name: string\n",
            "}\n",
            "VAR numbers: int[] = [1, 2, 3]\n",
            "VAR matrix: int[][] = [[1, 2], []]\n",
            "VAR party: Player[] = [{ hp: 10, name: \"Ada\" }]\n",
            "{numbers}|{matrix}|{party}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "[1, 2, 3]|[[1, 2], []]|[{hp: 10, name: Ada}]\n");
}

#[test]
fn empty_struct_arrays_load_as_values_at_runtime() {
    let compiled = compile_language_source(
        "empty-struct-array.ink",
        concat!(
            "STRUCT Marker {\n",
            "}\n",
            "VAR markers: Marker[] = [{}, {}]\n",
            "{LEN(markers)}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "2\n");
}

#[test]
fn typed_constants_support_struct_and_array_values() {
    let compiled = compile_language_source(
        "typed-constants.ink",
        concat!(
            "STRUCT Stats {\n",
            "hp: int\n",
            "ready: bool\n",
            "}\n",
            "CONST default_stats: Stats = { hp: 7 }\n",
            "CONST party: Stats[] = [{ hp: 1 }, {}]\n",
            "VAR copied_stats: Stats = default_stats\n",
            "VAR copied_party: Stats[] = party\n",
            "{default_stats}|{party}|{copied_stats}|{copied_party}\n",
            "-> DONE",
        ),
    );

    assert_story_output(
        &compiled,
        "{hp: 7, ready: false}|[{hp: 1, ready: false}, {hp: 0, ready: false}]|{hp: 7, ready: false}|[{hp: 1, ready: false}, {hp: 0, ready: false}]\n",
    );
}

#[test]
fn struct_literals_run_at_runtime() {
    let compiled = compile_language_source(
        "struct-literals.ink",
        concat!(
            "STRUCT Stats {\n",
            "hp: int\n",
            "ready: bool\n",
            "}\n",
            "STRUCT Player {\n",
            "name: string\n",
            "stats: Stats\n",
            "tags: string[]\n",
            "}\n",
            "VAR full: Player = { name: \"Ada\", stats: { hp: 10, ready: true }, tags: [\"scout\"] }\n",
            "VAR partial: Player = { name: \"Bea\" }\n",
            "VAR nested: Stats = { hp: 3 }\n",
            "{full}|{partial}|{nested}\n",
            "-> DONE",
        ),
    );

    assert_story_output(
        &compiled,
        "{name: Ada, stats: {hp: 10, ready: true}, tags: [scout]}|{name: Bea, stats: {hp: 0, ready: false}, tags: []}|{hp: 3, ready: false}\n",
    );
}

#[test]
fn field_access_reads_struct_fields_at_runtime() {
    let compiled = compile_language_source(
        "field-access.ink",
        concat!(
            "STRUCT Stats {\n",
            "hp: int\n",
            "ready: bool\n",
            "}\n",
            "VAR state: Stats = { hp: 9, ready: true }\n",
            "{state.hp}|{state.ready}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "9|true\n");
    assert_json_sequence(
        &compiled.program.to_json_value(),
        vec![json!({"VAR?": "state"}), json!("^hp"), json!("FIELD")],
    );
}

#[test]
fn field_access_prefers_visible_variables_over_matching_story_paths() {
    let compiled = compile_language_source(
        "field-access-label-shadow.ink",
        concat!(
            "STRUCT Player {\n",
            "hp: int\n",
            "}\n",
            "VAR player: Player = { hp: 7 }\n",
            "{player.hp}\n",
            "-> DONE\n",
            "== player ==\n",
            "= hp\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "7\n");
    assert_json_sequence(
        &compiled.program.to_json_value(),
        vec![json!({"VAR?": "player"}), json!("^hp"), json!("FIELD")],
    );
}

#[test]
fn index_access_reads_array_items_at_runtime() {
    let compiled = compile_language_source(
        "index-access.ink",
        concat!(
            "VAR items: int[] = [4, 9]\n",
            "{items[0]}|{items[1]}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "4|9\n");
    assert_json_sequence(
        &compiled.program.to_json_value(),
        vec![json!({"VAR?": "items"}), json!(0), json!("INDEX")],
    );
}

#[test]
fn field_assignment_writes_struct_fields_at_runtime() {
    let compiled = compile_language_source(
        "field-assignment.ink",
        concat!(
            "STRUCT Stats {\n",
            "hp: int\n",
            "ready: bool\n",
            "}\n",
            "VAR state: Stats = { hp: 2, ready: false }\n",
            "~ state.hp = 5\n",
            "~ state.ready = true\n",
            "{state.hp}|{state.ready}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "5|true\n");
}

#[test]
fn field_assignment_copies_struct_values_at_runtime() {
    let compiled = compile_language_source(
        "field-assignment-copy.ink",
        concat!(
            "STRUCT Player {\n",
            "hp: int\n",
            "}\n",
            "VAR p1: Player = { hp: 3 }\n",
            "VAR p2: Player = p1\n",
            "~ p2.hp = 1\n",
            "{p1.hp}|{p2.hp}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "3|1\n");
}

#[test]
fn index_assignment_writes_array_items_at_runtime() {
    let compiled = compile_language_source(
        "index-assignment.ink",
        concat!(
            "VAR items: int[] = [1, 2, 3]\n",
            "~ items[1] = 10\n",
            "{items[0]}|{items[1]}|{items[2]}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "1|10|3\n");
}

#[test]
fn index_assignment_copies_array_values_at_runtime() {
    let compiled = compile_language_source(
        "index-assignment-copy.ink",
        concat!(
            "VAR items1: int[] = [4, 5]\n",
            "VAR items2: int[] = items1\n",
            "~ items2[0] = 9\n",
            "{items1[0]}|{items2[0]}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "4|9\n");
}

#[test]
fn compound_field_and_index_assignment_run_at_runtime() {
    let compiled = compile_language_source(
        "compound-field-index-assignment.ink",
        concat!(
            "STRUCT Player {\n",
            "hp: int\n",
            "name: string\n",
            "}\n",
            "VAR state: Player = { hp: 4, name: \"Ada\" }\n",
            "VAR items: int[] = [1]\n",
            "~ state.hp += 1\n",
            "~ state.name += \"!\"\n",
            "~ items[0] += 1\n",
            "{state.hp}|{state.name}|{items[0]}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "5|Ada!|2\n");
}

#[test]
fn compound_index_assignment_evaluates_index_once() {
    let compiled = compile_language_source(
        "compound-index-single-eval.ink",
        concat!(
            "VAR calls: int = 0\n",
            "VAR items: int[] = [1, 2]\n",
            "~ items[idx()] += 1\n",
            "{items[0]}|{calls}\n",
            "-> DONE\n",
            "=== function idx() => int ===\n",
            "~ calls += 1\n",
            "~ return 0",
        ),
    );

    assert_story_output(&compiled, "2|1\n");
}

#[test]
fn len_returns_array_length_at_runtime() {
    let compiled = compile_language_source(
        "len.ink",
        concat!(
            "STRUCT Player {\n",
            "hp: int\n",
            "}\n",
            "VAR empty: int[] = []\n",
            "VAR items: int[] = [1, 2, 3]\n",
            "VAR players: Player[] = [{ hp: 1 }, { hp: 2 }]\n",
            "{LEN(empty)}|{LEN(items)}|{LEN(players)}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "0|3|2\n");
}

#[test]
fn array_remove_mutates_arrays_and_returns_void_at_runtime() {
    let compiled = compile_language_source(
        "array-remove.ink",
        concat!(
            "STRUCT Player {\n",
            "hp: int\n",
            "}\n",
            "STRUCT Bag {\n",
            "items: int[]\n",
            "}\n",
            "VAR items: int[] = [1, 2, 3, 4]\n",
            "VAR players: Player[] = [{ hp: 1 }, { hp: 2 }]\n",
            "VAR nested: int[][] = [[1], [2, 3]]\n",
            "VAR bag: Bag = { items: [8, 9] }\n",
            "before{ARRAY_REMOVE(items, 0)}after|{items[0]}|{LEN(items)}\n",
            "~ ARRAY_REMOVE(items, 1)\n",
            "{items[0]}|{items[1]}|{LEN(items)}\n",
            "~ ARRAY_REMOVE(items, 1)\n",
            "{items[0]}|{LEN(items)}\n",
            "~ ARRAY_REMOVE(players, 0)\n",
            "{players[0].hp}|{LEN(players)}\n",
            "~ ARRAY_REMOVE(nested[1], 0)\n",
            "{nested[1][0]}|{LEN(nested[1])}\n",
            "~ ARRAY_REMOVE(bag.items, 0)\n",
            "{bag.items[0]}|{LEN(bag.items)}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "beforeafter|2|3\n2|4|2\n2|1\n2|1\n3|1\n9|1\n");
}

#[test]
fn string_concatenation_runs_at_runtime() {
    let compiled = compile_language_source(
        "string-concat.ink",
        concat!(
            "VAR greeting: string = \"Hello\"\n",
            "VAR name: string = \"Ada\"\n",
            "{greeting + \", \" + name + \"!\"}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "Hello, Ada!\n");
}

#[test]
fn mixed_string_addition_reports_diagnostics() {
    let cases = [
        (
            "string-int.ink",
            concat!(
                "VAR text: string = \"Ada\"\n",
                "VAR score: int = 1\n",
                "VAR result: string = text + score\n",
                "-> DONE",
            ),
            "Operator '+' is not defined for types string and int",
        ),
        (
            "int-string.ink",
            concat!(
                "VAR text: string = \"Ada\"\n",
                "VAR score: int = 1\n",
                "VAR result: string = score + text\n",
                "-> DONE",
            ),
            "Operator '+' is not defined for types int and string",
        ),
    ];

    for (name, source, expected_message) in cases {
        let diagnostics = diagnostics_for_language_source(name, source);
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
    }
}

struct TypedExternal;

impl ExternalFunction for TypedExternal {
    fn call(&mut self, func_name: &str, args: Vec<ValueType>) -> Option<ValueType> {
        match func_name {
            "next_score" => {
                assert!(args == vec![ValueType::Int(4)]);
                Some(ValueType::Int(5))
            }
            "make_scores" => {
                assert!(args.is_empty());
                Some(ValueType::Array(vec![ValueType::Int(2), ValueType::Int(3)]))
            }
            "make_player" => {
                assert!(args.is_empty());
                let mut fields = BTreeMap::new();
                fields.insert("hp".to_string(), ValueType::Int(7));
                Some(ValueType::Object(fields))
            }
            _ => panic!("unexpected external function: {func_name}"),
        }
    }
}

#[test]
fn typed_external_fixture_runs() {
    let compiled = compile_language_fixture("typed/externals.ink");
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");
    for name in ["next_score", "make_scores", "make_player"] {
        story
            .bind_external_function(name, Rc::new(RefCell::new(TypedExternal)), true)
            .expect("external binding should succeed");
    }
    let output = story
        .continue_maximally()
        .expect("typed external story should run");

    assert_eq!(output, "5|2|3|7\n");
    assert!(
        story.get_current_errors().is_empty(),
        "story should not emit runtime errors: {:#?}",
        story.get_current_errors()
    );
}

#[test]
fn typed_external_calls_keep_runtime_shape_and_return_values() {
    let compiled = compile_language_source(
        "typed-externals.ink",
        concat!(
            "STRUCT Player {\n",
            "hp: int\n",
            "}\n",
            "EXTERNAL next_score(value: int) => int\n",
            "EXTERNAL make_scores() => int[]\n",
            "EXTERNAL make_player() => Player\n",
            "~ temp score: int = next_score(4)\n",
            "~ temp scores: int[] = make_scores()\n",
            "~ temp player: Player = make_player()\n",
            "{score}|{scores[1]}|{player.hp}\n",
            "-> DONE",
        ),
    );

    let json = compiled.program.to_json_value();
    assert_json_sequence(
        &json,
        vec![json!(4), json!({"x()": "next_score", "exArgs": 1})],
    );
    assert_json_sequence(&json, vec![json!({"x()": "make_scores"})]);
    assert_json_sequence(&json, vec![json!({"x()": "make_player"})]);

    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");
    for name in ["next_score", "make_scores", "make_player"] {
        story
            .bind_external_function(name, Rc::new(RefCell::new(TypedExternal)), true)
            .expect("external binding should succeed");
    }
    let output = story
        .continue_maximally()
        .expect("typed external story should run");

    assert_eq!(output, "5|3|7\n");
    assert!(
        story.get_current_errors().is_empty(),
        "story should not emit runtime errors: {:#?}",
        story.get_current_errors()
    );
}

#[test]
fn tail_recursion_rewrites_parameters_and_preserves_other_recursion() {
    let compiled = compile_language_source(
        "tail-recursion.ink",
        concat!(
            "{count_down(1500, 0)}|{carry(3, 0)}|{fact(5)}\n",
            "-> DONE\n",
            "=== function count_down(n: int, acc: int) => int ===\n",
            "{ n <= 0:\n",
            "    ~ return acc\n",
            "- else:\n",
            "    ~ return count_down(n - 1, acc + 1)\n",
            "}\n",
            "=== function carry(n: int, seen: int) => int ===\n",
            "{ n <= 0:\n",
            "    ~ return seen\n",
            "- else:\n",
            "    ~ return carry(n - 1, n)\n",
            "}\n",
            "=== function fact(n: int) => int ===\n",
            "{ n <= 1:\n",
            "    ~ return 1\n",
            "- else:\n",
            "    ~ return n * fact(n - 1)\n",
            "}",
        ),
    );

    let json = compiled.program.to_json_value();
    assert!(
        json_contains_divert_target(&json, &|target| target.ends_with(".2")),
        "tail recursion should lower to a direct jump to the function body after parameter reassignment, got {json:#}"
    );
    assert_story_output(&compiled, "1500|1|120\n");
}

#[test]
fn language_diagnostic_helper_asserts_error_messages() {
    let diagnostics = diagnostics_for_language_source("diagnostic-smoke.ink", "-> missing_target");
    assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, "target not found");
}

#[test]
fn untyped_global_declaration_reports_missing_type() {
    let diagnostics = diagnostics_for_language_source(
        "untyped-global.ink",
        concat!("VAR score = 1\n", "-> DONE"),
    );

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Variable 'score' is missing a type",
    );
}

#[test]
fn untyped_constant_declaration_reports_missing_type() {
    let diagnostics = diagnostics_for_language_source(
        "untyped-constant.ink",
        concat!("CONST score = 1\n", "-> DONE"),
    );

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Constant 'score' is missing a type",
    );
}

#[test]
fn nested_global_var_declarations_report_current_syntax_error() {
    let cases = [
        (
            "nested-var-function.ink",
            "== function setup() => void ==\nVAR score: int = 0",
        ),
        (
            "nested-var-knot.ink",
            "== knot ==\nVAR score: int = 0\n-> DONE",
        ),
        (
            "nested-var-stitch.ink",
            "== knot ==\n= stitch\nVAR score: int = 0\n-> DONE",
        ),
        (
            "nested-var-conditional.ink",
            "{ true:\nVAR score: int = 0\n}\n-> DONE",
        ),
    ];

    for (name, source) in cases {
        let diagnostics = diagnostics_for_language_source(name, source);

        assert_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Global VAR declarations must appear at the story top level",
        );
    }
}

#[test]
fn untyped_temp_declaration_reports_missing_type() {
    let diagnostics = diagnostics_for_language_source(
        "untyped-temp.ink",
        concat!("~ temp score = 1\n", "-> DONE"),
    );

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Temp variable 'score' is missing a type",
    );
}

#[test]
fn untyped_function_parameter_reports_missing_type() {
    let diagnostics = diagnostics_for_language_source(
        "untyped-function-param.ink",
        concat!(
            "{add(1, 2)}\n",
            "-> DONE\n",
            "== function add(a, b: int) => int ==\n",
            "~ return b"
        ),
    );

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Function parameter 'a' is missing a type",
    );
}

#[test]
fn missing_function_return_type_reports_missing_type() {
    let diagnostics = diagnostics_for_language_source(
        "missing-function-return.ink",
        concat!(
            "{add(1, 2)}\n",
            "-> DONE\n",
            "== function add(a: int, b: int) ==\n",
            "~ return a + b"
        ),
    );

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Function 'add' is missing a return type",
    );
}

#[test]
fn untyped_external_parameter_reports_missing_type() {
    let diagnostics = diagnostics_for_language_source(
        "untyped-external-param.ink",
        concat!(
            "EXTERNAL ext(a, b: int) => int\n",
            "{ext(1, 2)}\n",
            "-> DONE"
        ),
    );

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "External parameter 'a' is missing a type",
    );
}

#[test]
fn missing_external_return_type_reports_missing_type() {
    let diagnostics = diagnostics_for_language_source(
        "missing-external-return.ink",
        concat!("EXTERNAL ext(a: int)\n", "{ext(1)}\n", "-> DONE"),
    );

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "External declaration 'ext' is missing a return type",
    );
}
