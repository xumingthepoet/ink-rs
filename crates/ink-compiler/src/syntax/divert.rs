use crate::{
    parsed::{Divert, DivertTarget, Expression, Object, TunnelOnwards},
    source::SourceSpan,
};

use super::{parse_initial_expression, rule::RuleParser, split_top_level_args};

pub(super) fn parse_divert_objects(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    let source = parser.line_remainder().to_string();
    let objects = parse_divert_objects_source(&source, span)?;
    parser.skip_to_end();
    Some(objects)
}

pub(super) fn parse_divert_objects_source(source: &str, span: SourceSpan) -> Option<Vec<Object>> {
    let source = source.trim();
    if let Some(after_thread_arrow) = source.strip_prefix("<-") {
        let divert = parse_divert_source(after_thread_arrow, span)?.with_thread();
        return Some(vec![Object::Divert(divert)]);
    }

    if !source.starts_with("->") {
        return None;
    }

    if let Some(rest) = source.strip_prefix("->->") {
        let override_target = rest
            .trim()
            .is_empty()
            .then_some(None)
            .unwrap_or_else(|| Some(DivertTarget::from_source(rest.trim())));
        return Some(vec![Object::TunnelOnwards(TunnelOnwards::new(
            override_target,
            span,
        ))]);
    }

    let after_arrow = source.strip_prefix("->")?.trim();
    let (segments, has_trailing_tunnel_arrow) = split_multidivert_segments(after_arrow);
    if segments.is_empty() {
        return None;
    }

    let last_index = segments.len() - 1;
    segments
        .into_iter()
        .enumerate()
        .map(|(index, segment)| {
            let is_tunnel = index < last_index || has_trailing_tunnel_arrow;
            let mut divert = parse_divert_source(segment, span.clone())?;
            if is_tunnel {
                divert = divert.with_tunnel();
            }
            Some(Object::Divert(divert))
        })
        .collect()
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

fn split_multidivert_segments(source: &str) -> (Vec<&str>, bool) {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut paren_depth = 0;

    while index < source.len() {
        let rest = &source[index..];
        let ch = rest.chars().next().expect("index is inside source");

        if escaped {
            escaped = false;
            index += ch.len_utf8();
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '(' if !in_string => paren_depth += 1,
            ')' if !in_string => paren_depth -= 1,
            '-' if !in_string && paren_depth == 0 && rest.starts_with("->") => {
                let segment = source[start..index].trim();
                if !segment.is_empty() {
                    segments.push(segment);
                }
                index += "->".len();
                start = index;
                continue;
            }
            _ => {}
        }

        index += ch.len_utf8();
    }

    let tail = source[start..].trim();
    let has_trailing_tunnel_arrow = tail.is_empty() && !segments.is_empty();
    if !tail.is_empty() {
        segments.push(tail);
    }

    (segments, has_trailing_tunnel_arrow)
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
