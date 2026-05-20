use crate::source::SourceSpan;

use super::{
    is_identifier_continue,
    token::{binary_operator_rule, symbol_operator_text, ExpressionToken, ExpressionTokenKind},
};

#[cfg(test)]
pub(super) fn tokenize_expression(source: &str) -> Vec<ExpressionToken> {
    tokenize_expression_at(source, &SourceSpan::new(None, 1, 1))
}

pub(super) fn tokenize_expression_at(source: &str, base_span: &SourceSpan) -> Vec<ExpressionToken> {
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < source.len() {
        let rest = &source[index..];
        let ch = rest.chars().next().expect("index is inside source");

        if ch.is_whitespace() {
            index += ch.len_utf8();
            continue;
        }

        if rest.starts_with("->") {
            tokens.push(token_at(
                source,
                index,
                ExpressionTokenKind::Arrow,
                base_span,
            ));
            index += "->".len();
            continue;
        }

        if rest.starts_with("::") {
            tokens.push(token_at(
                source,
                index,
                ExpressionTokenKind::DoubleColon,
                base_span,
            ));
            index += "::".len();
            continue;
        }

        if let Some(operator) = match_operator(rest) {
            tokens.push(token_at(
                source,
                index,
                ExpressionTokenKind::Operator(operator.to_string()),
                base_span,
            ));
            index += operator.len();
            continue;
        }

        match ch {
            '(' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::OpenParen,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            ')' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::CloseParen,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            '[' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::OpenBracket,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            ']' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::CloseBracket,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            '{' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::OpenBrace,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            '}' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::CloseBrace,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            ':' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::Colon,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            '.' => {
                tokens.push(token_at(source, index, ExpressionTokenKind::Dot, base_span));
                index += ch.len_utf8();
            }
            ',' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::Comma,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            '"' => {
                let (literal, next_index) = read_string_literal(source, index);
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::StringLiteral(literal),
                    base_span,
                ));
                index = next_index;
            }
            _ if is_token_word_start(ch) => {
                let (word, next_index) = read_token_word(source, index);
                let (kind, next_index) =
                    classify_word_token_with_decimal_suffix(source, index, word, next_index);
                tokens.push(token_at(source, index, kind, base_span));
                index = next_index;
            }
            _ => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::Operator(ch.to_string()),
                    base_span,
                ));
                index += ch.len_utf8();
            }
        }
    }

    tokens
}

fn token_at(
    source: &str,
    byte_index: usize,
    kind: ExpressionTokenKind,
    base_span: &SourceSpan,
) -> ExpressionToken {
    ExpressionToken {
        kind,
        byte_index,
        span: SourceSpan::new(
            base_span.source_name.clone(),
            base_span.line,
            base_span.column + source[..byte_index].chars().count(),
        ),
    }
}

pub(super) fn match_operator(source: &str) -> Option<&'static str> {
    symbol_operator_text(source)
}

fn read_string_literal(source: &str, start: usize) -> (String, usize) {
    let mut literal = String::new();
    let mut index = start + '"'.len_utf8();
    let mut escaped = false;
    let mut brace_depth = 0;
    let mut in_nested_string = false;

    while index < source.len() {
        let ch = source[index..]
            .chars()
            .next()
            .expect("index is inside source");
        index += ch.len_utf8();

        if escaped {
            if brace_depth == 0 {
                match ch {
                    'n' => literal.push('\n'),
                    'r' => literal.push('\r'),
                    't' => literal.push('\t'),
                    '"' => literal.push('"'),
                    '\\' => literal.push('\\'),
                    other => literal.push(other),
                }
            } else {
                literal.push('\\');
                literal.push(ch);
            }
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' if brace_depth == 0 => return (literal, index),
            '"' => {
                in_nested_string = !in_nested_string;
                literal.push(ch);
            }
            '{' if !in_nested_string => {
                brace_depth += 1;
                literal.push(ch);
            }
            '}' if !in_nested_string && brace_depth > 0 => {
                brace_depth -= 1;
                literal.push(ch);
            }
            other => literal.push(other),
        }
    }

    if escaped {
        literal.push('\\');
    }

    (literal, index)
}

fn is_token_word_start(ch: char) -> bool {
    is_identifier_continue(ch)
}

fn read_token_word(source: &str, start: usize) -> (&str, usize) {
    let mut end = start;

    for (relative_index, ch) in source[start..].char_indices() {
        if is_identifier_continue(ch) {
            end = start + relative_index + ch.len_utf8();
        } else {
            break;
        }
    }

    (&source[start..end], end)
}

fn classify_word_token_with_decimal_suffix(
    source: &str,
    start: usize,
    word: &str,
    word_end: usize,
) -> (ExpressionTokenKind, usize) {
    let kind = classify_word_token(word);
    if !matches!(kind, ExpressionTokenKind::IntLiteral(_)) {
        return (kind, word_end);
    }

    let Some(fraction_start) = source[word_end..]
        .strip_prefix('.')
        .map(|_| word_end + '.'.len_utf8())
    else {
        return (kind, word_end);
    };

    let mut fraction_end = fraction_start;
    for (relative_index, ch) in source[fraction_start..].char_indices() {
        if ch.is_ascii_digit() {
            fraction_end = fraction_start + relative_index + ch.len_utf8();
        } else {
            break;
        }
    }

    if fraction_end == fraction_start {
        return (kind, word_end);
    }

    (
        ExpressionTokenKind::FloatLiteral(source[start..fraction_end].to_string()),
        fraction_end,
    )
}

fn classify_word_token(word: &str) -> ExpressionTokenKind {
    if binary_operator_rule(word).is_some() || word == "not" {
        return ExpressionTokenKind::Operator(word.to_string());
    }

    if word.chars().all(|ch| ch.is_ascii_digit()) {
        return ExpressionTokenKind::IntLiteral(word.to_string());
    }

    if word.contains('.')
        && word.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
        && word.parse::<f64>().is_ok()
    {
        return ExpressionTokenKind::FloatLiteral(word.to_string());
    }

    ExpressionTokenKind::Identifier(word.to_string())
}
