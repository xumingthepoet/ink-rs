mod choice;
mod conditional;
mod declaration;
mod divert;
mod error;
mod gather;
mod knot;
mod logic;
mod rule;
mod scan;
mod sequence;
mod state;
mod text;
mod variable;
mod weave;

mod parser;

pub(crate) use parser::parse;

use crate::{
    parsed::{
        AuthorWarning, BinaryOperator, Choice, ContentList, Expression, FloatLiteral, Object,
        UnaryOperator,
    },
    source::SourceLine,
};

use self::rule::RuleParser;

fn parse_choice_from_line(line: &SourceLine) -> Option<Choice> {
    let mut line_parser = RuleParser::new(line);
    let choice = line_parser.parse_rule(choice::parse_choice);
    if line_parser.had_error() {
        None
    } else {
        choice
    }
}

fn choice_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    choice::parse_choice(parser).map(|choice| vec![Object::Choice(choice)])
}

fn author_warning_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    let identifier = parser.take_while(is_identifier_continue)?;
    if identifier != "TODO" {
        return None;
    }
    parser.skip_horizontal_whitespace();
    let _ = parser.match_string(":");
    parser.skip_horizontal_whitespace();
    let message = parser.line_remainder().trim_end().to_string();
    parser.skip_to_end();
    Some(vec![Object::AuthorWarning(AuthorWarning::new(
        message, span,
    ))])
}

fn leading_whitespace_count(source: &str) -> usize {
    source
        .chars()
        .take_while(|ch| ch.is_whitespace() && *ch != '\n' && *ch != '\r')
        .count()
}

fn is_choice_continuation_boundary(trimmed: &str) -> bool {
    trimmed.starts_with('*')
        || trimmed.starts_with('+')
        || trimmed.starts_with('-')
        || trimmed.starts_with('=')
        || trimmed.starts_with("->")
        || knot::is_knot_declaration_line(trimmed)
        || knot::is_stitch_declaration_line(trimmed)
}

pub(super) fn parse_initial_expression(source: &str) -> Option<Expression> {
    parse_expression(source.trim())
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

pub(super) fn is_identifier(source: &str) -> bool {
    !source.is_empty()
        && source.chars().all(is_identifier_continue)
        && source.chars().any(|ch| !ch.is_ascii_digit())
}

fn is_path_identifier(source: &str) -> bool {
    source.split('.').all(is_identifier)
}

pub(super) fn is_identifier_start(ch: char) -> bool {
    is_identifier_continue(ch)
}

pub(super) fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric() || is_supported_unicode_identifier(ch)
}

fn is_supported_unicode_identifier(ch: char) -> bool {
    matches!(
        ch,
        '\u{0080}'..='\u{00ff}'
            | '\u{0100}'..='\u{017f}'
            | '\u{0180}'..='\u{024f}'
            | '\u{0370}'..='\u{03ff}'
            | '\u{0400}'..='\u{04ff}'
            | '\u{0531}'..='\u{0556}'
            | '\u{0561}'..='\u{0587}'
            | '\u{0590}'..='\u{05ff}'
            | '\u{0600}'..='\u{06ff}'
            | '\u{3041}'..='\u{3096}'
            | '\u{30a0}'..='\u{30fc}'
            | '\u{4e00}'..='\u{9fff}'
            | '\u{ac00}'..='\u{d7af}'
    ) && !matches!(ch, '\u{0374}' | '\u{0375}' | '\u{0378}'..='\u{0385}' | '\u{0387}' | '\u{038b}' | '\u{038d}' | '\u{03a2}' | '\u{0482}'..='\u{0489}')
}

fn divert_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    divert::parse_divert_objects(parser)
}

fn text_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    text::parse_text_line(parser)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::SourceInput;

    #[test]
    fn parses_plain_text_lines() {
        let output = parse(SourceInput::new("Line.\nOther line."));
        assert!(output.diagnostics.is_empty());
        let story = output.artifact.unwrap();
        assert_eq!(story.root_weave().content().len(), 4);
        assert!(story.flows().is_empty());
    }

    #[test]
    fn parses_choice() {
        let output = parse(SourceInput::new("* Choice"));
        assert!(output.diagnostics.is_empty());
        let story = output.artifact.unwrap();
        assert_eq!(story.root_weave().content().len(), 1);
    }

    #[test]
    fn parses_choice_after_same_line_gather() {
        let output = parse(SourceInput::new("- * Choice"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();
        assert!(matches!(story.root_weave().content()[0], Object::Gather(_)));
        assert!(matches!(story.root_weave().content()[1], Object::Choice(_)));
    }

    #[test]
    fn parses_inline_choice_segments() {
        let output = parse(SourceInput::new("* Hello [back!] right back to you!"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        assert_eq!(choice.start_content().unwrap().objects().len(), 1);
        assert_eq!(choice.choice_only_content().unwrap().objects().len(), 1);
        assert_eq!(choice.inner_content().objects().len(), 2);
        assert!(choice.has_weave_style_inline_brackets());
    }

    #[test]
    fn parses_choice_only_inline_choice() {
        let output = parse(SourceInput::new("* [Hello back!]"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        assert!(!choice.has_start_content());
        assert!(choice.has_choice_only_content());
        assert_eq!(choice.inner_content().objects().len(), 1);
    }

    #[test]
    fn parses_choice_with_inner_divert() {
        let output = parse(SourceInput::new("* [Open the gate] -> paragraph_2"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        assert!(matches!(
            choice.inner_content().objects()[1],
            Object::Divert(_)
        ));
    }

    #[test]
    fn parses_choice_condition_with_dotted_path() {
        let output = parse(SourceInput::new("* {knot.stitch.label} Text"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        let Some(crate::parsed::Expression::VariableReference(name)) = choice.condition() else {
            panic!("expected variable reference condition");
        };
        assert_eq!(name, "knot.stitch.label");
    }

    #[test]
    fn parses_empty_inline_choice_brackets_without_choice_only_content() {
        let output = parse(SourceInput::new("* Text[] inner"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        assert!(choice.has_weave_style_inline_brackets());
        assert!(!choice.has_choice_only_content());
    }

    #[test]
    fn parses_inline_divert_in_text() {
        let output = parse(SourceInput::new("A line. -> END"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        assert_eq!(story.root_weave().content().len(), 3);
        assert!(matches!(story.root_weave().content()[1], Object::Divert(_)));
    }

    #[test]
    fn parses_glue_and_inline_divert_in_text() {
        let output = parse(SourceInput::new("A line <> -> knot"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        assert!(matches!(story.root_weave().content()[0], Object::Text(_)));
        assert!(matches!(story.root_weave().content()[1], Object::Glue(_)));
        assert!(matches!(story.root_weave().content()[2], Object::Text(_)));
        assert!(matches!(story.root_weave().content()[3], Object::Divert(_)));
        assert!(matches!(story.root_weave().content()[4], Object::Text(_)));
    }

    #[test]
    fn trims_extra_separator_whitespace_before_terminal_divert() {
        let output = parse(SourceInput::new("<>as fast as we could.  -> END"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Text(text) = &story.root_weave().content()[1] else {
            panic!("expected text");
        };
        assert_eq!(text.text(), "as fast as we could. ");
    }

    #[test]
    fn parses_knot_definition() {
        let output = parse(SourceInput::new(
            "Top line.\n-> knot_name\n\n== knot_name ===\nInside knot. -> END",
        ));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        assert_eq!(story.flows().len(), 1);
        assert_eq!(story.flows()[0].name(), "knot_name");
        assert_eq!(story.flows()[0].weave().content().len(), 3);
    }

    #[test]
    fn parses_identifiers_that_start_with_numbers() {
        assert!(is_identifier("2tests"));
        assert!(is_identifier("512x2"));
        assert!(!is_identifier("512"));

        let output = parse(SourceInput::new("== 2tests ==\n-> DONE"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();
        assert_eq!(story.flows()[0].name(), "2tests");
    }

    #[test]
    fn parses_postfix_increment_logic_line() {
        let output = parse(SourceInput::new("~ x++\n~ x--"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::IncDec(increment) = &story.root_weave().content()[0] else {
            panic!("expected increment");
        };
        assert_eq!(increment.name(), "x");
        assert!(increment.is_increment());

        let Object::IncDec(decrement) = &story.root_weave().content()[1] else {
            panic!("expected decrement");
        };
        assert_eq!(decrement.name(), "x");
        assert!(!decrement.is_increment());
    }

    #[test]
    fn parses_return_without_expression() {
        let output = parse(SourceInput::new("=== function f() ===\n~ return"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Return(ret) = &story.flows()[0].weave().content()[0] else {
            panic!("expected return");
        };
        assert!(ret.returned_expression().is_none());
    }

    #[test]
    fn return_prefix_identifier_stays_expression() {
        let output = parse(SourceInput::new("~ returnValue()"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::LogicLine(Expression::FunctionCall { name, args }) =
            &story.root_weave().content()[0]
        else {
            panic!("expected function-call logic line");
        };
        assert_eq!(name, "returnValue");
        assert!(args.is_empty());
    }

    #[test]
    fn parses_sticky_choice() {
        let output = parse(SourceInput::new("+ Choice"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        assert!(!choice.once_only());
    }
}
