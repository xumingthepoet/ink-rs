use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

const ALLOWED_NON_MODULE_INK_FIXTURES: &[&str] = &[];

const ALLOWED_SOURCE_CONSTRUCTION_TESTS: &[&str] = &[];

const DISABLED_TEST_HARNESS_SOURCE_ROOTS: &[(&str, &str)] = &[
    ("Cargo.toml", "src"),
    (
        "crates/text-games-app/Cargo.toml",
        "crates/text-games-app/src",
    ),
    ("crates/ink-test/Cargo.toml", "crates/ink-test/src"),
    ("crates/ink-tools/Cargo.toml", "crates/ink-tools/src"),
];

const BANNED_ORIGIN_LABELS: &[&str] = &[
    "language",
    "conformance",
    "compiler_conformance",
    "inkling",
    "inkfiles",
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
    "get_json_string(",
    ".ink.json",
    "SourceInput::new(\"",
    "SourceInput::named(\"",
];

const REMOVED_COMPILED_JSON_STRING_TOKENS: &[&str] = &[
    "done",
    "end",
    "thread",
    "choiceCnt",
    "turn",
    "turns",
    "readc",
    "visit",
    "seq",
];

const REMOVED_COMPILED_JSON_OBJECT_KEYS: &[&str] = &["CNT?", "#f"];

#[test]
fn module_fixture_policy_tracks_non_module_ink_files() {
    let fixture_root = ink_test::fixture_root();
    let non_module_fixtures = ink_files_under(&fixture_root)
        .into_iter()
        .filter_map(|path| non_module_fixture_path(&fixture_root, &path))
        .collect::<BTreeSet<_>>();
    let allowed = ALLOWED_NON_MODULE_INK_FIXTURES
        .iter()
        .map(|path| path.to_string())
        .collect::<BTreeSet<_>>();

    let unexpected = non_module_fixtures
        .difference(&allowed)
        .cloned()
        .collect::<Vec<_>>();
    let stale_allowlist = allowed
        .difference(&non_module_fixtures)
        .cloned()
        .collect::<Vec<_>>();

    assert!(
        unexpected.is_empty(),
        "unexpected non-module .ink fixtures; convert them to explicit module syntax or add a task-specific allowlist entry: {unexpected:#?}"
    );
    assert!(
        stale_allowlist.is_empty(),
        "non-module .ink allowlist entries are stale; remove stale entries from ALLOWED_NON_MODULE_INK_FIXTURES: {stale_allowlist:#?}"
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
        "unexpected integration tests construct ink-rs source directly; move the source to .ink fixtures: {unexpected:#?}"
    );
    assert!(
        stale_allowlist.is_empty(),
        "inline source-construction allowlist entries are stale; remove stale entries from ALLOWED_SOURCE_CONSTRUCTION_TESTS: {stale_allowlist:#?}"
    );
}

#[test]
fn origin_label_policy_tracks_test_and_fixture_names() {
    let tests_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let fixture_root = ink_test::fixture_root();
    let offenders = files_under(&tests_root, |_| true)
        .into_iter()
        .chain(files_under(&fixture_root, |_| true))
        .filter_map(|path| origin_label_offender(&tests_root, &fixture_root, &path))
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "origin/example-suite labels must not appear in test targets, helper module paths, fixture paths, or test names: {offenders:#?}"
    );
}

#[test]
fn compiled_json_fixtures_have_source_siblings() {
    let fixture_root = ink_test::fixture_root();
    let orphaned_json = files_under(&fixture_root, |path| {
        path.file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with(".ink.json"))
    })
    .into_iter()
    .filter(|path| {
        let source_path = PathBuf::from(path.to_string_lossy().trim_end_matches(".json"));
        !source_path.exists()
    })
    .map(|path| relative_path(&fixture_root, &path))
    .collect::<Vec<_>>();

    assert!(
        orphaned_json.is_empty(),
        "compiled JSON fixtures must be snapshots for source .ink fixtures, not runtime integration inputs: {orphaned_json:#?}"
    );
}

#[test]
fn current_compiled_json_surfaces_reject_removed_legacy_format() {
    let workspace_root = workspace_root();
    let fixture_root = ink_test::fixture_root();

    let mut offenders = Vec::new();
    for path in files_under(&fixture_root, |path| {
        path.file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with(".ink.json"))
    }) {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read fixture {}: {error}", path.display()));
        let json: Value = serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("fixture {} is not JSON: {error}", path.display()));
        collect_removed_compiled_json_residue(
            &json,
            &relative_path(&workspace_root, &path),
            "$",
            &mut offenders,
        );
    }

    let format_doc = workspace_root.join("docs/ink_JSON_runtime_format.md");
    let format_doc_text = fs::read_to_string(&format_doc)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", format_doc.display()));
    for pattern in removed_compiled_json_doc_patterns() {
        if format_doc_text.contains(&pattern) {
            offenders.push(format!(
                "{} contains removed current-format token {pattern}",
                relative_path(&workspace_root, &format_doc)
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "current compiled JSON fixtures and format docs must not contain removed legacy format residue: {offenders:#?}"
    );
}

#[test]
fn disabled_test_harness_targets_stay_test_free() {
    let workspace_root = workspace_root();
    let disabled_manifests = manifests_with_disabled_test_harnesses(&workspace_root)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let tracked_manifests = DISABLED_TEST_HARNESS_SOURCE_ROOTS
        .iter()
        .map(|(manifest, _)| manifest.to_string())
        .collect::<BTreeSet<_>>();

    let unexpected = disabled_manifests
        .difference(&tracked_manifests)
        .cloned()
        .collect::<Vec<_>>();
    let stale = tracked_manifests
        .difference(&disabled_manifests)
        .cloned()
        .collect::<Vec<_>>();

    assert!(
        unexpected.is_empty(),
        "test = false targets must be tracked by DISABLED_TEST_HARNESS_SOURCE_ROOTS so ignored unit tests cannot be added silently: {unexpected:#?}"
    );
    assert!(
        stale.is_empty(),
        "DISABLED_TEST_HARNESS_SOURCE_ROOTS entries are stale; remove entries for targets that no longer set test = false: {stale:#?}"
    );

    let offenders = DISABLED_TEST_HARNESS_SOURCE_ROOTS
        .iter()
        .flat_map(|(_, source_root)| source_files_with_unit_tests(&workspace_root, source_root))
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "targets with test = false must not contain unit-test markers because Cargo would ignore them; remove test = false or move the tests: {offenders:#?}"
    );
}

fn collect_removed_compiled_json_residue(
    value: &Value,
    file: &str,
    json_path: &str,
    offenders: &mut Vec<String>,
) {
    match value {
        Value::String(token) => {
            if REMOVED_COMPILED_JSON_STRING_TOKENS.contains(&token.as_str()) {
                offenders.push(format!(
                    "{file}:{json_path} contains removed token {token:?}"
                ));
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_removed_compiled_json_residue(
                    item,
                    file,
                    &format!("{json_path}[{index}]"),
                    offenders,
                );
            }
        }
        Value::Object(fields) => {
            if let Some(flags) = fields.get("flg").and_then(Value::as_i64) {
                if flags & !0x0b != 0 {
                    offenders.push(format!(
                        "{file}:{json_path}.flg contains unsupported choice flag bits {flags}"
                    ));
                }
            }
            for (key, item) in fields {
                if REMOVED_COMPILED_JSON_OBJECT_KEYS.contains(&key.as_str()) {
                    offenders.push(format!("{file}:{json_path} contains removed key {key:?}"));
                }
                collect_removed_compiled_json_residue(
                    item,
                    file,
                    &format!("{json_path}.{key}"),
                    offenders,
                );
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn removed_compiled_json_doc_patterns() -> Vec<String> {
    REMOVED_COMPILED_JSON_STRING_TOKENS
        .iter()
        .map(|token| format!("`\"{token}\"`"))
        .chain(
            REMOVED_COMPILED_JSON_OBJECT_KEYS
                .iter()
                .map(|key| format!("`{key}`")),
        )
        .collect()
}

fn non_module_fixture_path(root: &Path, path: &Path) -> Option<String> {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read fixture {}: {error}", path.display()));
    let first_content = text
        .trim_start_matches('\u{feff}')
        .lines()
        .find(|line| !line.trim().is_empty())?
        .trim_start();

    if first_content.starts_with("=== module ") || first_content.starts_with("=== interface ") {
        return None;
    }

    Some(relative_path(root, path))
}

fn manifests_with_disabled_test_harnesses(workspace_root: &Path) -> Vec<String> {
    workspace_manifests(workspace_root)
        .into_iter()
        .filter(|path| {
            let text = fs::read_to_string(path).unwrap_or_else(|error| {
                panic!("failed to read manifest {}: {error}", path.display())
            });
            text.lines().any(|line| line.trim() == "test = false")
        })
        .map(|path| relative_path(workspace_root, &path))
        .collect()
}

fn workspace_manifests(workspace_root: &Path) -> Vec<PathBuf> {
    let mut manifests = vec![workspace_root.join("Cargo.toml")];
    manifests.extend(files_under(&workspace_root.join("crates"), |path| {
        path.file_name().is_some_and(|name| name == "Cargo.toml")
    }));
    manifests.sort();
    manifests
}

fn source_files_with_unit_tests(workspace_root: &Path, source_root: &str) -> Vec<String> {
    let source_root = workspace_root.join(source_root);
    rust_files_under(&source_root)
        .into_iter()
        .filter(|path| source_file_contains_unit_test_marker(path))
        .map(|path| relative_path(workspace_root, &path))
        .collect()
}

fn source_file_contains_unit_test_marker(path: &Path) -> bool {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read source {}: {error}", path.display()));
    text.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("#[test]") || line.starts_with("#[cfg(test)]")
    })
}

fn origin_label_offender(tests_root: &Path, fixture_root: &Path, path: &Path) -> Option<String> {
    if path
        .file_name()
        .is_some_and(|name| name == "integration_policy.rs")
    {
        return None;
    }

    let relative = if let Ok(path) = path.strip_prefix(tests_root) {
        format!("tests/{}", relative_path(Path::new(""), path))
    } else if let Ok(path) = path.strip_prefix(fixture_root) {
        format!("fixtures/{}", relative_path(Path::new(""), path))
    } else {
        relative_path(Path::new(""), path)
    };

    if BANNED_ORIGIN_LABELS
        .iter()
        .any(|label| relative.split('/').any(|segment| segment.contains(label)))
    {
        return Some(relative);
    }

    if path.extension().is_some_and(|extension| extension == "rs") {
        let text = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read test {}: {error}", path.display()));
        if BANNED_ORIGIN_LABELS
            .iter()
            .any(|label| text.contains(label))
        {
            return Some(relative);
        }
    }

    None
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

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|error| panic!("failed to resolve workspace root: {error}"))
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
