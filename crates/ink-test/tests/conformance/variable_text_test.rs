use crate::conformance::api::{story::Story, story_error::StoryError};

use crate::conformance::common;

#[test]
fn sequence_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/variabletext/sequence.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life. \"Three!\"", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life. \"Two!\"", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life. \"One!\"", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!(
        "The radio hissed into life. There was the white noise racket of an explosion.",
        text[0]
    );

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!(
        "The radio hissed into life. There was the white noise racket of an explosion.",
        text[0]
    );

    Ok(())
}

#[test]
fn cycle_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/variabletext/cycle.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life. \"Three!\"", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life. \"Two!\"", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life. \"One!\"", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life. \"Three!\"", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life. \"Two!\"", text[0]);

    Ok(())
}

#[test]
fn once_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/variabletext/once.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life. \"Three!\"", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life. \"Two!\"", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life. \"One!\"", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life.", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life.", text[0]);

    Ok(())
}

#[test]
fn empty_elements_test() -> Result<(), StoryError> {
    let json_string = common::get_json_string("inkfiles/variabletext/empty-elements.ink.json");
    let mut story = Story::new(&json_string);
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life.", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life.", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("The radio hissed into life. \"One!\"", text[0]);

    Ok(())
}
