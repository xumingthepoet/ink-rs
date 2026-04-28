use crate::parsed::{Choice, ContentList, Expression};

use super::{is_identifier, rule::RuleParser, scan, text};

pub(super) fn parse_choice(parser: &mut RuleParser<'_>) -> Option<Choice> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();

    // `*` and `+` are both repeatable in ink-rs. The bullet still controls
    // nesting style by repetition, but no longer changes runtime visibility.
    let bullet = if parser.match_string("*").is_some() {
        '*'
    } else if parser.match_string("+").is_some() {
        '+'
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

    let choice_body = parser.line_remainder().to_string();
    parser.skip_to_end();

    let (identifier, choice_body) = parse_choice_identifier(&choice_body);
    let (condition, choice_body) = parse_choice_conditions(&choice_body)?;

    // Handle fallback choices like "* -> " which have no text content
    // and the -> is a divert to empty (fall through to gather)
    let trimmed_body = choice_body.trim();
    if trimmed_body == "->" || trimmed_body.is_empty() {
        if trimmed_body.is_empty() {
            parser.warning(
                "Choice is completely empty. Interpretting as a default fallback choice. Add a divert arrow to remove this warning: * ->",
            );
        }

        // Invisible default choice - inner content is just a newline
        let inner = append_newline(ContentList::new(vec![]), span.clone());
        let mut choice = Choice::new(None, inner, span);
        choice.set_identifier(identifier);
        choice.set_once_only(false);
        choice.set_is_invisible_default(true);
        choice.set_condition(condition);
        choice.set_indentation_depth(indentation_depth);
        return Some(choice);
    }

    let is_divert_only_choice = find_top_level_divert(&choice_body)
        .is_some_and(|divert_index| choice_body[..divert_index].trim().is_empty());

    let segments = parse_choice_segments(&choice_body);

    let start_content = content_list_from_segment(segments.start, span.clone(), false, false);

    let mut choice = Choice::new(
        start_content,
        append_newline(
            content_list_from_segment(segments.inner, span.clone(), true, true).unwrap_or_default(),
            span.clone(),
        ),
        span,
    );
    choice.set_identifier(identifier);
    choice.set_once_only(false);
    choice.set_condition(condition);
    choice.set_indentation_depth(indentation_depth);

    // Check if this is an invisible default (empty content)
    let is_invisible_default = is_divert_only_choice
        || (!choice.has_start_content() && choice.inner_content().objects().len() <= 1); // Only newline
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
        let close_index = scan::find_matching_delimiter(after_open, '{', '}')?;
        let condition = parse_condition_expression(&after_open[..close_index])?;
        conditions.push(condition);
        remaining = after_open[close_index + 1..].trim_start();
        if let Some(after_boundary) = remaining.strip_prefix(':') {
            remaining = after_boundary.trim_start();
            break;
        }
    }

    Some((
        Expression::multiple_condition(conditions),
        remaining.to_string(),
    ))
}

fn parse_condition_expression(source: &str) -> Option<Expression> {
    super::parse_initial_expression(source)
}

struct ChoiceSegments {
    start: String,
    inner: String,
}

fn parse_choice_segments(choice_body: &str) -> ChoiceSegments {
    if let Some(divert_index) = find_top_level_divert(choice_body) {
        return ChoiceSegments {
            start: choice_body[..divert_index].to_string(),
            inner: choice_body[divert_index..].to_string(),
        };
    }

    ChoiceSegments {
        start: choice_body.to_string(),
        inner: String::new(),
    }
}

fn find_top_level_divert(source: &str) -> Option<usize> {
    scan::find_top_level_token(source, &["->"]).map(|(index, _)| index)
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

#[cfg(test)]
mod tests {
    use crate::{
        parsed::Object,
        source::{SourceLine, SourceSpan},
        syntax::rule::RuleParser,
    };

    use super::*;

    fn parse_choice_line(source: &str) -> Choice {
        let line = SourceLine {
            text: source.to_string(),
            span: SourceSpan::new(None, 1, 1),
        };
        let mut parser = RuleParser::new(&line);
        let choice = parse_choice(&mut parser).expect("expected choice");
        assert!(!parser
            .finish()
            .iter()
            .any(|diagnostic| diagnostic.severity == crate::diagnostic::DiagnosticSeverity::Error));
        choice
    }

    fn assert_single_dynamic_text(choice: &Choice, expected_name: &str) {
        let start_content = choice.start_content().expect("expected display text");
        assert!(matches!(
            start_content.objects(),
            [Object::ContentList(content)]
                if matches!(
                    content.objects(),
                    [Object::Expression(Expression::VariableReference(name))]
                        if name == expected_name
                )
        ));
    }

    #[test]
    fn top_level_divert_ignores_arrows_inside_braced_strings() {
        assert_eq!(
            find_top_level_divert(r#"visible {"->"} -> target"#),
            Some(15)
        );
        assert_eq!(find_top_level_divert(r#"visible {"->"}"#), None);
    }

    #[test]
    fn colon_ends_choice_condition_before_dynamic_text() {
        let choice = parse_choice_line("* {enabled}: {label}");

        assert!(matches!(
            choice.condition(),
            Some(Expression::VariableReference(name)) if name == "enabled"
        ));
        assert_single_dynamic_text(&choice, "label");
    }

    #[test]
    fn colon_ends_multiple_choice_conditions_before_dynamic_text() {
        let choice = parse_choice_line("* {enabled} {visible}: {label}");

        let Some(Expression::MultipleCondition(conditions)) = choice.condition() else {
            panic!("expected multiple condition");
        };
        assert_eq!(conditions.len(), 2);
        assert!(matches!(
            &conditions[0],
            Expression::VariableReference(name) if name == "enabled"
        ));
        assert!(matches!(
            &conditions[1],
            Expression::VariableReference(name) if name == "visible"
        ));
        assert_single_dynamic_text(&choice, "label");
    }

    #[test]
    fn adjacent_leading_braces_remain_multiple_conditions() {
        let choice = parse_choice_line("* {enabled}{label}");

        let Some(Expression::MultipleCondition(conditions)) = choice.condition() else {
            panic!("expected multiple condition");
        };
        assert_eq!(conditions.len(), 2);
        assert!(choice.start_content().is_none());
    }

    #[test]
    fn multiple_conditions_without_colon_keep_existing_text_boundary() {
        let choice = parse_choice_line("* {enabled} {visible} Label");

        let Some(Expression::MultipleCondition(conditions)) = choice.condition() else {
            panic!("expected multiple condition");
        };
        assert_eq!(conditions.len(), 2);
        let start_content = choice.start_content().expect("expected display text");
        assert!(matches!(
            start_content.objects(),
            [Object::Text(text)] if text.text() == "Label"
        ));
    }
}
