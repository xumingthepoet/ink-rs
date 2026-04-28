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
fixture!(divert_invisible_divert, "inkfiles/divert/invisible-divert.ink");
fixture!(divert_simple_divert, "inkfiles/divert/simple-divert.ink");
fixture!(glue_glue_with_divert, "inkfiles/glue/glue-with-divert.ink");
fixture!(glue_simple_glue, "inkfiles/glue/simple-glue.ink");
fixture!(runtime_jump_knot, "inkfiles/runtime/jump-knot.ink");
fixture!(runtime_saving_loading, "inkfiles/runtime/saving-loading.ink");
fixture!(
    runtime_multiflow_basics,
    "inkfiles/runtime/multiflow-basics.ink"
);
fixture!(runtime_jump_stitch, "inkfiles/runtime/jump-stitch.ink");
fixture!(tags_tags, "inkfiles/tags/tags.ink");
fixture!(
    variable_variable_declaration,
    "inkfiles/variable/variable-declaration.ink"
);
fixture!(tags_tags_dynamic_content, "inkfiles/tags/tagsDynamicContent.ink");
fixture!(variable_varcalc, "inkfiles/variable/varcalc.ink");
fixture!(typed_array_literals, "inkfiles/typed/array-literals.ink");
fixture!(typed_struct_literals, "inkfiles/typed/struct-literals.ink");
fixture!(function_rnd_func, "inkfiles/function/rnd-func.ink");
fixture_count_all_visits!(misc_operations, "inkfiles/misc/operations.ink");
fixture!(conditional_ifelse, "inkfiles/conditional/ifelse.ink");
fixture!(conditional_iffalse, "inkfiles/conditional/iffalse.ink");
fixture!(conditional_iftrue, "inkfiles/conditional/iftrue.ink");
fixture!(function_test_error, "inkfiles/function/test-error.ink");
fixture_count_all_visits!(
    runtime_read_visit_counts,
    "inkfiles/runtime/read-visit-counts.ink"
);
fixture!(conditional_ifelse_ext, "inkfiles/conditional/ifelse-ext.ink");
fixture!(misc_issue15, "inkfiles/misc/issue15.ink");
fixture!(knot_param_recurse, "inkfiles/knot/param-recurse.ink");
fixture!(function_func_basic, "inkfiles/function/func-basic.ink");
fixture!(function_func_none, "inkfiles/function/func-none.ink");
fixture!(
    glue_left_right_glue_matching,
    "inkfiles/glue/left-right-glue-matching.ink"
);
fixture!(glue_testbugfix1, "inkfiles/glue/testbugfix1.ink");
fixture!(glue_testbugfix2, "inkfiles/glue/testbugfix2.ink");
fixture_count_all_visits!(
    misc_newlines_with_string_eval,
    "inkfiles/misc/newlines_with_string_eval.ink"
);
fixture!(function_complex_func1, "inkfiles/function/complex-func1.ink");
fixture!(function_complex_func2, "inkfiles/function/complex-func2.ink");
fixture!(function_complex_func3, "inkfiles/function/complex-func3.ink");
fixture!(function_setvar_func, "inkfiles/function/setvar-func.ink");
fixture!(function_func_inline, "inkfiles/function/func-inline.ink");
fixture!(
    runtime_external_function_0_arg,
    "inkfiles/runtime/external-function-0-arg.ink"
);
fixture!(
    runtime_external_function_1_arg,
    "inkfiles/runtime/external-function-1-arg.ink"
);
fixture!(
    runtime_external_function_2_arg,
    "inkfiles/runtime/external-function-2-arg.ink"
);
fixture!(
    runtime_external_function_3_arg,
    "inkfiles/runtime/external-function-3-arg.ink"
);
fixture!(
    function_evaluating_function_variablestate_bug,
    "inkfiles/function/evaluating-function-variablestate-bug.ink"
);
fixture!(
    tunnels_tunnel_onwards_divert_override,
    "inkfiles/tunnels/tunnel-onwards-divert-override.ink"
);
fixture!(tags_tags_in_seq, "inkfiles/tags/tagsInSeq.ink");
fixture_count_all_visits!(misc_i18n, "inkfiles/misc/i18n.ink");
