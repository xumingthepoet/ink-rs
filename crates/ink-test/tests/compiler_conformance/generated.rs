use crate::compiler_conformance::common;

macro_rules! fixture {
    ($name:ident, $path:literal) => {
        #[test]
        fn $name() {
            common::assert_parse_and_json_match_fixture($path);
        }
    };
}

macro_rules! fixture_count_all_visits {
    ($name:ident, $path:literal) => {
        #[test]
        fn $name() {
            common::assert_parse_and_json_match_count_all_visits_fixture($path);
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
fixture!(variabletext_once, "inkfiles/variabletext/once.ink");
fixture!(variabletext_sequence, "inkfiles/variabletext/sequence.ink");
fixture!(
    variabletext_empty_elements,
    "inkfiles/variabletext/empty-elements.ink"
);
fixture!(
    variabletext_list_in_choice,
    "inkfiles/variabletext/list-in-choice.ink"
);
fixture!(choices_multi_choice, "inkfiles/choices/multi-choice.ink");
fixture!(gather_gather_basic, "inkfiles/gather/gather-basic.ink");
fixture!(test1, "inkfiles/test1.ink");
fixture!(gather_gather_chain, "inkfiles/gather/gather-chain.ink");
fixture!(gather_nested_flow, "inkfiles/gather/nested-flow.ink");
fixture!(gather_nested_gather, "inkfiles/gather/nested-gather.ink");
fixture!(gather_complex_flow, "inkfiles/gather/complex-flow.ink");
fixture!(gather_deep_nesting, "inkfiles/gather/deep-nesting.ink");
fixture!(choices_label_flow, "inkfiles/choices/label-flow.ink");
fixture!(
    choices_label_scope_error,
    "inkfiles/choices/label-scope-error.ink"
);
fixture!(choices_label_scope, "inkfiles/choices/label-scope.ink");
fixture!(choices_divert_choice, "inkfiles/choices/divert-choice.ink");
fixture!(tags_tags, "inkfiles/tags/tags.ink");
fixture!(variable_var_divert, "inkfiles/variable/var-divert.ink");
fixture!(
    tags_tags_in_choice_dynamic,
    "inkfiles/tags/tagsInChoiceDynamic.ink"
);
fixture!(
    variable_variable_declaration,
    "inkfiles/variable/variable-declaration.ink"
);
fixture!(
    runtime_variable_observers,
    "inkfiles/runtime/variable-observers.ink"
);
fixture!(variable_varstringinc, "inkfiles/variable/varstringinc.ink");
fixture!(tags_tags_dynamic_content, "inkfiles/tags/tagsDynamicContent.ink");
fixture!(variable_varcalc, "inkfiles/variable/varcalc.ink");
fixture!(function_rnd_func, "inkfiles/function/rnd-func.ink");
fixture_count_all_visits!(misc_operations, "inkfiles/misc/operations.ink");
fixture!(conditional_ifelse, "inkfiles/conditional/ifelse.ink");
fixture!(conditional_iffalse, "inkfiles/conditional/iffalse.ink");
fixture!(conditional_iftrue, "inkfiles/conditional/iftrue.ink");
fixture!(function_test_error, "inkfiles/function/test-error.ink");
