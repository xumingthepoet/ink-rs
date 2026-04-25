use crate::parsed::{ContentList, Expression, IncDec, Object, Text, VariableAssignment};

use super::{
    is_identifier, is_identifier_continue, logic::expression_contains_function_call,
    parse_initial_expression, rule::RuleParser,
};

pub(super) fn declaration_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    parser.match_string("VAR")?;
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

    Some(vec![Object::VariableAssignment(VariableAssignment::new(
        name, expression, true, false, span,
    ))])
}

pub(super) fn temp_declaration_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    parser.match_string("~")?;
    parser.skip_horizontal_whitespace();
    let keyword = parser.take_while(is_identifier_continue)?;
    if keyword != "temp" {
        return None;
    }
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

    let assignment = VariableAssignment::new(name, expression, false, true, span.clone());
    if expression_contains_function_call(assignment.expression()) {
        Some(vec![Object::ContentList(ContentList::new(vec![
            Object::VariableAssignment(assignment),
            Object::Text(Text::new("\n", span)),
        ]))])
    } else {
        Some(vec![Object::VariableAssignment(assignment)])
    }
}

pub(super) fn assignment_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    parser.match_string("~")?;
    parser.skip_horizontal_whitespace();
    let name = parser.take_while(is_identifier_continue)?;
    if !is_identifier(&name) {
        return None;
    }
    parser.skip_horizontal_whitespace();
    if parser.match_string("++").is_some() {
        parser.skip_horizontal_whitespace();
        if !parser.line_remainder().is_empty() {
            return None;
        }
        parser.skip_to_end();
        return Some(vec![Object::IncDec(IncDec::new(
            name,
            Expression::NumberInt(1),
            true,
            span,
        ))]);
    }
    if parser.match_string("--").is_some() {
        parser.skip_horizontal_whitespace();
        if !parser.line_remainder().is_empty() {
            return None;
        }
        parser.skip_to_end();
        return Some(vec![Object::IncDec(IncDec::new(
            name,
            Expression::NumberInt(1),
            false,
            span,
        ))]);
    }
    if parser.match_string("+=").is_some() {
        parser.skip_horizontal_whitespace();
        let expression = parse_initial_expression(parser.line_remainder().trim())?;
        parser.skip_to_end();
        let inc = IncDec::new(name, expression, true, span.clone());
        if expression_contains_function_call(inc.expression()) {
            return Some(vec![Object::ContentList(ContentList::new(vec![
                Object::IncDec(inc),
                Object::Text(Text::new("\n", span)),
            ]))]);
        }
        return Some(vec![Object::IncDec(inc)]);
    }
    if parser.match_string("-=").is_some() {
        parser.skip_horizontal_whitespace();
        let expression = parse_initial_expression(parser.line_remainder().trim())?;
        parser.skip_to_end();
        let dec = IncDec::new(name, expression, false, span.clone());
        if expression_contains_function_call(dec.expression()) {
            return Some(vec![Object::ContentList(ContentList::new(vec![
                Object::IncDec(dec),
                Object::Text(Text::new("\n", span)),
            ]))]);
        }
        return Some(vec![Object::IncDec(dec)]);
    }
    parser.match_string("=")?;
    parser.skip_horizontal_whitespace();
    let expression = parse_initial_expression(parser.line_remainder().trim())?;
    parser.skip_to_end();

    let assignment = VariableAssignment::new(name, expression, false, false, span.clone());
    if expression_contains_function_call(assignment.expression()) {
        Some(vec![Object::ContentList(ContentList::new(vec![
            Object::VariableAssignment(assignment),
            Object::Text(Text::new("\n", span)),
        ]))])
    } else {
        Some(vec![Object::VariableAssignment(assignment)])
    }
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
    fn parses_global_variable_declaration() {
        let line = line("VAR score = 1");
        let mut parser = RuleParser::new(&line);
        let objects = declaration_statement(&mut parser).expect("expected variable declaration");

        assert!(parser.finish().is_empty());
        let Object::VariableAssignment(assignment) = &objects[0] else {
            panic!("expected variable assignment");
        };
        assert_eq!(assignment.name(), "score");
        assert!(assignment.is_global());
    }

    #[test]
    fn parses_postfix_increment_statement() {
        let line = line("~ score++");
        let mut parser = RuleParser::new(&line);
        let objects = assignment_statement(&mut parser).expect("expected increment");

        assert!(parser.finish().is_empty());
        let Object::IncDec(inc) = &objects[0] else {
            panic!("expected inc/dec");
        };
        assert_eq!(inc.name(), "score");
        assert!(inc.is_increment());
    }
}
