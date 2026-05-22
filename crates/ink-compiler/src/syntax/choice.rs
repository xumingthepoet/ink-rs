use std::collections::HashSet;

use crate::{
    diagnostic::Diagnostic,
    parsed::{Choice, ContentList, DynamicChoiceBinding, DynamicChoiceVariable, Expression},
    source::SourceSpan,
};

use super::{is_identifier, rule::RuleParser, scan, text};

pub(super) fn parse_choice(parser: &mut RuleParser<'_>) -> Option<Choice> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();

    if parser.match_string("+").is_some() {
        parser.diagnostic(Diagnostic::error(
            span,
            "`+` choices are not supported; use `*` for choices",
        ));
        return None;
    }

    if parser.match_string("*").is_none() {
        return None;
    }

    let mut indentation_depth = 1;
    loop {
        parser.skip_horizontal_whitespace();
        if parser.match_string("*").is_none() {
            break;
        }
        indentation_depth += 1;
    }

    parser.skip_horizontal_whitespace();

    let choice_body = parser.line_remainder().to_string();
    parser.skip_to_end();

    let (dynamic_binding, choice_body) = match parse_dynamic_choice_binding(&choice_body, &span) {
        Ok(parsed) => parsed,
        Err(message) => {
            parser.diagnostic(Diagnostic::error(span, message));
            return None;
        }
    };
    let (identifier, choice_body) = parse_choice_identifier(&choice_body);
    if dynamic_binding.is_some() && identifier.is_some() {
        parser.diagnostic(Diagnostic::error(
            span,
            "Dynamic choices do not support labels; use item data, explicit state, or gather labels instead",
        ));
        return None;
    }
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
        choice.set_dynamic_binding(dynamic_binding);
        choice.set_is_invisible_default(true);
        choice.set_condition(condition);
        choice.set_indentation_depth(indentation_depth);
        return Some(choice);
    }

    let is_divert_only_choice = find_top_level_divert(&choice_body)
        .is_some_and(|divert_index| choice_body[..divert_index].trim().is_empty());

    let segments = parse_choice_segments(&choice_body);

    if let Some(diagnostic) =
        text::unsupported_sequence_diagnostic_for_inline_text(&choice_body, &span)
    {
        parser.diagnostic(diagnostic);
        return None;
    }

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
    choice.set_dynamic_binding(dynamic_binding);
    choice.set_condition(condition);
    choice.set_indentation_depth(indentation_depth);

    // Check if this is an invisible default (empty content)
    let is_invisible_default = is_divert_only_choice
        || (!choice.has_start_content() && choice.inner_content().objects().len() <= 1); // Only newline
    choice.set_is_invisible_default(is_invisible_default);

    Some(choice)
}

fn parse_dynamic_choice_binding(
    choice_body: &str,
    span: &SourceSpan,
) -> Result<(Option<DynamicChoiceBinding>, String), String> {
    let remaining = choice_body.trim_start();
    let Some(after_open) = remaining.strip_prefix('[') else {
        return Ok((None, choice_body.to_string()));
    };
    let Some(close_index) = scan::find_matching_delimiter(after_open, '[', ']') else {
        return Ok((None, choice_body.to_string()));
    };

    let header = &after_open[..close_index];
    let Some((variables_source, iterable_source)) = split_dynamic_choice_header(header) else {
        return Ok((None, choice_body.to_string()));
    };

    let variable_names =
        scan::split_top_level_with_options(variables_source, ',', scan::ScanOptions::expression());
    if !(1..=2).contains(&variable_names.len()) {
        return Err(format!(
            "Dynamic choice binding expects one item variable or `index, item` variables but got {}",
            variable_names.len()
        ));
    }
    if !variable_names.iter().all(|name| is_identifier(name.trim())) {
        return Err("Dynamic choice binding variables must be identifiers".to_string());
    }
    if has_duplicate_variable_names(&variable_names) {
        return Err("Dynamic choice binding variables must be unique".to_string());
    }

    let iterable = super::parse_initial_expression(iterable_source.trim()).ok_or_else(|| {
        "Dynamic choice binding expects an array expression after `in`".to_string()
    })?;
    let prefix = format!("$choice{}_{}", span.line, span.column);
    let variables = variable_names
        .into_iter()
        .enumerate()
        .map(|(index, name)| DynamicChoiceVariable::new(name.trim(), format!("{prefix}_v{index}")))
        .collect();
    let binding = DynamicChoiceBinding::new(
        variables,
        iterable,
        format!("{prefix}_arr"),
        format!("{prefix}_i"),
        format!("{prefix}_n"),
    );
    let after_binding = after_open[close_index + ']'.len_utf8()..]
        .trim_start()
        .to_string();

    Ok((Some(binding), after_binding))
}

fn split_dynamic_choice_header(source: &str) -> Option<(&str, &str)> {
    for (index, token) in scan::top_level_token_matches_with_options(
        source,
        &[" in "],
        scan::ScanOptions::expression(),
    ) {
        if token == " in " {
            return Some((&source[..index], &source[index + token.len()..]));
        }
    }
    None
}

fn has_duplicate_variable_names(variable_names: &[&str]) -> bool {
    let mut seen = HashSet::new();
    variable_names.iter().any(|name| !seen.insert(name.trim()))
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
    let original_body = choice_body.to_string();
    let mut remaining = choice_body.trim_start();
    let mut conditions = Vec::new();
    let mut found_boundary = false;

    while let Some(after_open) = remaining.strip_prefix('{') {
        let close_index = scan::find_matching_delimiter(after_open, '{', '}')?;
        let condition = parse_condition_expression(&after_open[..close_index])?;
        conditions.push(condition);
        remaining = after_open[close_index + 1..].trim_start();
        if let Some(after_boundary) = remaining.strip_prefix(':') {
            remaining = after_boundary.trim_start();
            found_boundary = true;
            break;
        }
    }

    if !conditions.is_empty() && !found_boundary {
        return Some((None, original_body));
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
        assert_single_expression_text(choice, expected_name);
    }

    fn assert_single_expression_text(choice: &Choice, expected_source: &str) {
        let start_content = choice.start_content().expect("expected display text");
        assert!(matches!(
            start_content.objects(),
            [Object::ContentList(content)]
                if matches!(
                    content.objects(),
                    [Object::Expression(expression)]
                        if expression.to_source_string() == expected_source
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
    fn braced_text_without_colon_is_display_text() {
        let choice = parse_choice_line("* {ready} Label");

        assert!(choice.dynamic_binding().is_none());
        assert!(choice.condition().is_none());
        let start_content = choice.start_content().expect("expected display text");
        assert!(matches!(
            start_content.objects(),
            [Object::ContentList(content), Object::Text(text)]
                if matches!(
                    content.objects(),
                    [Object::Expression(Expression::VariableReference(name))]
                        if name == "ready"
                ) && text.text() == " Label"
        ));
    }

    #[test]
    fn dynamic_choice_braced_text_without_colon_is_display_text() {
        let choice = parse_choice_line("* [move in moves] {move.text}");

        assert!(choice.dynamic_binding().is_some());
        assert!(choice.condition().is_none());
        assert_single_expression_text(&choice, "move.text");
    }

    #[test]
    fn dynamic_choice_condition_requires_colon_boundary() {
        let choice = parse_choice_line("* [option in options] {option.enabled}: {option.text}");

        assert!(choice.dynamic_binding().is_some());
        assert!(matches!(
            choice.condition(),
            Some(Expression::FieldAccess { field, .. }) if field == "enabled"
        ));
        assert_single_expression_text(&choice, "option.text");
    }

    #[test]
    fn adjacent_leading_braces_without_colon_are_display_text() {
        let choice = parse_choice_line("* {enabled}{label}");

        assert!(choice.condition().is_none());
        let start_content = choice.start_content().expect("expected display text");
        assert!(matches!(
            start_content.objects(),
            [Object::ContentList(_), Object::ContentList(_)]
        ));
    }

    #[test]
    fn multiple_conditions_without_colon_are_display_text() {
        let choice = parse_choice_line("* {enabled} {visible} Label");

        assert!(choice.condition().is_none());
        let start_content = choice.start_content().expect("expected display text");
        assert!(matches!(
            start_content.objects(),
            [Object::ContentList(_), Object::Text(text), Object::ContentList(_), Object::Text(label)]
                if text.text() == " " && label.text() == " Label"
        ));
    }
}
