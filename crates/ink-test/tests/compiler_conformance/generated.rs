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
fixture!(choices_one, "inkfiles/choices/one.ink");
