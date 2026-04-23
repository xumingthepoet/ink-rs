use crate::parsed::{Divert, DivertTarget};

use super::rule::RuleParser;

pub(super) fn parse_divert(parser: &mut RuleParser<'_>) -> Option<Divert> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    parser.match_string("->")?;
    let target = parser.line_remainder().to_string();
    parser.skip_to_end();

    Some(Divert::new(DivertTarget::from_source(&target), span))
}
