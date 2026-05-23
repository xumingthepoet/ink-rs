use crate::support::{runtime::StoryError, story_runner as common};

#[test]
fn choice_loop_retries_until_success() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("choice_loops/choice-loop.ink");
    println!("{}", story.build_string_of_hierarchy());

    assert_eq!(
        "Here is some gold. Do you want it?\n",
        story.continue_maximally()
    );
    assert_eq!(2, story.get_current_choices().len());
    assert_eq!("No", story.get_current_choices()[0].text);
    assert_eq!("Yes", story.get_current_choices()[1].text);
    story.choose_choice_index(0);

    assert_eq!("No\nTry again!\n", story.continue_maximally());
    assert_eq!(2, story.get_current_choices().len());
    assert_eq!("No", story.get_current_choices()[0].text);
    assert_eq!("Yes", story.get_current_choices()[1].text);
    story.choose_choice_index(1);

    assert_eq!("Yes\nYou win!\n", story.continue_maximally());

    Ok(())
}

#[test]
fn choice_loop_save_load_regenerates_current_choices() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("choice_loops/choice-loop.ink");
    println!("{}", story.build_string_of_hierarchy());

    assert_eq!(
        "Here is some gold. Do you want it?\n",
        story.continue_maximally()
    );
    assert_eq!(2, story.get_current_choices().len());
    assert_eq!("No", story.get_current_choices()[0].text);
    assert_eq!("Yes", story.get_current_choices()[1].text);

    let save_string = story.save_state();
    println!("{}", save_string);
    let mut story = common::story_from_fixture("choice_loops/choice-loop.ink");
    story.load_state(&save_string);

    assert_eq!("", story.continue_maximally());
    assert_eq!(2, story.get_current_choices().len());
    assert_eq!("No", story.get_current_choices()[0].text);
    assert_eq!("Yes", story.get_current_choices()[1].text);
    story.choose_choice_index(0);

    assert_eq!("No\nTry again!\n", story.continue_maximally());
    assert_eq!(2, story.get_current_choices().len());
    assert_eq!("No", story.get_current_choices()[0].text);
    assert_eq!("Yes", story.get_current_choices()[1].text);
    story.choose_choice_index(1);

    assert_eq!("Yes\nYou win!\n", story.continue_maximally());

    Ok(())
}
