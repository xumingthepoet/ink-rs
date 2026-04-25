use crate::parsed::{ConstantDeclaration, ExternalDeclaration, Object};

use super::{is_identifier, is_identifier_continue, parse_initial_expression, rule::RuleParser};

pub(super) fn constant_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    parser.match_string("CONST")?;
    parser.skip_horizontal_whitespace();
    let name = parser.take_while(is_identifier_continue)?;
    if !is_identifier(&name) {
        return None;
    }
    parser.skip_horizontal_whitespace();
    parser.match_string("=")?;
    parser.skip_horizontal_whitespace();
    let expression = parse_initial_expression(parser.line_remainder().trim())?;
    parser.skip_to_end();

    Some(vec![Object::ConstantDeclaration(ConstantDeclaration::new(
        name, expression, span,
    ))])
}

pub(super) fn external_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    parser.match_string("EXTERNAL")?;
    parser.skip_horizontal_whitespace();
    let name = parser.take_while(is_identifier_continue)?;
    if !is_identifier(&name) {
        return None;
    }
    parser.skip_horizontal_whitespace();
    parser.match_string("(")?;
    parser.skip_horizontal_whitespace();

    let mut arguments = Vec::new();
    if parser.match_string(")").is_none() {
        loop {
            parser.skip_horizontal_whitespace();
            let argument = parser.take_while(is_identifier_continue)?;
            if !is_identifier(&argument) {
                return None;
            }
            arguments.push(argument);
            parser.skip_horizontal_whitespace();

            if parser.match_string(")").is_some() {
                break;
            }
            parser.match_string(",")?;
        }
    }
    parser.skip_horizontal_whitespace();
    if !parser.line_remainder().is_empty() {
        return None;
    }
    parser.skip_to_end();

    Some(vec![Object::ExternalDeclaration(ExternalDeclaration::new(
        name, arguments,
    ))])
}

#[cfg(test)]
mod tests {
    use crate::{
        source::{SourceLine, SourceSpan},
        syntax::rule::RuleParser,
    };

    use super::*;

    fn line(text: &str) -> SourceLine {
        SourceLine {
            text: text.to_string(),
            span: SourceSpan::new(None, 1, 1),
        }
    }

    #[test]
    fn parses_constant_declaration() {
        let line = line("CONST max_score = 5");
        let mut parser = RuleParser::new(&line);
        let objects = constant_statement(&mut parser).expect("expected constant declaration");

        assert!(parser.finish().is_empty());
        let Object::ConstantDeclaration(declaration) = &objects[0] else {
            panic!("expected constant declaration");
        };
        assert_eq!(declaration.name(), "max_score");
    }

    #[test]
    fn parses_external_declaration_arguments() {
        let line = line("EXTERNAL play_sound(name, volume)");
        let mut parser = RuleParser::new(&line);
        let objects = external_statement(&mut parser).expect("expected external declaration");

        assert!(parser.finish().is_empty());
        let Object::ExternalDeclaration(declaration) = &objects[0] else {
            panic!("expected external declaration");
        };
        assert_eq!(declaration.name(), "play_sound");
        assert_eq!(declaration.argument_names(), ["name", "volume"]);
    }
}
