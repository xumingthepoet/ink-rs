#![allow(dead_code)]

use crate::compiler_conformance::api::Story;
use crate::compiler_conformance::parse_snapshot::render_story as render_parse_story;
use ink_compiler::{Compiler, CompilerOptions};
use std::time::Instant;
use std::{fs, path::Path};

use rand::Rng;
use serde_json::Value;

const SAFETY_STEP_LIMIT: usize = 512;
const STUCK_STEP_LIMIT: usize = 16;
const CONTINUE_SLICE_MILLIS: f32 = 50.0;

pub fn next_all(story: &mut Story, text: &mut Vec<String>) {
    let mut safety_steps = 0usize;
    let mut stuck_steps = 0usize;
    let mut last_snapshot = (String::new(), 0usize);
    while story.can_continue() {
        safety_steps += 1;
        if safety_steps > SAFETY_STEP_LIMIT {
            panic!("compiler_conformance story exceeded step limit in next_all");
        }
        let line = story.cont_async(CONTINUE_SLICE_MILLIS);
        print!("{line}");

        let current_snapshot = (story.get_current_text(), story.get_current_choices().len());
        if line.trim().is_empty() && current_snapshot == last_snapshot {
            stuck_steps += 1;
            if stuck_steps > STUCK_STEP_LIMIT {
                panic!(
                    "compiler_conformance story stopped making progress in next_all: text='{}' choices={}",
                    current_snapshot.0,
                    current_snapshot.1
                );
            }
        } else {
            stuck_steps = 0;
        }
        last_snapshot = current_snapshot;

        if !line.trim().is_empty() {
            text.push(line.trim().to_string());
        }
    }

    if !story.get_current_errors().is_empty() {
        panic!("{}", join_text(&story.get_current_errors()));
    }
}

pub fn join_text(text: &[String]) -> String {
    let mut sb = String::new();
    for s in text {
        sb.push_str(s);
    }
    sb
}

pub fn run_story(
    filename: &str,
    choice_list: Option<Vec<usize>>,
    errors: &mut Vec<String>,
) -> Vec<String> {
    let mut story = compile_story(filename);
    let mut text = Vec::new();
    let mut choice_list_index = 0;
    let mut rng = rand::rng();
    let mut safety_steps = 0usize;
    let mut stuck_steps = 0usize;
    let mut last_snapshot = (String::new(), 0usize);

    while story.can_continue() || !story.get_current_choices().is_empty() {
        safety_steps += 1;
        if safety_steps > SAFETY_STEP_LIMIT {
            panic!("compiler_conformance story exceeded step limit in run_story");
        }
        println!("{}", story.build_string_of_hierarchy());

        while story.can_continue() {
            let line = story.cont_async(CONTINUE_SLICE_MILLIS);
            print!("{}", line);
            text.push(line);

            let current_snapshot = (story.get_current_text(), story.get_current_choices().len());
            if current_snapshot == last_snapshot {
                stuck_steps += 1;
                if stuck_steps > STUCK_STEP_LIMIT {
                    panic!(
                        "compiler_conformance story stopped making progress in run_story: text='{}' choices={}",
                        current_snapshot.0,
                        current_snapshot.1
                    );
                }
            } else {
                stuck_steps = 0;
            }
            last_snapshot = current_snapshot;
        }

        if !story.get_current_errors().is_empty() {
            for error_msg in story.get_current_errors() {
                println!("{}", error_msg);
                errors.push(error_msg.to_string());
            }
        }

        let current_choices = story.get_current_choices();
        if !current_choices.is_empty() {
            let len = current_choices.len();

            for choice in current_choices {
                println!("{}", choice.text);
                text.push(format!("{}\n", choice.text));
            }

            if let Some(choice_list) = &choice_list {
                if choice_list_index < choice_list.len() {
                    story.choose_choice_index(choice_list[choice_list_index]);
                    choice_list_index += 1;
                } else {
                    let random_choice_index = rng.random_range(0..len);
                    story.choose_choice_index(random_choice_index);
                }
            } else {
                let random_choice_index = rng.random_range(0..len);
                story.choose_choice_index(random_choice_index);
            }
        }
    }

    text
}

pub fn compile_story(filename: &str) -> Story {
    let source = get_ink_string(filename);
    assert!(
        profile_step("compare_parsed_story", || {
            assert_parsed_story_matches_fixture(filename, &source)
        }),
        "parsed story differs for {filename}"
    );
    let compiled_story = profile_step("compile_story", || Story::new(&source));
    let mut compiled_story = compiled_story;
    assert!(
        profile_step("compare_compiled_json", || {
            assert_compiled_json_matches_fixture(filename, &mut compiled_story)
        }),
        "compiled json differs for {filename}"
    );
    compiled_story
}

fn profile_step<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = f();
    if std::env::var_os("INK_PROFILE_COMPILE_CONFORMANCE").is_some() {
        eprintln!("profile {label}: {:?}", start.elapsed());
    }
    result
}

pub fn get_ink_string(filename: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/conformance")
        .join(filename);
    fs::read_to_string(path).expect("fixture ink must exist")
}

fn get_fixture_json_string(filename: &str) -> String {
    let filename = if let Some(prefix) = filename.strip_suffix(".ink") {
        format!("{prefix}.ink.json")
    } else {
        format!("{filename}.ink.json")
    };
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/conformance")
        .join(filename);
    strip_utf8_bom(fs::read_to_string(path).expect("fixture json must exist"))
}

fn get_fixture_parse_string(filename: &str) -> String {
    let filename = if let Some(prefix) = filename.strip_suffix(".ink") {
        format!("{prefix}.ink.parse")
    } else {
        format!("{filename}.ink.parse")
    };
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/conformance")
        .join(filename);
    strip_utf8_bom(fs::read_to_string(path).expect("fixture parse must exist"))
}

pub fn assert_parsed_story_matches_fixture(filename: &str, source: &str) -> bool {
    let mut compiler = Compiler::new(
        source.to_string(),
        Some(CompilerOptions {
            source_filename: Some(filename.to_string()),
            ..Default::default()
        }),
    );
    let parse_result = compiler.parse();
    let Some(parsed_story) = parse_result.parsed_story else {
        panic!("parse should succeed: {:?}", parse_result.diagnostics);
    };

    let generated_parse = render_parse_story(&parsed_story);
    let fixture_parse = get_fixture_parse_string(filename);
    if generated_parse != fixture_parse {
        eprintln!(
            "{}",
            format_parsed_story_mismatch(filename, &generated_parse, &fixture_parse)
        );
        return false;
    }

    true
}

fn format_parsed_story_mismatch(
    filename: &str,
    generated_parse: &str,
    fixture_parse: &str,
) -> String {
    let fixture_path = if let Some(prefix) = filename.strip_suffix(".ink") {
        format!("fixtures/conformance/{prefix}.ink.parse")
    } else {
        format!("fixtures/conformance/{filename}.ink.parse")
    };

    format!(
        "parsed story differs for {filename}\n--- generated ---\n{generated_parse}\n--- fixture ({fixture_path}) ---\n{fixture_parse}"
    )
}

pub fn assert_compiled_json_matches_fixture(filename: &str, story: &mut Story) -> bool {
    let generated_json = story.to_json();
    let fixture_json = get_fixture_json_string(filename);
    let generated_value = parse_json_value(filename, &generated_json, "generated");
    let fixture_value = parse_json_value(filename, &fixture_json, "fixture");
    if generated_value != fixture_value {
        eprintln!(
            "{}",
            format_compiled_json_mismatch(filename, &generated_json, &fixture_json)
        );
        return false;
    }

    true
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
    generated_json: &str,
    fixture_json: &str,
) -> String {
    let fixture_path = if let Some(prefix) = filename.strip_suffix(".ink") {
        format!("fixtures/conformance/{prefix}.ink.json")
    } else {
        format!("fixtures/conformance/{filename}.ink.json")
    };

    format!(
        "compiled json differs for {filename}\n--- generated ---\n{generated_json}\n--- fixture ({fixture_path}) ---\n{fixture_json}"
    )
}

pub fn is_ended(story: &Story) -> bool {
    let mut story = story.clone();
    !story.can_continue() && story.get_current_choices().is_empty()
}
