use std::{cell::RefCell, collections::BTreeMap, error::Error, rc::Rc};

mod support;

use support::{
    compiler::compile_fixture,
    runtime::{ExternalFunction, Story, ValueType},
    story_runner as common,
};

struct ExtFunc1;
struct ExtFunc2;
struct ExtFunc3;
struct ExtFunc4;
struct MultiplyExternal;

impl ExternalFunction for ExtFunc1 {
    fn call(&mut self, func_name: &str, args: Vec<ValueType>) -> Option<ValueType> {
        println!("Calling {func_name}...");

        let x = args[0].coerce_to_int().unwrap_or_default();
        let y = args[1].coerce_to_int().unwrap_or_default();

        Some(ValueType::Int(x - y))
    }
}

impl ExternalFunction for ExtFunc2 {
    fn call(&mut self, _: &str, _: Vec<ValueType>) -> Option<ValueType> {
        Some(ValueType::new::<&str>("Hello world"))
    }
}

impl ExternalFunction for ExtFunc3 {
    fn call(&mut self, _: &str, args: Vec<ValueType>) -> Option<ValueType> {
        Some(ValueType::Bool(args[0].get::<i32>().unwrap() != 1))
    }
}

impl ExternalFunction for ExtFunc4 {
    fn call(&mut self, _: &str, args: Vec<ValueType>) -> Option<ValueType> {
        Some(ValueType::Bool(!args[0].coerce_to_bool().unwrap()))
    }
}

impl ExternalFunction for MultiplyExternal {
    fn call(&mut self, _func_name: &str, args: Vec<ValueType>) -> Option<ValueType> {
        let left = args.first()?.get::<i32>()?;
        let right = args.get(1)?.get::<i32>()?;
        Some(ValueType::Int(left * right))
    }
}

#[test]
fn external_function() -> Result<(), Box<dyn Error>> {
    let mut story = common::story_from_fixture("runtime_api/external-function-2-arg.ink");
    let mut text: Vec<String> = Vec::new();

    story.bind_external_function(
        "game::externalFunction",
        Rc::new(RefCell::new(ExtFunc1 {})),
        true,
    );

    common::next_all(&mut story, &mut text);
    assert_eq!(1, text.len());
    assert_eq!("The value is -1.", text[0]);

    Ok(())
}

#[test]
fn external_function_zero_arguments() -> Result<(), Box<dyn Error>> {
    let mut story = common::story_from_fixture("runtime_api/external-function-0-arg.ink");
    let mut text: Vec<String> = Vec::new();

    story.bind_external_function(
        "game::externalFunction",
        Rc::new(RefCell::new(ExtFunc2 {})),
        true,
    );

    common::next_all(&mut story, &mut text);
    assert_eq!(1, text.len());
    assert_eq!("The value is Hello world.", text[0]);

    Ok(())
}

#[test]
fn external_function_one_arguments() -> Result<(), Box<dyn Error>> {
    let mut story = common::story_from_fixture("runtime_api/external-function-1-arg.ink");
    let mut text: Vec<String> = Vec::new();

    story.bind_external_function(
        "game::externalFunction",
        Rc::new(RefCell::new(ExtFunc3 {})),
        true,
    );

    common::next_all(&mut story, &mut text);
    assert_eq!(1, text.len());
    assert_eq!("The value is false.", text[0]);

    Ok(())
}

#[test]
fn external_function_coerce_test() -> Result<(), Box<dyn Error>> {
    let mut story = common::story_from_fixture("runtime_api/external-function-1-arg.ink");
    let mut text: Vec<String> = Vec::new();

    story.bind_external_function(
        "game::externalFunction",
        Rc::new(RefCell::new(ExtFunc4 {})),
        true,
    );

    common::next_all(&mut story, &mut text);
    assert_eq!(1, text.len());
    assert_eq!("The value is false.", text[0]);

    Ok(())
}

#[test]
fn set_and_get_variable_test() -> Result<(), Box<dyn Error>> {
    let mut story = common::story_from_fixture("runtime_api/set-get-variables.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    assert_eq!(
        10,
        story.get_variable("game::x").unwrap().get::<i32>().unwrap()
    );

    let _ = story.set_variable("game::x", &ValueType::Int(15));

    assert_eq!(
        15,
        story.get_variable("game::x").unwrap().get::<i32>().unwrap()
    );

    story.choose_choice_index(0);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("OK", text[0]);

    Ok(())
}

#[test]
fn set_non_existant_variable_test() -> Result<(), Box<dyn Error>> {
    let mut story = common::story_from_fixture("runtime_api/set-get-variables.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    let result = story.set_variable("y", &ValueType::new::<&str>("earth"));
    assert!(result.is_err());

    assert_eq!(
        10,
        story.get_variable("game::x").unwrap().get::<i32>().unwrap()
    );

    let _ = story.set_variable("game::x", &ValueType::Int(15));

    assert_eq!(
        15,
        story.get_variable("game::x").unwrap().get::<i32>().unwrap()
    );

    story.choose_choice_index(0);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("OK", text[0]);

    Ok(())
}

#[test]
fn jump_knot_test() -> Result<(), Box<dyn Error>> {
    let mut story = common::story_from_fixture("runtime_api/jump-knot.ink");
    let mut text: Vec<String> = Vec::new();

    story.choose_path_string("game.two", true, None);
    common::next_all(&mut story, &mut text);
    assert_eq!("Two", text.first().unwrap());

    text.clear();
    story.choose_path_string("game.three", true, None);
    common::next_all(&mut story, &mut text);
    assert_eq!("Three", text.first().unwrap());

    text.clear();
    story.choose_path_string("game.one", true, None);
    common::next_all(&mut story, &mut text);
    assert_eq!("One", text.first().unwrap());

    text.clear();
    story.choose_path_string("game.two", true, None);
    common::next_all(&mut story, &mut text);
    assert_eq!("Two", text.first().unwrap());

    Ok(())
}

#[test]
fn jump_stitch_test() -> Result<(), Box<dyn Error>> {
    let mut story = common::story_from_fixture("runtime_api/jump-stitch.ink");
    let mut text: Vec<String> = Vec::new();

    story.choose_path_string("game.two.sthree", true, None);
    common::next_all(&mut story, &mut text);
    assert_eq!("Two.3", text.first().unwrap());

    text.clear();
    story.choose_path_string("game.one.stwo", true, None);
    common::next_all(&mut story, &mut text);
    assert_eq!("One.2", text.first().unwrap());

    text.clear();
    story.choose_path_string("game.one.sone", true, None);
    common::next_all(&mut story, &mut text);
    assert_eq!("One.1", text.first().unwrap());

    text.clear();
    story.choose_path_string("game.two.stwo", true, None);
    common::next_all(&mut story, &mut text);
    assert_eq!("Two.2", text.first().unwrap());

    Ok(())
}

#[test]
fn load_save_test() -> Result<(), Box<dyn Error>> {
    let mut story = common::story_from_fixture("runtime_api/load-save.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    assert_eq!(1, text.len());
    assert_eq!(
        "We arrived into London at 9.45pm exactly.",
        text.first().unwrap()
    );

    // save the game state
    let save_string = story.save_state();

    println!("{}", save_string);

    // recreate game and load state
    let mut story = common::story_from_fixture("runtime_api/load-save.ink");
    story.load_state(&save_string);

    story.choose_choice_index(0);

    common::next_all(&mut story, &mut text);
    assert_eq!(
        "\"There is not a moment to lose!\" I declared.",
        text.get(1).unwrap()
    );
    assert_eq!(
        "We hurried home to Savile Row as fast as we could.",
        text.get(2).unwrap()
    );

    // check that we are at the end
    assert!(!story.can_continue());
    assert_eq!(0, story.get_current_choices().len());

    Ok(())
}

#[test]
fn external_binding_fixture_runs() {
    let compiled = compile_fixture("runtime_api/external-binding.ink");
    let mut story = Story::new(&compiled.json);

    story.bind_external_function(
        "game::multiply",
        Rc::new(RefCell::new(MultiplyExternal)),
        true,
    );

    assert_eq!(story.continue_maximally(), "42\n");
}

#[test]
fn variable_get_set_fixture_runs() {
    let compiled = compile_fixture("runtime_api/variable-get-set.ink");
    let mut story = Story::new(&compiled.json);

    assert_eq!(story.continue_maximally(), "5\n");
    story
        .set_variable("game::observed", &ValueType::Int(10))
        .expect("variable set should succeed");
    story.choose_choice_index(0);

    assert_eq!(story.continue_maximally(), "10\n");
}

#[test]
fn internal_function_metadata_is_emitted_for_host_calls() {
    let compiled = compile_fixture("runtime_api/internal-functions.ink");
    let json: serde_json::Value =
        serde_json::from_str(&compiled.json).expect("compiled JSON should parse");

    assert_eq!(
        json["internalFunctions"]["game::read_config"],
        serde_json::json!({
            "path": "game.read_config",
            "args": 0,
            "argTypes": [],
            "returnType": "string"
        })
    );
    assert_eq!(
        json["internalFunctions"]["game::double"],
        serde_json::json!({
            "path": "game.double",
            "args": 1,
            "argTypes": ["int"],
            "returnType": "int"
        })
    );
    assert_eq!(
        json["internalFunctions"]["host_config::read_unimported_config"],
        serde_json::json!({
            "path": "host_config.read_unimported_config",
            "args": 0,
            "argTypes": [],
            "returnType": "string"
        })
    );
}

#[test]
fn internal_host_calls_return_values_and_keep_side_effects() {
    let compiled = compile_fixture("runtime_api/internal-functions.ink");
    let mut story = Story::new(&compiled.json);

    let result = story
        .call_internal("game::read_config", None)
        .expect("internal call should succeed");

    assert!(matches!(
        result,
        Some(ValueType::String(value)) if value.string == "enabled"
    ));
    assert!(matches!(
        story.get_variable("game::counter"),
        Some(ValueType::Int(1))
    ));
}

#[test]
fn interface_values_save_as_existing_json_values_and_restore_defaults() {
    let compiled = compile_fixture("typed/interface-values.ink");

    let fresh = Story::new(&compiled.json);
    let fresh_save = fresh.save_state();
    let fresh_save_json: serde_json::Value =
        serde_json::from_str(&fresh_save).expect("fresh save should be JSON");
    assert_eq!(fresh_save_json["variablesState"], serde_json::json!({}));

    let mut story = Story::new(&compiled.json);
    story
        .set_variable("game::route", &ValueType::from("right"))
        .expect("interface global should be settable as string");
    story
        .set_variable(
            "game::routes",
            &ValueType::Array(vec![ValueType::from("right"), ValueType::from("left")]),
        )
        .expect("interface array global should be settable as string array");
    let mut config = BTreeMap::new();
    config.insert("route".to_string(), ValueType::from("left"));
    config.insert(
        "routes".to_string(),
        ValueType::Array(vec![ValueType::from("right")]),
    );
    story
        .set_variable("game::config", &ValueType::Object(config))
        .expect("struct with interface fields should be settable");

    let save = story.save_state();
    let save_json: serde_json::Value = serde_json::from_str(&save).expect("save should be JSON");
    assert_eq!(
        save_json["variablesState"]["game::route"],
        serde_json::json!("^right")
    );
    assert_eq!(
        save_json["variablesState"]["game::routes"],
        serde_json::json!(["^right", "^left"])
    );
    assert_eq!(
        save_json["variablesState"]["game::config"],
        serde_json::json!({
            "route": "^left",
            "routes": ["^right"]
        })
    );

    let mut reloaded = Story::new(&compiled.json);
    reloaded.load_state(&save);
    assert!(matches!(
        reloaded.get_variable("game::route"),
        Some(ValueType::String(value)) if value.string == "right"
    ));
    assert!(matches!(
        reloaded.get_variable("game::routes"),
        Some(ValueType::Array(values))
            if matches!(
                values.as_slice(),
                [ValueType::String(first), ValueType::String(second)]
                    if first.string == "right" && second.string == "left"
            )
    ));
    assert!(matches!(
        reloaded.get_variable("game::config"),
        Some(ValueType::Object(fields))
            if matches!(fields.get("route"), Some(ValueType::String(value)) if value.string == "left")
                && matches!(fields.get("routes"), Some(ValueType::Array(values))
                    if matches!(values.as_slice(), [ValueType::String(value)] if value.string == "right"))
    ));

    let mut reset = Story::new(&compiled.json);
    reset
        .set_variable("game::route", &ValueType::from("right"))
        .expect("interface global should be settable");
    reset.load_state(&fresh_save);
    assert!(matches!(
        reset.get_variable("game::route"),
        Some(ValueType::String(value)) if value.string == "left"
    ));
    assert!(matches!(
        reset.get_variable("game::routes"),
        Some(ValueType::Array(values))
            if matches!(
                values.as_slice(),
                [ValueType::String(first), ValueType::String(second)]
                    if first.string == "left" && second.string == "right"
            )
    ));
    assert!(matches!(
        reset.get_variable("game::config"),
        Some(ValueType::Object(fields))
            if matches!(fields.get("route"), Some(ValueType::String(value)) if value.string == "right")
                && matches!(fields.get("routes"), Some(ValueType::Array(values))
                    if matches!(values.as_slice(), [ValueType::String(value)] if value.string == "left"))
    ));
}

#[test]
fn internal_host_calls_accept_typed_arguments_and_composite_returns() {
    let compiled = compile_fixture("runtime_api/internal-functions.ink");
    let mut story = Story::new(&compiled.json);

    assert!(matches!(
        story
            .call_internal("game::double", Some(vec![ValueType::Int(21)]))
            .expect("int call should succeed"),
        Some(ValueType::Int(42))
    ));

    let mut story = Story::new(&compiled.json);
    let scores = story
        .call_internal(
            "game::identity_scores",
            Some(vec![ValueType::Array(vec![
                ValueType::Int(4),
                ValueType::Int(5),
            ])]),
        )
        .expect("array call should succeed");
    assert!(matches!(
        scores,
        Some(ValueType::Array(values))
            if matches!(values.as_slice(), [ValueType::Int(4), ValueType::Int(5)])
    ));

    let mut story = Story::new(&compiled.json);
    let mut fields = BTreeMap::new();
    fields.insert("hp".to_string(), ValueType::Int(7));
    let player = story
        .call_internal(
            "game::identity_player",
            Some(vec![ValueType::Object(fields)]),
        )
        .expect("object call should succeed");
    assert!(matches!(
        player,
        Some(ValueType::Object(fields)) if matches!(fields.get("hp"), Some(ValueType::Int(7)))
    ));

    let mut story = Story::new(&compiled.json);
    let scores = story
        .call_internal("game::build_scores", Some(vec![ValueType::Int(6)]))
        .expect("array temp assignment should succeed");
    assert!(matches!(
        scores,
        Some(ValueType::Array(values))
            if matches!(values.as_slice(), [ValueType::Int(6), ValueType::Int(7)])
    ));

    let mut story = Story::new(&compiled.json);
    let player = story
        .call_internal("game::build_player", Some(vec![ValueType::Int(9)]))
        .expect("struct temp assignment should succeed");
    assert!(matches!(
        player,
        Some(ValueType::Object(fields)) if matches!(fields.get("hp"), Some(ValueType::Int(9)))
    ));
}

#[test]
fn internal_host_calls_unimported_internal_module_and_dependency() {
    let compiled = compile_fixture("runtime_api/internal-functions.ink");
    let mut story = Story::new(&compiled.json);

    let result = story
        .call_internal("host_config::read_unimported_config", None)
        .expect("unimported internal module should be callable");

    assert!(matches!(
        result,
        Some(ValueType::String(value)) if value.string == "remote"
    ));
    assert!(matches!(
        story.get_variable("host_config::reads"),
        Some(ValueType::Int(1))
    ));
    assert!(story
        .call_internal("config_data::default_value", None)
        .is_err());
}

#[test]
fn internal_host_calls_reject_private_functions_bad_args_and_output_text() {
    let compiled = compile_fixture("runtime_api/internal-functions.ink");
    let mut story = Story::new(&compiled.json);

    assert!(story.call_internal("game::private_value", None).is_err());
    assert!(story
        .call_internal(
            "game::double",
            Some(vec![ValueType::Int(1), ValueType::Int(2)])
        )
        .is_err());
    assert!(story
        .call_internal("game::double", Some(vec![ValueType::new("bad")]))
        .is_err());
    assert!(story.call_internal("game::noisy", None).is_err());
}

#[test]
fn internal_host_calls_validate_return_metadata_at_runtime() {
    let compiled = compile_fixture("runtime_api/internal-functions.ink");
    let mut json: serde_json::Value =
        serde_json::from_str(&compiled.json).expect("compiled JSON should parse");
    json["internalFunctions"]["game::read_config"]["returnType"] = serde_json::json!("int");
    let mut story = Story::new(&json.to_string());

    let error = match story.call_internal("game::read_config", None) {
        Ok(_) => panic!("return type mismatch should fail"),
        Err(error) => error,
    };

    assert!(error
        .to_string()
        .contains("expected return type int, got string"));
}

#[test]
fn internal_host_calls_survive_save_load() {
    let compiled = compile_fixture("runtime_api/internal-functions.ink");
    let mut story = Story::new(&compiled.json);
    story
        .call_internal("game::read_config", None)
        .expect("internal call should succeed");
    let save = story.save_state();
    let mut reloaded = Story::new(&compiled.json);
    reloaded.load_state(&save);

    let result = reloaded
        .call_internal("game::read_config", None)
        .expect("internal call after load should succeed");

    assert!(matches!(
        result,
        Some(ValueType::String(value)) if value.string == "enabled"
    ));
    assert!(matches!(
        reloaded.get_variable("game::counter"),
        Some(ValueType::Int(2))
    ));
}
