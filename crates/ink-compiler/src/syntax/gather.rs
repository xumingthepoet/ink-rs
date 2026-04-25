use crate::{
    parsed::Object,
    source::{SourceLine, SourceSpan},
};

use super::{is_identifier, is_identifier_continue, rule::RuleParser};

pub(super) fn parse_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
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

    Some(vec![Object::Gather(gather_object(
        span,
        indentation_depth,
        identifier,
    ))])
}

pub(super) fn parse_multiline_conditional_prefix(line: &SourceLine) -> Option<(Vec<Object>, &str)> {
    let trimmed = line.text.trim_start();
    if trimmed.starts_with('{') {
        return Some((Vec::new(), trimmed));
    }

    let mut rest = trimmed;
    let mut indentation_depth = 0;
    loop {
        if rest.starts_with("->") {
            return None;
        }
        let Some(after_dash) = rest.strip_prefix('-') else {
            break;
        };
        indentation_depth += 1;
        rest = after_dash.trim_start();
    }

    if indentation_depth == 0 {
        return None;
    }

    let (identifier, after_identifier) = parse_optional_identifier(rest);
    rest = after_identifier.trim_start();
    if !rest.starts_with('{') {
        return None;
    }

    Some((
        vec![Object::Gather(gather_object(
            line.span.clone(),
            indentation_depth,
            identifier,
        ))],
        rest,
    ))
}

fn gather_object(
    span: SourceSpan,
    indentation_depth: usize,
    identifier: Option<String>,
) -> crate::parsed::Gather {
    let mut gather = crate::parsed::Gather::new(span, indentation_depth);
    gather.set_identifier(identifier);
    gather
}

fn parse_bracketed_identifier(parser: &mut RuleParser<'_>) -> Option<String> {
    parser.parse_rule(|parser| {
        parser.skip_horizontal_whitespace();
        parser.match_string("(")?;
        parser.skip_horizontal_whitespace();
        let name = parser.take_while(is_identifier_continue)?;
        if !is_identifier(&name) {
            return None;
        }
        parser.skip_horizontal_whitespace();
        parser.match_string(")")?;
        Some(name)
    })
}

fn parse_optional_identifier(source: &str) -> (Option<String>, &str) {
    let Some(after_open) = source.strip_prefix('(') else {
        return (None, source);
    };
    let Some(close_index) = after_open.find(')') else {
        return (None, source);
    };
    let name = after_open[..close_index].trim();
    if !is_identifier(name) {
        return (None, source);
    }
    (Some(name.to_string()), &after_open[close_index + 1..])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text: &str) -> SourceLine {
        SourceLine {
            text: text.to_string(),
            span: SourceSpan::new(None, 1, 1),
        }
    }

    #[test]
    fn parses_named_gather_statement() {
        let line = line("-- (again)");
        let mut parser = RuleParser::new(&line);
        let objects = parse_statement(&mut parser).expect("expected gather");

        assert!(parser.finish().is_empty());
        let Object::Gather(gather) = &objects[0] else {
            panic!("expected gather");
        };
        assert_eq!(gather.indentation_depth(), 2);
        assert_eq!(gather.identifier(), Some("again"));
    }

    #[test]
    fn parses_gather_prefixed_multiline_conditional() {
        let line = line("- (again) { true:");
        let (objects, rest) =
            parse_multiline_conditional_prefix(&line).expect("expected gather prefix");

        assert_eq!(rest, "{ true:");
        let Object::Gather(gather) = &objects[0] else {
            panic!("expected gather");
        };
        assert_eq!(gather.indentation_depth(), 1);
        assert_eq!(gather.identifier(), Some("again"));
    }
}
