use crate::{
    parsed::{Divert, DivertTarget, Glue, Object, Tag, Text},
    source::SourceSpan,
};

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
    text.starts_with('#')
        || text.starts_with("INCLUDE ")
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
}

pub(super) fn parse_inline_content(text: &str, span: &SourceSpan) -> Option<Vec<Object>> {
    let mut remaining = text;
    let mut objects = Vec::new();
    let mut tag_active = false;

    while !remaining.is_empty() {
        if let Some(rest) = remaining.strip_prefix('#') {
            if tag_active {
                objects.push(Object::Tag(Tag::new(false, false)));
            }
            objects.push(Object::Tag(Tag::new(true, false)));
            tag_active = true;
            remaining = rest.trim_start_matches([' ', '\t']);
            continue;
        }

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
            if tag_active {
                objects.push(Object::Tag(Tag::new(false, false)));
                tag_active = false;
            }
            objects.push(Object::Divert(Divert::new(
                DivertTarget::from_source(target),
                span.clone(),
            )));
            break;
        }

        let next_token = [
            remaining.find("<>"),
            remaining.find("->"),
            remaining.find('#'),
        ]
        .into_iter()
        .flatten()
        .min();

        match next_token {
            Some(0) => return None,
            Some(index) => {
                let prefix = if remaining[index..].starts_with("->") {
                    trim_separator_whitespace(&remaining[..index])
                } else {
                    &remaining[..index]
                };
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

    if tag_active {
        objects.push(Object::Tag(Tag::new(false, false)));
    }

    if objects.is_empty() {
        None
    } else {
        Some(objects)
    }
}

fn trim_separator_whitespace(text: &str) -> &str {
    let trimmed = text.trim_end_matches([' ', '\t']);
    if trimmed.len() == text.len() {
        return text;
    }

    if trimmed.is_empty() {
        return " ";
    }

    &text[..trimmed.len() + 1]
}
