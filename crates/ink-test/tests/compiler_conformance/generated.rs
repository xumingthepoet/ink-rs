use crate::compiler_conformance::common;

fn assert_fixture(filename: &str) {
    common::assert_compiled_json_matches_fixture(filename);
}

#[test]
fn theintercept() {
    assert_fixture("inkfiles/TheIntercept.ink");
}

#[test]
fn basictext_oneline() {
    assert_fixture("inkfiles/basictext/oneline.ink");
}

#[test]
fn basictext_twolines() {
    assert_fixture("inkfiles/basictext/twolines.ink");
}

#[test]
fn choices_conditional_choice() {
    assert_fixture("inkfiles/choices/conditional-choice.ink");
}

#[test]
fn choices_divert_choice() {
    assert_fixture("inkfiles/choices/divert-choice.ink");
}

#[test]
fn choices_fallback_choice() {
    assert_fixture("inkfiles/choices/fallback-choice.ink");
}

#[test]
fn choices_label_flow() {
    assert_fixture("inkfiles/choices/label-flow.ink");
}

#[test]
fn choices_label_scope_error() {
    assert_fixture("inkfiles/choices/label-scope-error.ink");
}

#[test]
fn choices_label_scope() {
    assert_fixture("inkfiles/choices/label-scope.ink");
}

#[test]
fn choices_mixed_choice() {
    assert_fixture("inkfiles/choices/mixed-choice.ink");
}

#[test]
fn choices_multi_choice() {
    assert_fixture("inkfiles/choices/multi-choice.ink");
}

#[test]
fn choices_no_choice_text() {
    assert_fixture("inkfiles/choices/no-choice-text.ink");
}

#[test]
fn choices_one() {
    assert_fixture("inkfiles/choices/one.ink");
}

#[test]
fn choices_single_choice() {
    assert_fixture("inkfiles/choices/single-choice.ink");
}

#[test]
fn choices_sticky_choice() {
    assert_fixture("inkfiles/choices/sticky-choice.ink");
}

#[test]
fn choices_suppress_choice() {
    assert_fixture("inkfiles/choices/suppress-choice.ink");
}

#[test]
fn choices_varying_choice() {
    assert_fixture("inkfiles/choices/varying-choice.ink");
}

#[test]
fn conditional_condopt() {
    assert_fixture("inkfiles/conditional/condopt.ink");
}

#[test]
fn conditional_condtext() {
    assert_fixture("inkfiles/conditional/condtext.ink");
}

#[test]
fn conditional_cycle() {
    assert_fixture("inkfiles/conditional/cycle.ink");
}

#[test]
fn conditional_ifelse_ext_text1() {
    assert_fixture("inkfiles/conditional/ifelse-ext-text1.ink");
}

#[test]
fn conditional_ifelse_ext_text2() {
    assert_fixture("inkfiles/conditional/ifelse-ext-text2.ink");
}

#[test]
fn conditional_ifelse_ext_text3() {
    assert_fixture("inkfiles/conditional/ifelse-ext-text3.ink");
}

#[test]
fn conditional_ifelse_ext() {
    assert_fixture("inkfiles/conditional/ifelse-ext.ink");
}

#[test]
fn conditional_ifelse() {
    assert_fixture("inkfiles/conditional/ifelse.ink");
}

#[test]
fn conditional_iffalse() {
    assert_fixture("inkfiles/conditional/iffalse.ink");
}

#[test]
fn conditional_iftrue() {
    assert_fixture("inkfiles/conditional/iftrue.ink");
}

#[test]
fn conditional_multiline_choice() {
    assert_fixture("inkfiles/conditional/multiline-choice.ink");
}

#[test]
fn conditional_multiline_divert() {
    assert_fixture("inkfiles/conditional/multiline-divert.ink");
}

#[test]
fn conditional_multiline() {
    assert_fixture("inkfiles/conditional/multiline.ink");
}

#[test]
fn conditional_once() {
    assert_fixture("inkfiles/conditional/once.ink");
}

#[test]
fn conditional_shuffle() {
    assert_fixture("inkfiles/conditional/shuffle.ink");
}

#[test]
fn conditional_shuffle_once() {
    assert_fixture("inkfiles/conditional/shuffle_once.ink");
}

#[test]
fn conditional_shuffle_stopping() {
    assert_fixture("inkfiles/conditional/shuffle_stopping.ink");
}

#[test]
fn conditional_stopping() {
    assert_fixture("inkfiles/conditional/stopping.ink");
}

#[test]
fn divert_complex_branching() {
    assert_fixture("inkfiles/divert/complex-branching.ink");
}

#[test]
fn divert_divert_on_choice() {
    assert_fixture("inkfiles/divert/divert-on-choice.ink");
}

#[test]
fn divert_invisible_divert() {
    assert_fixture("inkfiles/divert/invisible-divert.ink");
}

#[test]
fn divert_simple_divert() {
    assert_fixture("inkfiles/divert/simple-divert.ink");
}

#[test]
fn function_complex_func1() {
    assert_fixture("inkfiles/function/complex-func1.ink");
}

#[test]
fn function_complex_func2() {
    assert_fixture("inkfiles/function/complex-func2.ink");
}

#[test]
fn function_complex_func3() {
    assert_fixture("inkfiles/function/complex-func3.ink");
}

#[test]
fn function_evaluating_function_variablestate_bug() {
    assert_fixture("inkfiles/function/evaluating-function-variablestate-bug.ink");
}

#[test]
fn function_func_basic() {
    assert_fixture("inkfiles/function/func-basic.ink");
}

#[test]
fn function_func_inline() {
    assert_fixture("inkfiles/function/func-inline.ink");
}

#[test]
fn function_func_none() {
    assert_fixture("inkfiles/function/func-none.ink");
}

#[test]
fn function_rnd_func() {
    assert_fixture("inkfiles/function/rnd-func.ink");
}

#[test]
fn function_setvar_func() {
    assert_fixture("inkfiles/function/setvar-func.ink");
}

#[test]
fn function_test_error() {
    assert_fixture("inkfiles/function/test-error.ink");
}

#[test]
fn gather_complex_flow() {
    assert_fixture("inkfiles/gather/complex-flow.ink");
}

#[test]
fn gather_deep_nesting() {
    assert_fixture("inkfiles/gather/deep-nesting.ink");
}

#[test]
fn gather_gather_basic() {
    assert_fixture("inkfiles/gather/gather-basic.ink");
}

#[test]
fn gather_gather_chain() {
    assert_fixture("inkfiles/gather/gather-chain.ink");
}

#[test]
fn gather_nested_flow() {
    assert_fixture("inkfiles/gather/nested-flow.ink");
}

#[test]
fn gather_nested_gather() {
    assert_fixture("inkfiles/gather/nested-gather.ink");
}

#[test]
fn glue_glue_with_divert() {
    assert_fixture("inkfiles/glue/glue-with-divert.ink");
}

#[test]
fn glue_left_right_glue_matching() {
    assert_fixture("inkfiles/glue/left-right-glue-matching.ink");
}

#[test]
fn glue_simple_glue() {
    assert_fixture("inkfiles/glue/simple-glue.ink");
}

#[test]
fn glue_testbugfix1() {
    assert_fixture("inkfiles/glue/testbugfix1.ink");
}

#[test]
fn glue_testbugfix2() {
    assert_fixture("inkfiles/glue/testbugfix2.ink");
}

#[test]
fn knot_multi_line() {
    assert_fixture("inkfiles/knot/multi-line.ink");
}

#[test]
fn knot_param_floats() {
    assert_fixture("inkfiles/knot/param-floats.ink");
}

#[test]
fn knot_param_ints() {
    assert_fixture("inkfiles/knot/param-ints.ink");
}

#[test]
fn knot_param_multi() {
    assert_fixture("inkfiles/knot/param-multi.ink");
}

#[test]
fn knot_param_recurse() {
    assert_fixture("inkfiles/knot/param-recurse.ink");
}

#[test]
fn knot_param_strings() {
    assert_fixture("inkfiles/knot/param-strings.ink");
}

#[test]
fn knot_param_vars() {
    assert_fixture("inkfiles/knot/param-vars.ink");
}

#[test]
fn knot_single_line() {
    assert_fixture("inkfiles/knot/single-line.ink");
}

#[test]
fn knot_strip_empty_lines() {
    assert_fixture("inkfiles/knot/strip-empty-lines.ink");
}

#[test]
fn lists_basic_operations() {
    assert_fixture("inkfiles/lists/basic-operations.ink");
}

#[test]
fn lists_bug_adding_element() {
    assert_fixture("inkfiles/lists/bug-adding-element.ink");
}

#[test]
fn lists_empty_list_origin_after_assignment() {
    assert_fixture("inkfiles/lists/empty-list-origin-after-assignment.ink");
}

#[test]
fn lists_empty_list_origin() {
    assert_fixture("inkfiles/lists/empty-list-origin.ink");
}

#[test]
fn lists_list_all() {
    assert_fixture("inkfiles/lists/list-all.ink");
}

#[test]
fn lists_list_comparison() {
    assert_fixture("inkfiles/lists/list-comparison.ink");
}

#[test]
fn lists_list_mixed_items() {
    assert_fixture("inkfiles/lists/list-mixed-items.ink");
}

#[test]
fn lists_list_range() {
    assert_fixture("inkfiles/lists/list-range.ink");
}

#[test]
fn lists_list_save_load() {
    assert_fixture("inkfiles/lists/list-save-load.ink");
}

#[test]
fn lists_more_list_operations() {
    assert_fixture("inkfiles/lists/more-list-operations.ink");
}

#[test]
fn lists_more_list_operations2() {
    assert_fixture("inkfiles/lists/more-list-operations2.ink");
}

#[test]
fn misc_i18n() {
    assert_fixture("inkfiles/misc/i18n.ink");
}

#[test]
fn misc_issue15() {
    assert_fixture("inkfiles/misc/issue15.ink");
}

#[test]
fn misc_newlines_with_string_eval() {
    assert_fixture("inkfiles/misc/newlines_with_string_eval.ink");
}

#[test]
fn misc_operations() {
    assert_fixture("inkfiles/misc/operations.ink");
}

#[test]
fn misc_read_counts() {
    assert_fixture("inkfiles/misc/read-counts.ink");
}

#[test]
fn misc_turns_since() {
    assert_fixture("inkfiles/misc/turns-since.ink");
}

#[test]
fn runtime_external_function_0_arg() {
    assert_fixture("inkfiles/runtime/external-function-0-arg.ink");
}

#[test]
fn runtime_external_function_1_arg() {
    assert_fixture("inkfiles/runtime/external-function-1-arg.ink");
}

#[test]
fn runtime_external_function_2_arg() {
    assert_fixture("inkfiles/runtime/external-function-2-arg.ink");
}

#[test]
fn runtime_external_function_3_arg() {
    assert_fixture("inkfiles/runtime/external-function-3-arg.ink");
}

#[test]
fn runtime_jump_knot() {
    assert_fixture("inkfiles/runtime/jump-knot.ink");
}

#[test]
fn runtime_jump_stitch() {
    assert_fixture("inkfiles/runtime/jump-stitch.ink");
}

#[test]
fn runtime_load_save() {
    assert_fixture("inkfiles/runtime/load-save.ink");
}

#[test]
fn runtime_multiflow_basics() {
    assert_fixture("inkfiles/runtime/multiflow-basics.ink");
}

#[test]
fn runtime_multiflow_saveloadthreads() {
    assert_fixture("inkfiles/runtime/multiflow-saveloadthreads.ink");
}

#[test]
fn runtime_read_visit_counts() {
    assert_fixture("inkfiles/runtime/read-visit-counts.ink");
}

#[test]
fn runtime_saving_loading() {
    assert_fixture("inkfiles/runtime/saving-loading.ink");
}

#[test]
fn runtime_set_get_variables() {
    assert_fixture("inkfiles/runtime/set-get-variables.ink");
}

#[test]
fn runtime_variable_observers() {
    assert_fixture("inkfiles/runtime/variable-observers.ink");
}

#[test]
fn stitch_auto_stitch() {
    assert_fixture("inkfiles/stitch/auto-stitch.ink");
}

#[test]
fn stitch_manual_stitch() {
    assert_fixture("inkfiles/stitch/manual-stitch.ink");
}

#[test]
fn tags_tags() {
    assert_fixture("inkfiles/tags/tags.ink");
}

#[test]
fn tags_tagsdynamiccontent() {
    assert_fixture("inkfiles/tags/tagsDynamicContent.ink");
}

#[test]
fn tags_tagsinchoice() {
    assert_fixture("inkfiles/tags/tagsInChoice.ink");
}

#[test]
fn tags_tagsinchoicedynamic() {
    assert_fixture("inkfiles/tags/tagsInChoiceDynamic.ink");
}

#[test]
fn tags_tagsinseq() {
    assert_fixture("inkfiles/tags/tagsInSeq.ink");
}

#[test]
fn test1() {
    assert_fixture("inkfiles/test1.ink");
}

#[test]
fn threads_thread_bug() {
    assert_fixture("inkfiles/threads/thread-bug.ink");
}

#[test]
fn tunnels_tunnel_onwards_divert_override() {
    assert_fixture("inkfiles/tunnels/tunnel-onwards-divert-override.ink");
}

#[test]
fn variable_var_divert() {
    assert_fixture("inkfiles/variable/var-divert.ink");
}

#[test]
fn variable_varcalc() {
    assert_fixture("inkfiles/variable/varcalc.ink");
}

#[test]
fn variable_variable_declaration() {
    assert_fixture("inkfiles/variable/variable-declaration.ink");
}

#[test]
fn variable_varstringinc() {
    assert_fixture("inkfiles/variable/varstringinc.ink");
}

#[test]
fn variabletext_cycle() {
    assert_fixture("inkfiles/variabletext/cycle.ink");
}

#[test]
fn variabletext_empty_elements() {
    assert_fixture("inkfiles/variabletext/empty-elements.ink");
}

#[test]
fn variabletext_list_in_choice() {
    assert_fixture("inkfiles/variabletext/list-in-choice.ink");
}

#[test]
fn variabletext_once() {
    assert_fixture("inkfiles/variabletext/once.ink");
}

#[test]
fn variabletext_sequence() {
    assert_fixture("inkfiles/variabletext/sequence.ink");
}

