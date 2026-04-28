use ink_compiler::{CompiledStory, Compiler, Diagnostic, DiagnosticSeverity, SourceInput};
use serde_json::Value;

use super::runtime::Story;

pub fn fixture_text(filename: &str) -> String {
    ink_test::load_fixture_text(filename)
}

pub fn compile_fixture(filename: &str) -> CompiledStory {
    let output = Compiler::default().compile(SourceInput::named(fixture_text(filename), filename));
    assert!(
        output.diagnostics.is_empty(),
        "compile for {filename} should not emit diagnostics: {:#?}",
        output.diagnostics
    );
    output.artifact.expect("expected compiled story")
}

pub fn compile_fixture_allowing_warnings(filename: &str) -> CompiledStory {
    let output = Compiler::default().compile(SourceInput::named(fixture_text(filename), filename));
    let errors = output
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .collect::<Vec<_>>();
    assert!(
        errors.is_empty(),
        "compile for {filename} should not emit errors: {errors:#?}"
    );
    output.artifact.expect("expected compiled story")
}

pub fn compile_fixture_to_story(filename: &str) -> Story {
    let compiled = compile_fixture(filename);
    Story::new(&compiled.json)
}

pub fn compile_fixture_to_story_allowing_warnings(filename: &str) -> Story {
    let compiled = compile_fixture_allowing_warnings(filename);
    Story::new(&compiled.json)
}

pub fn diagnostics_for_fixture(filename: &str) -> Vec<Diagnostic> {
    let output = Compiler::default().compile(SourceInput::named(fixture_text(filename), filename));
    assert!(
        output.artifact.is_none(),
        "compile for {filename} should fail when asserting diagnostics"
    );
    output.diagnostics
}

pub fn compile_fixture_error_messages(filename: &str) -> Vec<String> {
    Compiler::default()
        .compile(SourceInput::named(fixture_text(filename), filename))
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .map(|diagnostic| {
            format!(
                "line {} column {}: {}",
                diagnostic.line, diagnostic.column, diagnostic.message
            )
        })
        .collect()
}

pub fn assert_fixture_compile_errors(filename: &str) -> Vec<String> {
    let errors = compile_fixture_error_messages(filename);
    assert!(!errors.is_empty(), "expected errors for {filename}");
    errors
}

pub fn assert_diagnostic(
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

pub fn assert_story_output(compiled: &CompiledStory, expected: &str) {
    let mut story = Story::new(&compiled.json);
    let output = story.continue_maximally();
    assert_eq!(output, expected);
    assert!(
        story.get_current_errors().is_empty(),
        "story should not emit runtime errors: {:#?}",
        story.get_current_errors()
    );
}

pub fn assert_json_sequence(json: &Value, sequence: Vec<Value>) {
    assert!(
        json_contains_sequence(json, &sequence),
        "expected compiled JSON to contain sequence {sequence:?}, got {json:#}"
    );
}

pub fn json_contains_divert_target(value: &Value, predicate: &impl Fn(&str) -> bool) -> bool {
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
