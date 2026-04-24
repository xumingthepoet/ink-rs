use crate::{
    parsed::{ContentList, Glue, Object, Sequence, SequenceType, Tag, Text},
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

    let is_tag_line = text.starts_with('#');
    let mut objects = parse_inline_content(&text, &span)?;

    if !is_tag_line {
        objects.push(Object::Text(Text::new("\n", span)));
    }
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
}

pub(super) fn parse_inline_content(text: &str, span: &SourceSpan) -> Option<Vec<Object>> {
    parse_inline_content_with_options(text, span, true)
}

pub(super) fn parse_inline_content_preserving_divert_whitespace(
    text: &str,
    span: &SourceSpan,
) -> Option<Vec<Object>> {
    parse_inline_content_with_options(text, span, false)
}

fn parse_inline_content_with_options(
    text: &str,
    span: &SourceSpan,
    trim_divert_separator_whitespace: bool,
) -> Option<Vec<Object>> {
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

        if let Some(rest) = remaining.strip_prefix('{') {
            let close_index = rest.find('}')?;
            let inner = &rest[..close_index];
            let object = if inner.contains('|') {
                Object::Sequence(parse_inline_sequence(inner, span)?)
            } else {
                Object::Expression(super::parse_initial_expression(inner.trim())?)
            };
            objects.push(Object::ContentList(ContentList::new(vec![object])));
            remaining = &rest[close_index + 1..];
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
            objects.push(Object::Divert(super::divert::parse_divert_source(
                target,
                span.clone(),
            )?));
            break;
        }

        let next_token = [
            remaining.find("<>"),
            remaining.find("->"),
            remaining.find('#'),
            remaining.find('{'),
        ]
        .into_iter()
        .flatten()
        .min();

        match next_token {
            Some(0) => return None,
            Some(index) => {
                let prefix =
                    if trim_divert_separator_whitespace && remaining[index..].starts_with("->") {
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

fn parse_inline_sequence(source: &str, span: &SourceSpan) -> Option<Sequence> {
    let (sequence_type, elements_source) = parse_sequence_type(source.trim_start());
    let elements = elements_source
        .split('|')
        .map(|element| {
            let objects = parse_inline_content(element.trim(), span).unwrap_or_default();
            ContentList::new(objects)
        })
        .collect::<Vec<_>>();

    Some(Sequence::new(sequence_type, elements))
}

pub(super) fn parse_sequence_type_annotation(source: &str) -> Option<(SequenceType, &str)> {
    let source = source.trim_start();
    let first = source.chars().next()?;

    if matches!(first, '&' | '!' | '$' | '~') {
        let mut sequence_type: Option<SequenceType> = None;
        let mut rest_start = 0;
        for (index, ch) in source.char_indices() {
            let flag = match ch {
                '&' => Some(SequenceType::CYCLE),
                '!' => Some(SequenceType::ONCE),
                '$' => Some(SequenceType::STOPPING),
                '~' => Some(SequenceType::SHUFFLE),
                ' ' | '\t' => None,
                _ => break,
            };
            rest_start = index + ch.len_utf8();
            if let Some(flag) = flag {
                sequence_type = Some(sequence_type.map_or(flag, |current| current.union(flag)));
            }
        }
        return sequence_type.map(|sequence_type| (sequence_type, &source[rest_start..]));
    }

    let (words, rest) = source.split_once(':')?;
    let mut sequence_type: Option<SequenceType> = None;
    for word in words.split_whitespace() {
        let flag = match word {
            "stopping" => SequenceType::STOPPING,
            "cycle" => SequenceType::CYCLE,
            "shuffle" => SequenceType::SHUFFLE,
            "once" => SequenceType::ONCE,
            _ => return None,
        };
        sequence_type = Some(sequence_type.map_or(flag, |current| current.union(flag)));
    }
    sequence_type.map(|sequence_type| (sequence_type, rest))
}

fn parse_sequence_type(source: &str) -> (SequenceType, &str) {
    parse_sequence_type_annotation(source).unwrap_or((SequenceType::STOPPING, source))
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
