use crate::{
    parsed::{Divert, DivertTarget, Expression},
    source::SourceSpan,
};

use super::{parse_initial_expression, rule::RuleParser, split_top_level_args};

pub(super) fn parse_divert(parser: &mut RuleParser<'_>) -> Option<Divert> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    parser.match_string("->")?;
    let target = parser.line_remainder().to_string();
    parser.skip_to_end();

    parse_divert_source(&target, span)
}

pub(super) fn parse_divert_source(source: &str, span: SourceSpan) -> Option<Divert> {
    let source = source.trim();
    let (target, arguments) = parse_divert_target_and_arguments(source)?;
    Some(Divert::with_arguments(
        DivertTarget::from_source(target),
        arguments,
        span,
    ))
}

fn parse_divert_target_and_arguments(source: &str) -> Option<(&str, Vec<Expression>)> {
    let Some(open_index) = source.find('(') else {
        return Some((source, Vec::new()));
    };
    if !source.ends_with(')') {
        return None;
    }

    let target = source[..open_index].trim();
    if !is_divert_path(target) {
        return None;
    }

    let args_source = &source[open_index + 1..source.len() - 1];
    let arguments = if args_source.trim().is_empty() {
        Vec::new()
    } else {
        split_top_level_args(args_source)
            .into_iter()
            .map(parse_initial_expression)
            .collect::<Option<Vec<_>>>()?
    };

    Some((target, arguments))
}

fn is_divert_path(source: &str) -> bool {
    !source.is_empty()
        && source
            .split('.')
            .all(|part| super::is_identifier(part.trim()))
}
