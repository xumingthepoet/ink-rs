use ink_compiler::{
    CheckedStory, Compiler, CompilerOptions, DiagnosticCode, DiagnosticSeverity, ParsedStory,
    RuntimeContainer, RuntimeObject, RuntimeProgram, SourceInput,
};
use std::collections::BTreeMap;

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
fn public_compile_sources_accepts_explicit_source_list() {
    let compiler = Compiler::default();
    let source = SourceInput::new("Line.");

    let single_output = compiler.compile(source.clone());
    let source_list_output = compiler.compile_sources(vec![source]);

    assert!(
        !single_output.has_errors(),
        "{:#?}",
        single_output.diagnostics
    );
    assert!(
        !source_list_output.has_errors(),
        "{:#?}",
        source_list_output.diagnostics
    );
    assert_eq!(
        single_output.artifact.expect("single source json").json,
        source_list_output.artifact.expect("source list json").json
    );
}

#[test]
fn public_compile_sources_accepts_multiple_sources_in_any_order() {
    let compiler = Compiler::default();
    let entry = SourceInput::named("-> start", "entry.ink");
    let flow = SourceInput::named("=== start ===\nHello.\n-> END", "flow.ink");

    let entry_first = compiler.compile_sources(vec![entry.clone(), flow.clone()]);
    let flow_first = compiler.compile_sources(vec![flow, entry]);

    assert!(!entry_first.has_errors(), "{:#?}", entry_first.diagnostics);
    assert!(!flow_first.has_errors(), "{:#?}", flow_first.diagnostics);
    assert_eq!(
        entry_first.artifact.expect("entry first json").json,
        flow_first.artifact.expect("flow first json").json
    );
}

#[test]
fn public_compile_sources_rejects_empty_input() {
    let output = Compiler::default().compile_sources(Vec::new());

    assert!(output.has_errors());
    assert!(output.artifact.is_none());
    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].message,
        "Compiler::compile_sources requires at least one source input"
    );
}

#[test]
fn public_compile_sources_preserves_source_filenames_in_diagnostics() {
    let output = Compiler::default().compile_sources(vec![
        SourceInput::named("Line.", "ok.ink"),
        SourceInput::named("VAR score = 1", "bad.ink"),
    ]);

    assert!(output.has_errors());
    let diagnostic = output
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .expect("expected an error diagnostic");
    assert_eq!(diagnostic.source_filename.as_deref(), Some("bad.ink"));
    assert_eq!(diagnostic.line, 1);
}

#[test]
fn public_parse_sources_combines_modules_from_multiple_inputs() {
    let output = Compiler::default().parse_sources(vec![
        SourceInput::named(
            "=== module game ===\n\
             IMPORT sword FROM items\n\
             == main ==\n\
             -> END",
            "game.ink",
        ),
        SourceInput::named(
            "=== module items ===\n\
             == sword ==\n\
             -> END",
            "items.ink",
        ),
    ]);

    assert!(!output.has_errors(), "{:#?}", output.diagnostics);
    let story = output.artifact.expect("expected parsed story");
    assert_eq!(module_names(&story), vec!["game", "items"]);
    assert_eq!(story.modules()[0].imports()[0].source_module(), "items");
    assert_eq!(
        story.modules()[0].name_span().source_name.as_deref(),
        Some("game.ink")
    );
    assert_eq!(
        story.modules()[0].imports()[0]
            .source_module_span()
            .source_name
            .as_deref(),
        Some("game.ink")
    );
    assert_eq!(
        story.modules()[1].name_span().source_name.as_deref(),
        Some("items.ink")
    );
}

#[test]
fn public_parse_sources_preserves_module_ownership_across_input_orders() {
    let compiler = Compiler::default();
    let game = SourceInput::named("=== module game ===\n== main ==\n-> END", "game.ink");
    let items = SourceInput::named("=== module items ===\n== sword ==\n-> END", "items.ink");

    let game_first = compiler.parse_sources(vec![game.clone(), items.clone()]);
    let items_first = compiler.parse_sources(vec![items, game]);

    assert!(!game_first.has_errors(), "{:#?}", game_first.diagnostics);
    assert!(!items_first.has_errors(), "{:#?}", items_first.diagnostics);
    let game_first_story = game_first.artifact.expect("expected parsed story");
    let items_first_story = items_first.artifact.expect("expected parsed story");
    assert_eq!(
        module_source_map(&game_first_story),
        module_source_map(&items_first_story)
    );
}

#[test]
fn public_parse_sources_preserves_original_source_in_diagnostics() {
    let output = Compiler::default().parse_sources(vec![
        SourceInput::named("=== module ok ===", "ok.ink"),
        SourceInput::named("Line.\n=== module bad ===", "bad.ink"),
    ]);

    assert!(output.has_errors());
    assert!(output.artifact.is_none());
    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].source_filename.as_deref(),
        Some("bad.ink")
    );
    assert_eq!(output.diagnostics[0].line, 1);
    assert_eq!(
        output.diagnostics[0].message,
        "Content and module-scoped declarations must appear after an explicit module declaration"
    );
}

#[test]
fn public_parse_sources_keeps_declared_module_names_independent_from_filenames() {
    let output = Compiler::default().parse_sources(vec![SourceInput::named(
        "=== module declared ===\n\
         == main ==\n\
         -> END",
        "not_declared.ink",
    )]);

    assert!(!output.has_errors(), "{:#?}", output.diagnostics);
    let story = output.artifact.expect("expected parsed story");
    assert_eq!(module_names(&story), vec!["declared"]);
    assert_eq!(
        story.modules()[0].name_span().source_name.as_deref(),
        Some("not_declared.ink")
    );
}

#[test]
fn public_parse_sources_accepts_one_source_with_multiple_modules() {
    let output = Compiler::default().parse_sources(vec![SourceInput::named(
        "=== module game ===\n\
         == main ==\n\
         -> END\n\
         === module items ===\n\
         == sword ==\n\
         -> END",
        "bundle.ink",
    )]);

    assert!(!output.has_errors(), "{:#?}", output.diagnostics);
    let story = output.artifact.expect("expected parsed story");
    assert_eq!(module_names(&story), vec!["game", "items"]);
    assert!(story
        .modules()
        .iter()
        .all(|module| module.name_span().source_name.as_deref() == Some("bundle.ink")));
}

#[test]
fn public_diagnostics_expose_codes() {
    let output = Compiler::default().compile(SourceInput::new("Line {x + 1"));

    assert!(output.has_errors());
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == Some(DiagnosticCode::InvalidInlineSyntax)));
}

fn module_names(story: &ParsedStory) -> Vec<&str> {
    story.modules().iter().map(|module| module.name()).collect()
}

fn module_source_map(story: &ParsedStory) -> BTreeMap<String, Option<String>> {
    story
        .modules()
        .iter()
        .map(|module| {
            (
                module.name().to_string(),
                module.name_span().source_name.clone(),
            )
        })
        .collect()
}
