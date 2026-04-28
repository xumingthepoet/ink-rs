mod support;

use ink_compiler::DiagnosticSeverity;
use support::compiler::{
    assert_diagnostic, assert_fixture_compile_errors, compile_fixture_error_messages,
    diagnostics_for_fixture,
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
fn address_validation_reports_alternative_sequence_divert_errors() {
    assert_fixture_compile_errors("diagnostics/addresses/alternative-sequence-divert-errors.ink");
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
