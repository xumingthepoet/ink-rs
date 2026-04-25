use ink_compiler::{Compiler, Diagnostic, DiagnosticCode, DiagnosticSeverity, SourceInput};
use ink_runtime::story::Story;

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

fn assert_diagnostic_code(diagnostics: &[Diagnostic], code: DiagnosticCode) {
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == Some(code)),
        "expected diagnostic code {code:?}, got {diagnostics:#?}"
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

#[test]
fn language_fixture_smoke_test_runs_compiled_story() {
    let compiled = compile_language_fixture("smoke.ink");
    assert_story_output(&compiled, "Hello from the language test surface.\n");
}

#[test]
fn language_diagnostic_helper_asserts_error_messages() {
    let diagnostics = diagnostics_for_language_source("diagnostic-smoke.ink", "-> missing_target");
    assert_diagnostic(&diagnostics, DiagnosticSeverity::Error, "target not found");
}

#[test]
fn removed_list_declaration_reports_removed_feature_diagnostic() {
    let diagnostics =
        diagnostics_for_language_source("removed-list.ink", "LIST inventory = sword, shield");

    assert_diagnostic_code(&diagnostics, DiagnosticCode::RemovedFeature);
    assert_diagnostic(
        &diagnostics,
        DiagnosticSeverity::Error,
        "removed feature: LIST declarations",
    );
}
