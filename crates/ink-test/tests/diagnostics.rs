mod support;

use ink_compiler::DiagnosticSeverity;
use support::compiler::{
    assert_diagnostic, assert_fixture_compile_errors, compile_fixture,
    compile_fixture_error_messages, diagnostics_for_fixture,
};

#[test]
fn module_level_content_is_rejected() {
    let diagnostics = diagnostics_for_fixture("diagnostics/module-level-content.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Module-level story content is not allowed",
    );
    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Module-level tags are not allowed",
    );
    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Stitch declarations must appear inside a knot",
    );
}

#[test]
fn removed_list_declaration_reports_removed_feature_diagnostic() {
    let diagnostics = diagnostics_for_fixture("diagnostics/removed-list-module.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "LIST declarations are no longer supported",
    );
    assert!(
        diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("Module-level story content is not allowed")),
        "LIST diagnostics should not fall back to module-level content errors: {diagnostics:#?}"
    );
}

#[test]
fn removed_source_sequences_report_removed_feature_diagnostic() {
    let diagnostics = diagnostics_for_fixture("diagnostics/removed-sequence.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Source sequences, cycles, shuffles, and once-only alternatives are no longer supported",
    );
}

#[test]
fn enum_diagnostics_cover_invalid_declarations_and_values() {
    let cases = [
        (
            "diagnostics/enums/empty.ink",
            "Enum 'State' must declare at least one member",
        ),
        (
            "diagnostics/enums/duplicate-member.ink",
            "Duplicate member 'Idle' in enum 'State'",
        ),
        (
            "diagnostics/enums/explicit-member-value.ink",
            "Enum members do not support explicit values",
        ),
        (
            "diagnostics/enums/unknown-member.ink",
            "Unknown member 'Missing' in enum 'State'",
        ),
        (
            "diagnostics/enums/string-interop.ink",
            "has type string but expected State",
        ),
        (
            "diagnostics/enums/invalid-operator.ink",
            "Operator '>' is not defined for types State and State",
        ),
    ];

    for (fixture, message) in cases {
        let diagnostics = diagnostics_for_fixture(fixture);
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn dict_literal_diagnostics_report_invalid_literals() {
    let diagnostics = diagnostics_for_fixture("diagnostics/dict-literals.ink");

    for message in [
        "Dict literal key for 'wrongKey' has type string but expected int",
        "Value for 'wrongValue[\"one\"]' has type string but expected int",
        "Value for 'wrongNested[\"row\"][1]' has type int but expected string",
        "Value for 'wrongExpected' is a Dict literal but expected int",
    ] {
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn dict_type_name_diagnostics_report_invalid_generics() {
    let diagnostics = diagnostics_for_fixture("diagnostics/dicts/type-names.ink");

    for message in [
        "Dict key type must be string or int, got bool",
        "Dict type names must separate key and value types with `,`",
        "Dict type names must close with `>`",
    ] {
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn dict_index_diagnostics_report_invalid_indexing() {
    let diagnostics = diagnostics_for_fixture("diagnostics/dicts/indexing.ink");

    for message in [
        "Index expression has type int but expected string",
        "Cannot index non-array/non-Dict type int",
        "Index for assignment target 'scores[Number(1)]' has type int but expected string",
        "Cannot index non-array/non-Dict assignment target type int",
    ] {
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn dict_builtin_diagnostics_report_unsupported_collection_builtins() {
    let diagnostics = diagnostics_for_fixture("diagnostics/dicts/builtins.ink");

    for message in [
        "Argument for builtin 'LEN' has type Dict<string, int> but expected array",
        "First argument for builtin 'ARRAY_REMOVE' has type Dict<string, int> but expected array",
    ] {
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn dict_literal_diagnostics_report_duplicate_keys() {
    let diagnostics = diagnostics_for_fixture("diagnostics/dicts/duplicate-keys.ink");

    for message in [
        "Duplicate key \"ada\" in Dict literal for 'scores'",
        "Duplicate key 1 in Dict literal for 'names'",
        "Duplicate key 1 in Dict literal for 'nested[\"row\"]'",
    ] {
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn interface_body_diagnostics_reject_executable_content() {
    let diagnostics = diagnostics_for_fixture("diagnostics/interface-invalid-body.ink");

    for message in [
        "Interface bodies do not support variable declarations",
        "Interface bodies do not support external declarations",
        "Interface bodies do not support stitch declarations",
        "Interface bodies only support knot and function signatures",
    ] {
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn interface_analysis_diagnostics_report_names_and_unknown_types() {
    let diagnostics = diagnostics_for_fixture("diagnostics/interface-analysis.ink");

    for message in [
        "Interface 'IItem' is already declared in this compilation",
        "Interface 'IItem' conflicts with module 'IItem'",
        "Unknown interface type 'IMissing'",
    ] {
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn interface_implementation_diagnostics_report_contract_errors() {
    let diagnostics = diagnostics_for_fixture("diagnostics/interface-implementation.ink");

    for message in [
        "Module 'unknown' implements unknown interface 'IMissing'",
        "Module 'missing' is missing knot 'target' required by interface 'IItem'",
        "Module 'wrongKind' defines function 'target' but interface 'IItem' requires a knot",
        "Module 'wrongSignature' knot 'target' parameter 1 has type string but interface 'IItem' requires int",
        "Module 'wrongSignature' function 'score' parameter 1 has type string but interface 'IItem' requires int",
        "Module 'wrongSignature' function 'score' returns string but interface 'IItem' requires int",
        "External 'score' in module 'externalImpl' cannot implement function 'score' required by interface 'IItem'",
    ] {
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn interface_value_diagnostics_report_invalid_module_literals() {
    let diagnostics = diagnostics_for_fixture("diagnostics/interface-values.ink");

    for message in [
        "Module literal 'left' requires a bare import in module 'game': FROM left",
        "Unknown module 'missing' for interface value",
        "Module 'wrong' does not implement interface 'IItem'",
        "Variable 'defaultRoute' of type interface<IItem> cannot be default-initialized",
    ] {
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn interface_assignment_diagnostics_report_invalid_module_literals() {
    let diagnostics = diagnostics_for_fixture("diagnostics/interface-assignments.ink");

    for message in [
        "Module literal 'right' requires a bare import in module 'game': FROM right",
        "Module 'wrong' does not implement interface 'IItem'",
    ] {
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn interface_dynamic_target_diagnostics_report_invalid_member_access() {
    let diagnostics = diagnostics_for_fixture("diagnostics/interface-dynamic-targets.ink");

    for message in [
        "Dynamic interface target 'target' has base type string but expected interface",
        "Interface 'IItem' does not declare member 'missing'",
        "Interface 'IItem' member 'score' is a function but dynamic target access requires a knot",
        "Dynamic interface target 'target' expects 1 arguments but got 0",
        "Argument 'amount' for dynamic interface target 'target' has type string but expected int",
    ] {
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn interface_dynamic_target_lowering_is_implemented() {
    let _ = compile_fixture("diagnostics/interface-dynamic-lowering.ink");
}

#[test]
fn interface_dynamic_function_diagnostics_report_invalid_member_access() {
    let diagnostics = diagnostics_for_fixture("diagnostics/interface-dynamic-functions.ink");

    for message in [
        "Dynamic interface function 'score' has base type string but expected interface",
        "Interface 'IItem' does not declare member 'missing'",
        "Interface 'IItem' member 'target' is a knot but dynamic function call requires a function",
        "Dynamic interface function 'score' expects 1 arguments but got 0",
        "Argument 'amount' for dynamic interface function 'score' has type string but expected int",
    ] {
        assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, message);
    }
}

#[test]
fn interface_dynamic_function_lowering_is_implemented() {
    let _ = compile_fixture("diagnostics/interface-dynamic-function-lowering.ink");
}

#[test]
fn imports_diagnostics_report_obsolete_import_syntax() {
    let diagnostics = diagnostics_for_fixture("diagnostics/imports-obsolete.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Old import syntax `IMPORT sword FROM items` has been replaced by `FROM items IMPORT sword`",
    );
}

#[test]
fn imports_diagnostics_report_bare_module_import_errors_and_warnings() {
    let diagnostics = diagnostics_for_fixture("diagnostics/imports-bare.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Imported module 'missing' does not exist",
    );
    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Warning,
        "Imported module 'items' is never used",
    );
}

#[test]
fn diagnostic_helper_asserts_error_messages() {
    let diagnostics = diagnostics_for_fixture("diagnostics/diagnostic-smoke.ink");
    assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, "target not found");
}

#[test]
fn untyped_global_declaration_reports_missing_type() {
    let diagnostics = diagnostics_for_fixture("diagnostics/untyped-global.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Variable 'score' is missing a type",
    );
}

#[test]
fn untyped_constant_declaration_reports_missing_type() {
    let diagnostics = diagnostics_for_fixture("diagnostics/untyped-constant.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Constant 'score' is missing a type",
    );
}

#[test]
fn nested_global_var_declarations_are_rejected() {
    let cases = [
        "diagnostics/nested-var-function.ink",
        "diagnostics/nested-var-knot.ink",
        "diagnostics/nested-var-stitch.ink",
        "diagnostics/nested-var-conditional.ink",
    ];

    for fixture in cases {
        let diagnostics = diagnostics_for_fixture(fixture);

        assert_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Global VAR declarations must appear at the story top level",
        );
    }
}

#[test]
fn untyped_temp_declaration_reports_missing_type() {
    let diagnostics = diagnostics_for_fixture("diagnostics/untyped-temp.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Temp variable 'score' is missing a type",
    );
}

#[test]
fn multiline_temp_initializer_remains_single_line_syntax() {
    let diagnostics = diagnostics_for_fixture("diagnostics/multiline-temp-initializer.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "expected expression",
    );
}

#[test]
fn untyped_function_parameter_reports_missing_type() {
    let diagnostics = diagnostics_for_fixture("diagnostics/untyped-function-param.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Function parameter 'a' is missing a type",
    );
}

#[test]
fn missing_function_return_type_reports_missing_type() {
    let diagnostics = diagnostics_for_fixture("diagnostics/missing-function-return.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "Function 'add' is missing a return type",
    );
}

#[test]
fn untyped_external_parameter_reports_missing_type() {
    let diagnostics = diagnostics_for_fixture("diagnostics/untyped-external-param.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "External parameter 'a' is missing a type",
    );
}

#[test]
fn missing_external_return_type_reports_missing_type() {
    let diagnostics = diagnostics_for_fixture("diagnostics/missing-external-return.ink");

    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "External declaration 'ext' is missing a return type",
    );
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
        let diagnostics = diagnostics_for_fixture(fixture);
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
    for fixture in [
        "diagnostics/dynamic-diverts/wrong-type.ink",
        "diagnostics/dynamic-diverts/wrong-field-type.ink",
    ] {
        let diagnostics = diagnostics_for_fixture(fixture);

        assert_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Dynamic divert target has type int but expected ->",
        );
    }
}

#[test]
fn address_validation_reports_unknown_knot_divert() {
    assert_fixture_compile_errors("diagnostics/addresses/unknown-knot-divert.ink");
}

#[test]
fn address_validation_reports_unknown_stitch_divert() {
    assert_fixture_compile_errors("diagnostics/addresses/unknown-stitch-divert.ink");
}

#[test]
fn address_validation_reports_unknown_relative_stitch() {
    assert_fixture_compile_errors("diagnostics/addresses/unknown-relative-stitch.ink");
}

#[test]
fn address_validation_reports_choice_text_divert_target_errors() {
    assert_fixture_compile_errors("diagnostics/addresses/choice-text-divert-target-errors.ink");
}

#[test]
fn address_validation_reports_nested_branch_divert_errors() {
    assert_fixture_compile_errors("diagnostics/addresses/nested-branch-divert-errors.ink");
}

#[test]
fn address_validation_reports_condition_address_errors() {
    assert_fixture_compile_errors("diagnostics/addresses/condition-address-errors.ink");
}

#[test]
fn addresses_in_choices_are_validated_without_raw_parse_nodes() {
    let errors = compile_fixture_error_messages(
        "diagnostics/addresses/addresses-in-choices-validated-without-raw-parse-nodes.ink",
    );
    assert!(errors.is_empty(), "{errors:#?}");
}
