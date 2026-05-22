use crate::support::{compiler::compile_fixture_to_story_allowing_warnings, runtime::Story};

fn story_is_ended(story: &Story) -> bool {
    !story.can_continue() && story.get_current_choices().is_empty()
}

#[test]
fn root_flow_ends_naturally() {
    let mut story = compile_fixture_to_story_allowing_warnings("flow/natural-root-end.ink");

    assert_eq!("Root line.\n", story.cont_maximally());
    assert!(story_is_ended(&story));
}

#[test]
fn knot_ends_naturally_after_divert() {
    let mut story = compile_fixture_to_story_allowing_warnings("flow/natural-knot-end.ink");

    assert_eq!("Knot line.\n", story.cont_maximally());
    assert!(story_is_ended(&story));
}

#[test]
fn stitch_ends_naturally_after_divert() {
    let mut story = compile_fixture_to_story_allowing_warnings("flow/natural-stitch-end.ink");

    assert_eq!("Stitch line.\n", story.cont_maximally());
    assert!(story_is_ended(&story));
}

#[test]
fn unordered_knot_divert_story_runs_to_end() {
    let mut story = compile_fixture_to_story_allowing_warnings(
        "flow/unordered-knot-divert-story-runs-to-end.ink",
    );

    assert_eq!(
        "The detective arrives.\nThe witness points to the desk.\nThe letter is still on the desk.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn flat_story_comments_and_initial_divert_run_in_order() {
    let mut story = compile_fixture_to_story_allowing_warnings(
        "flow/flat-story-comments-and-initial-divert-run-in-order.ink",
    );

    assert_eq!(
        "The recorder starts.\nEvery switch is labeled.\nThe panel is already lit.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn stitch_shorthand_diverts_inside_a_knot() {
    let mut story = compile_fixture_to_story_allowing_warnings(
        "flow/stitch-shorthand-diverts-inside-a-knot.ink",
    );

    assert_eq!(
        "The map is unfolded.\nThe station clock rings.\nThe kettle whistles.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn story_can_be_read_with_unnamed_knots() {
    let mut story =
        compile_fixture_to_story_allowing_warnings("flow/story-can-be-read-with-unnamed-knots.ink");

    assert_eq!(
        "Mont Blanc was a world-renowned mountain guide.\nHe befriended thousands of climbers and children sightseeing in Switzerland.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn story_starts_from_top_even_if_knot_is_unnamed() {
    let mut story = compile_fixture_to_story_allowing_warnings(
        "flow/story-starts-from-top-even-if-knot-is-unnamed.ink",
    );

    assert_eq!(
        "Mont Blanc was a world-renowned mountain guide.\nHe befriended thousands of climbers and children sightseeing in Switzerland.\nGESICHT'S BEDROOM, MORNING\nGesicht is lying in his bed, eyes wide open and staring at the ceiling.\nHe just woke from a nightmare.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn story_can_start_with_named_knot() {
    let mut story =
        compile_fixture_to_story_allowing_warnings("flow/story-can-start-with-named-knot.ink");

    assert_eq!(
        "GESICHT'S BEDROOM (MORNING)\nGesicht is lying in his bed, eyes wide open and staring at the ceiling.\nHe just woke from a nightmare.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn story_can_divert_at_will_between_unordered_knots() {
    let mut story = compile_fixture_to_story_allowing_warnings(
        "flow/story-can-divert-at-will-between-unordered-knots.ink",
    );

    assert_eq!(
        "SCENE OF MURDER (TRASHED APARTMENT, DAYTIME)\nGesicht arrives at a grotesque murder scene.\nThe lead detective stop him as he enters the room.\nHe identifies himself as being from Europol and passes the barrier.\nThe body is lying face down in a pool of blood.\nA desk lamp and piece of broken wood have been stuck to his head.\nThey mimic the appearance of antlers.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn story_can_be_structured_using_stitches() {
    let mut story = compile_fixture_to_story_allowing_warnings(
        "flow/story-can-be-structured-using-stitches.ink",
    );

    assert_eq!(
        "Mont Blanc was a world-renowned mountain guide.\nHe befriended thousands of climbers and children sightseeing in Switzerland.\nGesicht is lying in his bed, eyes wide open and staring at the ceiling.\nHe just woke from a nightmare.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}

#[test]
fn stitches_can_be_diverted_to_inside_a_knot_without_full_address() {
    let mut story = compile_fixture_to_story_allowing_warnings(
        "flow/stitches-can-be-diverted-to-inside-a-knot-without-full-address.ink",
    );

    assert_eq!(
        "Mont Blanc was a world-renowned mountain guide.\nHe befriended thousands of climbers and children sightseeing in Switzerland.\nGesicht is lying in his bed, eyes wide open and staring at the ceiling.\nHe just woke from a nightmare.\nBefore he's had the time to eat breakfast a call about a murder comes in.\nAs Helena probes him about leaving he suggests that they take a vacation.\n",
        story.cont_maximally()
    );
    assert!(story_is_ended(&story));
}
