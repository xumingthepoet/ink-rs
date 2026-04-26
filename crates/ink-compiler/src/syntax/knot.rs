use crate::{
    diagnostic::Diagnostic,
    parsed::{FlowArgument, FlowLevel, TypeName},
};

use super::{is_identifier_continue, is_identifier_start, rule::RuleParser, type_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FlowDecl {
    pub level: FlowLevel,
    pub name: String,
    pub arguments: Vec<FlowArgument>,
    pub return_type: TypeName,
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
    let return_type = if is_function {
        parse_return_type(parser)
    } else {
        None
    };
    validate_function_signature(parser, is_function, &name, &arguments, return_type.as_ref());
    let return_type = return_type.unwrap_or_else(TypeName::void);

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
        return_type,
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

    let first_identifier = parser.expect("stitch name", parse_identifier, |parser| {
        parser.skip_to_end()
    })?;

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
    let return_type = if is_function {
        parse_return_type(parser)
    } else {
        None
    };
    validate_function_signature(parser, is_function, &name, &arguments, return_type.as_ref());
    let return_type = return_type.unwrap_or_else(TypeName::void);

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
        arguments,
        return_type,
        is_function,
    })
}

fn parse_return_type(parser: &mut RuleParser<'_>) -> Option<TypeName> {
    parser.parse_rule(|parser| {
        parser.skip_horizontal_whitespace();
        parser.match_string("->")?;
        parser.skip_horizontal_whitespace();
        parser.expect("return type", type_name::parse_type_name, |parser| {
            parser.skip_to_end();
        })
    })
}

fn validate_function_signature(
    parser: &mut RuleParser<'_>,
    is_function: bool,
    name: &str,
    arguments: &[FlowArgument],
    return_type: Option<&TypeName>,
) {
    if !is_function {
        return;
    }

    for argument in arguments {
        if argument.declared_type().is_none() {
            parser.diagnostic(Diagnostic::error(
                argument.span().clone(),
                format!("Function parameter '{}' is missing a type", argument.name()),
            ));
            return;
        }
    }

    if return_type.is_none() {
        parser.diagnostic(Diagnostic::error(
            parser.current_span(),
            format!("Function '{name}' is missing a return type"),
        ));
    }
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
    let declared_type = if parser.match_string(":").is_some() {
        let type_span = parser.current_span();
        parser.skip_horizontal_whitespace();
        let declared_type = type_name::parse_type_name(parser)?;
        if declared_type.is_void() {
            parser.diagnostic(Diagnostic::error(
                type_span,
                "Function parameters cannot be declared with type void",
            ));
        }
        Some(declared_type)
    } else {
        None
    };

    Some(FlowArgument::new(
        name,
        declared_type,
        is_by_reference,
        is_divert_target,
        span,
    ))
}

fn parse_identifier(parser: &mut RuleParser<'_>) -> Option<String> {
    let name = parser.take_while(is_identifier_continue)?;
    if !is_identifier_start(name.chars().next()?) || !super::is_identifier(&name) {
        return None;
    }
    Some(name)
}

fn parse_horizontal_whitespace(parser: &mut RuleParser<'_>) -> Option<String> {
    parser.take_while(|ch| matches!(ch, ' ' | '\t'))
}

#[cfg(test)]
mod tests {
    use crate::{
        diagnostic::DiagnosticSeverity,
        parsed::TypeName,
        source::{SourceLine, SourceSpan},
    };

    use super::*;

    fn line(text: &str) -> SourceLine {
        SourceLine {
            text: text.to_string(),
            span: SourceSpan::new(None, 1, 1),
        }
    }

    fn parse_knot(source: &str) -> (Option<FlowDecl>, Vec<crate::diagnostic::Diagnostic>) {
        let line = line(source);
        let mut parser = RuleParser::new(&line);
        let declaration = parser.parse_rule(parse_knot_declaration);
        let diagnostics = parser.finish();

        (declaration, diagnostics)
    }

    #[test]
    fn parses_primitive_function_signature() {
        let (declaration, diagnostics) = parse_knot("== function add(a: int, b: float) -> int ==");

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        let declaration = declaration.expect("expected function declaration");
        assert!(declaration.is_function);
        assert_eq!(declaration.return_type, TypeName::int());
        assert_eq!(declaration.arguments.len(), 2);
        assert_eq!(
            declaration.arguments[0].declared_type(),
            Some(&TypeName::int())
        );
        assert_eq!(
            declaration.arguments[1].declared_type(),
            Some(&TypeName::float())
        );
    }

    #[test]
    fn parses_array_struct_and_nested_array_function_return_types() {
        let cases = [
            (
                "== function ids(source: Player) -> int[] ==",
                TypeName::array(TypeName::int()),
            ),
            (
                "== function make_player(seed: int) -> Player ==",
                TypeName::struct_type("Player"),
            ),
            (
                "== function make_grid(rows: int) -> Player[][] ==",
                TypeName::array(TypeName::array(TypeName::struct_type("Player"))),
            ),
        ];

        for (source, expected_return_type) in cases {
            let (declaration, diagnostics) = parse_knot(source);

            assert!(diagnostics.is_empty(), "{source}: {diagnostics:#?}");
            let declaration = declaration.expect("expected function declaration");
            assert_eq!(declaration.return_type, expected_return_type);
            assert!(declaration
                .arguments
                .iter()
                .all(|argument| argument.declared_type().is_some()));
        }
    }

    #[test]
    fn rejects_missing_function_return_type() {
        let (declaration, diagnostics) = parse_knot("== function log(message: string) ==");

        assert!(declaration.is_some());
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Function 'log' is missing a return type"
        );
    }

    #[test]
    fn rejects_missing_parameter_type_in_typed_function_signature() {
        let (declaration, diagnostics) = parse_knot("== function add(a: int, b) -> int ==");

        assert!(declaration.is_some());
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Function parameter 'b' is missing a type"
        );
    }

    #[test]
    fn rejects_void_function_parameter_type() {
        let (declaration, diagnostics) = parse_knot("== function noop(value: void) -> void ==");

        assert!(declaration.is_some());
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Function parameters cannot be declared with type void"
        );
    }

    #[test]
    fn rejects_missing_types_in_function_signature() {
        let (declaration, diagnostics) = parse_knot("== function missing_types(a, b) ==");

        assert!(declaration.is_some());
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Function parameter 'a' is missing a type"
        );
    }
}
