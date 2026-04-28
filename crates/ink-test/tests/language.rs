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
    let output = Compiler::default().compile(SourceInput::named(
        language_fixture_text(filename),
        filename,
    ));
    assert!(
        output.diagnostics.is_empty(),
        "compile for {filename} should not emit diagnostics: {:#?}",
        output.diagnostics
    );
    output.artifact.expect("expected compiled story")
}

fn diagnostics_for_language_fixture(filename: &str) -> Vec<Diagnostic> {
    let output = Compiler::default().compile(SourceInput::named(
        language_fixture_text(filename),
        filename,
    ));
    assert!(
        output.artifact.is_none(),
        "compile for {filename} should fail when asserting diagnostics"
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

#[test]
fn explicit_module_level_content_emits_diagnostics() {
    let module_diagnostics =
        diagnostics_for_language_fixture("diagnostics/module-level-content.ink");

    assert_diagnostic(
        &module_diagnostics,
        DiagnosticSeverity::Error,
        "Module-level story content is not allowed",
    );
    assert_diagnostic(
        &module_diagnostics,
        DiagnosticSeverity::Error,
        "Module-level tags are not allowed",
    );
    assert_diagnostic(
        &module_diagnostics,
        DiagnosticSeverity::Error,
        "Stitch declarations must appear inside a knot",
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
fn module_imported_global_variable_reads_and_writes_run() {
    let compiled = compile_language_fixture("modules/module-imported-global-vars.ink");

    assert_story_output(&compiled, "1\n3\n6\n");
}

#[test]
fn docs_module_import_example_runs() {
    let compiled = compile_language_fixture("modules/docs-module-imports.ink");

    assert_story_output(
        &compiled,
        "The shop is open.\nGold: 5\nPrice: 3\nUpdated price: 5\n",
    );
}

#[test]
fn module_qualified_function_and_external_calls_run() {
    let compiled = compile_language_fixture("modules/module-qualified-calls.ink");

    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");
    story
        .bind_external_function("audio::play", Rc::new(RefCell::new(TypedExternal)), true)
        .expect("module external binding should succeed");
    let output = story
        .continue_maximally()
        .expect("module qualified call story should run");

    assert_eq!(output, "5\n7\n");
    assert!(
        story.get_current_errors().is_empty(),
        "story should not emit runtime errors: {:#?}",
        story.get_current_errors()
    );
}

#[test]
fn module_qualified_flow_paths_run_without_runtime_colon_separator() {
    let compiled = compile_language_fixture("modules/module-qualified-flow-paths.ink");

    assert_story_output(&compiled, "9\nTunnel.\nAfter tunnel.\nTarget.\n");
    assert!(
        !compiled.json.to_string().contains("::"),
        "runtime flow paths should use dot-separated container paths: {:#}",
        compiled.json
    );
}

#[test]
fn module_dynamic_qualified_divert_target_values_run() {
    let compiled = compile_language_fixture("modules/module-dynamic-qualified-divert-target.ink");

    assert_story_output(&compiled, "Dynamic target.\n");
    assert!(
        !compiled.json.to_string().contains("::"),
        "runtime divert target values should use dot-separated container paths: {:#}",
        compiled.json
    );
}

#[test]
fn module_same_module_stitch_paths_run() {
    let compiled = compile_language_fixture("modules/module-same-module-stitch-paths.ink");

    assert_story_output(&compiled, "Intro.\nOpen.\n");
}

#[test]
fn module_globals_and_externals_use_module_qualified_runtime_names() {
    let compiled = compile_language_fixture("modules/module-qualified-globals-and-externals.ink");

    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");
    for name in ["audio::play", "video::play"] {
        story
            .bind_external_function(name, Rc::new(RefCell::new(TypedExternal)), true)
            .expect("module external binding should succeed");
    }
    let output = story
        .continue_maximally()
        .expect("module qualified global story should run");

    assert_eq!(output, "7|8\nReady.\n");
    assert_eq!(
        story
            .get_variable("left::level")
            .and_then(|value| value.get::<i32>()),
        Some(11)
    );
    assert_eq!(
        story
            .get_variable("right::level")
            .and_then(|value| value.get::<i32>()),
        Some(22)
    );
    assert!(story.get_variable("level").is_none());

    let save_string = story.save_state().expect("module state should save");
    let save: Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert_eq!(save["variablesState"]["left::level"], json!(11));
    assert_eq!(save["variablesState"]["right::level"], json!(22));
    assert!(
        save["variablesState"].get("level").is_none(),
        "module globals should not save under unqualified names: {save:#}"
    );

    let mut reloaded = Story::new(&compiled.json).expect("compiled JSON should load");
    for name in ["audio::play", "video::play"] {
        reloaded
            .bind_external_function(name, Rc::new(RefCell::new(TypedExternal)), true)
            .expect("module external binding should succeed after reload");
    }
    reloaded
        .load_state(&save_string)
        .expect("module state should reload");
    reloaded.choose_choice_index(0).unwrap();
    assert_eq!(reloaded.continue_maximally().unwrap(), "11|22\n");
}

#[test]
fn explicit_dynamic_diverts_run_at_runtime() {
    let compiled = compile_language_fixture("diverts/explicit-dynamic-diverts.ink");

    assert_story_output(
        &compiled,
        "First.\nStruct.\nArray.\nConst.\nConst struct.\nConst array.\nFinal.\n",
    );
}

#[test]
fn explicit_dynamic_diverts_support_arguments() {
    let compiled = compile_language_fixture("diverts/explicit-dynamic-divert-args.ink");

    assert_story_output(&compiled, "Value 5.\n");
}

#[test]
fn explicit_dynamic_tunnels_run_at_runtime() {
    let compiled = compile_language_fixture("diverts/explicit-dynamic-tunnel.ink");

    assert_story_output(&compiled, "Inside.\nAfter.\n");
}

#[test]
fn static_diverts_to_variables_report_current_syntax_error() {
    let cases = [
        "diagnostics/static-diverts/old-global-divert.ink",
        "diagnostics/static-diverts/old-const-divert.ink",
        "diagnostics/static-diverts/old-param-divert.ink",
        "diagnostics/static-diverts/old-temp-divert.ink",
    ];

    for fixture in cases {
        let diagnostics = diagnostics_for_language_fixture(fixture);
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
    let diagnostics =
        diagnostics_for_language_fixture("diagnostics/dynamic-diverts/wrong-type.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Dynamic divert target has type int but expected ->",
    );

    let diagnostics =
        diagnostics_for_language_fixture("diagnostics/dynamic-diverts/wrong-field-type.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Dynamic divert target has type int but expected ->",
    );
}

#[test]
fn square_brackets_in_choice_text_are_literal() {
    let compiled = compile_language_fixture("choices/literal-choice-brackets.ink");
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
    let compiled = compile_language_fixture("choices/repeatable-choices.ink");
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
    let compiled = compile_language_fixture("choices/choice-display-only.ink");
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");

    assert_eq!(story.continue_maximally().unwrap(), "");
    assert_eq!(story.get_current_choices()[0].text, "Display text");
    story.choose_choice_index(0).unwrap();
    assert_eq!(story.continue_maximally().unwrap(), "Branch text.\n");
}

#[test]
fn choice_conditions_still_control_visibility() {
    let compiled = compile_language_fixture("choices/conditional-repeatable-choices.ink");
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
fn choice_condition_colon_boundary_allows_dynamic_choice_text() {
    let compiled = compile_language_source(
        "choice-condition-colon-boundary.ink",
        concat!(
            "VAR enabled: bool = true\n",
            "VAR label: string = \"Open path\"\n",
            "* {enabled}: {label}\n",
            "    Done.\n",
            "    -> DONE",
        ),
    );
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");

    assert_eq!(story.continue_maximally().unwrap(), "");
    let choices = story.get_current_choices();
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].text, "Open path");
    story.choose_choice_index(0).unwrap();
    assert_eq!(story.continue_maximally().unwrap(), "Done.\n");
}

#[test]
fn save_load_preserves_generated_choices_without_regeneration() {
    let compiled = compile_language_fixture("choices/choice-save-load.ink");
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
    let compiled = compile_language_fixture("choices/thread-choice-save-load.ink");
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
    let compiled = compile_language_fixture("choices/random-save-load.ink");
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
    let compiled = compile_language_fixture("typed/defaults.ink");

    assert_story_output(
        &compiled,
        "0|false||0|[]|{hp: 0, inventory: [], name: , ready: false}|0|[]|{hp: 0, inventory: [], name: , ready: false}\n",
    );
}

#[test]
fn typed_default_initializers_are_lowered_to_json() {
    let compiled = compile_language_fixture("typed/defaults-json.ink");
    let json = compiled.program.to_json_value();
    let default_player = json!({
        "hp": 0,
        "inventory": [],
        "name": "^",
        "ready": false,
    });

    assert_json_sequence(&json, vec![json!(0), json!({"VAR=": "game::global_score"})]);
    assert_json_sequence(
        &json,
        vec![json!([]), json!({"VAR=": "game::global_values"})],
    );
    assert_json_sequence(
        &json,
        vec![
            default_player.clone(),
            json!({"VAR=": "game::global_player"}),
        ],
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
    let compiled = compile_language_fixture("typed/array-literals-runtime.ink");

    assert_story_output(&compiled, "[1, 2, 3]|[[1, 2], []]|[{hp: 10, name: Ada}]\n");
}

#[test]
fn empty_struct_arrays_load_as_values_at_runtime() {
    let compiled = compile_language_fixture("typed/empty-struct-array.ink");

    assert_story_output(&compiled, "2\n");
}

#[test]
fn typed_constants_support_struct_and_array_values() {
    let compiled = compile_language_fixture("typed/constants.ink");

    assert_story_output(
        &compiled,
        "{hp: 7, ready: false}|[{hp: 1, ready: false}, {hp: 0, ready: false}]|{hp: 7, ready: false}|[{hp: 1, ready: false}, {hp: 0, ready: false}]\n",
    );
}

#[test]
fn multiline_var_and_const_composite_literals_run_at_runtime() {
    let compiled = compile_language_source(
        "multiline-composite-literals.ink",
        concat!(
            "=== module game ===\n",
            "STRUCT Stats {\n",
            "hp: int\n",
            "ready: bool\n",
            "}\n",
            "STRUCT Player {\n",
            "name: string\n",
            "stats: Stats\n",
            "tags: string[]\n",
            "}\n",
            "VAR party: Player[] = [\n",
            "{ name: \"Ada\", stats: { hp: 10, ready: true }, tags: [\"scout\"] },\n",
            "{ name: \"Bea\", stats: { hp: 8 }, tags: [] }\n",
            "]\n",
            "CONST fallback: Stats = {\n",
            "hp: 3,\n",
            "ready: true\n",
            "}\n",
            "CONST backups: Stats[] = [\n",
            "{ hp: 1 },\n",
            "{ hp: 2, ready: true }\n",
            "]\n",
            "== main ==\n",
            "{party[0].name}|{party[1].stats.hp}|{fallback.ready}|{backups[1].hp}|{LEN(party)}\n",
            "-> DONE",
        ),
    );

    assert_story_output(&compiled, "Ada|8|true|2|2\n");
}

#[test]
fn struct_literals_run_at_runtime() {
    let compiled = compile_language_fixture("typed/struct-literals-runtime.ink");

    assert_story_output(
        &compiled,
        "{name: Ada, stats: {hp: 10, ready: true}, tags: [scout]}|{name: Bea, stats: {hp: 0, ready: false}, tags: []}|{hp: 3, ready: false}\n",
    );
}

#[test]
fn field_access_reads_struct_fields_at_runtime() {
    let compiled = compile_language_fixture("typed/field-access.ink");

    assert_story_output(&compiled, "9|true\n");
    assert_json_sequence(
        &compiled.program.to_json_value(),
        vec![json!({"VAR?": "game::state"}), json!("^hp"), json!("FIELD")],
    );
}

#[test]
fn field_access_prefers_visible_variables_over_matching_story_paths() {
    let compiled = compile_language_fixture("typed/field-access-label-shadow.ink");

    assert_story_output(&compiled, "7\n");
    assert_json_sequence(
        &compiled.program.to_json_value(),
        vec![json!({"VAR?": "player"}), json!("^hp"), json!("FIELD")],
    );
}

#[test]
fn index_access_reads_array_items_at_runtime() {
    let compiled = compile_language_fixture("typed/index-access.ink");

    assert_story_output(&compiled, "4|9\n");
    assert_json_sequence(
        &compiled.program.to_json_value(),
        vec![json!({"VAR?": "game::items"}), json!(0), json!("INDEX")],
    );
}

#[test]
fn field_assignment_writes_struct_fields_at_runtime() {
    let compiled = compile_language_fixture("typed/field-assignment.ink");

    assert_story_output(&compiled, "5|true\n");
}

#[test]
fn field_assignment_copies_struct_values_at_runtime() {
    let compiled = compile_language_fixture("typed/field-assignment-copy.ink");

    assert_story_output(&compiled, "3|1\n");
}

#[test]
fn index_assignment_writes_array_items_at_runtime() {
    let compiled = compile_language_fixture("typed/index-assignment.ink");

    assert_story_output(&compiled, "1|10|3\n");
}

#[test]
fn index_assignment_copies_array_values_at_runtime() {
    let compiled = compile_language_fixture("typed/index-assignment-copy.ink");

    assert_story_output(&compiled, "4|9\n");
}

#[test]
fn compound_field_and_index_assignment_run_at_runtime() {
    let compiled = compile_language_fixture("typed/compound-field-index-assignment.ink");

    assert_story_output(&compiled, "5|Ada!|2\n");
}

#[test]
fn compound_index_assignment_evaluates_index_once() {
    let compiled = compile_language_fixture("typed/compound-index-single-eval.ink");

    assert_story_output(&compiled, "2|1\n");
}

#[test]
fn len_returns_array_length_at_runtime() {
    let compiled = compile_language_fixture("typed/len.ink");

    assert_story_output(&compiled, "0|3|2\n");
}

#[test]
fn array_remove_mutates_arrays_and_returns_void_at_runtime() {
    let compiled = compile_language_fixture("typed/array-remove.ink");

    assert_story_output(&compiled, "beforeafter|2|3\n2|4|2\n2|1\n2|1\n3|1\n9|1\n");
}

#[test]
fn string_concatenation_runs_at_runtime() {
    let compiled = compile_language_fixture("typed/string-concat.ink");

    assert_story_output(&compiled, "Hello, Ada!\n");
}

#[test]
fn mixed_string_addition_reports_diagnostics() {
    let cases = [
        (
            "diagnostics/typed/string-int.ink",
            "Operator '+' is not defined for types string and int",
        ),
        (
            "diagnostics/typed/int-string.ink",
            "Operator '+' is not defined for types int and string",
        ),
    ];

    for (fixture, expected_message) in cases {
        let diagnostics = diagnostics_for_language_fixture(fixture);
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
    }
}

struct TypedExternal;

impl ExternalFunction for TypedExternal {
    fn call(&mut self, func_name: &str, args: Vec<ValueType>) -> Option<ValueType> {
        match func_name {
            "next_score" | "game::next_score" => {
                assert!(args == vec![ValueType::Int(4)]);
                Some(ValueType::Int(5))
            }
            "make_scores" | "game::make_scores" => {
                assert!(args.is_empty());
                Some(ValueType::Array(vec![ValueType::Int(2), ValueType::Int(3)]))
            }
            "make_player" | "game::make_player" => {
                assert!(args.is_empty());
                let mut fields = BTreeMap::new();
                fields.insert("hp".to_string(), ValueType::Int(7));
                Some(ValueType::Object(fields))
            }
            "audio::play" => {
                assert!(args == vec![ValueType::from("intro")]);
                Some(ValueType::Int(7))
            }
            "video::play" => {
                assert!(args == vec![ValueType::from("intro")]);
                Some(ValueType::Int(8))
            }
            _ => panic!("unexpected external function: {func_name}"),
        }
    }
}

#[test]
fn typed_external_fixture_runs() {
    let compiled = compile_language_fixture("typed/externals.ink");
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");
    for name in ["game::next_score", "game::make_scores", "game::make_player"] {
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
    let compiled = compile_language_fixture("typed/externals-runtime-shape.ink");

    let json = compiled.program.to_json_value();
    assert_json_sequence(
        &json,
        vec![json!(4), json!({"x()": "game::next_score", "exArgs": 1})],
    );
    assert_json_sequence(&json, vec![json!({"x()": "game::make_scores"})]);
    assert_json_sequence(&json, vec![json!({"x()": "game::make_player"})]);

    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");
    for name in ["game::next_score", "game::make_scores", "game::make_player"] {
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
    let compiled = compile_language_fixture("typed/tail-recursion-preserves.ink");

    let json = compiled.program.to_json_value();
    assert!(
        json_contains_divert_target(&json, &|target| target.ends_with(".2")),
        "tail recursion should lower to a direct jump to the function body after parameter reassignment, got {json:#}"
    );
    assert_story_output(&compiled, "1500|1|120\n");
}

#[test]
fn language_diagnostic_helper_asserts_error_messages() {
    let diagnostics = diagnostics_for_language_fixture("diagnostics/diagnostic-smoke.ink");
    assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, "target not found");
}

#[test]
fn untyped_global_declaration_reports_missing_type() {
    let diagnostics = diagnostics_for_language_fixture("diagnostics/untyped-global.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Variable 'score' is missing a type",
    );
}

#[test]
fn untyped_constant_declaration_reports_missing_type() {
    let diagnostics = diagnostics_for_language_fixture("diagnostics/untyped-constant.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Constant 'score' is missing a type",
    );
}

#[test]
fn nested_global_var_declarations_report_current_syntax_error() {
    let cases = [
        "diagnostics/nested-var-function.ink",
        "diagnostics/nested-var-knot.ink",
        "diagnostics/nested-var-stitch.ink",
        "diagnostics/nested-var-conditional.ink",
    ];

    for fixture in cases {
        let diagnostics = diagnostics_for_language_fixture(fixture);

        assert_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Global VAR declarations must appear at the story top level",
        );
    }
}

#[test]
fn untyped_temp_declaration_reports_missing_type() {
    let diagnostics = diagnostics_for_language_fixture("diagnostics/untyped-temp.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Temp variable 'score' is missing a type",
    );
}

#[test]
fn multiline_temp_initializer_remains_single_line_syntax() {
    let diagnostics = diagnostics_for_language_source(
        "multiline-temp-initializer.ink",
        concat!(
            "=== module game ===\n",
            "== main ==\n",
            "~ temp values: int[] = [\n",
            "1,\n",
            "2\n",
            "]\n",
            "-> DONE",
        ),
    );

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "expected expression",
    );
}

#[test]
fn untyped_function_parameter_reports_missing_type() {
    let diagnostics = diagnostics_for_language_fixture("diagnostics/untyped-function-param.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Function parameter 'a' is missing a type",
    );
}

#[test]
fn missing_function_return_type_reports_missing_type() {
    let diagnostics = diagnostics_for_language_fixture("diagnostics/missing-function-return.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Function 'add' is missing a return type",
    );
}

#[test]
fn untyped_external_parameter_reports_missing_type() {
    let diagnostics = diagnostics_for_language_fixture("diagnostics/untyped-external-param.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "External parameter 'a' is missing a type",
    );
}

#[test]
fn missing_external_return_type_reports_missing_type() {
    let diagnostics = diagnostics_for_language_fixture("diagnostics/missing-external-return.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "External declaration 'ext' is missing a return type",
    );
}
