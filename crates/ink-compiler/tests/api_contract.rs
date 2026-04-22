use std::{
    collections::HashMap,
    io,
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

#[derive(Debug)]
struct IncludeFileHandler {
    resolve_calls: AtomicUsize,
    load_calls: AtomicUsize,
    files: HashMap<PathBuf, String>,
}

impl IncludeFileHandler {
    fn new(files: HashMap<PathBuf, String>) -> Self {
        Self {
            resolve_calls: AtomicUsize::new(0),
            load_calls: AtomicUsize::new(0),
            files,
        }
    }
}

impl FileHandler for IncludeFileHandler {
    fn resolve_ink_filename(&self, include_name: &str) -> PathBuf {
        self.resolve_calls.fetch_add(1, Ordering::SeqCst);
        PathBuf::from("/virtual").join(include_name)
    }

    fn load_ink_file_contents(&self, full_filename: &Path) -> io::Result<String> {
        self.load_calls.fetch_add(1, Ordering::SeqCst);
        self.files.get(full_filename).cloned().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("missing include: {}", full_filename.display()),
            )
        })
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
fn parse_returns_a_structured_plain_text_story() {
    let mut compiler = Compiler::new(
        "Hello world",
        Some(CompilerOptions {
            source_filename: Some("story.ink".to_string()),
            count_all_visits: true,
            file_handler: None,
        }),
    );

    let result = compiler.parse();

    assert!(result.parsed_story.is_some());
    assert!(result.diagnostics.is_empty());
    assert!(compiler.parsed_story().is_some());
}

#[test]
fn compile_json_returns_runtime_json_for_plain_text_story() {
    let mut compiler = Compiler::new(
        "Hello world",
        Some(CompilerOptions {
            source_filename: Some("story.ink".to_string()),
            count_all_visits: false,
            file_handler: None,
        }),
    );

    let result = compiler.compile_json();

    let json = result.json.expect("expected runtime JSON");
    assert!(json.contains("\"inkVersion\":21"));
    assert!(json.contains("\"root\""));
    assert!(json.contains("\"listDefs\":{}"));
    assert!(result.diagnostics.is_empty());
}

#[test]
fn compile_returns_a_runtime_story_for_plain_text_story() {
    let mut compiler = Compiler::new("Hello world", None);

    let result = compiler.compile();

    let mut story = result.story.expect("expected runtime story");
    assert!(result.diagnostics.is_empty());
    assert!(story.can_continue());
    assert_eq!(story.cont().unwrap(), "Hello world");
    assert!(!story.can_continue());
}

#[test]
fn compiler_strips_utf8_bom_from_source_text() {
    let mut compiler = Compiler::new("\u{feff}Hello world", None);

    let result = compiler.compile();

    let mut story = result.story.expect("expected runtime story");
    assert!(result.diagnostics.is_empty());
    assert!(story.can_continue());
    assert_eq!(story.cont().unwrap(), "Hello world");
    assert!(!story.can_continue());
}

#[test]
fn compiler_compile_json_expands_includes_through_file_handler() {
    let mut files = HashMap::new();
    files.insert(
        PathBuf::from("/virtual/chapter.ink"),
        "Included line.".to_string(),
    );

    let file_handler = Arc::new(IncludeFileHandler::new(files));
    let handler: Arc<dyn FileHandler> = file_handler.clone();
    let mut compiler = Compiler::new(
        "Prelude.\nINCLUDE chapter.ink\nPostlude.",
        Some(CompilerOptions {
            source_filename: Some("/virtual/main.ink".to_string()),
            count_all_visits: false,
            file_handler: Some(handler),
        }),
    );

    let result = compiler.compile_json();

    let json = result
        .json
        .expect("expected runtime JSON for include story");
    assert!(json.contains("Prelude."));
    assert!(json.contains("Included line."));
    assert!(json.contains("Postlude."));
    assert!(result.diagnostics.is_empty());
    assert_eq!(file_handler.resolve_calls.load(Ordering::SeqCst), 1);
    assert_eq!(file_handler.load_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn compiler_strips_utf8_bom_from_included_files() {
    let mut files = HashMap::new();
    files.insert(
        PathBuf::from("/virtual/chapter.ink"),
        "\u{feff}Included line.".to_string(),
    );

    let file_handler = Arc::new(IncludeFileHandler::new(files));
    let handler: Arc<dyn FileHandler> = file_handler.clone();
    let mut compiler = Compiler::new(
        "INCLUDE chapter.ink",
        Some(CompilerOptions {
            source_filename: Some("/virtual/main.ink".to_string()),
            count_all_visits: false,
            file_handler: Some(handler),
        }),
    );

    let result = compiler.compile_json();

    let json = result
        .json
        .expect("expected runtime JSON for include story");
    assert!(!json.contains('\u{feff}'));
    assert!(json.contains("Included line."));
    assert_eq!(file_handler.resolve_calls.load(Ordering::SeqCst), 1);
    assert_eq!(file_handler.load_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn compiler_reports_includes_diagnostics_with_included_source_filename() {
    let mut files = HashMap::new();
    files.insert(
        PathBuf::from("/virtual/broken.ink"),
        "* invalid".to_string(),
    );

    let file_handler = Arc::new(IncludeFileHandler::new(files));
    let handler: Arc<dyn FileHandler> = file_handler.clone();
    let mut compiler = Compiler::new(
        "INCLUDE broken.ink",
        Some(CompilerOptions {
            source_filename: Some("/virtual/main.ink".to_string()),
            count_all_visits: false,
            file_handler: Some(handler),
        }),
    );

    let result = compiler.parse();

    let diagnostic = result
        .diagnostics
        .first()
        .expect("expected include diagnostic");
    assert_eq!(
        diagnostic.source_filename.as_deref(),
        Some("/virtual/broken.ink")
    );
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert!(diagnostic.message.contains("unsupported syntax"));
    assert_eq!(file_handler.resolve_calls.load(Ordering::SeqCst), 1);
    assert_eq!(file_handler.load_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn compile_json_rejects_structured_story_for_now() {
    let mut compiler = Compiler::new("== start ==\nHello", None);

    let result = compiler.compile_json();

    assert!(result.json.is_none());
    assert!(result.has_errors());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, DiagnosticSeverity::Error);
}
