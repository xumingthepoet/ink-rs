use crate::parsed::{Divert, DivertTarget, Glue, Object, Text};

use super::rule::RuleParser;

pub(super) fn parse_text_line(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    let span = parser.current_span();
    let text = parser.line_remainder().trim_start().to_string();
    parser.skip_to_end();

    if text.is_empty() {
        return None;
    }

    if has_unsupported_text_syntax(&text) {
        return None;
    }

    let mut objects = parse_inline_content(&text, &span)?;

    objects.push(Object::Text(Text::new("\n", span)));
    Some(objects)
}

fn has_unsupported_text_syntax(text: &str) -> bool {
    text.starts_with("INCLUDE ")
        || text.starts_with("VAR ")
        || text.starts_with("LIST ")
        || text.starts_with("CONST ")
        || text.starts_with("EXTERNAL ")
        || text.starts_with("===")
        || text.starts_with('=')
        || text.starts_with('*')
        || text.starts_with('+')
        || (text.starts_with('-') && !text.starts_with("->"))
        || text.starts_with('~')
        || text.contains('{')
        || text.contains('}')
        || text.contains('#')
}

fn parse_inline_content(text: &str, span: &crate::source::SourceSpan) -> Option<Vec<Object>> {
    let mut remaining = text;
    let mut objects = Vec::new();

    while !remaining.is_empty() {
        if let Some(rest) = remaining.strip_prefix("<>") {
            objects.push(Object::Glue(Glue::new()));
            remaining = rest;
            continue;
        }

        if remaining.starts_with("->") {
            let target = remaining.strip_prefix("->")?.trim();
            if target.is_empty() {
                return None;
            }
            objects.push(Object::Divert(Divert::new(
                DivertTarget::from_source(target),
                span.clone(),
            )));
            break;
        }

        let next_glue = remaining.find("<>");
        let next_divert = remaining.find("->");
        let next_token = match (next_glue, next_divert) {
            (Some(glue), Some(divert)) => Some(glue.min(divert)),
            (Some(glue), None) => Some(glue),
            (None, Some(divert)) => Some(divert),
            (None, None) => None,
        };

        match next_token {
            Some(0) => return None,
            Some(index) => {
                let prefix = &remaining[..index];
                if !prefix.is_empty() {
                    objects.push(Object::Text(Text::new(prefix, span.clone())));
                }
                remaining = &remaining[index..];
            }
            None => {
                objects.push(Object::Text(Text::new(remaining, span.clone())));
                remaining = "";
            }
        }
    }

    if objects.is_empty() {
        None
    } else {
        Some(objects)
    }
}
