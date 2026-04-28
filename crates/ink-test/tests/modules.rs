use std::{cell::RefCell, rc::Rc};

mod support;

use support::{
    compiler::{assert_story_output, compile_fixture},
    runtime::{ExternalFunction, Story, ValueType},
};

#[test]
fn imported_global_variable_reads_and_writes_run() {
    let compiled = compile_fixture("modules/module-imported-global-vars.ink");

    assert_story_output(&compiled, "1\n3\n6\n");
}

#[test]
fn module_import_example_runs() {
    let compiled = compile_fixture("modules/docs-module-imports.ink");

    assert_story_output(
        &compiled,
        "The shop is open.\nGold: 5\nPrice: 3\nUpdated price: 5\n",
    );
}

#[test]
fn qualified_function_and_external_calls_run() {
    let compiled = compile_fixture("modules/module-qualified-calls.ink");

    let mut story = Story::new(&compiled.json);
    story.bind_external_function("audio::play", Rc::new(RefCell::new(ModuleExternal)), true);
    let output = story.continue_maximally();

    assert_eq!(output, "5\n7\n");
    assert!(
        story.get_current_errors().is_empty(),
        "story should not emit runtime errors: {:#?}",
        story.get_current_errors()
    );
}

#[test]
fn qualified_flow_paths_run_without_runtime_colon_separator() {
    let compiled = compile_fixture("modules/module-qualified-flow-paths.ink");

    assert_story_output(&compiled, "9\nTunnel.\nAfter tunnel.\nTarget.\n");
    assert!(
        !compiled.json.to_string().contains("::"),
        "runtime flow paths should use dot-separated container paths: {:#}",
        compiled.json
    );
}

#[test]
fn dynamic_qualified_divert_target_values_run() {
    let compiled = compile_fixture("modules/module-dynamic-qualified-divert-target.ink");

    assert_story_output(&compiled, "Dynamic target.\n");
    assert!(
        !compiled.json.to_string().contains("::"),
        "runtime divert target values should use dot-separated container paths: {:#}",
        compiled.json
    );
}

#[test]
fn same_module_stitch_paths_run() {
    let compiled = compile_fixture("modules/module-same-module-stitch-paths.ink");

    assert_story_output(&compiled, "Intro.\nOpen.\n");
}

#[test]
fn globals_and_externals_use_module_qualified_runtime_names() {
    let compiled = compile_fixture("modules/module-qualified-globals-and-externals.ink");

    let mut story = Story::new(&compiled.json);
    for name in ["audio::play", "video::play"] {
        story.bind_external_function(name, Rc::new(RefCell::new(ModuleExternal)), true);
    }
    let output = story.continue_maximally();

    assert_eq!(output, "7|8\nReady.\n");
    assert_eq!(
        story
            .get_variable("left::level")
            .and_then(|value| value.get::<i32>()),
        Some(11)
    );
    assert_eq!(
        story
            .get_variable("right::level")
            .and_then(|value| value.get::<i32>()),
        Some(22)
    );
    assert!(story.get_variable("level").is_none());

    let save_string = story.save_state();
    let save: serde_json::Value = serde_json::from_str(&save_string).expect("valid save JSON");
    assert_eq!(save["variablesState"]["left::level"], serde_json::json!(11));
    assert_eq!(
        save["variablesState"]["right::level"],
        serde_json::json!(22)
    );
    assert!(
        save["variablesState"].get("level").is_none(),
        "module globals should not save under unqualified names: {save:#}"
    );

    let mut reloaded = Story::new(&compiled.json);
    for name in ["audio::play", "video::play"] {
        reloaded.bind_external_function(name, Rc::new(RefCell::new(ModuleExternal)), true);
    }
    reloaded.load_state(&save_string);
    reloaded.choose_choice_index(0);
    assert_eq!(reloaded.continue_maximally(), "11|22\n");
}

struct ModuleExternal;

impl ExternalFunction for ModuleExternal {
    fn call(&mut self, func_name: &str, args: Vec<ValueType>) -> Option<ValueType> {
        match func_name {
            "audio::play" => {
                assert!(args == vec![ValueType::from("intro")]);
                Some(ValueType::Int(7))
            }
            "video::play" => {
                assert!(args == vec![ValueType::from("intro")]);
                Some(ValueType::Int(8))
            }
            _ => panic!("unexpected external function: {func_name}"),
        }
    }
}
