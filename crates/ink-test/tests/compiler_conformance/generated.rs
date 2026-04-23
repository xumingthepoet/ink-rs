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
fixture!(divert_invisible_divert, "inkfiles/divert/invisible-divert.ink");
fixture!(divert_complex_branching, "inkfiles/divert/complex-branching.ink");
fixture!(divert_on_choice, "inkfiles/divert/divert-on-choice.ink");
fixture!(divert_simple_divert, "inkfiles/divert/simple-divert.ink");
fixture!(glue_glue_with_divert, "inkfiles/glue/glue-with-divert.ink");
fixture!(glue_simple_glue, "inkfiles/glue/simple-glue.ink");
fixture!(runtime_jump_knot, "inkfiles/runtime/jump-knot.ink");
fixture!(runtime_load_save, "inkfiles/runtime/load-save.ink");
fixture!(runtime_saving_loading, "inkfiles/runtime/saving-loading.ink");
fixture!(
    runtime_multiflow_basics,
    "inkfiles/runtime/multiflow-basics.ink"
);
fixture!(runtime_jump_stitch, "inkfiles/runtime/jump-stitch.ink");
fixture!(stitch_auto_stitch, "inkfiles/stitch/auto-stitch.ink");
fixture!(stitch_manual_stitch, "inkfiles/stitch/manual-stitch.ink");
fixture!(choices_varying_choice, "inkfiles/choices/varying-choice.ink");
fixture!(choices_sticky_choice, "inkfiles/choices/sticky-choice.ink");
fixture!(choices_fallback_choice, "inkfiles/choices/fallback-choice.ink");
fixture!(tags_tags_in_choice, "inkfiles/tags/tagsInChoice.ink");
fixture!(
    choices_conditional_choice,
    "inkfiles/choices/conditional-choice.ink"
);
fixture!(variabletext_cycle, "inkfiles/variabletext/cycle.ink");
