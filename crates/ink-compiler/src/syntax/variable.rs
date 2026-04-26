use crate::parsed::{
    AssignmentTarget, ContentList, Expression, IncDec, Object, Text, VariableAssignment,
};

use super::{
    is_identifier, is_identifier_continue, logic::expression_contains_function_call,
    parse_expression_remainder, parse_initial_expression, rule::RuleParser, scan, type_name,
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
    if parser.match_string(":").is_none() {
        parser.error(format!("Variable '{name}' is missing a type"));
        parser.skip_to_end();
        return None;
    }
    let declared_type = type_name::parse_type_name(parser)?;
    if declared_type.is_void() {
        parser.error("Variables cannot be declared with type void");
        return None;
    }
    parser.skip_horizontal_whitespace();

    let expression = if parser.match_string("=").is_some() {
        parser.skip_horizontal_whitespace();
        Some(parse_expression_remainder(parser)?)
    } else {
        parser.skip_horizontal_whitespace();
        if !parser.line_remainder().is_empty() {
            parser.error("Expected '=' or end of line after typed global declaration");
            return None;
        }
        None
    };
    parser.skip_to_end();

    Some(vec![Object::VariableAssignment(VariableAssignment::new(
        name,
        expression,
        Some(declared_type),
        true,
        false,
        span,
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
    if parser.match_string(":").is_none() {
        parser.error(format!("Temp variable '{name}' is missing a type"));
        parser.skip_to_end();
        return None;
    }
    let declared_type = type_name::parse_type_name(parser)?;
    if declared_type.is_void() {
        parser.error("Variables cannot be declared with type void");
        return None;
    }
    parser.skip_horizontal_whitespace();

    let expression = if parser.match_string("=").is_some() {
        parser.skip_horizontal_whitespace();
        Some(parse_expression_remainder(parser)?)
    } else {
        parser.skip_horizontal_whitespace();
        if !parser.line_remainder().is_empty() {
            parser.error("Expected '=' or end of line after typed temp declaration");
            return None;
        }
        None
    };
    parser.skip_to_end();

    let assignment = VariableAssignment::new(
        name,
        expression,
        Some(declared_type),
        false,
        true,
        span.clone(),
    );
    if assignment
        .expression()
        .is_some_and(expression_contains_function_call)
    {
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
    let assignment_source = parser.line_remainder().to_string();
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
        let expression = parse_expression_remainder(parser)?;
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
        let expression = parse_expression_remainder(parser)?;
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
    if let Some(objects) = complex_inc_dec_statement(&assignment_source, span.clone()) {
        parser.skip_to_end();
        return Some(objects);
    }
    if let Some(objects) = complex_assignment_statement(&assignment_source, span.clone()) {
        parser.skip_to_end();
        return Some(objects);
    }
    parser.match_string("=")?;
    parser.skip_horizontal_whitespace();
    let expression = parse_expression_remainder(parser)?;
    parser.skip_to_end();

    let assignment =
        VariableAssignment::new(name, Some(expression), None, false, false, span.clone());
    if assignment
        .expression()
        .is_some_and(expression_contains_function_call)
    {
        Some(vec![Object::ContentList(ContentList::new(vec![
            Object::VariableAssignment(assignment),
            Object::Text(Text::new("\n", span)),
        ]))])
    } else {
        Some(vec![Object::VariableAssignment(assignment)])
    }
}

fn complex_inc_dec_statement(source: &str, span: crate::source::SourceSpan) -> Option<Vec<Object>> {
    let (target_source, operator, value_source) = split_compound_once(source)?;
    let target_source = target_source.trim();
    if !target_source.contains('.') && !target_source.contains('[') && !target_source.contains("::")
    {
        return None;
    }
    let target_expression = parse_initial_expression(target_source)?;
    let target = AssignmentTarget::from_expression(target_expression)?;
    let value_source = value_source.trim();
    let (expression, is_increment) = match operator {
        "++" => {
            if !value_source.is_empty() {
                return None;
            }
            (Expression::NumberInt(1), true)
        }
        "--" => {
            if !value_source.is_empty() {
                return None;
            }
            (Expression::NumberInt(1), false)
        }
        "+=" => (parse_initial_expression(value_source)?, true),
        "-=" => (parse_initial_expression(value_source)?, false),
        _ => return None,
    };

    let inc_dec = IncDec::with_target(target, expression, is_increment, span.clone());
    if expression_contains_function_call(inc_dec.expression()) {
        Some(vec![Object::ContentList(ContentList::new(vec![
            Object::IncDec(inc_dec),
            Object::Text(Text::new("\n", span)),
        ]))])
    } else {
        Some(vec![Object::IncDec(inc_dec)])
    }
}

fn split_compound_once(source: &str) -> Option<(&str, &str, &str)> {
    for (index, token) in scan::top_level_token_matches_with_options(
        source,
        &["++", "--", "+=", "-="],
        scan::ScanOptions::expression(),
    ) {
        return Some((&source[..index], token, &source[index + token.len()..]));
    }

    None
}

fn complex_assignment_statement(
    source: &str,
    span: crate::source::SourceSpan,
) -> Option<Vec<Object>> {
    let (target_source, value_source) = split_assignment_once(source)?;
    let target_source = target_source.trim();
    if !target_source.contains('.') && !target_source.contains('[') && !target_source.contains("::")
    {
        return None;
    }
    let target_expression = parse_initial_expression(target_source)?;
    let target = AssignmentTarget::from_expression(target_expression)?;
    let expression = parse_initial_expression(value_source.trim())?;

    let assignment =
        VariableAssignment::with_target(target, Some(expression), None, false, false, span.clone());
    if assignment
        .expression()
        .is_some_and(expression_contains_function_call)
    {
        Some(vec![Object::ContentList(ContentList::new(vec![
            Object::VariableAssignment(assignment),
            Object::Text(Text::new("\n", span)),
        ]))])
    } else {
        Some(vec![Object::VariableAssignment(assignment)])
    }
}

fn split_assignment_once(source: &str) -> Option<(&str, &str)> {
    for (index, token) in scan::top_level_token_matches_with_options(
        source,
        &["==", ">=", "<=", "!=", "="],
        scan::ScanOptions::expression(),
    ) {
        if token == "=" && !is_non_assignment_equal(source, index) {
            return Some((&source[..index], &source[index + token.len()..]));
        }
    }

    None
}

fn is_non_assignment_equal(source: &str, index: usize) -> bool {
    source[index + '='.len_utf8()..].starts_with('=')
        || source[..index]
            .chars()
            .next_back()
            .is_some_and(|ch| matches!(ch, '=' | '!' | '<' | '>'))
}

#[cfg(test)]
mod tests {
    use crate::{
        parsed::TypeName,
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
    fn rejects_untyped_global_variable_declaration() {
        let line = line("VAR score = 1");
        let mut parser = RuleParser::new(&line);

        assert!(declaration_statement(&mut parser).is_none());
        let diagnostics = parser.finish();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].message, "Variable 'score' is missing a type");
    }

    #[test]
    fn parses_typed_global_variable_declarations() {
        let cases = [
            ("VAR score: int = 1", TypeName::int(), true),
            (
                "VAR inventory: Item[]",
                TypeName::array(TypeName::struct_type("Item")),
                false,
            ),
        ];

        for (source, expected_type, expects_expression) in cases {
            let line = line(source);
            let mut parser = RuleParser::new(&line);
            let objects =
                declaration_statement(&mut parser).expect("expected variable declaration");

            assert!(parser.finish().is_empty());
            let Object::VariableAssignment(assignment) = &objects[0] else {
                panic!("expected variable assignment");
            };
            assert!(assignment.is_global());
            assert_eq!(assignment.declared_type(), Some(&expected_type));
            assert_eq!(assignment.expression().is_some(), expects_expression);
        }
    }

    #[test]
    fn rejects_void_global_variable_type() {
        let line = line("VAR result: void = 0");
        let mut parser = RuleParser::new(&line);

        assert!(declaration_statement(&mut parser).is_none());
        let diagnostics = parser.finish();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(
            diagnostics[0].message,
            "Variables cannot be declared with type void"
        );
    }

    #[test]
    fn parses_typed_temp_declarations_with_and_without_initializers() {
        let cases = [
            ("~ temp hp: int = 10", TypeName::int(), true),
            (
                "~ temp party: Player[]",
                TypeName::array(TypeName::struct_type("Player")),
                false,
            ),
        ];

        for (source, expected_type, expects_expression) in cases {
            let line = line(source);
            let mut parser = RuleParser::new(&line);
            let objects =
                temp_declaration_statement(&mut parser).expect("expected temp declaration");

            assert!(parser.finish().is_empty());
            let Object::VariableAssignment(assignment) = &objects[0] else {
                panic!("expected variable assignment");
            };
            assert!(assignment.is_temporary());
            assert_eq!(assignment.declared_type(), Some(&expected_type));
            assert_eq!(assignment.expression().is_some(), expects_expression);
        }
    }

    #[test]
    fn rejects_void_temp_variable_type() {
        let line = line("~ temp result: void");
        let mut parser = RuleParser::new(&line);

        assert!(temp_declaration_statement(&mut parser).is_none());
        let diagnostics = parser.finish();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(
            diagnostics[0].message,
            "Variables cannot be declared with type void"
        );
    }

    #[test]
    fn rejects_untyped_temp_variable_declaration() {
        let line = line("~ temp score = 1");
        let mut parser = RuleParser::new(&line);

        assert!(temp_declaration_statement(&mut parser).is_none());
        let diagnostics = parser.finish();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(
            diagnostics[0].message,
            "Temp variable 'score' is missing a type"
        );
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
        assert!(matches!(
            inc.target(),
            AssignmentTarget::Variable(name) if name == "score"
        ));
        assert!(inc.is_increment());
    }

    #[test]
    fn parses_simple_variable_assignment_regression() {
        let line = line("~ score = 2");
        let mut parser = RuleParser::new(&line);
        let objects = assignment_statement(&mut parser).expect("expected assignment");

        assert!(parser.finish().is_empty());
        let Object::VariableAssignment(assignment) = &objects[0] else {
            panic!("expected variable assignment");
        };
        assert_eq!(assignment.name(), "score");
        assert!(matches!(
            assignment.target(),
            AssignmentTarget::Variable(name) if name == "score"
        ));
    }

    #[test]
    fn parses_qualified_assignment_target() {
        let line = line("~ items::count = 2");
        let mut parser = RuleParser::new(&line);
        let objects = assignment_statement(&mut parser).expect("expected qualified assignment");

        assert!(parser.finish().is_empty());
        let Object::VariableAssignment(assignment) = &objects[0] else {
            panic!("expected variable assignment");
        };
        assert_eq!(assignment.name(), "items::count");
        let AssignmentTarget::QualifiedVariable(name) = assignment.target() else {
            panic!("expected qualified target");
        };
        assert_eq!(name.module(), "items");
        assert_eq!(name.symbol(), "count");
    }

    #[test]
    fn parses_qualified_compound_assignment_target() {
        let line = line("~ items::count += 1");
        let mut parser = RuleParser::new(&line);
        let objects = assignment_statement(&mut parser).expect("expected qualified compound");

        assert!(parser.finish().is_empty());
        let Object::IncDec(inc_dec) = &objects[0] else {
            panic!("expected inc/dec");
        };
        assert_eq!(inc_dec.name(), "items::count");
        assert!(inc_dec.is_increment());
        let AssignmentTarget::QualifiedVariable(name) = inc_dec.target() else {
            panic!("expected qualified target");
        };
        assert_eq!(name.module(), "items");
        assert_eq!(name.symbol(), "count");
    }

    #[test]
    fn parses_field_assignment_target() {
        let line = line("~ state.hp = 10");
        let mut parser = RuleParser::new(&line);
        let objects = assignment_statement(&mut parser).expect("expected field assignment");

        assert!(parser.finish().is_empty());
        let Object::VariableAssignment(assignment) = &objects[0] else {
            panic!("expected variable assignment");
        };
        assert_eq!(assignment.name(), "state.hp");
        let AssignmentTarget::FieldAccess { base, field } = assignment.target() else {
            panic!("expected field target");
        };
        assert_eq!(field, "hp");
        assert!(matches!(base.as_ref(), AssignmentTarget::Variable(name) if name == "state"));
    }

    #[test]
    fn parses_index_assignment_target() {
        let line = line("~ items[1] = item");
        let mut parser = RuleParser::new(&line);
        let objects = assignment_statement(&mut parser).expect("expected index assignment");

        assert!(parser.finish().is_empty());
        let Object::VariableAssignment(assignment) = &objects[0] else {
            panic!("expected variable assignment");
        };
        let AssignmentTarget::IndexAccess { base, index } = assignment.target() else {
            panic!("expected index target");
        };
        assert!(matches!(base.as_ref(), AssignmentTarget::Variable(name) if name == "items"));
        assert!(matches!(index, Expression::NumberInt(1)));
    }

    #[test]
    fn parses_field_compound_assignment_target() {
        let line = line("~ state.hp += 1");
        let mut parser = RuleParser::new(&line);
        let objects = assignment_statement(&mut parser).expect("expected field compound");

        assert!(parser.finish().is_empty());
        let Object::IncDec(inc_dec) = &objects[0] else {
            panic!("expected inc/dec");
        };
        assert_eq!(inc_dec.name(), "state.hp");
        assert!(inc_dec.is_increment());
        assert!(matches!(inc_dec.expression(), Expression::NumberInt(1)));
        let AssignmentTarget::FieldAccess { base, field } = inc_dec.target() else {
            panic!("expected field target");
        };
        assert_eq!(field, "hp");
        assert!(matches!(base.as_ref(), AssignmentTarget::Variable(name) if name == "state"));
    }

    #[test]
    fn parses_index_compound_assignment_target() {
        let line = line("~ items[0] += 1");
        let mut parser = RuleParser::new(&line);
        let objects = assignment_statement(&mut parser).expect("expected index compound");

        assert!(parser.finish().is_empty());
        let Object::IncDec(inc_dec) = &objects[0] else {
            panic!("expected inc/dec");
        };
        assert!(inc_dec.is_increment());
        assert!(matches!(inc_dec.expression(), Expression::NumberInt(1)));
        let AssignmentTarget::IndexAccess { base, index } = inc_dec.target() else {
            panic!("expected index target");
        };
        assert!(matches!(base.as_ref(), AssignmentTarget::Variable(name) if name == "items"));
        assert!(matches!(index, Expression::NumberInt(0)));
    }

    #[test]
    fn parses_field_decrement_assignment_target() {
        let line = line("~ state.hp -= damage");
        let mut parser = RuleParser::new(&line);
        let objects = assignment_statement(&mut parser).expect("expected field decrement");

        assert!(parser.finish().is_empty());
        let Object::IncDec(inc_dec) = &objects[0] else {
            panic!("expected inc/dec");
        };
        assert!(!inc_dec.is_increment());
        assert!(matches!(
            inc_dec.expression(),
            Expression::VariableReference(name) if name == "damage"
        ));
        let AssignmentTarget::FieldAccess { base, field } = inc_dec.target() else {
            panic!("expected field target");
        };
        assert_eq!(field, "hp");
        assert!(matches!(base.as_ref(), AssignmentTarget::Variable(name) if name == "state"));
    }

    #[test]
    fn parses_nested_field_and_index_assignment_target() {
        let line = line("~ party[0].stats.hp = hp");
        let mut parser = RuleParser::new(&line);
        let objects = assignment_statement(&mut parser).expect("expected nested assignment");

        assert!(parser.finish().is_empty());
        let Object::VariableAssignment(assignment) = &objects[0] else {
            panic!("expected variable assignment");
        };
        assert_eq!(assignment.name(), "party[Number(0)].stats.hp");
        let AssignmentTarget::FieldAccess { base, field } = assignment.target() else {
            panic!("expected outer field target");
        };
        assert_eq!(field, "hp");
        let AssignmentTarget::FieldAccess { base, field } = base.as_ref() else {
            panic!("expected stats field target");
        };
        assert_eq!(field, "stats");
        assert!(matches!(
            base.as_ref(),
            AssignmentTarget::IndexAccess { base, index }
                if matches!(base.as_ref(), AssignmentTarget::Variable(name) if name == "party")
                    && matches!(index, Expression::NumberInt(0))
        ));
    }

    #[test]
    fn rejects_non_lvalue_assignment_target() {
        let line = line("~ get_state().hp = 10");
        let mut parser = RuleParser::new(&line);

        assert!(assignment_statement(&mut parser).is_none());
    }
}
