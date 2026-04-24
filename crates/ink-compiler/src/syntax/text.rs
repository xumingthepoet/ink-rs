use crate::{
    parsed::{
        Conditional, ConditionalBranch, ContentList, Glue, Object, Sequence, SequenceType, Tag,
        Text, Weave,
    },
    source::SourceSpan,
};

use super::rule::RuleParser;

pub(super) fn parse_text_line(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    let span = parser.current_span();
    let text = parser.line_remainder().trim_start().to_string();

    if text.is_empty() {
        return None;
    }

    if has_unsupported_text_syntax(&text) {
        return None;
    }

    let is_tag_line = text.starts_with('#');
    let mut objects = parse_inline_content(&text, &span)?;
    if objects.first().is_some_and(
        |object| matches!(object, Object::Text(text) if text.text().starts_with("return")),
    ) {
        parser.warning(
            "Do you need a '~' before 'return'? If not, perhaps use a glue: <> (since it's lowercase) or rewrite somehow?",
        );
    }

    parser.skip_to_end();

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
    let mut tag_state = InlineTagState::default();
    parse_inline_content_inner(
        text,
        span,
        trim_divert_separator_whitespace,
        true,
        &mut tag_state,
    )
}

#[derive(Default)]
struct InlineTagState {
    active: bool,
}

fn parse_inline_content_inner(
    text: &str,
    span: &SourceSpan,
    trim_divert_separator_whitespace: bool,
    close_tag_at_end: bool,
    tag_state: &mut InlineTagState,
) -> Option<Vec<Object>> {
    let mut remaining = text;
    let mut objects = Vec::new();

    while !remaining.is_empty() {
        if let Some(rest) = remaining.strip_prefix('#') {
            if tag_state.active {
                objects.push(Object::ContentList(ContentList::new(vec![
                    Object::Tag(Tag::new(false, false)),
                    Object::Tag(Tag::new(true, false)),
                ])));
            } else {
                objects.push(Object::Tag(Tag::new(true, false)));
            }
            tag_state.active = true;
            remaining = rest.trim_start_matches([' ', '\t']);
            continue;
        }

        if let Some(rest) = remaining.strip_prefix('{') {
            let close_index = find_matching_brace(rest)?;
            let inner = &rest[..close_index];
            let braced_objects = parse_inline_braced_objects(inner, span, tag_state)?;
            objects.push(Object::ContentList(ContentList::new(braced_objects)));
            remaining = &rest[close_index + 1..];
            continue;
        }

        if let Some(rest) = remaining.strip_prefix("<>") {
            objects.push(Object::Glue(Glue::new()));
            remaining = rest;
            continue;
        }

        if remaining.starts_with("->") {
            if tag_state.active {
                objects.push(Object::Tag(Tag::new(false, false)));
                tag_state.active = false;
            }
            objects.extend(super::divert::parse_divert_objects_source(
                remaining,
                span.clone(),
            )?);
            break;
        }

        let next_token = find_next_unescaped_inline_token(remaining);

        match next_token {
            Some(0) => return None,
            Some(index) => {
                let prefix_text = unescape_content_text(&remaining[..index]);
                let prefix =
                    if trim_divert_separator_whitespace && remaining[index..].starts_with("->") {
                        normalize_divert_separator_whitespace(&prefix_text)
                    } else {
                        prefix_text
                    };
                if !prefix.is_empty() {
                    objects.push(Object::Text(Text::new(prefix, span.clone())));
                }
                remaining = &remaining[index..];
            }
            None => {
                objects.push(Object::Text(Text::new(
                    unescape_content_text(remaining),
                    span.clone(),
                )));
                remaining = "";
            }
        }
    }

    if close_tag_at_end && tag_state.active {
        objects.push(Object::Tag(Tag::new(false, false)));
        tag_state.active = false;
    }

    if objects.is_empty() {
        None
    } else {
        Some(objects)
    }
}

fn find_next_unescaped_inline_token(source: &str) -> Option<usize> {
    let mut escaped = false;

    for (index, ch) in source.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == '#'
            || ch == '{'
            || source[index..].starts_with("<>")
            || source[index..].starts_with("->")
        {
            return Some(index);
        }
    }

    None
}

fn unescape_content_text(source: &str) -> String {
    let mut output = String::new();
    let mut chars = source.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                output.push(next);
            }
        } else {
            output.push(ch);
        }
    }

    output
}

fn parse_inline_braced_objects(
    source: &str,
    span: &SourceSpan,
    tag_state: &mut InlineTagState,
) -> Option<Vec<Object>> {
    let was_tag_active = tag_state.active;
    let mut objects = vec![parse_inline_braced_object(source, span, tag_state)?];

    if !was_tag_active && tag_state.active {
        objects.push(Object::Tag(Tag::new(false, false)));
        tag_state.active = false;
    }

    Some(objects)
}

fn find_matching_brace(source_after_open: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in source_after_open.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                if depth == 0 {
                    return Some(index);
                }
                depth -= 1;
            }
            _ => {}
        }
    }

    None
}

fn parse_inline_braced_object(
    source: &str,
    span: &SourceSpan,
    tag_state: &mut InlineTagState,
) -> Option<Object> {
    let trimmed = source.trim();
    if parse_sequence_type_annotation(trimmed).is_some() {
        return Some(Object::Sequence(parse_inline_sequence(
            trimmed, span, tag_state,
        )?));
    }

    if let Some((condition_source, branch_source)) = split_top_level_once(source, ':') {
        let condition = super::parse_initial_expression(condition_source.trim())?;
        let alternatives = split_top_level(branch_source, '|');
        if alternatives.len() > 2 {
            return None;
        }

        let true_content = parse_inline_content(alternatives.first().copied().unwrap_or(""), span)
            .unwrap_or_default();
        let mut branches = vec![ConditionalBranch::new(
            true,
            false,
            true,
            Weave::new(true_content, 0),
            None,
        )];

        if let Some(else_source) = alternatives.get(1) {
            let else_content = parse_inline_content(else_source, span).unwrap_or_default();
            branches.push(ConditionalBranch::new(
                false,
                true,
                true,
                Weave::new(else_content, 0),
                None,
            ));
        }

        return Some(Object::Conditional(Conditional::new(
            Some(condition),
            branches,
        )));
    }

    if contains_top_level(trimmed, '|') {
        return Some(Object::Sequence(parse_inline_sequence(
            trimmed, span, tag_state,
        )?));
    }

    Some(Object::Expression(super::parse_initial_expression(
        trimmed,
    )?))
}

fn parse_inline_sequence(
    source: &str,
    span: &SourceSpan,
    tag_state: &mut InlineTagState,
) -> Option<Sequence> {
    let (sequence_type, elements_source) = parse_sequence_type(source.trim_start());
    let elements = split_top_level(elements_source, '|')
        .into_iter()
        .map(|element| {
            let objects = parse_inline_content_inner(element.trim(), span, true, false, tag_state)
                .unwrap_or_default();
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

fn contains_top_level(source: &str, needle: char) -> bool {
    split_top_level_once(source, needle).is_some()
}

fn split_top_level_once(source: &str, needle: char) -> Option<(&str, &str)> {
    let mut in_string = false;
    let mut escaped = false;
    let mut paren_depth = 0;
    let mut brace_depth = 0;

    for (index, ch) in source.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => in_string = !in_string,
            '(' if !in_string => paren_depth += 1,
            ')' if !in_string => paren_depth -= 1,
            '{' if !in_string => brace_depth += 1,
            '}' if !in_string => brace_depth -= 1,
            _ if ch == needle && !in_string && paren_depth == 0 && brace_depth == 0 => {
                let right_start = index + ch.len_utf8();
                return Some((&source[..index], &source[right_start..]));
            }
            _ => {}
        }
    }

    None
}

fn split_top_level(source: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut rest = source;
    while let Some((left, right)) = split_top_level_once(rest, separator) {
        parts.push(left);
        rest = right;
    }
    parts.push(rest);
    parts
}

fn normalize_divert_separator_whitespace(text: &str) -> String {
    let trimmed = text.trim_end_matches([' ', '\t']);
    if trimmed.len() == text.len() {
        return format!("{text} ");
    }

    if trimmed.is_empty() {
        return " ".to_string();
    }

    text[..trimmed.len() + 1].to_string()
}
