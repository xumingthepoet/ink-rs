use crate::compiler_conformance::api::{story::Story, story_error::StoryError};

use crate::compiler_conformance::common;

#[test]
fn variable_declaration_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/variable/variable-declaration.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!(
        "\"My name is Jean Passepartout, but my friend's call me Jackie. I'm 23 years old.\"",
        text[0]
    );

    Ok(())
}

#[test]
fn var_calc_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/variable/varcalc.ink");
    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The values are true and -1 and -6 and aa.", text[0]);

    Ok(())
}

#[test]
fn var_string_ink_bug_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/variable/varstringinc.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    assert_eq!("ab.", text[1]);

    Ok(())
}

#[test]
fn var_divert_test() -> Result<(), StoryError> {
    let mut story = common::compile_story("inkfiles/variable/var-divert.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(1);

    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("Everybody dies.", text[0]);

    Ok(())
}
