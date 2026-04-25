use ink_compiler::{
    CheckedStory, Compiler, CompilerOptions, DiagnosticCode, ParsedStory, RuntimeContainer,
    RuntimeObject, RuntimeProgram, SourceInput,
};

#[test]
fn public_compiler_api_exposes_pipeline_artifacts() {
    let compiler = Compiler::new();

    let parsed = compiler.parse(SourceInput::named("Line.", "api.ink"));
    assert!(!parsed.has_errors(), "{:#?}", parsed.diagnostics);
    let parsed_story: ParsedStory = parsed.artifact.expect("expected parsed story");

    let checked = compiler.analyze(parsed_story);
    assert!(!checked.has_errors(), "{:#?}", checked.diagnostics);
    let checked_story: CheckedStory = checked.artifact.expect("expected checked story");

    let lowered = compiler.lower(&checked_story);
    assert!(!lowered.has_errors(), "{:#?}", lowered.diagnostics);
    let program: RuntimeProgram = lowered.artifact.expect("expected runtime program");
    let root: RuntimeContainer = program.root.clone();
    let _first_runtime_object: Option<&RuntimeObject> = root.content.first();

    let emitted = compiler.emit_json(program);
    assert!(!emitted.has_errors(), "{:#?}", emitted.diagnostics);
    let json = emitted.artifact.expect("expected emitted JSON");
    assert!(json.contains("\"inkVersion\""));
}

#[test]
fn public_compiler_options_are_constructible() {
    let compiler = Compiler::with_options(CompilerOptions {
        source_filename: Some("api-options.ink".to_string()),
        count_all_visits: true,
        file_handler: None,
    });

    let output = compiler.compile(SourceInput::new("Line."));

    assert!(!output.has_errors(), "{:#?}", output.diagnostics);
    assert!(output
        .artifact
        .expect("expected compiled story")
        .json
        .contains("Line."));
}

#[test]
fn public_diagnostics_expose_codes() {
    let output = Compiler::default().compile(SourceInput::new("LIST inventory = sword"));

    assert!(output.has_errors());
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == Some(DiagnosticCode::RemovedFeature)));
}
