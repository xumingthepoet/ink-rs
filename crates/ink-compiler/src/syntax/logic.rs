use crate::parsed::{ContentList, Expression, Object, Return, Text};

use super::{is_identifier_continue, parse_expression_remainder, rule::RuleParser};

pub(super) fn return_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    parser.match_string("~")?;
    parser.skip_horizontal_whitespace();
    let keyword = parser.take_while(is_identifier_continue)?;
    if keyword != "return" {
        return None;
    }
    parser.skip_horizontal_whitespace();
    let expression = if parser.line_remainder().trim().is_empty() {
        None
    } else {
        Some(parse_expression_remainder(parser)?)
    };
    parser.skip_to_end();
    let ret = Object::Return(Return::new(expression, span.clone()));
    if object_contains_function_call(&ret) {
        Some(vec![Object::ContentList(ContentList::new(vec![
            ret,
            Object::Text(Text::new("\n", span)),
        ]))])
    } else {
        Some(vec![ret])
    }
}

pub(super) fn line_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    parser.match_string("~")?;
    parser.skip_horizontal_whitespace();
    let expression = parse_expression_remainder(parser)?;
    parser.skip_to_end();
    Some(vec![Object::LogicLine(expression)])
}

pub(super) fn expression_contains_function_call(expr: &Expression) -> bool {
    match expr {
        Expression::FunctionCall { .. } | Expression::QualifiedFunctionCall { .. } => true,
        Expression::StringContent(content) => {
            content.objects().iter().any(object_contains_function_call)
        }
        Expression::ArrayLiteral(elements) => {
            elements.iter().any(expression_contains_function_call)
        }
        Expression::StructLiteral(fields) => fields
            .iter()
            .any(|field| expression_contains_function_call(field.expression())),
        Expression::FieldAccess { base, .. } => expression_contains_function_call(base),
        Expression::IndexAccess { base, index } => {
            expression_contains_function_call(base) || expression_contains_function_call(index)
        }
        Expression::Binary { left, right, .. } => {
            expression_contains_function_call(left) || expression_contains_function_call(right)
        }
        Expression::Unary { expression, .. } => expression_contains_function_call(expression),
        Expression::MultipleCondition(expressions) => {
            expressions.iter().any(expression_contains_function_call)
        }
        _ => false,
    }
}

fn object_contains_function_call(object: &Object) -> bool {
    match object {
        Object::Expression(expression) | Object::LogicLine(expression) => {
            expression_contains_function_call(expression)
        }
        Object::ContentList(content) => content.objects().iter().any(object_contains_function_call),
        Object::Conditional(conditional) => {
            conditional
                .initial_condition()
                .is_some_and(expression_contains_function_call)
                || conditional.branches().iter().any(|branch| {
                    branch
                        .own_condition()
                        .is_some_and(expression_contains_function_call)
                        || branch
                            .content()
                            .content()
                            .iter()
                            .any(object_contains_function_call)
                })
        }
        Object::Choice(choice) => {
            choice
                .condition()
                .is_some_and(expression_contains_function_call)
                || choice.start_content().is_some_and(|content| {
                    content.objects().iter().any(object_contains_function_call)
                })
                || choice
                    .inner_content()
                    .objects()
                    .iter()
                    .any(object_contains_function_call)
        }
        Object::VariableAssignment(assignment) => assignment
            .expression()
            .is_some_and(expression_contains_function_call),
        Object::IncDec(inc_dec) => expression_contains_function_call(inc_dec.expression()),
        Object::Return(ret) => ret
            .returned_expression()
            .is_some_and(expression_contains_function_call),
        Object::Weave(weave) => weave.content().iter().any(object_contains_function_call),
        Object::Text(_)
        | Object::AuthorWarning(_)
        | Object::ConstantDeclaration(_)
        | Object::Glue(_)
        | Object::Divert(_)
        | Object::TunnelOnwards(_)
        | Object::Gather(_)
        | Object::ExternalDeclaration(_)
        | Object::EnumDeclaration(_)
        | Object::StructDeclaration(_)
        | Object::Tag(_) => false,
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
    fn parses_return_without_expression() {
        let line = line("~ return");
        let mut parser = RuleParser::new(&line);
        let objects = return_statement(&mut parser).expect("expected return");

        assert!(parser.finish().is_empty());
        let Object::Return(ret) = &objects[0] else {
            panic!("expected return");
        };
        assert!(ret.returned_expression().is_none());
    }

    #[test]
    fn parses_logic_line_expression() {
        let line = line("~ score + 1");
        let mut parser = RuleParser::new(&line);
        let objects = line_statement(&mut parser).expect("expected logic line");

        assert!(parser.finish().is_empty());
        assert!(matches!(objects[0], Object::LogicLine(_)));
    }
}
