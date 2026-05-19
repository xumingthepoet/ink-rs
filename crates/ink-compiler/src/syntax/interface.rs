use crate::source::SourceSpan;

use super::{is_identifier, is_identifier_continue, rule::RuleParser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InterfaceDecl {
    pub(super) name: String,
    pub(super) name_span: SourceSpan,
    pub(super) span: SourceSpan,
}

pub(crate) fn is_interface_like_declaration_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('=') {
        return false;
    }

    let without_equals = trimmed.trim_start_matches('=').trim_start();
    let keyword = without_equals
        .chars()
        .take_while(|ch| is_identifier_continue(*ch))
        .collect::<String>();
    keyword.eq_ignore_ascii_case("interface")
}

pub(super) fn parse_interface_declaration(parser: &mut RuleParser<'_>) -> Option<InterfaceDecl> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    let equals = parser.take_while(|ch| ch == '=')?;
    let equals_count = equals.chars().count();

    parser.skip_horizontal_whitespace();
    let keyword_span = parser.current_span();
    let keyword = parser.take_while(is_identifier_continue)?;
    if !keyword.eq_ignore_ascii_case("interface") {
        return None;
    }

    if keyword != "interface" {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            keyword_span,
            "Interface declarations must use lowercase `interface`",
        ));
        parser.skip_to_end();
        return None;
    }

    if equals_count != 3 {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            span,
            "Interface declarations must use `=== interface name ===`",
        ));
        parser.skip_to_end();
        return None;
    }

    parser.expect(
        "whitespace after 'interface'",
        parse_horizontal_whitespace,
        |parser| parser.skip_to_end(),
    )?;

    let name_span = parser.current_span();
    let Some(name) = parser.take_while(|ch| !ch.is_whitespace() && ch != '=') else {
        parser.error("Expected interface name but saw end of line");
        parser.skip_to_end();
        return None;
    };

    if name.contains('(') || name.contains(')') {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            name_span,
            "Interface declarations do not accept parameters",
        ));
        parser.skip_to_end();
        return None;
    }

    if name.contains('.') {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            name_span,
            "Interface names must be single identifiers; hierarchical interface names are not supported",
        ));
        parser.skip_to_end();
        return None;
    }

    if !is_identifier(&name) {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            name_span,
            "Interface name must be a single identifier",
        ));
        parser.skip_to_end();
        return None;
    }

    parser.skip_horizontal_whitespace();
    if parser.line_remainder().starts_with('(') {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            parser.current_span(),
            "Interface declarations do not accept parameters",
        ));
        parser.skip_to_end();
        return None;
    }

    let suffix = parser.parse_rule(|parser| parser.take_while(|ch| ch == '='));
    if suffix.is_none_or(|suffix| suffix.chars().count() != 3) {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            parser.current_span(),
            "Interface declarations must use `=== interface name ===`",
        ));
        parser.skip_to_end();
        return None;
    }

    parser.skip_horizontal_whitespace();
    if !parser.line_remainder().is_empty() {
        parser.error(format!(
            "Expected end of line after interface declaration but saw '{}'",
            parser.line_remainder()
        ));
        parser.skip_to_end();
        return None;
    }

    Some(InterfaceDecl {
        name,
        name_span,
        span,
    })
}

fn parse_horizontal_whitespace(parser: &mut RuleParser<'_>) -> Option<String> {
    parser.take_while(|ch| matches!(ch, ' ' | '\t'))
}

#[cfg(test)]
mod tests {
    use crate::{
        diagnostic::DiagnosticSeverity,
        source::{SourceLine, SourceSpan},
    };

    use super::*;

    fn parse_interface(
        source: &str,
    ) -> (Option<InterfaceDecl>, Vec<crate::diagnostic::Diagnostic>) {
        let line = SourceLine {
            text: source.to_string(),
            span: SourceSpan::new(Some("interface.ink".to_string()), 1, 1),
        };
        let mut parser = RuleParser::new(&line);
        let declaration = parser.parse_rule(parse_interface_declaration);
        let diagnostics = parser.finish();
        (declaration, diagnostics)
    }

    #[test]
    fn parses_interface_header() {
        let (declaration, diagnostics) = parse_interface("=== interface IItem ===");

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        let declaration = declaration.expect("expected interface declaration");
        assert_eq!(declaration.name, "IItem");
        assert_eq!(
            declaration.name_span,
            SourceSpan::new(Some("interface.ink".to_string()), 1, 15)
        );
    }

    #[test]
    fn rejects_invalid_interface_header_forms() {
        let cases = [
            (
                "== interface IItem ==",
                "Interface declarations must use `=== interface name ===`",
            ),
            (
                "= interface IItem",
                "Interface declarations must use `=== interface name ===`",
            ),
            (
                "=== Interface IItem ===",
                "Interface declarations must use lowercase `interface`",
            ),
            (
                "=== interface IItem(seed) ===",
                "Interface declarations do not accept parameters",
            ),
            (
                "=== interface game.IItem ===",
                "Interface names must be single identifiers; hierarchical interface names are not supported",
            ),
            (
                "=== interface 123 ===",
                "Interface name must be a single identifier",
            ),
        ];

        for (source, expected_message) in cases {
            let (declaration, diagnostics) = parse_interface(source);

            assert!(declaration.is_none(), "{source}");
            assert_eq!(diagnostics.len(), 1, "{source}: {diagnostics:#?}");
            assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
            assert_eq!(diagnostics[0].message, expected_message);
        }
    }
}
