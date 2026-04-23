use crate::compiler_conformance::common;

fn assert_fixtures_match(fixture_paths: Vec<String>) {
    for fixture in fixture_paths {
        common::assert_compiled_json_matches_fixture(&fixture);
    }
}

#[test]
fn compiler_conformance_top_level_fixtures_match() {
    assert_fixtures_match(common::top_level_fixture_paths());
}

#[test]
fn compiler_conformance_basictext_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("basictext"));
}

#[test]
fn compiler_conformance_choice_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("choices"));
}

#[test]
fn compiler_conformance_conditional_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("conditional"));
}

#[test]
fn compiler_conformance_divert_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("divert"));
}

#[test]
fn compiler_conformance_function_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("function"));
}

#[test]
fn compiler_conformance_gather_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("gather"));
}

#[test]
fn compiler_conformance_glue_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("glue"));
}

#[test]
fn compiler_conformance_knot_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("knot"));
}

#[test]
fn compiler_conformance_list_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("lists"));
}

#[test]
fn compiler_conformance_misc_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("misc"));
}

#[test]
fn compiler_conformance_runtime_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("runtime"));
}

#[test]
fn compiler_conformance_stitch_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("stitch"));
}

#[test]
fn compiler_conformance_tag_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("tags"));
}

#[test]
fn compiler_conformance_thread_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("threads"));
}

#[test]
fn compiler_conformance_tunnel_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("tunnels"));
}

#[test]
fn compiler_conformance_variable_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("variable"));
}

#[test]
fn compiler_conformance_variable_text_fixtures_match() {
    assert_fixtures_match(common::fixture_group_paths("variabletext"));
}
