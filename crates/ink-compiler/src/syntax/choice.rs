use crate::parsed::{BinaryOperator, Choice, ContentList, Expression};

use super::{rule::RuleParser, text};

pub(super) fn parse_choice(parser: &mut RuleParser<'_>) -> Option<Choice> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();

    // Match one or more '*' (once-only) or '+' (sticky) bullets.
    let (bullet, once_only) = if parser.match_string("*").is_some() {
        ('*', true)
    } else if parser.match_string("+").is_some() {
        ('+', false)
    } else {
        return None;
    };

    let mut indentation_depth = 1;
    loop {
        parser.skip_horizontal_whitespace();
        let matched = match bullet {
            '*' => parser.match_string("*").is_some(),
            '+' => parser.match_string("+").is_some(),
            _ => false,
        };
        if !matched {
            break;
        }
        indentation_depth += 1;
    }

    parser.skip_horizontal_whitespace();

    let choice_body = parser.expect(
        "choice text",
        |parser| {
            let text = parser.line_remainder().to_string();
            parser.skip_to_end();
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        },
        |parser| parser.skip_to_end(),
    )?;

    let (identifier, choice_body) = parse_choice_identifier(&choice_body);
    let (condition, choice_body) = parse_choice_conditions(&choice_body)?;

    // Handle fallback choices like "* -> " which have no text content
    // and the -> is a divert to empty (fall through to gather)
    let trimmed_body = choice_body.trim();
    if trimmed_body == "->" || trimmed_body.is_empty() {
        // Invisible default choice - inner content is just a newline
        let inner = append_newline(ContentList::new(vec![]), span.clone());
        let mut choice = Choice::new_with_inline_brackets(
            None, // no start content
            None, // no choice-only content
            inner, span, false, // no inline brackets
        );
        choice.set_identifier(identifier);
        choice.set_once_only(once_only);
        choice.set_is_invisible_default(true);
        choice.set_condition(condition);
        choice.set_indentation_depth(indentation_depth);
        return Some(choice);
    }

    let segments = parse_choice_segments(&choice_body)
        .map_err(|message| {
            parser.error(message);
        })
        .ok()?;

    let mut choice = Choice::new_with_inline_brackets(
        content_list_from_segment(segments.start, span.clone(), false, false),
        segments.choice_only.map(|segment| {
            content_list_from_segment(segment, span.clone(), true, true).unwrap_or_default()
        }),
        append_newline(
            content_list_from_segment(segments.inner, span.clone(), true, true).unwrap_or_default(),
            span.clone(),
        ),
        span,
        segments.has_inline_brackets,
    );
    choice.set_identifier(identifier);
    choice.set_once_only(once_only);
    choice.set_condition(condition);
    choice.set_indentation_depth(indentation_depth);

    // Check if this is an invisible default (empty content)
    let is_invisible_default = !choice.has_start_content()
        && !choice.has_choice_only_content()
        && choice.inner_content().objects().len() <= 1; // Only newline
    choice.set_is_invisible_default(is_invisible_default);

    Some(choice)
}

fn parse_choice_identifier(choice_body: &str) -> (Option<String>, String) {
    let remaining = choice_body.trim_start();
    let Some(after_open) = remaining.strip_prefix('(') else {
        return (None, choice_body.to_string());
    };
    let Some(close_index) = after_open.find(')') else {
        return (None, choice_body.to_string());
    };
    let name = after_open[..close_index].trim();
    if !is_identifier(name) {
        return (None, choice_body.to_string());
    }
    (
        Some(name.to_string()),
        after_open[close_index + 1..].trim_start().to_string(),
    )
}

fn parse_choice_conditions(choice_body: &str) -> Option<(Option<Expression>, String)> {
    let mut remaining = choice_body.trim_start();
    let mut conditions = Vec::new();

    while let Some(after_open) = remaining.strip_prefix('{') {
        let close_index = after_open.find('}')?;
        let condition = parse_condition_expression(&after_open[..close_index])?;
        conditions.push(condition);
        remaining = after_open[close_index + 1..].trim_start();
    }

    Some((
        Expression::multiple_condition(conditions),
        remaining.to_string(),
    ))
}

fn parse_condition_expression(source: &str) -> Option<Expression> {
    let terms = source
        .split(" and ")
        .map(|term| parse_bool_literal(term.trim()))
        .collect::<Option<Vec<_>>>()?;

    let mut terms = terms.into_iter();
    let first = terms.next()?;
    Some(terms.fold(first, |left, right| Expression::Binary {
        operator: BinaryOperator::And,
        left: Box::new(left),
        right: Box::new(right),
    }))
}

fn parse_bool_literal(source: &str) -> Option<Expression> {
    match source {
        "true" => Some(Expression::NumberBool(true)),
        "false" => Some(Expression::NumberBool(false)),
        _ if is_path_identifier(source) => Some(Expression::VariableReference(source.to_string())),
        _ => None,
    }
}

fn is_path_identifier(source: &str) -> bool {
    source.split('.').all(is_identifier)
}

fn is_identifier(source: &str) -> bool {
    let mut chars = source.chars();
    matches!(chars.next(), Some(ch) if ch == '_' || ch.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

struct ChoiceSegments {
    start: String,
    choice_only: Option<String>,
    inner: String,
    has_inline_brackets: bool,
}

fn parse_choice_segments(choice_body: &str) -> Result<ChoiceSegments, &'static str> {
    let Some(open_index) = choice_body.find('[') else {
        if choice_body.contains(']') {
            return Err("Expected opening '[' for weave-style option but saw ']'");
        }

        return Ok(ChoiceSegments {
            start: choice_body.to_string(),
            choice_only: None,
            inner: String::new(),
            has_inline_brackets: false,
        });
    };

    let close_index = choice_body[open_index + 1..]
        .find(']')
        .map(|relative| open_index + 1 + relative)
        .ok_or("Expected closing ']' for weave-style option but saw end of line")?;

    if choice_body[open_index + 1..close_index].contains('[')
        || choice_body[open_index + 1..close_index].contains(']')
    {
        return Err("unsupported syntax: choice");
    }

    if choice_body[close_index + 1..].contains('[') || choice_body[close_index + 1..].contains(']')
    {
        return Err("unsupported syntax: choice");
    }

    Ok(ChoiceSegments {
        start: choice_body[..open_index].to_string(),
        choice_only: (!choice_body[open_index + 1..close_index].is_empty())
            .then(|| choice_body[open_index + 1..close_index].to_string()),
        inner: choice_body[close_index + 1..].to_string(),
        has_inline_brackets: true,
    })
}

fn content_list_from_segment(
    segment: String,
    span: crate::source::SourceSpan,
    keep_empty: bool,
    preserve_divert_whitespace: bool,
) -> Option<ContentList> {
    if segment.is_empty() && !keep_empty {
        return None;
    }

    let objects = if preserve_divert_whitespace {
        text::parse_inline_content_preserving_divert_whitespace(&segment, &span)
    } else {
        text::parse_inline_content(&segment, &span)
    }
    .unwrap_or_default();
    Some(ContentList::new(objects))
}

fn append_newline(mut content: ContentList, span: crate::source::SourceSpan) -> ContentList {
    content.push(crate::parsed::Object::Text(crate::parsed::Text::new(
        "\n", span,
    )));
    content
}
