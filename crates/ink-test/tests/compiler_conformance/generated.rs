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
