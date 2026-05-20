use super::common;

macro_rules! fixture {
    ($name:ident, $path:literal) => {
        #[test]
        fn $name() {
            common::assert_parse_and_json_match_fixture($path);
        }
    };
}

#[test]
fn interface_dynamic_targets_lower_to_format_tokens() {
    use ink_compiler::{Compiler, SourceInput};
    use serde_json::{json, Value};
    use std::fs;

    let filename = "interface/dynamic-targets.ink";
    let source_path = ink_test::fixture_root().join(filename);
    let source = fs::read_to_string(&source_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
    let output = Compiler::default().compile(SourceInput::named(source, filename));

    assert!(
        output.diagnostics.is_empty(),
        "compile for {filename} should not emit diagnostics: {:#?}",
        output.diagnostics
    );

    let compiled = output.artifact.expect("expected compiled story");
    let json: Value = serde_json::from_str(&compiled.json)
        .unwrap_or_else(|error| panic!("compiled json for {filename} must be valid: {error}"));

    assert_json_sequence(
        &json,
        &[
            json!("ev"),
            json!(3),
            json!({"VAR?": "game::route"}),
            json!({"i->": "target", "interface": "IItem"}),
            json!({"temp=": "$divertTarget"}),
            json!("/ev"),
            json!({"->": "$divertTarget", "var": true}),
        ],
    );
    assert_json_sequence(
        &json,
        &[
            json!({"VAR?": "game::routes"}),
            json!(1),
            json!("INDEX"),
            json!({"i->": "fallback", "interface": "IItem"}),
        ],
    );
    assert_json_sequence(
        &json,
        &[
            json!(5),
            json!({"VAR?": "game::route"}),
            json!({"i()": "score", "interface": "IItem", "args": 1}),
        ],
    );
    assert_eq!(
        json["interfaces"]["IItem"],
        json!({
            "members": {
                "fallback": "knot",
                "score": "function",
                "target": "knot"
            },
            "implementations": ["left", "right"]
        })
    );
}

#[test]
fn interface_metadata_omits_uncompiled_implementations() {
    use ink_compiler::{Compiler, DiagnosticSeverity, SourceInput};
    use serde_json::{json, Value};

    let source = r#"
=== interface IItem ===
== target ==

=== module game ===
FROM left
VAR route: interface<IItem> = left
== main ==
-> {{route}::target}

=== module left implements IItem ===
== target ==
-> END

=== module unused implements IItem ===
== target ==
-> END
"#;
    let output = Compiler::default().compile(SourceInput::named(source, "inline-interface.ink"));
    let errors = output
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .collect::<Vec<_>>();
    assert!(
        errors.is_empty(),
        "compile should not emit errors: {errors:#?}"
    );

    let compiled = output.artifact.expect("expected compiled story");
    let json: Value = serde_json::from_str(&compiled.json)
        .unwrap_or_else(|error| panic!("compiled json must be valid: {error}"));

    assert_eq!(
        json["interfaces"]["IItem"],
        json!({
            "members": {
                "target": "knot"
            },
            "implementations": ["left"]
        })
    );
}

#[test]
fn interface_end_to_end_fixture_lowers_dynamic_tokens() {
    use ink_compiler::{Compiler, SourceInput};
    use serde_json::{json, Value};
    use std::fs;

    let filename = "interface/end-to-end.ink";
    common::assert_json_matches_fixture(filename);

    let source_path = ink_test::fixture_root().join(filename);
    let source = fs::read_to_string(&source_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
    let output = Compiler::default().compile(SourceInput::named(source, filename));

    assert!(
        output.diagnostics.is_empty(),
        "compile for {filename} should not emit diagnostics: {:#?}",
        output.diagnostics
    );

    let compiled = output.artifact.expect("expected compiled story");
    let json: Value = serde_json::from_str(&compiled.json)
        .unwrap_or_else(|error| panic!("compiled json for {filename} must be valid: {error}"));

    assert_json_sequence(
        &json,
        &[
            json!({"VAR?": "bonus::value"}),
            json!({"VAR?": "game::route"}),
            json!({"i()": "score", "interface": "IItem", "args": 1}),
        ],
    );
    assert_json_sequence(
        &json,
        &[
            json!({"VAR?": "game::labeler"}),
            json!({"i()": "label", "interface": "ILabel", "args": 0}),
        ],
    );
    assert_json_sequence(
        &json,
        &[
            json!(2),
            json!({"VAR?": "game::route"}),
            json!({"i->": "target", "interface": "IItem"}),
            json!({"temp=": "$divertTarget"}),
            json!("/ev"),
            json!({"->": "$divertTarget", "var": true}),
        ],
    );
    assert_eq!(
        json["interfaces"]["IItem"],
        json!({
            "members": {
                "score": "function",
                "target": "knot"
            },
            "implementations": ["left", "right"]
        })
    );
    assert_eq!(
        json["interfaces"]["ILabel"],
        json!({
            "members": {
                "label": "function"
            },
            "implementations": ["right"]
        })
    );
}

fn assert_json_sequence(value: &serde_json::Value, sequence: &[serde_json::Value]) {
    assert!(
        json_contains_sequence(value, sequence),
        "expected JSON sequence {sequence:#?} in {value:#}"
    );
}

fn json_contains_sequence(value: &serde_json::Value, sequence: &[serde_json::Value]) -> bool {
    match value {
        serde_json::Value::Array(items) => {
            items
                .windows(sequence.len())
                .any(|window| window == sequence)
                || items
                    .iter()
                    .any(|item| json_contains_sequence(item, sequence))
        }
        serde_json::Value::Object(map) => map
            .values()
            .any(|item| json_contains_sequence(item, sequence)),
        _ => false,
    }
}

fixture!(text_oneline, "text/oneline.ink");
fixture!(text_twolines, "text/twolines.ink");
fixture!(knot_multi_line, "knots/multi-line.ink");
fixture!(knot_single_line, "knots/single-line.ink");
fixture!(knot_strip_empty_lines, "knots/strip-empty-lines.ink");
fixture!(divert_invisible_divert, "diverts/invisible-divert.ink");
fixture!(divert_simple_divert, "diverts/simple-divert.ink");
fixture!(glue_glue_with_divert, "glue/glue-with-divert.ink");
fixture!(glue_simple_glue, "glue/simple-glue.ink");
fixture!(runtime_jump_knot, "runtime_api/jump-knot.ink");
fixture!(runtime_saving_loading, "runtime_api/saving-loading.ink");
fixture!(runtime_multiflow_basics, "runtime_api/multiflow-basics.ink");
fixture!(runtime_jump_stitch, "runtime_api/jump-stitch.ink");
fixture!(tags_tags, "tags/tags.ink");
fixture!(
    variable_variable_declaration,
    "variables/variable-declaration.ink"
);
fixture!(tags_tags_dynamic_content, "tags/tagsDynamicContent.ink");
fixture!(variable_varcalc, "variables/varcalc.ink");
fixture!(typed_array_literals, "typed/array-literals.ink");
fixture!(typed_enums, "typed/enums.ink");
fixture!(typed_struct_literals, "typed/struct-literals.ink");

#[test]
fn typed_dicts_parse_snapshot() {
    common::assert_parse_matches_fixture("typed/dicts.ink");
}

fixture!(function_rnd_func, "functions/rnd-func.ink");
fixture!(misc_operations, "misc/operations.ink");
fixture!(conditional_ifelse, "conditionals/ifelse.ink");
fixture!(conditional_iffalse, "conditionals/iffalse.ink");
fixture!(conditional_iftrue, "conditionals/iftrue.ink");
fixture!(function_test_error, "functions/test-error.ink");
fixture!(
    runtime_read_visit_counts,
    "runtime_api/read-visit-counts.ink"
);
fixture!(conditional_ifelse_ext, "conditionals/ifelse-ext.ink");
fixture!(misc_issue15, "misc/issue15.ink");
fixture!(knot_param_recurse, "knots/param-recurse.ink");
fixture!(function_func_basic, "functions/func-basic.ink");
fixture!(function_func_none, "functions/func-none.ink");
fixture!(
    glue_left_right_glue_matching,
    "glue/left-right-glue-matching.ink"
);
fixture!(glue_testbugfix1, "glue/testbugfix1.ink");
fixture!(glue_testbugfix2, "glue/testbugfix2.ink");
fixture!(
    misc_newlines_with_string_eval,
    "misc/newlines_with_string_eval.ink"
);
fixture!(function_complex_func1, "functions/complex-func1.ink");
fixture!(function_complex_func2, "functions/complex-func2.ink");
fixture!(function_complex_func3, "functions/complex-func3.ink");
fixture!(function_setvar_func, "functions/setvar-func.ink");
fixture!(function_func_inline, "functions/func-inline.ink");
fixture!(
    runtime_external_function_0_arg,
    "runtime_api/external-function-0-arg.ink"
);
fixture!(
    runtime_external_function_1_arg,
    "runtime_api/external-function-1-arg.ink"
);
fixture!(
    runtime_external_function_2_arg,
    "runtime_api/external-function-2-arg.ink"
);
fixture!(
    runtime_external_function_3_arg,
    "runtime_api/external-function-3-arg.ink"
);
fixture!(
    function_evaluating_function_variablestate_bug,
    "functions/evaluating-function-variablestate-bug.ink"
);
fixture!(
    tunnels_tunnel_onwards_divert_override,
    "tunnels/tunnel-onwards-divert-override.ink"
);
fixture!(misc_i18n, "misc/i18n.ink");
