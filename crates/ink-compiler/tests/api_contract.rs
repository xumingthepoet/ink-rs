use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use ink_compiler::{parser::InkParser, Compiler, CompilerOptions, DiagnosticSeverity, FileHandler};

#[derive(Debug)]
struct MockFileHandler {
    resolve_calls: AtomicUsize,
    load_calls: AtomicUsize,
}

impl MockFileHandler {
    fn new() -> Self {
        Self {
            resolve_calls: AtomicUsize::new(0),
            load_calls: AtomicUsize::new(0),
        }
    }
}

impl FileHandler for MockFileHandler {
    fn resolve_ink_filename(&self, include_name: &str) -> PathBuf {
        self.resolve_calls.fetch_add(1, Ordering::SeqCst);
        PathBuf::from("/virtual").join(include_name)
    }

    fn load_ink_file_contents(&self, _full_filename: &Path) -> std::io::Result<String> {
        self.load_calls.fetch_add(1, Ordering::SeqCst);
        Ok("-> done".to_string())
    }
}

#[test]
fn compiler_options_can_hold_a_file_handler() {
    let file_handler: Arc<dyn FileHandler> = Arc::new(MockFileHandler::new());
    let options = CompilerOptions {
        source_filename: Some("story.ink".to_string()),
        count_all_visits: true,
        file_handler: Some(file_handler.clone()),
    };

    let cloned = options.clone();
    let handler = cloned.file_handler.expect("file handler should clone");
    assert_eq!(
        handler.resolve_ink_filename("chapter.ink"),
        PathBuf::from("/virtual/chapter.ink")
    );
    assert_eq!(
        handler
            .load_ink_file_contents(Path::new("/virtual/chapter.ink"))
            .unwrap(),
        "-> done"
    );
}

#[test]
fn ink_parser_exposes_the_file_handler_contract() {
    let file_handler: Arc<dyn FileHandler> = Arc::new(MockFileHandler::new());
    let parser = InkParser::new("Hello", Some("story.ink"), Some(file_handler));

    assert_eq!(parser.input_string(), "Hello");
    assert_eq!(parser.source_filename(), Some("story.ink"));
    assert!(parser.file_handler().is_some());
}

#[test]
fn parse_returns_a_structured_unsupported_diagnostic() {
    let mut compiler = Compiler::new(
        "Hello world",
        Some(CompilerOptions {
            source_filename: Some("story.ink".to_string()),
            count_all_visits: true,
            file_handler: None,
        }),
    );

    let result = compiler.parse();

    assert!(result.parsed_story.is_none());
    assert!(result.has_errors());
    assert_eq!(result.diagnostics.len(), 1);

    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(diagnostic.source_filename.as_deref(), Some("story.ink"));
    assert_eq!(diagnostic.line, 0);
    assert_eq!(diagnostic.column, 0);
    assert!(diagnostic.message.contains("InkParser"));
    assert!(compiler.parsed_story().is_none());
}

#[test]
fn compile_json_returns_a_result_object_with_diagnostics() {
    let mut compiler = Compiler::new(
        "Hello world",
        Some(CompilerOptions {
            source_filename: Some("story.ink".to_string()),
            count_all_visits: false,
            file_handler: None,
        }),
    );

    let result = compiler.compile_json();

    assert!(result.json.is_none());
    assert!(result.has_errors());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, DiagnosticSeverity::Error);
}

#[test]
fn compile_returns_a_runtime_result_object_with_diagnostics() {
    let mut compiler = Compiler::new("Hello world", None);

    let result = compiler.compile();

    assert!(result.story.is_none());
    assert!(result.has_errors());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, DiagnosticSeverity::Error);
}
