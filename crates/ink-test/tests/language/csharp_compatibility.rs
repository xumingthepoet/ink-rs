use std::{cell::RefCell, rc::Rc};

use ink_runtime::{
    story::{external_functions::ExternalFunction, Story},
    value_type::ValueType,
};

use super::{assert_story_output, compile_language_fixture};

#[test]
fn csharp_hello_world_example_runs_as_module_fixture() {
    let compiled = compile_language_fixture("csharp_compatibility/hello-world.ink");

    assert_story_output(&compiled, "Hello world\n");
}

#[test]
fn csharp_arithmetic_example_runs_as_module_fixture() {
    let compiled = compile_language_fixture("csharp_compatibility/arithmetic.ink");

    assert_story_output(&compiled, "36\n2\n3\n2\n2.3333333\n8\n8\n");
}

#[test]
fn csharp_simple_glue_example_runs_as_module_fixture() {
    let compiled = compile_language_fixture("csharp_compatibility/simple-glue.ink");

    assert_story_output(&compiled, "Some content with glue.\n");
}

#[test]
fn csharp_external_binding_example_runs_as_module_fixture() {
    let compiled = compile_language_fixture("csharp_compatibility/external-binding.ink");
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");

    story
        .bind_external_function(
            "game::multiply",
            Rc::new(RefCell::new(MultiplyExternal)),
            true,
        )
        .expect("external binding should succeed");

    assert_eq!(story.continue_maximally().unwrap(), "42\n");
}

#[test]
fn csharp_variable_get_set_example_runs_as_module_fixture() {
    let compiled = compile_language_fixture("csharp_compatibility/variable-get-set.ink");
    let mut story = Story::new(&compiled.json).expect("compiled JSON should load");

    assert_eq!(story.continue_maximally().unwrap(), "5\n");
    story
        .set_variable("game::observed", &ValueType::Int(10))
        .expect("variable set should succeed");
    story.choose_choice_index(0).unwrap();

    assert_eq!(story.continue_maximally().unwrap(), "10\n");
}

struct MultiplyExternal;

impl ExternalFunction for MultiplyExternal {
    fn call(&mut self, _func_name: &str, args: Vec<ValueType>) -> Option<ValueType> {
        let left = args.first()?.get::<i32>()?;
        let right = args.get(1)?.get::<i32>()?;
        Some(ValueType::Int(left * right))
    }
}
