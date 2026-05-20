use crate::support::compiler::{assert_story_output, compile_fixture};
use crate::support::{runtime::StoryError, story_runner as common};

#[test]
fn simple_glue_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("glue/simple-glue.ink");

    let mut text: Vec<String> = Vec::new();
    common::next_all(&mut story, &mut text);
    assert_eq!(1, text.len());
    assert_eq!("Some content with glue.", text[0]);

    Ok(())
}

#[test]
fn glue_with_divert_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("glue/glue-with-divert.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(1, text.len());
    assert_eq!(
        "We hurried home to Savile Row as fast as we could.",
        text[0]
    );

    Ok(())
}

#[test]
fn has_left_right_glue_matching_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("glue/left-right-glue-matching.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    assert_eq!("A line.", text[0]);
    assert_eq!("Another line.", text[1]);

    Ok(())
}

#[test]
fn bugfix1_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("glue/testbugfix1.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    assert_eq!("A", text[0]);
    assert_eq!("C", text[1]);

    Ok(())
}

#[test]
fn bugfix2_test() -> Result<(), StoryError> {
    let mut story = common::story_from_fixture("glue/testbugfix2.ink");
    let mut text: Vec<String> = Vec::new();

    common::next_all(&mut story, &mut text);

    assert_eq!(2, text.len());
    //assert_eq!("A", text[0]);
    assert_eq!("X", text[1]);

    Ok(())
}

#[test]
fn source_simple_glue_fixture_runs() {
    let compiled = compile_fixture("glue/simple-glue.ink");

    assert_story_output(&compiled, "Some content with glue.\n");
}

#[test]
fn inline_glue_binds_content_without_spacing_padding() {
    let compiled = compile_fixture("glue/inline-glue.ink");

    assert_story_output(&compiled, "Some content with glue.\n");
}

#[test]
fn diverts_are_glue_and_add_single_whitespace_after_story_text() {
    let compiled =
        compile_fixture("glue/diverts-are-glue-and-add-single-whitespace-after-story-text.ink");

    assert_story_output(
        &compiled,
        "“We all loved Mont Blanc.\n“The memorial service will take place in three days ...\n“This place will be filled with tens of thousands of people, probably several hundred thousands.\n“all mourning the death of mont blanc.”\n",
    );
}

#[test]
fn glue_binds_lines_together_without_newline_markers() {
    let compiled = compile_fixture("glue/glue-binds-lines-together-without-newline-markers.ink");

    assert_story_output(
        &compiled,
        "“So she abandoned me ... she sent me to a boarding school in England ... and I never heard a thing from her again.”\n",
    );
}

#[test]
fn glue_binds_across_diverts() {
    let compiled = compile_fixture("glue/glue-binds-across-diverts.ink");

    assert_story_output(
        &compiled,
        "“So she abandoned me ... she sent me to a boarding school in England ... and I never heard a thing from her again.”\n",
    );
}
