use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use serde_json::json;

use crate::support::{
    compiler::{
        assert_diagnostic, assert_json_sequence, assert_story_output, compile_fixture,
        diagnostics_for_fixture, json_contains_divert_target,
    },
    runtime::{ExternalFunction, Story, ValueType},
};
use ink_runtime::value_type::{DictKey, DictKeyType, DictValue};

#[test]
fn primitives_run() {
    let compiled = compile_fixture("typed/primitives.ink");
    assert_story_output(&compiled, "0|1.5|true|Ada Lovelace|2|true\n");
}

#[test]
fn structs_run() {
    let compiled = compile_fixture("typed/structs.ink");
    assert_story_output(&compiled, "10|3|false|Ada\n");
}

#[test]
fn arrays_run() {
    let compiled = compile_fixture("typed/arrays.ink");
    assert_story_output(&compiled, "3|1|10|3\n2|10|3\n");
}

#[test]
fn inline_array_arguments_run() {
    let compiled = compile_fixture("typed/inline-array-arguments.ink");
    assert_story_output(&compiled, "First 101.\nCount 3.\nCopied 8.\n");
}

#[test]
fn divert_target_values_run() {
    let compiled = compile_fixture("typed/divert-targets.ink");
    assert_story_output(&compiled, "Here.\nStruct.\nArray.\nFallback.\n");
}

#[test]
fn enums_run_at_runtime() {
    let compiled = compile_fixture("typed/enums.ink");

    assert_story_output(
        &compiled,
        "data::State.Idle|true|data::State.Busy|data::State.Busy|data::State.Idle|data::State.Done|calm|data::Tone.Sharp\n|true\n",
    );
    assert_json_sequence(
        &compiled.program.to_json_value(),
        vec![json!("^data::State.Idle"), json!({"VAR=": "data::state"})],
    );
    assert_json_sequence(
        &compiled.program.to_json_value(),
        vec![
            json!("^data::State.Done"),
            json!("/ev"),
            json!({"VAR=": "data::state", "re": true}),
        ],
    );
}

#[test]
fn nested_values_run() {
    let compiled = compile_fixture("typed/nested.ink");
    assert_story_output(&compiled, "2|0|4|1|false\n");
}

#[test]
fn interface_values_lower_to_runtime_strings() {
    let compiled = compile_fixture("typed/interface-values.ink");

    assert_story_output(
        &compiled,
        "right|[right, right]|{route: left, routes: [left]}|right|left|[left, right]|{route: right, routes: [right]}\n",
    );

    let json = compiled.program.to_json_value();
    assert_json_sequence(&json, vec![json!("^left"), json!({"VAR=": "game::route"})]);
    assert_json_sequence(
        &json,
        vec![json!(["^left", "^right"]), json!({"VAR=": "game::routes"})],
    );
    assert_json_sequence(
        &json,
        vec![
            json!({"route": "^right", "routes": ["^left"]}),
            json!({"VAR=": "game::config"}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![
            json!("ev"),
            json!("^right"),
            json!("/ev"),
            json!({"VAR=": "game::route", "re": true}),
        ],
    );
}

#[test]
fn interface_module_literals_run_as_static_flow_arguments() {
    let compiled = compile_fixture("typed/interface-argument-module-literals.ink");

    assert_story_output(&compiled, "damage_event|heal_event\n");
}

#[test]
fn functions_return_typed_values() {
    let compiled = compile_fixture("typed/functions.ink");
    assert_story_output(&compiled, "5|Ada!|7\n2|3\n");
}

#[test]
fn tail_call_optimization_rewrites_self_recursion() {
    let compiled = compile_fixture("typed/tail-recursion.ink");
    let json = compiled.program.to_json_value();
    assert!(
        json_contains_divert_target(&json, &|target| target.ends_with(".2")),
        "tail recursion should lower to a direct jump to the function body after parameter reassignment, got {json:#}"
    );
    assert_story_output(&compiled, "1500|15\n");
}

#[test]
fn default_initializers_run_at_runtime() {
    let compiled = compile_fixture("typed/defaults.ink");

    assert_story_output(
        &compiled,
        "0|false||0|[]|{hp: 0, inventory: [], name: , ready: false}|0|[]|{hp: 0, inventory: [], name: , ready: false}\n",
    );
}

#[test]
fn default_initializers_are_lowered_to_json() {
    let compiled = compile_fixture("typed/defaults-json.ink");
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
fn dict_typed_values_defaults_and_literals_are_lowered_to_json() {
    let compiled = compile_fixture("typed/dict-literals-json.ink");
    let json = compiled.program.to_json_value();
    let empty_string_dict = json!(["dict", "string", []]);
    let literal_scores = json!(["dict", "string", [["ada", 10], ["grace", 11]]]);
    let base_scores = json!(["dict", "string", [["ada", 10]]]);
    let literal_by_id = json!(["dict", "int", [[1, "^one"]]]);
    let nested_scores = json!(["dict", "int", [[1, ["dict", "string", [["ada", 10]]]]]]);
    let score_tables = json!([["dict", "string", [["ada", 10]]], ["dict", "string", []]]);
    let sheet = json!({
        "hp": 3,
        "scores": ["dict", "string", [["luck", 7]]]
    });

    assert_json_sequence(
        &json,
        vec![
            empty_string_dict.clone(),
            json!({"VAR=": "game::default_scores"}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![
            literal_scores.clone(),
            json!({"VAR=": "game::literal_scores"}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![base_scores, json!({"VAR=": "game::copied_scores"})],
    );
    assert_json_sequence(
        &json,
        vec![literal_by_id, json!({"VAR=": "game::literal_by_id"})],
    );
    assert_json_sequence(
        &json,
        vec![nested_scores, json!({"VAR=": "game::nested_scores"})],
    );
    assert_json_sequence(
        &json,
        vec![score_tables, json!({"VAR=": "game::score_tables"})],
    );
    assert_json_sequence(&json, vec![sheet, json!({"VAR=": "game::sheet"})]);
    assert_json_sequence(
        &json,
        vec![
            json!(["dict", "string", []]),
            json!("^sum"),
            json!(1),
            json!(1),
            json!("+"),
            json!("SET_INDEX"),
            json!({"VAR=": "game::dynamic_scores"}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![
            json!("ev"),
            json!(["dict", "int", []]),
            json!("/ev"),
            json!({"temp=": "local_default"}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![
            json!("ev"),
            json!(["dict", "int", [[2, "^two"]]]),
            json!("/ev"),
            json!({"temp=": "local_literal"}),
        ],
    );
}

#[test]
fn composite_literal_arguments_run_at_runtime() {
    let compiled = compile_fixture("typed/composite-literal-arguments.ink");

    assert_story_output(
        &compiled,
        "Function 12. Dynamic 23.\nStatic 34.\nTunnel 56.\nOverride 67.\nOnward 78.\nTarget 45.\n",
    );
}

#[test]
fn array_literals_run_at_runtime() {
    let compiled = compile_fixture("typed/array-literals-runtime.ink");

    assert_story_output(&compiled, "[1, 2, 3]|[[1, 2], []]|[{hp: 10, name: Ada}]\n");
}

#[test]
fn empty_struct_arrays_load_as_values_at_runtime() {
    let compiled = compile_fixture("typed/empty-struct-array.ink");

    assert_story_output(&compiled, "2\n");
}

#[test]
fn constants_support_struct_and_array_values() {
    let compiled = compile_fixture("typed/constants.ink");

    assert_story_output(
        &compiled,
        "{hp: 7, ready: false}|[{hp: 1, ready: false}, {hp: 0, ready: false}]|{hp: 7, ready: false}|[{hp: 1, ready: false}, {hp: 0, ready: false}]\n",
    );
}

#[test]
fn multiline_var_and_const_composite_literals_run_at_runtime() {
    let compiled = compile_fixture("typed/multiline-composite-literals.ink");

    assert_story_output(&compiled, "Ada|8|true|2|2\n");
}

#[test]
fn struct_literals_run_at_runtime() {
    let compiled = compile_fixture("typed/struct-literals-runtime.ink");

    assert_story_output(
        &compiled,
        "{name: Ada, stats: {hp: 10, ready: true}, tags: [scout]}|{name: Bea, stats: {hp: 0, ready: false}, tags: []}|{hp: 3, ready: false}\n",
    );
}

#[test]
fn field_access_reads_struct_fields_at_runtime() {
    let compiled = compile_fixture("typed/field-access.ink");

    assert_story_output(&compiled, "9|true\n");
    assert_json_sequence(
        &compiled.program.to_json_value(),
        vec![json!({"VAR?": "game::state"}), json!("^hp"), json!("FIELD")],
    );
}

#[test]
fn field_access_prefers_visible_variables_over_matching_story_paths() {
    let compiled = compile_fixture("typed/field-access-label-shadow.ink");

    assert_story_output(&compiled, "7\n");
    assert_json_sequence(
        &compiled.program.to_json_value(),
        vec![json!({"VAR?": "player"}), json!("^hp"), json!("FIELD")],
    );
}

#[test]
fn index_access_reads_array_items_at_runtime() {
    let compiled = compile_fixture("typed/index-access.ink");

    assert_story_output(&compiled, "4|9\n");
    assert_json_sequence(
        &compiled.program.to_json_value(),
        vec![json!({"VAR?": "game::items"}), json!(0), json!("INDEX")],
    );
}

#[test]
fn dict_typed_values_index_reads_lower_to_index_tokens() {
    let compiled = compile_fixture("typed/dict-index-reads-json.ink");
    let json = compiled.program.to_json_value();

    assert_json_sequence(
        &json,
        vec![
            json!({"VAR?": "game::scores"}),
            json!("str"),
            json!("^ada"),
            json!("/str"),
            json!("INDEX"),
        ],
    );
    assert_json_sequence(
        &json,
        vec![json!({"VAR?": "game::names"}), json!(1), json!("INDEX")],
    );
    assert_json_sequence(
        &json,
        vec![
            json!({"VAR?": "game::nested"}),
            json!(1),
            json!("INDEX"),
            json!("str"),
            json!("^ada"),
            json!("/str"),
            json!("INDEX"),
        ],
    );
}

#[test]
fn field_assignment_writes_struct_fields_at_runtime() {
    let compiled = compile_fixture("typed/field-assignment.ink");

    assert_story_output(&compiled, "5|true\n");
}

#[test]
fn field_assignment_copies_struct_values_at_runtime() {
    let compiled = compile_fixture("typed/field-assignment-copy.ink");

    assert_story_output(&compiled, "3|1\n");
}

#[test]
fn index_assignment_writes_array_items_at_runtime() {
    let compiled = compile_fixture("typed/index-assignment.ink");

    assert_story_output(&compiled, "1|10|3\n");
}

#[test]
fn dict_typed_values_index_writes_lower_to_set_index_tokens() {
    let compiled = compile_fixture("typed/dict-index-writes-json.ink");
    let json = compiled.program.to_json_value();

    assert_json_sequence(
        &json,
        vec![
            json!("str"),
            json!("^bea"),
            json!("/str"),
            json!({"temp=": "$lvalue0"}),
            json!({"VAR?": "game::scores"}),
            json!({"VAR?": "$lvalue0"}),
            json!(11),
            json!("SET_INDEX"),
            json!("/ev"),
            json!({"VAR=": "game::scores", "re": true}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![
            json!("str"),
            json!("^ada"),
            json!("/str"),
            json!({"temp=": "$lvalue0"}),
            json!({"VAR?": "game::scores"}),
            json!({"VAR?": "$lvalue0"}),
            json!(12),
            json!("SET_INDEX"),
            json!("/ev"),
            json!({"VAR=": "game::scores", "re": true}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![
            json!(1),
            json!({"temp=": "$lvalue0"}),
            json!("str"),
            json!("^ada"),
            json!("/str"),
            json!({"temp=": "$lvalue1"}),
            json!({"VAR?": "game::nested"}),
            json!({"VAR?": "$lvalue0"}),
            json!({"VAR?": "game::nested"}),
            json!({"VAR?": "$lvalue0"}),
            json!("INDEX"),
            json!({"VAR?": "$lvalue1"}),
            json!(13),
            json!("SET_INDEX"),
            json!("SET_INDEX"),
            json!("/ev"),
            json!({"VAR=": "game::nested", "re": true}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![
            json!(0),
            json!({"temp=": "$lvalue0"}),
            json!("str"),
            json!("^ada"),
            json!("/str"),
            json!({"temp=": "$lvalue1"}),
            json!({"VAR?": "game::score_arrays"}),
            json!({"VAR?": "$lvalue0"}),
            json!({"VAR?": "game::score_arrays"}),
            json!({"VAR?": "$lvalue0"}),
            json!("INDEX"),
            json!({"VAR?": "$lvalue1"}),
            json!(14),
            json!("SET_INDEX"),
            json!("SET_INDEX"),
            json!("/ev"),
            json!({"VAR=": "game::score_arrays", "re": true}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![
            json!("str"),
            json!("^ada"),
            json!("/str"),
            json!({"temp=": "$lvalue0"}),
            json!({"VAR?": "game::sheet"}),
            json!("^scores"),
            json!({"VAR?": "game::sheet"}),
            json!("^scores"),
            json!("FIELD"),
            json!({"VAR?": "$lvalue0"}),
            json!(15),
            json!("SET_INDEX"),
            json!("SET_FIELD"),
            json!("/ev"),
            json!({"VAR=": "game::sheet", "re": true}),
        ],
    );
}

#[test]
fn dict_typed_values_run_at_runtime() {
    let compiled = compile_fixture("typed/dicts-runtime.ink");
    let json = compiled.program.to_json_value();

    assert_json_sequence(
        &json,
        vec![
            json!(["dict", "string", []]),
            json!({"VAR=": "game::default_scores"}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![
            json!(["dict", "string", [["ada", 10]]]),
            json!({"VAR=": "game::scores"}),
        ],
    );
    assert_json_sequence(
        &json,
        vec![
            json!(["dict", "int", [[1, "^one"]]]),
            json!({"VAR=": "game::names"}),
        ],
    );

    let mut story = Story::new(&compiled.json);
    for name in ["game::make_dict_scores", "game::pick_dict_score"] {
        story.bind_external_function(name, Rc::new(RefCell::new(TypedExternal)), true);
    }

    assert_eq!(
        story.continue_maximally(),
        "Dict<string>{}|10|one|one|10|one\n11|12|two|13|14\ntrue|true\n23|10\n5|6\n"
    );
    assert!(
        story.get_current_errors().is_empty(),
        "story should not emit runtime errors: {:#?}",
        story.get_current_errors()
    );
}

#[test]
fn index_assignment_copies_array_values_at_runtime() {
    let compiled = compile_fixture("typed/index-assignment-copy.ink");

    assert_story_output(&compiled, "4|9\n");
}

#[test]
fn compound_field_and_index_assignment_run_at_runtime() {
    let compiled = compile_fixture("typed/compound-field-index-assignment.ink");

    assert_story_output(&compiled, "5|Ada!|2\n");
}

#[test]
fn compound_index_assignment_evaluates_index_once() {
    let compiled = compile_fixture("typed/compound-index-single-eval.ink");

    assert_story_output(&compiled, "2|1\n");
}

#[test]
fn len_returns_array_length_at_runtime() {
    let compiled = compile_fixture("typed/len.ink");

    assert_story_output(&compiled, "0|3|2\n");
}

#[test]
fn array_remove_mutates_arrays_and_returns_void_at_runtime() {
    let compiled = compile_fixture("typed/array-remove.ink");

    assert_story_output(&compiled, "beforeafter|2|3\n2|4|2\n2|1\n2|1\n3|1\n9|1\n");
}

#[test]
fn string_concatenation_runs_at_runtime() {
    let compiled = compile_fixture("typed/string-concat.ink");

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
        let diagnostics = diagnostics_for_fixture(fixture);
        assert_diagnostic(
            &diagnostics,
            ink_compiler::DiagnosticSeverity::Error,
            expected_message,
        );
    }
}

#[test]
fn external_values_run() {
    let compiled = compile_fixture("typed/externals.ink");
    let mut story = Story::new(&compiled.json);
    for name in ["game::next_score", "game::make_scores", "game::make_player"] {
        story.bind_external_function(name, Rc::new(RefCell::new(TypedExternal)), true);
    }
    let output = story.continue_maximally();

    assert_eq!(output, "5|2|3|7\n");
    assert!(
        story.get_current_errors().is_empty(),
        "story should not emit runtime errors: {:#?}",
        story.get_current_errors()
    );
}

#[test]
fn external_calls_keep_runtime_shape_and_return_values() {
    let compiled = compile_fixture("typed/externals-runtime-shape.ink");

    let json = compiled.program.to_json_value();
    assert_json_sequence(
        &json,
        vec![json!(4), json!({"x()": "game::next_score", "exArgs": 1})],
    );
    assert_json_sequence(&json, vec![json!({"x()": "game::make_scores"})]);
    assert_json_sequence(&json, vec![json!({"x()": "game::make_player"})]);

    let mut story = Story::new(&compiled.json);
    for name in ["game::next_score", "game::make_scores", "game::make_player"] {
        story.bind_external_function(name, Rc::new(RefCell::new(TypedExternal)), true);
    }
    let output = story.continue_maximally();

    assert_eq!(output, "5|3|7\n");
    assert!(
        story.get_current_errors().is_empty(),
        "story should not emit runtime errors: {:#?}",
        story.get_current_errors()
    );
}

#[test]
fn tail_recursion_rewrites_parameters_and_preserves_other_recursion() {
    let compiled = compile_fixture("typed/tail-recursion-preserves.ink");

    let json = compiled.program.to_json_value();
    assert!(
        json_contains_divert_target(&json, &|target| target.ends_with(".2")),
        "tail recursion should lower to a direct jump to the function body after parameter reassignment, got {json:#}"
    );
    assert_story_output(&compiled, "1500|1|120\n");
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
            "make_dict_scores" | "game::make_dict_scores" => {
                let seed = args.first()?.get::<i32>()?;
                Some(string_int_dict([("seed", seed), ("next", seed + 1)]))
            }
            "pick_dict_score" | "game::pick_dict_score" => {
                let ValueType::Dict(scores) = args.first()? else {
                    panic!("expected Dict scores argument");
                };
                let ValueType::String(key) = args.get(1)? else {
                    panic!("expected string key argument");
                };
                match scores.get(&DictKey::String(key.string.clone())) {
                    Some(ValueType::Int(score)) => Some(ValueType::Int(*score)),
                    _ => panic!("expected int score"),
                }
            }
            _ => panic!("unexpected external function: {func_name}"),
        }
    }
}

fn string_int_dict<const N: usize>(entries: [(&str, i32); N]) -> ValueType {
    ValueType::Dict(
        DictValue::new(
            DictKeyType::String,
            entries
                .into_iter()
                .map(|(key, value)| (DictKey::String(key.to_string()), ValueType::Int(value)))
                .collect(),
        )
        .expect("valid Dict"),
    )
}
