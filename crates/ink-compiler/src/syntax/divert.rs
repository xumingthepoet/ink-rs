use crate::{
    parsed::{Divert, DivertTarget, Expression, Object, TunnelOnwards},
    source::SourceSpan,
};

use super::{parse_initial_expression, rule::RuleParser, scan, split_top_level_args};

pub(super) fn parse_divert_objects(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    let source = parser.line_remainder().to_string();
    if let Some(after_thread_arrow) = source.trim().strip_prefix("<-") {
        if after_thread_arrow.trim().is_empty() {
            parser.error("Expected target for new thread");
            parser.skip_to_end();
            return Some(vec![Object::Divert(
                Divert::new(DivertTarget::Empty, span).with_thread(),
            )]);
        }
    }
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
        let override_source = (!rest.trim().is_empty()).then_some(rest.trim());
        return Some(vec![Object::TunnelOnwards(parse_tunnel_onwards(
            override_source,
            span,
        )?)]);
    }

    let after_arrow = source.strip_prefix("->")?.trim();
    let (segments, trailing) = split_multidivert_segments(after_arrow);
    if segments.is_empty() {
        return Some(vec![Object::Divert(Divert::new(DivertTarget::Empty, span))]);
    }

    let last_index = segments.len() - 1;
    let mut objects = segments
        .into_iter()
        .enumerate()
        .map(|(index, segment)| {
            let is_tunnel = index < last_index || trailing.is_some();
            let mut divert = parse_divert_source(segment, span.clone())?;
            if is_tunnel {
                divert = divert.with_tunnel();
            }
            Some(Object::Divert(divert))
        })
        .collect::<Option<Vec<_>>>()?;

    if let TrailingDivertSyntax::TunnelOnwards(target) = trailing {
        objects.push(Object::TunnelOnwards(parse_tunnel_onwards(target, span)?));
    }

    Some(objects)
}

fn parse_tunnel_onwards(source: Option<&str>, span: SourceSpan) -> Option<TunnelOnwards> {
    let Some(source) = source else {
        return Some(TunnelOnwards::new(None, span));
    };
    let divert = parse_divert_source(source, span.clone())?;
    Some(TunnelOnwards::with_arguments(
        Some(divert.target().clone()),
        divert.arguments().to_vec(),
        span,
    ))
}

pub(super) fn parse_divert_source(source: &str, span: SourceSpan) -> Option<Divert> {
    let source = source.trim();
    let (target, arguments, has_argument_list) = parse_divert_target_and_arguments(source)?;
    if has_argument_list {
        Some(Divert::with_arguments(
            DivertTarget::from_source(target),
            arguments,
            span,
        ))
    } else {
        Some(Divert::new(DivertTarget::from_source(target), span))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrailingDivertSyntax<'a> {
    None,
    TunnelArrow,
    TunnelOnwards(Option<&'a str>),
}

impl TrailingDivertSyntax<'_> {
    fn is_some(self) -> bool {
        !matches!(self, Self::None)
    }
}

fn split_multidivert_segments(source: &str) -> (Vec<&str>, TrailingDivertSyntax<'_>) {
    let mut segments = Vec::new();
    let mut start = 0;

    for (index, token) in scan::top_level_token_matches_with_options(
        source,
        &["->->", "->"],
        scan::ScanOptions::expression(),
    ) {
        match token {
            "->->" => {
                let segment = source[start..index].trim();
                if !segment.is_empty() {
                    segments.push(segment);
                }
                let override_target = source[index + "->->".len()..].trim();
                let override_target = (!override_target.is_empty()).then_some(override_target);
                return (
                    segments,
                    TrailingDivertSyntax::TunnelOnwards(override_target),
                );
            }
            "->" => {
                let segment = source[start..index].trim();
                if !segment.is_empty() {
                    segments.push(segment);
                }
                start = index + "->".len();
            }
            _ => {}
        }
    }

    let tail = source[start..].trim();
    let trailing = if tail.is_empty() && !segments.is_empty() {
        TrailingDivertSyntax::TunnelArrow
    } else {
        TrailingDivertSyntax::None
    };
    if !tail.is_empty() {
        segments.push(tail);
    }

    (segments, trailing)
}

fn parse_divert_target_and_arguments(source: &str) -> Option<(&str, Vec<Expression>, bool)> {
    let Some(open_index) =
        scan::find_top_level_char_with_options(source, '(', scan::ScanOptions::expression())
    else {
        return Some((source, Vec::new(), false));
    };
    if !source.ends_with(')') {
        return None;
    }

    let target = source[..open_index].trim();
    if !is_divert_path(target) && parse_initial_expression(target).is_none() {
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

    Some((target, arguments, true))
}

fn is_divert_path(source: &str) -> bool {
    !source.is_empty()
        && source
            .split('.')
            .all(|part| super::is_identifier(part.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multidivert_split_ignores_arrows_inside_arguments() {
        let (segments, trailing) = split_multidivert_segments(r#"first("->") -> second"#);

        assert_eq!(segments, vec![r#"first("->")"#, "second"]);
        assert_eq!(trailing, TrailingDivertSyntax::None);
    }

    #[test]
    fn multidivert_split_preserves_tunnel_onwards_override() {
        let (segments, trailing) =
            split_multidivert_segments(r#"first -> second ->-> escape("->")"#);

        assert_eq!(segments, vec!["first", "second"]);
        assert_eq!(
            trailing,
            TrailingDivertSyntax::TunnelOnwards(Some(r#"escape("->")"#))
        );
    }
}
