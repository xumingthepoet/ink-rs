use crate::compiler_conformance::common;

fn assert_fixture(filename: &str) {
    common::assert_compiled_json_matches_fixture(filename);
}

#[test]
fn compiler_conformanceinkfiles_theintercept_ink() {
    assert_fixture("inkfiles/TheIntercept.ink");
}

#[test]
fn compiler_conformanceinkfiles_basictext_oneline_ink() {
    assert_fixture("inkfiles/basictext/oneline.ink");
}

#[test]
fn compiler_conformanceinkfiles_basictext_twolines_ink() {
    assert_fixture("inkfiles/basictext/twolines.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_conditional_choice_ink() {
    assert_fixture("inkfiles/choices/conditional-choice.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_divert_choice_ink() {
    assert_fixture("inkfiles/choices/divert-choice.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_fallback_choice_ink() {
    assert_fixture("inkfiles/choices/fallback-choice.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_label_flow_ink() {
    assert_fixture("inkfiles/choices/label-flow.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_label_scope_error_ink() {
    assert_fixture("inkfiles/choices/label-scope-error.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_label_scope_ink() {
    assert_fixture("inkfiles/choices/label-scope.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_mixed_choice_ink() {
    assert_fixture("inkfiles/choices/mixed-choice.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_multi_choice_ink() {
    assert_fixture("inkfiles/choices/multi-choice.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_no_choice_text_ink() {
    assert_fixture("inkfiles/choices/no-choice-text.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_one_ink() {
    assert_fixture("inkfiles/choices/one.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_single_choice_ink() {
    assert_fixture("inkfiles/choices/single-choice.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_sticky_choice_ink() {
    assert_fixture("inkfiles/choices/sticky-choice.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_suppress_choice_ink() {
    assert_fixture("inkfiles/choices/suppress-choice.ink");
}

#[test]
fn compiler_conformanceinkfiles_choices_varying_choice_ink() {
    assert_fixture("inkfiles/choices/varying-choice.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_condopt_ink() {
    assert_fixture("inkfiles/conditional/condopt.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_condtext_ink() {
    assert_fixture("inkfiles/conditional/condtext.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_cycle_ink() {
    assert_fixture("inkfiles/conditional/cycle.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_ifelse_ext_text1_ink() {
    assert_fixture("inkfiles/conditional/ifelse-ext-text1.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_ifelse_ext_text2_ink() {
    assert_fixture("inkfiles/conditional/ifelse-ext-text2.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_ifelse_ext_text3_ink() {
    assert_fixture("inkfiles/conditional/ifelse-ext-text3.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_ifelse_ext_ink() {
    assert_fixture("inkfiles/conditional/ifelse-ext.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_ifelse_ink() {
    assert_fixture("inkfiles/conditional/ifelse.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_iffalse_ink() {
    assert_fixture("inkfiles/conditional/iffalse.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_iftrue_ink() {
    assert_fixture("inkfiles/conditional/iftrue.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_multiline_choice_ink() {
    assert_fixture("inkfiles/conditional/multiline-choice.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_multiline_divert_ink() {
    assert_fixture("inkfiles/conditional/multiline-divert.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_multiline_ink() {
    assert_fixture("inkfiles/conditional/multiline.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_once_ink() {
    assert_fixture("inkfiles/conditional/once.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_shuffle_ink() {
    assert_fixture("inkfiles/conditional/shuffle.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_shuffle_once_ink() {
    assert_fixture("inkfiles/conditional/shuffle_once.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_shuffle_stopping_ink() {
    assert_fixture("inkfiles/conditional/shuffle_stopping.ink");
}

#[test]
fn compiler_conformanceinkfiles_conditional_stopping_ink() {
    assert_fixture("inkfiles/conditional/stopping.ink");
}

#[test]
fn compiler_conformanceinkfiles_divert_complex_branching_ink() {
    assert_fixture("inkfiles/divert/complex-branching.ink");
}

#[test]
fn compiler_conformanceinkfiles_divert_divert_on_choice_ink() {
    assert_fixture("inkfiles/divert/divert-on-choice.ink");
}

#[test]
fn compiler_conformanceinkfiles_divert_invisible_divert_ink() {
    assert_fixture("inkfiles/divert/invisible-divert.ink");
}

#[test]
fn compiler_conformanceinkfiles_divert_simple_divert_ink() {
    assert_fixture("inkfiles/divert/simple-divert.ink");
}

#[test]
fn compiler_conformanceinkfiles_function_complex_func1_ink() {
    assert_fixture("inkfiles/function/complex-func1.ink");
}

#[test]
fn compiler_conformanceinkfiles_function_complex_func2_ink() {
    assert_fixture("inkfiles/function/complex-func2.ink");
}

#[test]
fn compiler_conformanceinkfiles_function_complex_func3_ink() {
    assert_fixture("inkfiles/function/complex-func3.ink");
}

#[test]
fn compiler_conformanceinkfiles_function_evaluating_function_variablestate_bug_ink() {
    assert_fixture("inkfiles/function/evaluating-function-variablestate-bug.ink");
}

#[test]
fn compiler_conformanceinkfiles_function_func_basic_ink() {
    assert_fixture("inkfiles/function/func-basic.ink");
}

#[test]
fn compiler_conformanceinkfiles_function_func_inline_ink() {
    assert_fixture("inkfiles/function/func-inline.ink");
}

#[test]
fn compiler_conformanceinkfiles_function_func_none_ink() {
    assert_fixture("inkfiles/function/func-none.ink");
}

#[test]
fn compiler_conformanceinkfiles_function_rnd_func_ink() {
    assert_fixture("inkfiles/function/rnd-func.ink");
}

#[test]
fn compiler_conformanceinkfiles_function_setvar_func_ink() {
    assert_fixture("inkfiles/function/setvar-func.ink");
}

#[test]
fn compiler_conformanceinkfiles_function_test_error_ink() {
    assert_fixture("inkfiles/function/test-error.ink");
}

#[test]
fn compiler_conformanceinkfiles_gather_complex_flow_ink() {
    assert_fixture("inkfiles/gather/complex-flow.ink");
}

#[test]
fn compiler_conformanceinkfiles_gather_deep_nesting_ink() {
    assert_fixture("inkfiles/gather/deep-nesting.ink");
}

#[test]
fn compiler_conformanceinkfiles_gather_gather_basic_ink() {
    assert_fixture("inkfiles/gather/gather-basic.ink");
}

#[test]
fn compiler_conformanceinkfiles_gather_gather_chain_ink() {
    assert_fixture("inkfiles/gather/gather-chain.ink");
}

#[test]
fn compiler_conformanceinkfiles_gather_nested_flow_ink() {
    assert_fixture("inkfiles/gather/nested-flow.ink");
}

#[test]
fn compiler_conformanceinkfiles_gather_nested_gather_ink() {
    assert_fixture("inkfiles/gather/nested-gather.ink");
}

#[test]
fn compiler_conformanceinkfiles_glue_glue_with_divert_ink() {
    assert_fixture("inkfiles/glue/glue-with-divert.ink");
}

#[test]
fn compiler_conformanceinkfiles_glue_left_right_glue_matching_ink() {
    assert_fixture("inkfiles/glue/left-right-glue-matching.ink");
}

#[test]
fn compiler_conformanceinkfiles_glue_simple_glue_ink() {
    assert_fixture("inkfiles/glue/simple-glue.ink");
}

#[test]
fn compiler_conformanceinkfiles_glue_testbugfix1_ink() {
    assert_fixture("inkfiles/glue/testbugfix1.ink");
}

#[test]
fn compiler_conformanceinkfiles_glue_testbugfix2_ink() {
    assert_fixture("inkfiles/glue/testbugfix2.ink");
}

#[test]
fn compiler_conformanceinkfiles_knot_multi_line_ink() {
    assert_fixture("inkfiles/knot/multi-line.ink");
}

#[test]
fn compiler_conformanceinkfiles_knot_param_floats_ink() {
    assert_fixture("inkfiles/knot/param-floats.ink");
}

#[test]
fn compiler_conformanceinkfiles_knot_param_ints_ink() {
    assert_fixture("inkfiles/knot/param-ints.ink");
}

#[test]
fn compiler_conformanceinkfiles_knot_param_multi_ink() {
    assert_fixture("inkfiles/knot/param-multi.ink");
}

#[test]
fn compiler_conformanceinkfiles_knot_param_recurse_ink() {
    assert_fixture("inkfiles/knot/param-recurse.ink");
}

#[test]
fn compiler_conformanceinkfiles_knot_param_strings_ink() {
    assert_fixture("inkfiles/knot/param-strings.ink");
}

#[test]
fn compiler_conformanceinkfiles_knot_param_vars_ink() {
    assert_fixture("inkfiles/knot/param-vars.ink");
}

#[test]
fn compiler_conformanceinkfiles_knot_single_line_ink() {
    assert_fixture("inkfiles/knot/single-line.ink");
}

#[test]
fn compiler_conformanceinkfiles_knot_strip_empty_lines_ink() {
    assert_fixture("inkfiles/knot/strip-empty-lines.ink");
}

#[test]
fn compiler_conformanceinkfiles_lists_basic_operations_ink() {
    assert_fixture("inkfiles/lists/basic-operations.ink");
}

#[test]
fn compiler_conformanceinkfiles_lists_bug_adding_element_ink() {
    assert_fixture("inkfiles/lists/bug-adding-element.ink");
}

#[test]
fn compiler_conformanceinkfiles_lists_empty_list_origin_after_assignment_ink() {
    assert_fixture("inkfiles/lists/empty-list-origin-after-assignment.ink");
}

#[test]
fn compiler_conformanceinkfiles_lists_empty_list_origin_ink() {
    assert_fixture("inkfiles/lists/empty-list-origin.ink");
}

#[test]
fn compiler_conformanceinkfiles_lists_list_all_ink() {
    assert_fixture("inkfiles/lists/list-all.ink");
}

#[test]
fn compiler_conformanceinkfiles_lists_list_comparison_ink() {
    assert_fixture("inkfiles/lists/list-comparison.ink");
}

#[test]
fn compiler_conformanceinkfiles_lists_list_mixed_items_ink() {
    assert_fixture("inkfiles/lists/list-mixed-items.ink");
}

#[test]
fn compiler_conformanceinkfiles_lists_list_range_ink() {
    assert_fixture("inkfiles/lists/list-range.ink");
}

#[test]
fn compiler_conformanceinkfiles_lists_list_save_load_ink() {
    assert_fixture("inkfiles/lists/list-save-load.ink");
}

#[test]
fn compiler_conformanceinkfiles_lists_more_list_operations_ink() {
    assert_fixture("inkfiles/lists/more-list-operations.ink");
}

#[test]
fn compiler_conformanceinkfiles_lists_more_list_operations2_ink() {
    assert_fixture("inkfiles/lists/more-list-operations2.ink");
}

#[test]
fn compiler_conformanceinkfiles_misc_i18n_ink() {
    assert_fixture("inkfiles/misc/i18n.ink");
}

#[test]
fn compiler_conformanceinkfiles_misc_issue15_ink() {
    assert_fixture("inkfiles/misc/issue15.ink");
}

#[test]
fn compiler_conformanceinkfiles_misc_newlines_with_string_eval_ink() {
    assert_fixture("inkfiles/misc/newlines_with_string_eval.ink");
}

#[test]
fn compiler_conformanceinkfiles_misc_operations_ink() {
    assert_fixture("inkfiles/misc/operations.ink");
}

#[test]
fn compiler_conformanceinkfiles_misc_read_counts_ink() {
    assert_fixture("inkfiles/misc/read-counts.ink");
}

#[test]
fn compiler_conformanceinkfiles_misc_turns_since_ink() {
    assert_fixture("inkfiles/misc/turns-since.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_external_function_0_arg_ink() {
    assert_fixture("inkfiles/runtime/external-function-0-arg.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_external_function_1_arg_ink() {
    assert_fixture("inkfiles/runtime/external-function-1-arg.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_external_function_2_arg_ink() {
    assert_fixture("inkfiles/runtime/external-function-2-arg.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_external_function_3_arg_ink() {
    assert_fixture("inkfiles/runtime/external-function-3-arg.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_jump_knot_ink() {
    assert_fixture("inkfiles/runtime/jump-knot.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_jump_stitch_ink() {
    assert_fixture("inkfiles/runtime/jump-stitch.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_load_save_ink() {
    assert_fixture("inkfiles/runtime/load-save.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_multiflow_basics_ink() {
    assert_fixture("inkfiles/runtime/multiflow-basics.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_multiflow_saveloadthreads_ink() {
    assert_fixture("inkfiles/runtime/multiflow-saveloadthreads.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_read_visit_counts_ink() {
    assert_fixture("inkfiles/runtime/read-visit-counts.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_saving_loading_ink() {
    assert_fixture("inkfiles/runtime/saving-loading.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_set_get_variables_ink() {
    assert_fixture("inkfiles/runtime/set-get-variables.ink");
}

#[test]
fn compiler_conformanceinkfiles_runtime_variable_observers_ink() {
    assert_fixture("inkfiles/runtime/variable-observers.ink");
}

#[test]
fn compiler_conformanceinkfiles_stitch_auto_stitch_ink() {
    assert_fixture("inkfiles/stitch/auto-stitch.ink");
}

#[test]
fn compiler_conformanceinkfiles_stitch_manual_stitch_ink() {
    assert_fixture("inkfiles/stitch/manual-stitch.ink");
}

#[test]
fn compiler_conformanceinkfiles_tags_tags_ink() {
    assert_fixture("inkfiles/tags/tags.ink");
}

#[test]
fn compiler_conformanceinkfiles_tags_tagsdynamiccontent_ink() {
    assert_fixture("inkfiles/tags/tagsDynamicContent.ink");
}

#[test]
fn compiler_conformanceinkfiles_tags_tagsinchoice_ink() {
    assert_fixture("inkfiles/tags/tagsInChoice.ink");
}

#[test]
fn compiler_conformanceinkfiles_tags_tagsinchoicedynamic_ink() {
    assert_fixture("inkfiles/tags/tagsInChoiceDynamic.ink");
}

#[test]
fn compiler_conformanceinkfiles_tags_tagsinseq_ink() {
    assert_fixture("inkfiles/tags/tagsInSeq.ink");
}

#[test]
fn compiler_conformanceinkfiles_test1_ink() {
    assert_fixture("inkfiles/test1.ink");
}

#[test]
fn compiler_conformanceinkfiles_threads_thread_bug_ink() {
    assert_fixture("inkfiles/threads/thread-bug.ink");
}

#[test]
fn compiler_conformanceinkfiles_tunnels_tunnel_onwards_divert_override_ink() {
    assert_fixture("inkfiles/tunnels/tunnel-onwards-divert-override.ink");
}

#[test]
fn compiler_conformanceinkfiles_variable_var_divert_ink() {
    assert_fixture("inkfiles/variable/var-divert.ink");
}

#[test]
fn compiler_conformanceinkfiles_variable_varcalc_ink() {
    assert_fixture("inkfiles/variable/varcalc.ink");
}

#[test]
fn compiler_conformanceinkfiles_variable_variable_declaration_ink() {
    assert_fixture("inkfiles/variable/variable-declaration.ink");
}

#[test]
fn compiler_conformanceinkfiles_variable_varstringinc_ink() {
    assert_fixture("inkfiles/variable/varstringinc.ink");
}

#[test]
fn compiler_conformanceinkfiles_variabletext_cycle_ink() {
    assert_fixture("inkfiles/variabletext/cycle.ink");
}

#[test]
fn compiler_conformanceinkfiles_variabletext_empty_elements_ink() {
    assert_fixture("inkfiles/variabletext/empty-elements.ink");
}

#[test]
fn compiler_conformanceinkfiles_variabletext_list_in_choice_ink() {
    assert_fixture("inkfiles/variabletext/list-in-choice.ink");
}

#[test]
fn compiler_conformanceinkfiles_variabletext_once_ink() {
    assert_fixture("inkfiles/variabletext/once.ink");
}

#[test]
fn compiler_conformanceinkfiles_variabletext_sequence_ink() {
    assert_fixture("inkfiles/variabletext/sequence.ink");
}

