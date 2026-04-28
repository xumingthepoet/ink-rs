use std::{cell::RefCell, error::Error, rc::Rc};

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
