use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const ALLOWED_LEGACY_INK_FIXTURES: &[&str] = &[
    "conformance/inkfiles/basictext/oneline.ink",
    "conformance/inkfiles/basictext/twolines.ink",
    "conformance/inkfiles/conditional/ifelse-ext.ink",
    "conformance/inkfiles/conditional/ifelse.ink",
    "conformance/inkfiles/conditional/iffalse.ink",
    "conformance/inkfiles/conditional/iftrue.ink",
    "conformance/inkfiles/divert/invisible-divert.ink",
    "conformance/inkfiles/divert/simple-divert.ink",
    "conformance/inkfiles/function/complex-func1.ink",
    "conformance/inkfiles/function/complex-func2.ink",
    "conformance/inkfiles/function/complex-func3.ink",
    "conformance/inkfiles/function/evaluating-function-variablestate-bug.ink",
    "conformance/inkfiles/function/func-basic.ink",
    "conformance/inkfiles/function/func-inline.ink",
    "conformance/inkfiles/function/func-none.ink",
    "conformance/inkfiles/function/rnd-func.ink",
    "conformance/inkfiles/function/setvar-func.ink",
    "conformance/inkfiles/function/test-error.ink",
    "conformance/inkfiles/glue/glue-with-divert.ink",
    "conformance/inkfiles/glue/left-right-glue-matching.ink",
    "conformance/inkfiles/glue/simple-glue.ink",
    "conformance/inkfiles/glue/testbugfix1.ink",
    "conformance/inkfiles/glue/testbugfix2.ink",
    "conformance/inkfiles/knot/multi-line.ink",
    "conformance/inkfiles/knot/param-recurse.ink",
    "conformance/inkfiles/knot/single-line.ink",
    "conformance/inkfiles/knot/strip-empty-lines.ink",
    "conformance/inkfiles/misc/i18n.ink",
    "conformance/inkfiles/misc/issue15.ink",
    "conformance/inkfiles/misc/newlines_with_string_eval.ink",
    "conformance/inkfiles/misc/operations.ink",
    "conformance/inkfiles/runtime/external-function-0-arg.ink",
    "conformance/inkfiles/runtime/external-function-1-arg.ink",
    "conformance/inkfiles/runtime/external-function-2-arg.ink",
    "conformance/inkfiles/runtime/external-function-3-arg.ink",
    "conformance/inkfiles/runtime/jump-knot.ink",
    "conformance/inkfiles/runtime/jump-stitch.ink",
    "conformance/inkfiles/runtime/multiflow-basics.ink",
    "conformance/inkfiles/runtime/read-visit-counts.ink",
    "conformance/inkfiles/runtime/saving-loading.ink",
    "conformance/inkfiles/tags/tags.ink",
    "conformance/inkfiles/tags/tagsDynamicContent.ink",
    "conformance/inkfiles/tags/tagsInSeq.ink",
    "conformance/inkfiles/tunnels/tunnel-onwards-divert-override.ink",
    "conformance/inkfiles/typed/array-literals.ink",
    "conformance/inkfiles/typed/struct-literals.ink",
    "conformance/inkfiles/variable/varcalc.ink",
    "conformance/inkfiles/variable/variable-declaration.ink",
];

const ALLOWED_SOURCE_CONSTRUCTION_TESTS: &[&str] = &[
    "compiler_api.rs",
    "csharp_tests/mod.rs",
    "inkling_examples.rs",
    "language.rs",
];

const SOURCE_CONSTRUCTION_PATTERNS: &[&str] = &[
    "explicit_game_module",
    "compile_language_source(",
    "diagnostics_for_language_source(",
    "compile_story(",
    "compile_error_messages(",
    "assert_compile_errors(",
    "compile_string(",
    "compile_string_without_runtime(",
    "SourceInput::new(\"",
    "SourceInput::named(\"",
];

#[test]
fn module_fixture_policy_tracks_legacy_ink_files() {
    let fixture_root = ink_test::fixture_root();
    let legacy_fixtures = ink_files_under(&fixture_root)
        .into_iter()
        .filter_map(|path| legacy_fixture_path(&fixture_root, &path))
        .collect::<BTreeSet<_>>();
    let allowed = ALLOWED_LEGACY_INK_FIXTURES
        .iter()
        .map(|path| path.to_string())
        .collect::<BTreeSet<_>>();

    let unexpected = legacy_fixtures
        .difference(&allowed)
        .cloned()
        .collect::<Vec<_>>();
    let stale_allowlist = allowed
        .difference(&legacy_fixtures)
        .cloned()
        .collect::<Vec<_>>();

    assert!(
        unexpected.is_empty(),
        "unexpected legacy .ink fixtures; convert them to explicit module syntax or add a task-specific allowlist entry: {unexpected:#?}"
    );
    assert!(
        stale_allowlist.is_empty(),
        "legacy .ink allowlist entries are stale; remove migrated fixtures from ALLOWED_LEGACY_INK_FIXTURES: {stale_allowlist:#?}"
    );
}

#[test]
fn source_construction_policy_tracks_inline_ink_helpers() {
    let tests_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let offenders = rust_files_under(&tests_root)
        .into_iter()
        .filter_map(|path| source_construction_file(&tests_root, &path))
        .collect::<BTreeSet<_>>();
    let allowed = ALLOWED_SOURCE_CONSTRUCTION_TESTS
        .iter()
        .map(|path| path.to_string())
        .collect::<BTreeSet<_>>();

    let unexpected = offenders.difference(&allowed).cloned().collect::<Vec<_>>();
    let stale_allowlist = allowed.difference(&offenders).cloned().collect::<Vec<_>>();

    assert!(
        unexpected.is_empty(),
        "unexpected integration tests construct Ink source directly; move the source to .ink fixtures: {unexpected:#?}"
    );
    assert!(
        stale_allowlist.is_empty(),
        "inline Ink/source-construction allowlist entries are stale; remove migrated files from ALLOWED_SOURCE_CONSTRUCTION_TESTS: {stale_allowlist:#?}"
    );
}

fn legacy_fixture_path(root: &Path, path: &Path) -> Option<String> {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read fixture {}: {error}", path.display()));
    let first_content = text
        .trim_start_matches('\u{feff}')
        .lines()
        .find(|line| !line.trim().is_empty())?
        .trim_start();

    if first_content.starts_with("=== module ") {
        return None;
    }

    Some(relative_path(root, path))
}

fn source_construction_file(root: &Path, path: &Path) -> Option<String> {
    if path
        .file_name()
        .is_some_and(|name| name == "integration_policy.rs")
    {
        return None;
    }

    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read test {}: {error}", path.display()));
    if SOURCE_CONSTRUCTION_PATTERNS
        .iter()
        .any(|pattern| text.contains(pattern))
    {
        Some(relative_path(root, path))
    } else {
        None
    }
}

fn ink_files_under(root: &Path) -> Vec<PathBuf> {
    files_under(root, |path| {
        path.extension().is_some_and(|extension| extension == "ink")
    })
}

fn rust_files_under(root: &Path) -> Vec<PathBuf> {
    files_under(root, |path| {
        path.extension().is_some_and(|extension| extension == "rs")
    })
}

fn files_under(root: &Path, include: impl Fn(&Path) -> bool + Copy) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files(root, include, &mut files);
    files.sort();
    files
}

fn collect_files(root: &Path, include: impl Fn(&Path) -> bool + Copy, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read directory {}: {error}", root.display()))
    {
        let path = entry
            .unwrap_or_else(|error| panic!("failed to read directory entry: {error}"))
            .path();
        if path.is_dir() {
            collect_files(&path, include, files);
        } else if include(&path) {
            files.push(path);
        }
    }
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
