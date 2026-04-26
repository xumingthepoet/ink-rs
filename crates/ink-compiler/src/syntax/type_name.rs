use crate::parsed::TypeName;

use super::{error, is_identifier, is_identifier_continue, rule::RuleParser};

pub(super) fn parse_type_name(parser: &mut RuleParser<'_>) -> Option<TypeName> {
    parser.parse_rule(|parser| {
        parser.skip_horizontal_whitespace();

        let mut type_name = if parser.match_string("->").is_some() {
            TypeName::divert_target()
        } else {
            let name = parser.take_while(is_identifier_continue)?;
            if !is_identifier(&name) {
                parser.error(format!("Expected type name but saw '{name}'"));
                return None;
            }

            match name.as_str() {
                "int" => TypeName::int(),
                "float" => TypeName::float(),
                "bool" => TypeName::bool(),
                "string" => TypeName::string(),
                "void" => TypeName::void(),
                _ => TypeName::struct_type(name),
            }
        };

        loop {
            parser.skip_horizontal_whitespace();
            if parser.match_string("[").is_none() {
                break;
            }
            parser.skip_horizontal_whitespace();
            if parser.match_string("]").is_none() {
                parser.error(error::expected_message("']'", parser.line_remainder()));
                parser.skip_to_end();
                return None;
            }

            type_name = TypeName::array(type_name);
        }

        Some(type_name)
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        diagnostic::{Diagnostic, DiagnosticSeverity},
        parsed::TypeName,
        source::{SourceLine, SourceSpan},
    };

    use super::{parse_type_name, RuleParser};

    fn parse_type_name_text(source: &str) -> (Option<TypeName>, String, Vec<Diagnostic>) {
        let line = SourceLine {
            text: source.to_string(),
            span: SourceSpan::new(None, 1, 1),
        };
        let mut parser = RuleParser::new(&line);
        let type_name = parser.expect("type name", parse_type_name, |parser| {
            parser.skip_to_end();
        });
        let remainder = parser.line_remainder().to_string();
        let diagnostics = parser.finish();

        (type_name, remainder, diagnostics)
    }

    #[test]
    fn parses_primitive_struct_and_void_type_names() {
        let cases = [
            ("int", TypeName::int()),
            ("float", TypeName::float()),
            ("bool", TypeName::bool()),
            ("string", TypeName::string()),
            ("->", TypeName::divert_target()),
            ("void", TypeName::void()),
            ("Player", TypeName::struct_type("Player")),
        ];

        for (source, expected) in cases {
            let (parsed, remainder, diagnostics) = parse_type_name_text(source);

            assert_eq!(parsed, Some(expected));
            assert_eq!(remainder, "");
            assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        }
    }

    #[test]
    fn parses_nested_array_type_names() {
        let cases = [
            ("int[]", TypeName::array(TypeName::int())),
            ("->[]", TypeName::array(TypeName::divert_target())),
            (
                "Player[][]",
                TypeName::array(TypeName::array(TypeName::struct_type("Player"))),
            ),
        ];

        for (source, expected) in cases {
            let (parsed, remainder, diagnostics) = parse_type_name_text(source);

            assert_eq!(parsed, Some(expected));
            assert_eq!(remainder, "");
            assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        }
    }

    #[test]
    fn reports_invalid_type_syntax() {
        let cases = [
            ("", "Expected type name but saw end of line"),
            ("[]", "Expected type name but saw '[]'"),
            ("123", "Expected type name but saw '123'"),
            ("int[", "Expected ']' but saw end of line"),
        ];

        for (source, expected_message) in cases {
            let (parsed, _remainder, diagnostics) = parse_type_name_text(source);

            assert_eq!(parsed, None);
            assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
            assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
            assert_eq!(diagnostics[0].message, expected_message);
        }
    }
}
