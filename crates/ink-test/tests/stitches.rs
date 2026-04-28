mod support;

use support::{runtime::StoryError, story_runner as common};

#[test]
fn auto_stitch_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("stitches/auto-stitch.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("I settled my master.", text[0]);

    Ok(())
}

#[test]
fn auto_stitch2_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("stitches/auto-stitch.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);
    story.choose_choice_index(1);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("I settled my master.", text[0]);

    Ok(())
}

#[test]
fn manual_stitch_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("stitches/manual-stitch.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("How shall we travel?", text[0]);

    story.choose_choice_index(1);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("I put myself in third.", text[0]);

    Ok(())
}

#[test]
fn manual_stitch2_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("stitches/manual-stitch.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("How shall we travel?", text[0]);

    story.choose_choice_index(0);
    text.clear();
    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!("I settled my master.", text[0]);

    Ok(())
}
