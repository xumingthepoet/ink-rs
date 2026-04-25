use crate::{
    parsed::{BinaryOperator, ContentList, Expression, FloatLiteral, Object, UnaryOperator},
    source::SourceSpan,
};

use super::{is_identifier, is_identifier_continue, scan, text};

pub(super) fn parse_initial_expression(source: &str) -> Option<Expression> {
    let _tokens = tokenize_expression(source.trim());
    parse_expression(source.trim())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpressionToken {
    kind: ExpressionTokenKind,
    byte_index: usize,
    span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpressionTokenKind {
    Identifier(String),
    IntLiteral(String),
    FloatLiteral(String),
    StringLiteral(String),
    Operator(String),
    OpenParen,
    CloseParen,
    Comma,
    Arrow,
}

fn tokenize_expression(source: &str) -> Vec<ExpressionToken> {
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
            tokens.push(token_at(source, index, ExpressionTokenKind::Arrow));
            index += "->".len();
            continue;
        }

        if let Some(operator) = match_operator(rest) {
            tokens.push(token_at(
                source,
                index,
                ExpressionTokenKind::Operator(operator.to_string()),
            ));
            index += operator.len();
            continue;
        }

        match ch {
            '(' => {
                tokens.push(token_at(source, index, ExpressionTokenKind::OpenParen));
                index += ch.len_utf8();
            }
            ')' => {
                tokens.push(token_at(source, index, ExpressionTokenKind::CloseParen));
                index += ch.len_utf8();
            }
            ',' => {
                tokens.push(token_at(source, index, ExpressionTokenKind::Comma));
                index += ch.len_utf8();
            }
            '"' => {
                let (literal, next_index) = read_string_literal(source, index);
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::StringLiteral(literal),
                ));
                index = next_index;
            }
            _ if is_token_word_start(ch) => {
                let (word, next_index) = read_token_word(source, index);
                tokens.push(token_at(source, index, classify_word_token(word)));
                index = next_index;
            }
            _ => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::Operator(ch.to_string()),
                ));
                index += ch.len_utf8();
            }
        }
    }

    tokens
}

fn token_at(source: &str, byte_index: usize, kind: ExpressionTokenKind) -> ExpressionToken {
    ExpressionToken {
        kind,
        byte_index,
        span: SourceSpan::new(None, 1, source[..byte_index].chars().count() + 1),
    }
}

fn match_operator(source: &str) -> Option<&'static str> {
    ["&&", "||", "==", "!=", ">=", "<=", "!?"]
        .into_iter()
        .find(|operator| source.starts_with(operator))
}

fn read_string_literal(source: &str, start: usize) -> (String, usize) {
    let mut literal = String::new();
    let mut index = start + '"'.len_utf8();
    let mut escaped = false;

    while index < source.len() {
        let ch = source[index..]
            .chars()
            .next()
            .expect("index is inside source");
        index += ch.len_utf8();

        if escaped {
            literal.push('\\');
            literal.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return (literal, index),
            _ => literal.push(ch),
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
        if is_identifier_continue(ch) || ch == '.' {
            end = start + relative_index + ch.len_utf8();
        } else {
            break;
        }
    }

    (&source[start..end], end)
}

fn classify_word_token(word: &str) -> ExpressionTokenKind {
    if matches!(word, "and" | "or" | "has" | "hasnt" | "mod" | "not") {
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

fn parse_expression(source: &str) -> Option<Expression> {
    let source = strip_enclosing_parentheses(source.trim());
    if let Some((left, operator, right)) = split_top_level_text_operators(
        source,
        &[
            TextOperator::symbol("&&", BinaryOperator::AndSymbol),
            TextOperator::symbol("||", BinaryOperator::OrSymbol),
            TextOperator::word("and", BinaryOperator::And),
            TextOperator::word("or", BinaryOperator::Or),
        ],
    ) {
        return Some(Expression::Binary {
            operator,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some((left, operator, right)) = split_top_level_word_operator(
        source,
        &[
            ("==", BinaryOperator::Equals),
            ("!=", BinaryOperator::NotEquals),
            (">=", BinaryOperator::GreaterThanOrEquals),
            ("<=", BinaryOperator::LessThanOrEquals),
            (">", BinaryOperator::GreaterThan),
            ("<", BinaryOperator::LessThan),
        ],
    ) {
        return Some(Expression::Binary {
            operator,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some((left, operator, right)) =
        split_top_level_word_operator(source, &[("!?", BinaryOperator::Hasnt)])
    {
        return Some(Expression::Binary {
            operator,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some((left, operator, right)) = split_top_level_text_operators(
        source,
        &[
            TextOperator::word("hasnt", BinaryOperator::Hasnt),
            TextOperator::word("has", BinaryOperator::Has),
            TextOperator::symbol("?", BinaryOperator::Has),
        ],
    ) {
        return Some(Expression::Binary {
            operator,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some((left, operator, right)) = split_top_level_operator(
        source,
        &[("+", BinaryOperator::Add), ("-", BinaryOperator::Subtract)],
    ) {
        return Some(Expression::Binary {
            operator,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some((left, operator, right)) =
        split_top_level_operator(source, &[("*", BinaryOperator::Multiply)])
    {
        return Some(Expression::Binary {
            operator,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some((left, operator, right)) =
        split_top_level_operator(source, &[("/", BinaryOperator::Divide)])
    {
        return Some(Expression::Binary {
            operator,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some((left, right)) = split_top_level_word_text_operator(source, "mod") {
        return Some(Expression::Binary {
            operator: BinaryOperator::Modulo,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some((left, operator, right)) =
        split_top_level_operator(source, &[("%", BinaryOperator::Modulo)])
    {
        return Some(Expression::Binary {
            operator,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some(value) = parse_quoted_string_literal(source) {
        return Some(parse_string_expression(&value));
    }
    if let Some((name, args)) = parse_function_call(source) {
        return Some(Expression::FunctionCall { name, args });
    }
    if let Some(target) = source.strip_prefix("->") {
        return Some(Expression::DivertTarget(
            crate::parsed::DivertTarget::from_source(target).to_snapshot_string(),
        ));
    }
    if let Some((operator, inner)) = parse_unary_prefix(source) {
        return Some(unary_expression(operator, parse_expression(inner)?));
    }
    if source == "true" {
        return Some(Expression::NumberBool(true));
    }
    if source == "false" {
        return Some(Expression::NumberBool(false));
    }
    if let Ok(value) = source.parse::<i32>() {
        return Some(Expression::NumberInt(value));
    }
    if source.contains('.') {
        if let Ok(value) = source.parse::<f64>() {
            return Some(Expression::NumberFloat(FloatLiteral::new(value)));
        }
    }
    is_path_identifier(source).then(|| Expression::VariableReference(source.to_string()))
}

fn parse_unary_prefix(source: &str) -> Option<(UnaryOperator, &str)> {
    if let Some(inner) = source.strip_prefix('-') {
        return Some((UnaryOperator::Negate, inner.trim_start()));
    }
    if let Some(inner) = source.strip_prefix('!') {
        return Some((UnaryOperator::Not, inner.trim_start()));
    }
    if let Some(inner) = source.strip_prefix("not") {
        let next_is_identifier = inner.chars().next().is_some_and(is_identifier_continue);
        if !next_is_identifier {
            return Some((UnaryOperator::Not, inner.trim_start()));
        }
    }
    None
}

fn unary_expression(operator: UnaryOperator, expression: Expression) -> Expression {
    match (operator, expression) {
        (UnaryOperator::Negate, Expression::NumberInt(value)) => Expression::NumberInt(-value),
        (UnaryOperator::Negate, Expression::NumberFloat(value)) => {
            Expression::NumberFloat(FloatLiteral::new(-value.value()))
        }
        (operator, expression) => Expression::Unary {
            operator,
            expression: Box::new(expression),
        },
    }
}

fn parse_function_call(source: &str) -> Option<(String, Vec<Expression>)> {
    let open_index = source.find('(')?;
    if !source.ends_with(')') {
        return None;
    }

    let name = source[..open_index].trim();
    if !is_identifier(name) {
        return None;
    }

    let args_source = &source[open_index + 1..source.len() - 1];
    let args = if args_source.trim().is_empty() {
        Vec::new()
    } else {
        split_top_level_args(args_source)
            .into_iter()
            .map(parse_expression)
            .collect::<Option<Vec<_>>>()?
    };
    Some((name.to_string(), args))
}

fn parse_string_expression(value: &str) -> Expression {
    let span = crate::source::SourceSpan::new(None, 1, 1);
    let Some(objects) = text::parse_inline_content(value, &span) else {
        return Expression::String(value.to_string());
    };
    let objects = flatten_string_expression_content(objects);

    if objects.len() == 1 {
        if let Object::Text(text) = &objects[0] {
            if text.text() == value {
                return Expression::String(value.to_string());
            }
        }
    }

    Expression::StringContent(ContentList::new(objects))
}

fn flatten_string_expression_content(objects: Vec<Object>) -> Vec<Object> {
    let mut flattened = Vec::new();
    for object in objects {
        match object {
            Object::ContentList(content) => flattened.extend(content.objects().iter().cloned()),
            other => flattened.push(other),
        }
    }
    flattened
}

pub(super) fn split_top_level_args(source: &str) -> Vec<&str> {
    scan::split_top_level_with_options(source, ',', scan::ScanOptions::expression())
}

fn split_top_level_operator<'a>(
    source: &'a str,
    operators: &[(&'static str, BinaryOperator)],
) -> Option<(&'a str, BinaryOperator, &'a str)> {
    split_top_level_operator_text(source, operators, false, true)
}

fn split_top_level_word_operator<'a>(
    source: &'a str,
    operators: &[(&'static str, BinaryOperator)],
) -> Option<(&'a str, BinaryOperator, &'a str)> {
    split_top_level_operator_text(source, operators, false, true)
}

fn split_top_level_operator_text<'a>(
    source: &'a str,
    operators: &[(&'static str, BinaryOperator)],
    require_word_boundaries: bool,
    reject_unary_position: bool,
) -> Option<(&'a str, BinaryOperator, &'a str)> {
    let tokens = operators
        .iter()
        .map(|(operator_text, _)| *operator_text)
        .collect::<Vec<_>>();
    scan::top_level_token_matches_with_options(source, &tokens, scan::ScanOptions::expression())
        .into_iter()
        .rev()
        .find_map(|(index, operator_text)| {
            if require_word_boundaries && !has_word_boundaries(source, index, operator_text.len()) {
                return None;
            }

            let (_, operator) = operators
                .iter()
                .find(|(candidate, _)| *candidate == operator_text)?;
            let left = source[..index].trim();
            let right = source[index + operator_text.len()..].trim();

            if left.is_empty() || right.is_empty() {
                return None;
            }

            if reject_unary_position && is_unary_operator_position(source, index) {
                return None;
            }

            Some((left, *operator, right))
        })
}

fn split_top_level_word_text_operator<'a>(
    source: &'a str,
    operator: &str,
) -> Option<(&'a str, &'a str)> {
    split_top_level_text_operator_with_boundaries(source, operator, true)
}

#[derive(Clone, Copy)]
struct TextOperator {
    text: &'static str,
    operator: BinaryOperator,
    require_word_boundaries: bool,
}

impl TextOperator {
    fn symbol(text: &'static str, operator: BinaryOperator) -> Self {
        Self {
            text,
            operator,
            require_word_boundaries: false,
        }
    }

    fn word(text: &'static str, operator: BinaryOperator) -> Self {
        Self {
            text,
            operator,
            require_word_boundaries: true,
        }
    }
}

fn split_top_level_text_operators<'a>(
    source: &'a str,
    operators: &[TextOperator],
) -> Option<(&'a str, BinaryOperator, &'a str)> {
    let tokens = operators
        .iter()
        .map(|operator| operator.text)
        .collect::<Vec<_>>();
    scan::top_level_token_matches_with_options(source, &tokens, scan::ScanOptions::expression())
        .into_iter()
        .rev()
        .find_map(|(index, operator_text)| {
            let operator = operators
                .iter()
                .find(|operator| operator.text == operator_text)?;

            if operator.require_word_boundaries
                && !has_word_boundaries(source, index, operator.text.len())
            {
                return None;
            }

            let left = source[..index].trim();
            let right = source[index + operator.text.len()..].trim();
            if left.is_empty() || right.is_empty() {
                return None;
            }

            Some((left, operator.operator, right))
        })
}

fn split_top_level_text_operator_with_boundaries<'a>(
    source: &'a str,
    operator: &str,
    require_word_boundaries: bool,
) -> Option<(&'a str, &'a str)> {
    scan::top_level_token_matches_with_options(source, &[operator], scan::ScanOptions::expression())
        .into_iter()
        .rev()
        .find_map(|(index, _)| {
            if require_word_boundaries && !has_word_boundaries(source, index, operator.len()) {
                return None;
            }

            let left = source[..index].trim();
            let right = source[index + operator.len()..].trim();
            if left.is_empty() || right.is_empty() {
                return None;
            }

            Some((left, right))
        })
}

fn has_word_boundaries(source: &str, index: usize, length: usize) -> bool {
    let before = source[..index].chars().next_back();
    let after = source[index + length..].chars().next();
    before.is_none_or(|ch| !is_identifier_continue(ch))
        && after.is_none_or(|ch| !is_identifier_continue(ch))
}

fn is_unary_operator_position(source: &str, index: usize) -> bool {
    let left = source[..index].trim_end();
    let Some(previous) = left.chars().next_back() else {
        return true;
    };
    matches!(
        previous,
        '(' | ',' | '+' | '-' | '*' | '/' | '%' | '<' | '>' | '=' | '!' | '?' | '^'
    )
}

fn strip_enclosing_parentheses(source: &str) -> &str {
    let mut current = source.trim();
    loop {
        let Some(inner) = current
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
        else {
            return current;
        };
        if !parentheses_wrap_entire_expression(current) {
            return current;
        }
        current = inner.trim();
    }
}

fn parentheses_wrap_entire_expression(source: &str) -> bool {
    let Some(after_open) = source.strip_prefix('(') else {
        return false;
    };
    scan::find_matching_delimiter(after_open, '(', ')')
        .is_some_and(|close_index| close_index + 1 == after_open.len())
}

fn parse_quoted_string_literal(source: &str) -> Option<String> {
    let mut chars = source.chars();
    if chars.next()? != '"' {
        return None;
    }

    let mut value = String::new();
    let mut escaped = false;
    let mut brace_depth = 0;
    let mut in_nested_string = false;
    for ch in chars.by_ref() {
        if escaped {
            if brace_depth == 0 {
                match ch {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    other => value.push(other),
                }
            } else {
                value.push('\\');
                value.push(ch);
            }
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' if brace_depth == 0 => {
                return if chars.as_str().trim().is_empty() {
                    Some(value)
                } else {
                    None
                };
            }
            '"' => {
                in_nested_string = !in_nested_string;
                value.push(ch);
            }
            '{' if !in_nested_string => {
                brace_depth += 1;
                value.push(ch);
            }
            '}' if !in_nested_string && brace_depth > 0 => {
                brace_depth -= 1;
                value.push(ch);
            }
            other => value.push(other),
        }
    }

    None
}

fn is_path_identifier(source: &str) -> bool {
    source.split('.').all(is_identifier)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(source: &str) -> String {
        let expression = parse_initial_expression(source)
            .unwrap_or_else(|| panic!("expected expression for {source:?}"));
        let mut output = String::new();
        expression.write_parse_snapshot(&mut output, 0);
        output
    }

    fn token(kind: ExpressionTokenKind, byte_index: usize, column: usize) -> ExpressionToken {
        ExpressionToken {
            kind,
            byte_index,
            span: SourceSpan::new(None, 1, column),
        }
    }

    #[test]
    fn parses_current_expression_behavior_baseline() {
        let cases = [
            (
                "1 + 2 * 3",
                "Binary(+, Number(1), Binary(*, Number(2), Number(3)))",
            ),
            (
                "(1 + 2) * 3",
                "Binary(*, Binary(+, Number(1), Number(2)), Number(3))",
            ),
            (
                "1 - 2 - 3",
                "Binary(-, Binary(-, Number(1), Number(2)), Number(3))",
            ),
            (
                "8 / 4 / 2",
                "Binary(/, Binary(/, Number(8), Number(4)), Number(2))",
            ),
            ("8 mod 3", "Binary(%, Number(8), Number(3))"),
            ("8 % 3", "Binary(%, Number(8), Number(3))"),
            ("-5", "Number(-5)"),
            ("-x", "Unary(-, VariableReference(x))"),
            ("not ready", "Unary(not, VariableReference(ready))"),
            ("!ready", "Unary(not, VariableReference(ready))"),
            ("notebook", "VariableReference(notebook)"),
            ("true", "Number(true)"),
            ("false", "Number(false)"),
            ("3.5", "Number(3.5)"),
            (r#""hello""#, r#"String("hello")"#),
            ("foo(1, bar(2, 3), \"x,y\")", "FunctionCall(foo, args=3)"),
            ("-> knot.stitch", "DivertTarget(-> knot.stitch)"),
            ("knot.stitch.label", "VariableReference(knot.stitch.label)"),
            (
                "a and b or c",
                "Binary(or, Binary(and, VariableReference(a), VariableReference(b)), VariableReference(c))",
            ),
            (
                "a && b || c",
                "Binary(||, Binary(&&, VariableReference(a), VariableReference(b)), VariableReference(c))",
            ),
            (
                "list ? item",
                "Binary(?, VariableReference(list), VariableReference(item))",
            ),
            (
                "list hasnt item",
                "Binary(!?, VariableReference(list), VariableReference(item))",
            ),
            (
                "a >= b",
                "Binary(>=, VariableReference(a), VariableReference(b))",
            ),
            (
                "1 < 2 == true",
                "Binary(==, Binary(<, Number(1), Number(2)), Number(true))",
            ),
        ];

        for (source, expected) in cases {
            assert_eq!(snapshot(source), expected, "source: {source}");
        }
    }

    #[test]
    fn tokenizer_covers_expression_token_categories() {
        let tokens = tokenize_expression(r#"foo(1, 2.5, "a,b", -> knot, list ? item)"#);

        assert_eq!(
            tokens,
            vec![
                token(ExpressionTokenKind::Identifier("foo".to_string()), 0, 1),
                token(ExpressionTokenKind::OpenParen, 3, 4),
                token(ExpressionTokenKind::IntLiteral("1".to_string()), 4, 5),
                token(ExpressionTokenKind::Comma, 5, 6),
                token(ExpressionTokenKind::FloatLiteral("2.5".to_string()), 7, 8),
                token(ExpressionTokenKind::Comma, 10, 11),
                token(
                    ExpressionTokenKind::StringLiteral("a,b".to_string()),
                    12,
                    13
                ),
                token(ExpressionTokenKind::Comma, 17, 18),
                token(ExpressionTokenKind::Arrow, 19, 20),
                token(ExpressionTokenKind::Identifier("knot".to_string()), 22, 23),
                token(ExpressionTokenKind::Comma, 26, 27),
                token(ExpressionTokenKind::Identifier("list".to_string()), 28, 29),
                token(ExpressionTokenKind::Operator("?".to_string()), 33, 34),
                token(ExpressionTokenKind::Identifier("item".to_string()), 35, 36),
                token(ExpressionTokenKind::CloseParen, 39, 40),
            ]
        );
    }

    #[test]
    fn tokenizer_keeps_word_operators_distinct_from_identifiers() {
        let tokens = tokenize_expression("not ready and notebook or list hasnt item mod 2");

        assert_eq!(
            tokens,
            vec![
                token(ExpressionTokenKind::Operator("not".to_string()), 0, 1),
                token(ExpressionTokenKind::Identifier("ready".to_string()), 4, 5),
                token(ExpressionTokenKind::Operator("and".to_string()), 10, 11),
                token(
                    ExpressionTokenKind::Identifier("notebook".to_string()),
                    14,
                    15
                ),
                token(ExpressionTokenKind::Operator("or".to_string()), 23, 24),
                token(ExpressionTokenKind::Identifier("list".to_string()), 26, 27),
                token(ExpressionTokenKind::Operator("hasnt".to_string()), 31, 32),
                token(ExpressionTokenKind::Identifier("item".to_string()), 37, 38),
                token(ExpressionTokenKind::Operator("mod".to_string()), 42, 43),
                token(ExpressionTokenKind::IntLiteral("2".to_string()), 46, 47),
            ]
        );
    }

    #[test]
    fn tokenizer_covers_symbol_operators_and_paths() {
        let tokens = tokenize_expression("a.b >= c && x != y || z <= 3");

        assert_eq!(
            tokens,
            vec![
                token(ExpressionTokenKind::Identifier("a.b".to_string()), 0, 1),
                token(ExpressionTokenKind::Operator(">=".to_string()), 4, 5),
                token(ExpressionTokenKind::Identifier("c".to_string()), 7, 8),
                token(ExpressionTokenKind::Operator("&&".to_string()), 9, 10),
                token(ExpressionTokenKind::Identifier("x".to_string()), 12, 13),
                token(ExpressionTokenKind::Operator("!=".to_string()), 14, 15),
                token(ExpressionTokenKind::Identifier("y".to_string()), 17, 18),
                token(ExpressionTokenKind::Operator("||".to_string()), 19, 20),
                token(ExpressionTokenKind::Identifier("z".to_string()), 22, 23),
                token(ExpressionTokenKind::Operator("<=".to_string()), 24, 25),
                token(ExpressionTokenKind::IntLiteral("3".to_string()), 27, 28),
            ]
        );
    }

    #[test]
    fn tokenizer_tracks_character_columns_for_unicode_prefixes() {
        let tokens = tokenize_expression("变量 + \"ok\"");

        assert_eq!(
            tokens,
            vec![
                token(ExpressionTokenKind::Identifier("变量".to_string()), 0, 1),
                token(ExpressionTokenKind::Operator("+".to_string()), 7, 4),
                token(ExpressionTokenKind::StringLiteral("ok".to_string()), 9, 6),
            ]
        );
    }
}
