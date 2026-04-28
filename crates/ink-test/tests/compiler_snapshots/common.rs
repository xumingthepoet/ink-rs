use std::fs;

use ink_compiler::{Compiler, SourceInput};
use serde_json::Value;

pub fn assert_parse_and_json_match_fixture(filename: &str) {
    assert_parse_matches_fixture(filename);
    assert_json_matches_fixture(filename);
}

pub fn assert_parse_matches_fixture(filename: &str) {
    let source = get_fixture_text(filename);
    let compiler = Compiler::default();
    let output = compiler.parse(SourceInput::named(source, filename));

    assert!(
        output.diagnostics.is_empty(),
        "parse for {filename} should not emit diagnostics: {:#?}",
        output.diagnostics
    );

    let parsed = output.artifact.expect("expected parsed story");
    let generated = parsed.to_parse_snapshot();
    let fixture = get_fixture_text(&fixture_path_with_extension(filename, "parse"));

    assert_eq!(
        normalize_snapshot(&generated),
        normalize_snapshot(&fixture),
        "parse snapshot differs for {filename}"
    );
}

pub fn assert_json_matches_fixture(filename: &str) {
    let source = get_fixture_text(filename);
    let compiler = Compiler::default();
    let output = compiler.compile(SourceInput::named(source, filename));

    assert!(
        output.diagnostics.is_empty(),
        "compile for {filename} should not emit diagnostics: {:#?}",
        output.diagnostics
    );

    let compiled = output.artifact.expect("expected compiled story");
    let generated = parse_json_value(filename, &compiled.json, "generated");
    let fixture_json = get_fixture_text(&fixture_path_with_extension(filename, "json"));
    let fixture = parse_json_value(filename, &fixture_json, "fixture");

    assert_eq!(
        generated,
        fixture,
        "{}",
        format_json_mismatch(filename, &generated, &fixture)
    );
}

fn get_fixture_text(filename: &str) -> String {
    let path = ink_test::fixture_root().join(filename);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read fixture {}: {error}", path.display()));
    strip_utf8_bom(text)
}

fn fixture_path_with_extension(filename: &str, extension: &str) -> String {
    let Some(prefix) = filename.strip_suffix(".ink") else {
        panic!("compiler snapshot fixture must be an .ink file: {filename}");
    };
    format!("{prefix}.ink.{extension}")
}

fn parse_json_value(filename: &str, json: &str, label: &str) -> Value {
    serde_json::from_str(json).unwrap_or_else(|error| {
        panic!("compiler snapshot {label} json for {filename} must be valid json: {error}")
    })
}

fn normalize_snapshot(text: &str) -> String {
    text.replace("\r\n", "\n").trim_end().to_string()
}

fn strip_utf8_bom(text: String) -> String {
    text.strip_prefix('\u{feff}')
        .map(str::to_string)
        .unwrap_or(text)
}

fn format_json_mismatch(filename: &str, generated: &Value, fixture: &Value) -> String {
    format!(
        "compiled json differs for {filename}\n--- generated ---\n{}\n--- fixture ---\n{}",
        serde_json::to_string_pretty(generated).unwrap_or_else(|_| generated.to_string()),
        serde_json::to_string_pretty(fixture).unwrap_or_else(|_| fixture.to_string())
    )
}
