use crate::parsed::ConditionalKind;

use super::ParsedConditionalHeader;
use crate::syntax::{is_identifier_continue, parse_initial_expression};

pub(super) fn parse_header(header: &str) -> Option<ParsedConditionalHeader> {
    let rest = strip_control_keyword(header, "switch")?;
    let selector_source = rest.trim();
    if selector_source.is_empty() {
        return None;
    }

    Some(ParsedConditionalHeader {
        kind: ConditionalKind::Switch,
        initial_condition: Some(parse_initial_expression(selector_source)?),
    })
}

fn strip_control_keyword<'a>(source: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = source.strip_prefix(keyword)?;
    if rest.chars().next().is_some_and(is_identifier_continue) {
        return None;
    }
    Some(rest)
}
