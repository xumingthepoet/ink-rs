use crate::parsed::{FlowArgument, FlowLevel};

use super::rule::RuleParser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FlowDecl {
    pub level: FlowLevel,
    pub name: String,
    pub arguments: Vec<FlowArgument>,
    pub is_function: bool,
}

pub(super) fn is_knot_declaration_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    let equals = trimmed.chars().take_while(|ch| *ch == '=').count();
    equals >= 2
}

pub(super) fn is_stitch_declaration_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('=') && !is_knot_declaration_line(trimmed)
}

pub(super) fn parse_knot_declaration(parser: &mut RuleParser<'_>) -> Option<FlowDecl> {
    parser.skip_horizontal_whitespace();

    let equals = parser.take_while(|ch| ch == '=')?;
    if equals.chars().count() < 2 {
        return None;
    }

    parser.skip_horizontal_whitespace();

    let first_identifier =
        parser.expect("knot name", parse_identifier, |parser| parser.skip_to_end())?;

    let (name, is_function) = if first_identifier == "function" {
        parser.expect(
            "whitespace after 'function'",
            parse_horizontal_whitespace,
            |_| {},
        )?;
        let name = parser.expect("function name", parse_identifier, |parser| {
            parser.skip_to_end();
        })?;
        (name, true)
    } else {
        (first_identifier, false)
    };

    parser.skip_horizontal_whitespace();

    let arguments = parse_arguments(parser).unwrap_or_default();

    parser.skip_horizontal_whitespace();
    let _ = parser.parse_rule(|parser| {
        let suffix = parser.take_while(|ch| ch == '=')?;
        if suffix.chars().count() < 2 {
            return None;
        }
        Some(suffix)
    });
    parser.skip_horizontal_whitespace();

    if !parser.line_remainder().is_empty() {
        parser.error(format!(
            "Expected end of line after knot declaration but saw '{}'",
            parser.line_remainder()
        ));
        parser.skip_to_end();
    }

    Some(FlowDecl {
        level: FlowLevel::Knot,
        name,
        arguments,
        is_function,
    })
}

pub(super) fn parse_stitch_declaration(parser: &mut RuleParser<'_>) -> Option<FlowDecl> {
    parser.skip_horizontal_whitespace();

    let equals = parser.take_while(|ch| ch == '=')?;
    if equals.chars().count() != 1 {
        return None;
    }

    parser.skip_horizontal_whitespace();

    let name = parser.expect("stitch name", parse_identifier, |parser| {
        parser.skip_to_end()
    })?;

    parser.skip_horizontal_whitespace();

    if !parser.line_remainder().is_empty() {
        parser.error(format!(
            "Expected end of line after stitch declaration but saw '{}'",
            parser.line_remainder()
        ));
        parser.skip_to_end();
    }

    Some(FlowDecl {
        level: FlowLevel::Stitch,
        name,
        arguments: Vec::new(),
        is_function: false,
    })
}

fn parse_arguments(parser: &mut RuleParser<'_>) -> Option<Vec<FlowArgument>> {
    parser.parse_rule(|parser| {
        parser.match_string("(")?;
        parser.skip_horizontal_whitespace();

        let mut arguments = Vec::new();
        if parser.match_string(")").is_some() {
            return Some(arguments);
        }

        loop {
            arguments.push(parse_argument(parser)?);
            parser.skip_horizontal_whitespace();

            if parser.match_string(")").is_some() {
                break;
            }

            parser.expect(
                "',' between arguments",
                |parser| parser.match_string(","),
                |parser| {
                    parser.skip_to_end();
                },
            )?;
            parser.skip_horizontal_whitespace();
        }

        Some(arguments)
    })
}

fn parse_argument(parser: &mut RuleParser<'_>) -> Option<FlowArgument> {
    let mut is_by_reference = false;
    let mut is_divert_target = false;

    if parser
        .parse_rule(|parser| {
            let keyword = parse_identifier(parser)?;
            (keyword == "ref").then_some(())
        })
        .is_some()
    {
        is_by_reference = true;
        parser.skip_horizontal_whitespace();
    }

    if parser.match_string("->").is_some() {
        is_divert_target = true;
        parser.skip_horizontal_whitespace();
    }

    let span = parser.current_span();
    let name = parser.expect("parameter name", parse_identifier, |parser| {
        parser.skip_to_end();
    })?;

    Some(FlowArgument::new(
        name,
        is_by_reference,
        is_divert_target,
        span,
    ))
}

fn parse_identifier(parser: &mut RuleParser<'_>) -> Option<String> {
    let first = parser.take_while(|ch| ch == '_' || ch.is_ascii_alphabetic())?;
    let rest = parser
        .take_while(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        .unwrap_or_default();
    Some(format!("{first}{rest}"))
}

fn parse_horizontal_whitespace(parser: &mut RuleParser<'_>) -> Option<String> {
    parser.take_while(|ch| matches!(ch, ' ' | '\t'))
}
