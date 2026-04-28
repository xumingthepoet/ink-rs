mod support;

use support::compiler::compile_fixture_to_story;
use support::{runtime::StoryError, story_runner as common};

#[test]
fn tags_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("tags/tags.ink");

    let global_tags = story.get_global_tags();
    assert_eq!(0, global_tags.len());

    assert_eq!("This is the content\n", story.cont());

    let current_tags = story.get_current_tags();
    assert_eq!(2, current_tags.len());
    assert_eq!("author: Joe", current_tags[0]);
    assert_eq!("title: My Great Story", current_tags[1]);

    story.choose_path_string("game.knot", false, None);
    assert_eq!("Knot content\n", story.cont());
    let current_tags = story.get_current_tags();
    assert_eq!(1, current_tags.len());
    assert_eq!("knot tag", current_tags[0]);

    assert_eq!("", story.cont());
    let current_tags = story.get_current_tags();
    assert_eq!("end of knot tag", current_tags[0]);

    Ok(())
}

#[test]
fn tags_in_choice_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("tags/tagsInChoice.ink");

    story.cont();
    let current_tags = story.get_current_tags();
    assert_eq!(0, current_tags.len());
    assert_eq!(1, story.get_current_choices().len());
    assert_eq!(2, story.get_current_choices()[0].tags.len());
    assert_eq!("one", story.get_current_choices()[0].tags[0]);
    assert_eq!("two", story.get_current_choices()[0].tags[1]);

    story.choose_choice_index(0);

    assert_eq!("one three\n", story.cont());
    let current_tags = story.get_current_tags();
    assert_eq!(2, current_tags.len());
    assert_eq!("one", current_tags[0]);
    assert_eq!("three", current_tags[1]);

    Ok(())
}

#[test]
fn tags_in_choice_dynamic_content_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("tags/tagsInChoiceDynamic.ink");

    story.cont(); // Avanzar una vez
    let current_tags = story.get_current_tags();
    assert_eq!(0, current_tags.len());

    let choices = story.get_current_choices();
    assert_eq!(3, choices.len());

    assert_eq!(1, choices[0].tags.len());
    assert_eq!("tag Name", choices[0].tags[0]);

    assert_eq!(1, choices[1].tags.len());
    assert_eq!("tag 1 Name 2 3 4", choices[1].tags[0]);

    assert_eq!(1, choices[2].tags.len());
    assert_eq!("Name tag 1 2 3 4", choices[2].tags[0]);

    Ok(())
}

#[test]
fn tags_are_included_with_lines_and_choices() {
    let mut story = compile_fixture_to_story("tags/tags-are-included-with-lines-and-choices.ink");

    assert_eq!("SCOTLAND, EURO FEDERATION\n", story.cont());
    assert_eq!(
        vec!["location card".to_string(), "europe".to_string()],
        story.get_current_tags()
    );
    assert_eq!("An old castle heaves in front of you.\n", story.cont());
    assert_eq!(vec!["description".to_string()], story.get_current_tags());
    assert_eq!("", story.cont_maximally());
    let choices = story.get_current_choices();
    assert_eq!("Enter it.", choices[0].text);
    assert_eq!(vec!["action".to_string()], choices[0].tags);
}
