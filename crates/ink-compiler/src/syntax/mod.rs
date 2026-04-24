mod choice;
mod divert;
mod error;
mod knot;
mod rule;
mod state;
mod text;

use crate::{
    compiler::StageOutput,
    diagnostic::Diagnostic,
    parsed::{
        BinaryOperator, Expression, FloatLiteral, Flow, IncDec, Object, Story, UnaryOperator,
        VariableAssignment, Weave,
    },
    source::{SourceFile, SourceInput, SourceLine},
};

use self::rule::RuleParser;

type StatementRule = for<'source> fn(&mut RuleParser<'source>) -> Option<Vec<Object>>;

pub(crate) fn parse(input: SourceInput) -> StageOutput<Story> {
    let source = SourceFile::from_input(input);
    let mut parser = Parser::new(source);
    let story = parser.parse_story();

    StageOutput {
        artifact: Some(story),
        diagnostics: parser.diagnostics,
    }
}

struct Parser {
    source: SourceFile,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    fn new(source: SourceFile) -> Self {
        Self {
            source,
            diagnostics: Vec::new(),
        }
    }

    fn parse_story(&mut self) -> Story {
        let lines = self.source.lines.clone();
        let mut objects = Vec::new();
        let mut flows = Vec::new();
        let mut index = 0;

        while index < lines.len() {
            let line = &lines[index];

            if line.text.trim().is_empty() {
                index += 1;
                continue;
            }

            if knot::is_knot_declaration_line(&line.text) {
                if let Some(flow) = self.parse_flow(&lines, &mut index) {
                    flows.push(flow);
                } else {
                    index += 1;
                }
                continue;
            }

            objects.extend(self.parse_statement(line));
            index += 1;
        }

        Story::new(group_nested_weaves(objects, 1), flows)
    }

    fn parse_statement(&mut self, line: &SourceLine) -> Vec<Object> {
        if line.text.trim().is_empty() {
            return Vec::new();
        }

        let mut line_parser = RuleParser::new(line);
        let statement_rules: &[StatementRule] = &[
            variable_declaration_statement,
            variable_assignment_statement,
            logic_line_statement,
            choice_statement,
            gather_statement,
            divert_statement,
            text_statement,
        ];

        for rule in statement_rules {
            if let Some(objects) = line_parser.parse_rule(*rule) {
                self.diagnostics.extend(line_parser.finish());
                return objects;
            }

            if line_parser.had_error() {
                self.diagnostics.extend(line_parser.finish());
                return Vec::new();
            }
        }

        let diagnostics = line_parser.finish();
        self.diagnostics.extend(diagnostics);

        if let Some(diagnostic) = self.try_unsupported_statement(line) {
            self.diagnostics.push(diagnostic);
        }
        Vec::new()
    }

    fn try_unsupported_statement(&self, line: &SourceLine) -> Option<Diagnostic> {
        let trimmed = line.text.trim_start();

        let feature = if trimmed.starts_with("INCLUDE ") {
            Some("include")
        } else if trimmed.starts_with("VAR ") {
            Some("global variable declaration")
        } else if trimmed.starts_with("LIST ") {
            Some("list declaration")
        } else if trimmed.starts_with("CONST ") {
            Some("constant declaration")
        } else if trimmed.starts_with("EXTERNAL ") {
            Some("external declaration")
        } else if knot::is_knot_declaration_line(trimmed) {
            Some("knot declaration")
        } else if trimmed.starts_with('*') || trimmed.starts_with('+') {
            Some("choice")
        } else if trimmed.starts_with('~') {
            Some("logic line")
        } else if trimmed.contains('{') || trimmed.contains('}') {
            Some("inline logic")
        } else if trimmed.contains("<>") {
            Some("glue")
        } else if trimmed.contains('#') {
            Some("tag")
        } else {
            None
        };

        feature.map(|feature| Diagnostic::unsupported(line.span.clone(), feature))
    }

    fn parse_flow(&mut self, lines: &[SourceLine], index: &mut usize) -> Option<Flow> {
        let line = &lines[*index];
        let mut line_parser = RuleParser::new(line);
        let declaration = line_parser.parse_rule(knot::parse_knot_declaration);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());

        let Some(declaration) = declaration else {
            return None;
        };

        if had_error {
            return None;
        }

        *index += 1;
        let mut content = Vec::new();
        let mut child_flows = Vec::new();

        while *index < lines.len() {
            let next_line = &lines[*index];
            if next_line.text.trim().is_empty() {
                *index += 1;
                continue;
            }

            if knot::is_knot_declaration_line(&next_line.text) {
                break;
            }

            if knot::is_stitch_declaration_line(&next_line.text) {
                if let Some(stitch) = self.parse_stitch(lines, index) {
                    child_flows.push(stitch);
                } else {
                    *index += 1;
                }
                continue;
            }

            content.extend(self.parse_statement(next_line));
            *index += 1;
        }

        Some(Flow::new(
            declaration.level,
            declaration.name,
            group_nested_weaves(content, 1),
            child_flows,
            declaration.arguments,
            declaration.is_function,
        ))
    }

    fn parse_stitch(&mut self, lines: &[SourceLine], index: &mut usize) -> Option<Flow> {
        let line = &lines[*index];
        let mut line_parser = RuleParser::new(line);
        let declaration = line_parser.parse_rule(knot::parse_stitch_declaration);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());

        let Some(declaration) = declaration else {
            return None;
        };

        if had_error {
            return None;
        }

        *index += 1;
        let mut content = Vec::new();

        while *index < lines.len() {
            let next_line = &lines[*index];
            if next_line.text.trim().is_empty() {
                *index += 1;
                continue;
            }

            if knot::is_knot_declaration_line(&next_line.text)
                || knot::is_stitch_declaration_line(&next_line.text)
            {
                break;
            }

            content.extend(self.parse_statement(next_line));
            *index += 1;
        }

        Some(Flow::new(
            declaration.level,
            declaration.name,
            group_nested_weaves(content, 1),
            Vec::new(),
            declaration.arguments,
            declaration.is_function,
        ))
    }
}

fn group_nested_weaves(objects: Vec<Object>, base_depth: usize) -> Vec<Object> {
    let mut grouped = Vec::new();
    let mut index = 0;

    while index < objects.len() {
        if object_depth(&objects[index]).is_some_and(|depth| depth > base_depth) {
            let mut nested = Vec::new();
            while index < objects.len() {
                if object_depth(&objects[index]).is_some_and(|depth| depth <= base_depth) {
                    break;
                }
                nested.push(objects[index].clone());
                index += 1;
            }
            grouped.push(Object::Weave(Weave::new(
                group_nested_weaves(nested, base_depth + 1),
                base_depth,
            )));
        } else {
            grouped.push(objects[index].clone());
            index += 1;
        }
    }

    grouped
}

fn object_depth(object: &Object) -> Option<usize> {
    match object {
        Object::Choice(choice) => Some(choice.indentation_depth()),
        Object::Gather(gather) => Some(gather.indentation_depth()),
        _ => None,
    }
}

fn choice_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    choice::parse_choice(parser).map(|choice| vec![Object::Choice(choice)])
}

fn variable_declaration_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    parser.match_string("VAR")?;
    parser.skip_horizontal_whitespace();
    let name = parser.take_while(|ch| ch == '_' || ch.is_ascii_alphanumeric())?;
    if !is_identifier(&name) {
        return None;
    }
    parser.skip_horizontal_whitespace();
    parser.match_string("=")?;
    parser.skip_horizontal_whitespace();
    let expression = parse_initial_expression(parser.line_remainder().trim())?;
    parser.skip_to_end();

    Some(vec![Object::VariableAssignment(VariableAssignment::new(
        name, expression, true, false, span,
    ))])
}

fn variable_assignment_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    parser.match_string("~")?;
    parser.skip_horizontal_whitespace();
    let name = parser.take_while(|ch| ch == '_' || ch.is_ascii_alphanumeric())?;
    if !is_identifier(&name) {
        return None;
    }
    parser.skip_horizontal_whitespace();
    if parser.match_string("+=").is_some() {
        parser.skip_horizontal_whitespace();
        let expression = parse_initial_expression(parser.line_remainder().trim())?;
        parser.skip_to_end();
        return Some(vec![Object::IncDec(IncDec::new(
            name, expression, true, span,
        ))]);
    }
    if parser.match_string("-=").is_some() {
        parser.skip_horizontal_whitespace();
        let expression = parse_initial_expression(parser.line_remainder().trim())?;
        parser.skip_to_end();
        return Some(vec![Object::IncDec(IncDec::new(
            name, expression, false, span,
        ))]);
    }
    parser.match_string("=")?;
    parser.skip_horizontal_whitespace();
    let expression = parse_initial_expression(parser.line_remainder().trim())?;
    parser.skip_to_end();

    Some(vec![Object::VariableAssignment(VariableAssignment::new(
        name, expression, false, false, span,
    ))])
}

fn logic_line_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    parser.match_string("~")?;
    parser.skip_horizontal_whitespace();
    let expression = parse_initial_expression(parser.line_remainder().trim())?;
    parser.skip_to_end();
    Some(vec![Object::LogicLine(expression)])
}

pub(super) fn parse_initial_expression(source: &str) -> Option<Expression> {
    parse_expression(source.trim())
}

fn parse_expression(source: &str) -> Option<Expression> {
    let source = strip_enclosing_parentheses(source.trim());
    if let Some((left, operator, right)) = split_top_level_operator(
        source,
        &[('+', BinaryOperator::Add), ('-', BinaryOperator::Subtract)],
    ) {
        return Some(Expression::Binary {
            operator,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some((left, operator, right)) =
        split_top_level_operator(source, &[('*', BinaryOperator::Multiply)])
    {
        return Some(Expression::Binary {
            operator,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some((left, operator, right)) =
        split_top_level_operator(source, &[('/', BinaryOperator::Divide)])
    {
        return Some(Expression::Binary {
            operator,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some((left, operator, right)) =
        split_top_level_operator(source, &[('%', BinaryOperator::Modulo)])
    {
        return Some(Expression::Binary {
            operator,
            left: Box::new(parse_expression(left)?),
            right: Box::new(parse_expression(right)?),
        });
    }
    if let Some(value) = parse_quoted_string_literal(source) {
        return Some(Expression::String(value));
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
    is_identifier(source).then(|| Expression::VariableReference(source.to_string()))
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

fn split_top_level_args(source: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut start = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut paren_depth = 0;

    for (index, ch) in source.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '(' if !in_string => paren_depth += 1,
            ')' if !in_string => paren_depth -= 1,
            ',' if !in_string && paren_depth == 0 => {
                args.push(source[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    args.push(source[start..].trim());
    args
}

fn split_top_level_operator<'a>(
    source: &'a str,
    operators: &[(char, BinaryOperator)],
) -> Option<(&'a str, BinaryOperator, &'a str)> {
    let mut in_string = false;
    let mut escaped = false;
    let mut paren_depth = 0;

    for (index, ch) in source.char_indices().rev() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            ')' if !in_string => paren_depth += 1,
            '(' if !in_string => paren_depth -= 1,
            _ if !in_string && paren_depth == 0 => {
                let Some((_, operator)) = operators
                    .iter()
                    .find(|(operator_char, _)| *operator_char == ch)
                else {
                    continue;
                };
                let left = &source[..index];
                let right = &source[index + ch.len_utf8()..];
                if !left.trim().is_empty()
                    && !right.trim().is_empty()
                    && !is_unary_operator_position(source, index)
                {
                    return Some((left.trim(), *operator, right.trim()));
                }
            }
            _ => {}
        }
    }

    None
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
    let mut in_string = false;
    let mut escaped = false;
    let mut depth = 0;
    for (index, ch) in source.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '(' if !in_string => depth += 1,
            ')' if !in_string => {
                depth -= 1;
                if depth == 0 && index + ch.len_utf8() != source.len() {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn parse_quoted_string_literal(source: &str) -> Option<String> {
    let mut chars = source.chars();
    if chars.next()? != '"' {
        return None;
    }

    let mut value = String::new();
    let mut escaped = false;
    for ch in chars.by_ref() {
        if escaped {
            match ch {
                'n' => value.push('\n'),
                'r' => value.push('\r'),
                't' => value.push('\t'),
                '"' => value.push('"'),
                '\\' => value.push('\\'),
                other => value.push(other),
            }
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => {
                return if chars.as_str().trim().is_empty() {
                    Some(value)
                } else {
                    None
                };
            }
            other => value.push(other),
        }
    }

    None
}

fn gather_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();

    let mut indentation_depth = 0;
    loop {
        // Match one or more '-' gather dashes, but never consume a divert arrow.
        if parser.line_remainder().starts_with("->") || parser.match_string("-").is_none() {
            break;
        }
        indentation_depth += 1;
        parser.skip_horizontal_whitespace();
    }

    if indentation_depth == 0 {
        return None;
    }

    let identifier = parse_bracketed_identifier(parser);
    parser.skip_horizontal_whitespace();

    let mut gather = crate::parsed::Gather::new(span.clone(), indentation_depth);
    gather.set_identifier(identifier);
    let mut objects = vec![Object::Gather(gather)];

    // Parse any remaining content on the line
    let remaining = parser.line_remainder().trim();
    if !remaining.is_empty() {
        let parsed = text::parse_inline_content(remaining, &span).unwrap_or_default();
        let has_text = parsed.iter().any(|o| matches!(o, Object::Text(_)));
        objects.extend(parsed);
        parser.skip_to_end();
        // Add trailing newline only when there's actual text content
        if has_text {
            objects.push(Object::Text(crate::parsed::Text::new("\n", span)));
        }
    } else {
        parser.skip_to_end();
    }

    Some(objects)
}

fn parse_bracketed_identifier(parser: &mut RuleParser<'_>) -> Option<String> {
    parser.parse_rule(|parser| {
        parser.skip_horizontal_whitespace();
        parser.match_string("(")?;
        parser.skip_horizontal_whitespace();
        let name = parser.take_while(|ch| ch == '_' || ch.is_ascii_alphanumeric())?;
        if !is_identifier(&name) {
            return None;
        }
        parser.skip_horizontal_whitespace();
        parser.match_string(")")?;
        Some(name)
    })
}

fn is_identifier(source: &str) -> bool {
    let mut chars = source.chars();
    matches!(chars.next(), Some(ch) if ch == '_' || ch.is_ascii_alphabetic())
        && chars.all(is_identifier_continue)
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn divert_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    divert::parse_divert(parser).map(|divert| vec![Object::Divert(divert)])
}

fn text_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    text::parse_text_line(parser)
}

#[cfg(test)]
mod tests {
    use super::*;

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
