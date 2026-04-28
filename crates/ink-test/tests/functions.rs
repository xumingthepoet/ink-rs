mod support;

use support::{runtime::StoryError, story_runner as common};

#[test]
fn fun_basic_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/func-basic.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value of x is 4.4.", text[0]);

    Ok(())
}

#[test]
fn fun_none_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/func-none.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value of x is 3.8.", text[0]);

    Ok(())
}

#[test]
fn fun_inline_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/func-inline.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value of x is 4.4.", text[0]);

    Ok(())
}

#[test]
fn setvar_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/setvar-func.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The value is 6.", text[0]);

    Ok(())
}

#[test]
fn complex_func1_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/complex-func1.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The values are 6 and 10.", text[0]);

    Ok(())
}

#[test]
fn complex_func2_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/complex-func2.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The values are -1 and 0 and 1.", text[0]);

    Ok(())
}

#[test]
fn complex_func3_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/complex-func3.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("\"I will pay you 120 reales if you get the goods to their destination. The goods will take up 20 cargo spaces.\"",
    text[0]);

    Ok(())
}

#[test]
fn rnd() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("functions/rnd-func.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(4, text.len());
    assert_eq!("Rolling dice 1: 6.", text[0]);
    assert_eq!("Rolling dice 2: 6.", text[1]);
    assert_eq!("Rolling dice 3: 4.", text[2]);
    assert_eq!("Rolling dice 4: 2.", text[3]);

    Ok(())
}
