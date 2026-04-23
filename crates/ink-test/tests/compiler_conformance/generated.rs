use crate::compiler_conformance::common;

macro_rules! fixture {
    ($name:ident, $path:literal) => {
        #[test]
        fn $name() {
            common::assert_parse_and_json_match_fixture($path);
        }
    };
}

fixture!(basictext_oneline, "inkfiles/basictext/oneline.ink");
fixture!(basictext_twolines, "inkfiles/basictext/twolines.ink");
fixture!(knot_multi_line, "inkfiles/knot/multi-line.ink");
fixture!(knot_single_line, "inkfiles/knot/single-line.ink");
fixture!(knot_strip_empty_lines, "inkfiles/knot/strip-empty-lines.ink");
fixture!(choices_mixed_choice, "inkfiles/choices/mixed-choice.ink");
fixture!(choices_no_choice_text, "inkfiles/choices/no-choice-text.ink");
fixture!(choices_one, "inkfiles/choices/one.ink");
fixture!(choices_single_choice, "inkfiles/choices/single-choice.ink");
fixture!(choices_suppress_choice, "inkfiles/choices/suppress-choice.ink");
fixture!(divert_simple_divert, "inkfiles/divert/simple-divert.ink");
fixture!(runtime_jump_knot, "inkfiles/runtime/jump-knot.ink");
fixture!(
    runtime_multiflow_basics,
    "inkfiles/runtime/multiflow-basics.ink"
);
