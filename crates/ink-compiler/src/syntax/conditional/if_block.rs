use crate::parsed::ConditionalKind;

use super::ParsedConditionalHeader;
use crate::syntax::{is_identifier_continue, parse_initial_expression};

pub(super) fn parse_header(header: &str) -> Option<ParsedConditionalHeader> {
    let rest = strip_control_keyword(header, "if")?;
    let condition_source = rest.trim();
    let initial_condition = if condition_source.is_empty() {
        None
    } else {
        Some(parse_initial_expression(condition_source)?)
    };

    Some(ParsedConditionalHeader {
        kind: ConditionalKind::If,
        initial_condition,
    })
}

fn strip_control_keyword<'a>(source: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = source.strip_prefix(keyword)?;
    if rest.chars().next().is_some_and(is_identifier_continue) {
        return None;
    }
    Some(rest)
}
