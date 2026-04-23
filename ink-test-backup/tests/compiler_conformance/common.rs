#![allow(dead_code)]

use std::{fs, path::Path};

use ink_compiler::{Compiler, CompilerOptions};
use serde_json::Value;

pub fn compile_json(filename: &str) -> Value {
    let source = get_ink_string(filename);
    let mut compiler = Compiler::new(
        source,
        Some(CompilerOptions {
            source_filename: Some(filename.to_string()),
            ..Default::default()
        }),
    );
    let result = compiler.compile_json();
    assert!(
        result.diagnostics.is_empty(),
        "compiler_conformance compile_json for {filename} should not emit diagnostics: {:#?}",
        result.diagnostics
    );
    let json = result.json.expect("expected compiled json");
    parse_json_value(filename, &json, "generated")
}

pub fn assert_compiled_json_matches_fixture(filename: &str) {
    let generated_value = compile_json(filename);
    let fixture_json = get_fixture_json_string(filename);
    let fixture_value = parse_json_value(filename, &fixture_json, "fixture");
    assert_eq!(
        generated_value,
        fixture_value,
        "{}",
        format_compiled_json_mismatch(filename, &generated_value, &fixture_value)
    );
}

pub fn all_fixture_paths() -> Vec<String> {
    let root = ink_test::fixture_root().join("conformance");
    let inkfiles_root = root.join("inkfiles");
    let mut fixtures = Vec::new();
    collect_ink_files(&root, &inkfiles_root, &mut fixtures);
    fixtures.sort();
    fixtures
}

pub fn top_level_fixture_paths() -> Vec<String> {
    all_fixture_paths()
        .into_iter()
        .filter(|path| {
            Path::new(path)
                .parent()
                .is_some_and(|parent| parent == Path::new("inkfiles"))
        })
        .collect()
}

pub fn fixture_group_paths(group: &str) -> Vec<String> {
    let prefix = format!("inkfiles/{group}/");
    all_fixture_paths()
        .into_iter()
        .filter(|path| path.starts_with(&prefix))
        .collect()
}

fn collect_ink_files(base: &Path, dir: &Path, out: &mut Vec<String>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|error| {
        panic!(
            "failed to read fixture directory {}: {error}",
            dir.display()
        )
    });

    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            panic!("failed to read fixture entry in {}: {error}", dir.display())
        });
        let path = entry.path();
        if path.is_dir() {
            collect_ink_files(base, &path, out);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("ink") {
            continue;
        }

        let relative = path.strip_prefix(base).unwrap_or_else(|_| {
            panic!(
                "fixture path {} must be inside {}",
                path.display(),
                base.display()
            )
        });
        out.push(relative.to_string_lossy().replace('\\', "/"));
    }
}

pub fn get_ink_string(filename: &str) -> String {
    let path = ink_test::fixture_root().join("conformance").join(filename);
    fs::read_to_string(path).expect("fixture ink must exist")
}

fn get_fixture_json_string(filename: &str) -> String {
    let filename = if let Some(prefix) = filename.strip_suffix(".ink") {
        format!("{prefix}.ink.json")
    } else {
        format!("{filename}.ink.json")
    };
    let path = ink_test::fixture_root().join("conformance").join(filename);
    strip_utf8_bom(fs::read_to_string(path).expect("fixture json must exist"))
}

fn parse_json_value(filename: &str, json: &str, label: &str) -> Value {
    serde_json::from_str(json).unwrap_or_else(|error| {
        panic!("compiler_conformance {label} json for {filename} must be valid json: {error}")
    })
}

fn strip_utf8_bom(text: String) -> String {
    if let Some(stripped) = text.strip_prefix('\u{feff}') {
        stripped.to_string()
    } else {
        text
    }
}

fn format_compiled_json_mismatch(
    filename: &str,
    generated_value: &Value,
    fixture_value: &Value,
) -> String {
    let fixture_path = if let Some(prefix) = filename.strip_suffix(".ink") {
        format!("fixtures/conformance/{prefix}.ink.json")
    } else {
        format!("fixtures/conformance/{filename}.ink.json")
    };

    format!(
        "compiled json differs for {filename}\n--- generated ---\n{}\n--- fixture ({fixture_path}) ---\n{}",
        serde_json::to_string_pretty(generated_value).unwrap_or_else(|_| generated_value.to_string()),
        serde_json::to_string_pretty(fixture_value).unwrap_or_else(|_| fixture_value.to_string())
    )
}
