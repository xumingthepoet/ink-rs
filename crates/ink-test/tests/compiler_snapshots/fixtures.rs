use super::common;

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

fixture!(text_oneline, "text/oneline.ink");
fixture!(text_twolines, "text/twolines.ink");
fixture!(knot_multi_line, "knots/multi-line.ink");
fixture!(knot_single_line, "knots/single-line.ink");
fixture!(knot_strip_empty_lines, "knots/strip-empty-lines.ink");
fixture!(divert_invisible_divert, "diverts/invisible-divert.ink");
fixture!(divert_simple_divert, "diverts/simple-divert.ink");
fixture!(glue_glue_with_divert, "glue/glue-with-divert.ink");
fixture!(glue_simple_glue, "glue/simple-glue.ink");
fixture!(runtime_jump_knot, "runtime_api/jump-knot.ink");
fixture!(runtime_saving_loading, "runtime_api/saving-loading.ink");
fixture!(runtime_multiflow_basics, "runtime_api/multiflow-basics.ink");
fixture!(runtime_jump_stitch, "runtime_api/jump-stitch.ink");
fixture!(tags_tags, "tags/tags.ink");
fixture!(
    variable_variable_declaration,
    "variables/variable-declaration.ink"
);
fixture!(tags_tags_dynamic_content, "tags/tagsDynamicContent.ink");
fixture!(variable_varcalc, "variables/varcalc.ink");
fixture!(typed_array_literals, "typed/array-literals.ink");
fixture!(typed_struct_literals, "typed/struct-literals.ink");
fixture!(function_rnd_func, "functions/rnd-func.ink");
fixture_count_all_visits!(misc_operations, "misc/operations.ink");
fixture!(conditional_ifelse, "conditionals/ifelse.ink");
fixture!(conditional_iffalse, "conditionals/iffalse.ink");
fixture!(conditional_iftrue, "conditionals/iftrue.ink");
fixture!(function_test_error, "functions/test-error.ink");
fixture_count_all_visits!(
    runtime_read_visit_counts,
    "runtime_api/read-visit-counts.ink"
);
fixture!(conditional_ifelse_ext, "conditionals/ifelse-ext.ink");
fixture!(misc_issue15, "misc/issue15.ink");
fixture!(knot_param_recurse, "knots/param-recurse.ink");
fixture!(function_func_basic, "functions/func-basic.ink");
fixture!(function_func_none, "functions/func-none.ink");
fixture!(
    glue_left_right_glue_matching,
    "glue/left-right-glue-matching.ink"
);
fixture!(glue_testbugfix1, "glue/testbugfix1.ink");
fixture!(glue_testbugfix2, "glue/testbugfix2.ink");
fixture_count_all_visits!(
    misc_newlines_with_string_eval,
    "misc/newlines_with_string_eval.ink"
);
fixture!(function_complex_func1, "functions/complex-func1.ink");
fixture!(function_complex_func2, "functions/complex-func2.ink");
fixture!(function_complex_func3, "functions/complex-func3.ink");
fixture!(function_setvar_func, "functions/setvar-func.ink");
fixture!(function_func_inline, "functions/func-inline.ink");
fixture!(
    runtime_external_function_0_arg,
    "runtime_api/external-function-0-arg.ink"
);
fixture!(
    runtime_external_function_1_arg,
    "runtime_api/external-function-1-arg.ink"
);
fixture!(
    runtime_external_function_2_arg,
    "runtime_api/external-function-2-arg.ink"
);
fixture!(
    runtime_external_function_3_arg,
    "runtime_api/external-function-3-arg.ink"
);
fixture!(
    function_evaluating_function_variablestate_bug,
    "functions/evaluating-function-variablestate-bug.ink"
);
fixture!(
    tunnels_tunnel_onwards_divert_override,
    "tunnels/tunnel-onwards-divert-override.ink"
);
fixture!(tags_tags_in_seq, "tags/tagsInSeq.ink");
fixture_count_all_visits!(misc_i18n, "misc/i18n.ink");
