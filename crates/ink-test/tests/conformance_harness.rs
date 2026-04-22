use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use ink_compiler::{Compiler, CompilerOptions, FileHandler};
use ink_test::load_fixture_text;

#[derive(Debug)]
struct HarnessFileHandler {
    files: HashMap<PathBuf, String>,
}

impl HarnessFileHandler {
    fn new(files: HashMap<PathBuf, String>) -> Self {
        Self { files }
    }
}

impl FileHandler for HarnessFileHandler {
    fn resolve_ink_filename(&self, include_name: &str) -> PathBuf {
        PathBuf::from("/virtual").join(include_name)
    }

    fn load_ink_file_contents(&self, full_filename: &Path) -> io::Result<String> {
        self.files.get(full_filename).cloned().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("missing include: {}", full_filename.display()),
            )
        })
    }
}

fn compile_json(
    source_text: String,
    source_filename: &str,
    file_handler: Option<Arc<dyn FileHandler>>,
) -> String {
    let mut compiler = Compiler::new(
        source_text,
        Some(CompilerOptions {
            source_filename: Some(source_filename.to_string()),
            count_all_visits: false,
            file_handler,
        }),
    );

    let result = compiler.compile_json();
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:#?}",
        result.diagnostics
    );
    result.json.expect("expected compiled JSON")
}

fn run_story(json: &str) -> String {
    let mut story = ink_runtime::story::Story::new(json).expect("load runtime story");
    let mut output = String::new();
    while story.can_continue() {
        output.push_str(&story.cont().expect("continue story"));
    }
    output
}

#[test]
fn blade_basictext_oneline_matches_trusted_runtime_output() {
    let cases = [
        (
            "conformance-tests/inkfiles/basictext/oneline.ink",
            "conformance-tests/inkfiles/basictext/oneline.ink.json",
            "/virtual/oneline.ink",
            "Line.\n",
        ),
        (
            "conformance-tests/inkfiles/basictext/twolines.ink",
            "conformance-tests/inkfiles/basictext/twolines.ink.json",
            "/virtual/twolines.ink",
            "Line.\nOther line.\n",
        ),
        (
            "conformance-tests/inkfiles/knot/multi-line.ink",
            "conformance-tests/inkfiles/knot/multi-line.ink.json",
            "/virtual/multi-line.ink",
            "Hello, world!\nHello?\nHello, are you there?\n",
        ),
        (
            "conformance-tests/inkfiles/knot/strip-empty-lines.ink",
            "conformance-tests/inkfiles/knot/strip-empty-lines.ink.json",
            "/virtual/strip-empty-lines.ink",
            "Hello, world!\nHello?\nHello, are you there?\n",
        ),
        (
            "conformance-tests/inkfiles/knot/single-line.ink",
            "conformance-tests/inkfiles/knot/single-line.ink.json",
            "/virtual/single-line.ink",
            "Hello, world!\n",
        ),
    ];

    for (source_path, expected_path, virtual_path, expected_output) in cases {
        let source = load_fixture_text(source_path);
        let expected = load_fixture_text(expected_path);

        let actual = compile_json(source, virtual_path, None);
        assert_eq!(run_story(&expected), run_story(&actual));
        assert_eq!(expected_output, run_story(&actual));
    }
}

#[test]
fn csharp_include_story_matches_expected_runtime_output() {
    let source = "\
INCLUDE test_included_file.ink\n\
  INCLUDE test_included_file2.ink\n\
\n\
This is the main file.\n";

    let mut files = HashMap::new();
    files.insert(
        PathBuf::from("/virtual/test_included_file.ink"),
        load_fixture_text("ink-csharp/tests/test_included_file.ink"),
    );
    files.insert(
        PathBuf::from("/virtual/test_included_file2.ink"),
        load_fixture_text("ink-csharp/tests/test_included_file2.ink"),
    );

    let file_handler: Arc<dyn FileHandler> = Arc::new(HarnessFileHandler::new(files));
    let actual = compile_json(source.to_string(), "/virtual/main.ink", Some(file_handler));

    assert_eq!(
        "This is include 1.\nThis is include 2.\nThis is the main file.\n",
        run_story(&actual)
    );
}
