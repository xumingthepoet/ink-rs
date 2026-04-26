use crate::{
    diagnostic::Diagnostic,
    parsed::{ConstantDeclaration, ExternalDeclaration, Object, TypeName},
};

use super::{
    is_identifier, is_identifier_continue, parse_expression_remainder, rule::RuleParser, type_name,
};

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
    if parser.match_string(":").is_none() {
        parser.error(format!("Constant '{name}' is missing a type"));
        parser.skip_to_end();
        return None;
    }
    let declared_type = type_name::parse_type_name(parser)?;
    if declared_type.is_void() {
        parser.error("Constants cannot be declared with type void");
        return None;
    }
    parser.skip_horizontal_whitespace();
    parser.match_string("=")?;
    parser.skip_horizontal_whitespace();
    let expression = parse_expression_remainder(parser)?;
    parser.skip_to_end();

    Some(vec![Object::ConstantDeclaration(ConstantDeclaration::new(
        name,
        declared_type,
        expression,
        span,
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
            arguments.push(parse_external_argument(parser)?);
            parser.skip_horizontal_whitespace();

            if parser.match_string(")").is_some() {
                break;
            }
            parser.match_string(",")?;
        }
    }
    parser.skip_horizontal_whitespace();
    let return_type = parse_external_return_type(parser);
    validate_external_signature(parser, &name, &arguments, return_type.as_ref());
    if parser.had_error() {
        return None;
    }

    if !parser.line_remainder().is_empty() {
        return None;
    }
    parser.skip_to_end();

    let return_type = return_type?;
    let mut typed_arguments = Vec::with_capacity(arguments.len());
    for (name, declared_type) in arguments {
        typed_arguments.push((name, declared_type?));
    }
    let declaration = ExternalDeclaration::with_signature(name, typed_arguments, return_type);

    Some(vec![Object::ExternalDeclaration(declaration)])
}

fn parse_external_argument(parser: &mut RuleParser<'_>) -> Option<(String, Option<TypeName>)> {
    parser.skip_horizontal_whitespace();
    let argument = parser.take_while(is_identifier_continue)?;
    if !is_identifier(&argument) {
        return None;
    }
    parser.skip_horizontal_whitespace();
    let declared_type = if parser.match_string(":").is_some() {
        let type_span = parser.current_span();
        parser.skip_horizontal_whitespace();
        let declared_type = type_name::parse_type_name(parser)?;
        if declared_type.is_void() {
            parser.diagnostic(Diagnostic::error(
                type_span,
                "External parameters cannot be declared with type void",
            ));
        }
        Some(declared_type)
    } else {
        None
    };

    Some((argument, declared_type))
}

fn parse_external_return_type(parser: &mut RuleParser<'_>) -> Option<TypeName> {
    parser.parse_rule(|parser| {
        parser.skip_horizontal_whitespace();
        parser.match_string("=>")?;
        parser.skip_horizontal_whitespace();
        parser.expect("return type", type_name::parse_type_name, |parser| {
            parser.skip_to_end();
        })
    })
}

fn validate_external_signature(
    parser: &mut RuleParser<'_>,
    name: &str,
    arguments: &[(String, Option<TypeName>)],
    return_type: Option<&TypeName>,
) {
    for (argument, declared_type) in arguments {
        if declared_type.is_none() {
            parser.diagnostic(Diagnostic::error(
                parser.current_span(),
                format!("External parameter '{argument}' is missing a type"),
            ));
            return;
        }
    }

    if return_type.is_none() {
        parser.diagnostic(Diagnostic::error(
            parser.current_span(),
            format!("External declaration '{name}' is missing a return type"),
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        diagnostic::DiagnosticSeverity,
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
    fn parses_constant_declaration() {
        let line = line("CONST max_score: int = 5");
        let mut parser = RuleParser::new(&line);
        let objects = constant_statement(&mut parser).expect("expected constant declaration");

        assert!(parser.finish().is_empty());
        let Object::ConstantDeclaration(declaration) = &objects[0] else {
            panic!("expected constant declaration");
        };
        assert_eq!(declaration.name(), "max_score");
        assert_eq!(declaration.declared_type(), &TypeName::int());
    }

    #[test]
    fn rejects_untyped_constant_declaration() {
        let line = line("CONST max_score = 5");
        let mut parser = RuleParser::new(&line);

        assert!(constant_statement(&mut parser).is_none());
        let diagnostics = parser.finish();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Constant 'max_score' is missing a type"
        );
    }

    #[test]
    fn rejects_void_constant_declaration() {
        let line = line("CONST value: void = 5");
        let mut parser = RuleParser::new(&line);

        assert!(constant_statement(&mut parser).is_none());
        let diagnostics = parser.finish();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Constants cannot be declared with type void"
        );
    }

    #[test]
    fn rejects_untyped_external_declaration_arguments() {
        let line = line("EXTERNAL play_sound(name, volume)");
        let mut parser = RuleParser::new(&line);

        assert!(external_statement(&mut parser).is_none());
        let diagnostics = parser.finish();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "External parameter 'name' is missing a type"
        );
    }

    #[test]
    fn parses_external_divert_target_return_type() {
        let line = line("EXTERNAL choose(name: string) => ->");
        let mut parser = RuleParser::new(&line);
        let objects = external_statement(&mut parser).expect("expected external declaration");

        assert!(parser.finish().is_empty());
        let Object::ExternalDeclaration(declaration) = &objects[0] else {
            panic!("expected external declaration");
        };
        assert_eq!(declaration.return_type(), &TypeName::divert_target());
    }

    #[test]
    fn parses_typed_external_declaration_signature() {
        let line = line("EXTERNAL is_ready(a: int, b: string) => bool");
        let mut parser = RuleParser::new(&line);
        let objects = external_statement(&mut parser).expect("expected external declaration");

        assert!(parser.finish().is_empty());
        let Object::ExternalDeclaration(declaration) = &objects[0] else {
            panic!("expected external declaration");
        };
        assert_eq!(declaration.name(), "is_ready");
        assert_eq!(declaration.argument_names(), ["a", "b"]);
        assert_eq!(
            declaration.argument_types(),
            &[TypeName::int(), TypeName::string()]
        );
        assert_eq!(declaration.return_type(), &TypeName::bool());
    }

    #[test]
    fn rejects_typed_external_with_missing_argument_type() {
        let line = line("EXTERNAL is_ready(a: int, b) => bool");
        let mut parser = RuleParser::new(&line);

        assert!(external_statement(&mut parser).is_none());
        let diagnostics = parser.finish();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "External parameter 'b' is missing a type"
        );
    }

    #[test]
    fn rejects_typed_external_with_missing_return_type() {
        let line = line("EXTERNAL is_ready(a: int, b: string)");
        let mut parser = RuleParser::new(&line);

        assert!(external_statement(&mut parser).is_none());
        let diagnostics = parser.finish();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "External declaration 'is_ready' is missing a return type"
        );
    }

    #[test]
    fn rejects_void_external_parameter_type() {
        let line = line("EXTERNAL noop(value: void) => void");
        let mut parser = RuleParser::new(&line);

        assert!(external_statement(&mut parser).is_none());
        let diagnostics = parser.finish();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "External parameters cannot be declared with type void"
        );
    }
}
