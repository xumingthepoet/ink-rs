use crate::{
    diagnostic::{Diagnostic, DiagnosticCode},
    parsed::{Conditional, ConditionalBranch, ContentList, Glue, Object, Tag, Text, Weave},
    source::SourceSpan,
};

use super::{rule::RuleParser, scan};

pub(super) const REMOVED_SEQUENCE_MESSAGE: &str =
    "Source sequences, cycles, shuffles, and once-only alternatives are no longer supported; use explicit variables and conditionals instead.";

pub(super) fn parse_text_line(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    let span = parser.current_span();
    let raw_text = parser.line_remainder().to_string();
    let leading_whitespace = raw_text
        .chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .count();
    let text = raw_text.trim_start().to_string();

    if text.is_empty() {
        return None;
    }

    if is_structured_syntax_prefix(&text) {
        return None;
    }

    let text_span = SourceSpan::new(
        span.source_name.clone(),
        span.line,
        span.column + leading_whitespace,
    );

    if let Some(char_offset) = find_unmatched_open_brace_char_offset(&text) {
        parser.diagnostic(
            Diagnostic::error(
                SourceSpan::new(
                    text_span.source_name.clone(),
                    text_span.line,
                    text_span.column + char_offset,
                ),
                "expected closing `}` for inline expression before end of line",
            )
            .with_code(DiagnosticCode::InvalidInlineSyntax),
        );
        return None;
    }

    if let Some(diagnostic) = removed_sequence_diagnostic_for_inline_text(&text, &text_span) {
        parser.diagnostic(diagnostic);
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

fn is_structured_syntax_prefix(text: &str) -> bool {
    text.starts_with("INCLUDE ")
        || text.starts_with("VAR ")
        || text.starts_with("CONST ")
        || text.starts_with("EXTERNAL ")
        || text.starts_with("===")
        || text.starts_with('=')
        || text.starts_with('*')
        || text.starts_with('+')
        || (text.starts_with('-') && !text.starts_with("->"))
        || text.starts_with('~')
}

fn find_unmatched_open_brace_char_offset(text: &str) -> Option<usize> {
    let mut byte_offset = 0;
    while let Some(relative_open) = text[byte_offset..].find('{') {
        let open_index = byte_offset + relative_open;
        if is_escaped(text, open_index) {
            byte_offset = open_index + '{'.len_utf8();
            continue;
        }

        let rest = &text[open_index + '{'.len_utf8()..];
        let Some(close_index) = scan::find_matching_delimiter(rest, '{', '}') else {
            return Some(text[..open_index].chars().count());
        };
        byte_offset = open_index + '{'.len_utf8() + close_index + '}'.len_utf8();
    }
    None
}

pub(super) fn removed_sequence_diagnostic_for_inline_text(
    text: &str,
    span: &SourceSpan,
) -> Option<Diagnostic> {
    let char_offset = find_removed_inline_sequence_char_offset(text)?;
    Some(
        Diagnostic::error(
            SourceSpan::new(
                span.source_name.clone(),
                span.line,
                span.column + char_offset,
            ),
            REMOVED_SEQUENCE_MESSAGE,
        )
        .with_code(DiagnosticCode::InvalidInlineSyntax),
    )
}

fn find_removed_inline_sequence_char_offset(text: &str) -> Option<usize> {
    let mut byte_offset = 0;
    while let Some(relative_open) = text[byte_offset..].find('{') {
        let open_index = byte_offset + relative_open;
        if is_escaped(text, open_index) {
            byte_offset = open_index + '{'.len_utf8();
            continue;
        }

        let rest = &text[open_index + '{'.len_utf8()..];
        let Some(close_index) = scan::find_matching_delimiter(rest, '{', '}') else {
            return None;
        };
        let inner = &rest[..close_index];
        if is_removed_inline_sequence(inner) {
            return Some(text[..open_index].chars().count());
        }
        byte_offset = open_index + '{'.len_utf8() + close_index + '}'.len_utf8();
    }

    None
}

fn is_removed_inline_sequence(source: &str) -> bool {
    let trimmed = source.trim();
    if removed_sequence_type_annotation_rest(trimmed).is_some() {
        return true;
    }

    if split_inline_conditional(source).is_some() {
        return false;
    }

    scan::split_top_level_once_with_options(trimmed, '|', scan::ScanOptions::inline_text())
        .is_some()
}

fn is_escaped(source: &str, byte_index: usize) -> bool {
    let mut slash_count = 0;
    for ch in source[..byte_index].chars().rev() {
        if ch == '\\' {
            slash_count += 1;
        } else {
            break;
        }
    }
    slash_count % 2 == 1
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
            let close_index = scan::find_matching_delimiter(rest, '{', '}')?;
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

        if remaining.starts_with("->") || remaining.starts_with("<-") {
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

        let next_token = scan::find_top_level_token_with_options(
            remaining,
            &["#", "{", "<>", "->", "<-"],
            scan::ScanOptions::inline_tokens(),
        )
        .map(|(index, _)| index);

        match next_token {
            Some(0) => return None,
            Some(index) => {
                let prefix_text = unescape_content_text(&remaining[..index]);
                let prefix = if trim_divert_separator_whitespace
                    && (remaining[index..].starts_with("->")
                        || remaining[index..].starts_with("<-"))
                {
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

fn parse_inline_braced_object(
    source: &str,
    span: &SourceSpan,
    _tag_state: &mut InlineTagState,
) -> Option<Object> {
    let trimmed = source.trim();

    if let Some((condition_source, branch_source)) = split_inline_conditional(source) {
        let condition = super::parse_initial_expression(condition_source.trim())?;
        let alternatives = scan::split_top_level_preserving_whitespace_with_options(
            branch_source,
            '|',
            scan::ScanOptions::inline_text(),
        );
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

    Some(Object::Expression(super::parse_initial_expression(
        trimmed,
    )?))
}

fn split_inline_conditional(source: &str) -> Option<(&str, &str)> {
    let index = scan::top_level_token_matches_with_options(
        source,
        &[":"],
        scan::ScanOptions::inline_text(),
    )
    .into_iter()
    .map(|(index, _)| index)
    .find(|index| {
        !source[..*index].ends_with(':') && !source[index + ':'.len_utf8()..].starts_with(':')
    })?;

    Some((&source[..index], &source[index + ':'.len_utf8()..]))
}

pub(super) fn removed_sequence_type_annotation_rest(source: &str) -> Option<&str> {
    let source = source.trim_start();
    let first = source.chars().next()?;

    if matches!(first, '&' | '!' | '$' | '~') {
        let mut saw_flag = false;
        let mut rest_start = 0;
        for (index, ch) in source.char_indices() {
            let is_flag = match ch {
                '&' | '!' | '$' | '~' => Some(true),
                ' ' | '\t' => Some(false),
                _ => break,
            };
            rest_start = index + ch.len_utf8();
            if is_flag == Some(true) {
                saw_flag = true;
            }
        }
        return saw_flag.then_some(&source[rest_start..]);
    }

    let (words, rest) = source.split_once(':')?;
    let mut saw_word = false;
    for word in words.split_whitespace() {
        match word {
            "stopping" | "cycle" | "shuffle" | "once" => saw_word = true,
            _ => return None,
        }
    }
    saw_word.then_some(rest)
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

#[cfg(test)]
mod tests {
    use crate::{
        parsed::{Expression, Object},
        source::SourceSpan,
    };

    use super::*;

    #[test]
    fn inline_braced_qualified_reference_is_not_split_as_conditional() {
        let objects = parse_inline_content(
            "{state::score}",
            &SourceSpan::new(Some("module.ink".to_string()), 1, 1),
        )
        .expect("inline content should parse");

        let [Object::ContentList(content)] = objects.as_slice() else {
            panic!("expected braced content list, got {objects:#?}");
        };
        let [Object::Expression(Expression::QualifiedReference(name))] = content.objects() else {
            panic!("expected qualified reference expression, got {content:#?}");
        };

        assert_eq!(name.as_str(), "state::score");
    }

    #[test]
    fn inline_braced_single_colon_still_parses_as_conditional() {
        let objects = parse_inline_content(
            "{ready: go}",
            &SourceSpan::new(Some("conditional.ink".to_string()), 1, 1),
        )
        .expect("inline content should parse");

        let [Object::ContentList(content)] = objects.as_slice() else {
            panic!("expected braced content list, got {objects:#?}");
        };

        assert!(matches!(content.objects(), [Object::Conditional(_)]));
    }

    #[test]
    fn removed_inline_sequence_reports_specific_error_span() {
        let diagnostic = removed_sequence_diagnostic_for_inline_text(
            "Line {one|two}",
            &SourceSpan::new(Some("sequence.ink".to_string()), 4, 3),
        )
        .expect("expected removed sequence diagnostic");

        assert_eq!(diagnostic.code, Some(DiagnosticCode::InvalidInlineSyntax));
        assert_eq!(diagnostic.line, 4);
        assert_eq!(diagnostic.column, 8);
        assert_eq!(diagnostic.message, REMOVED_SEQUENCE_MESSAGE);
    }
}
