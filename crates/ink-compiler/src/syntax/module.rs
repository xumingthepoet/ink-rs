use std::collections::BTreeSet;

use crate::{parsed::ImplementedInterface, source::SourceSpan};

use super::{is_identifier, is_identifier_continue, rule::RuleParser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ModuleDecl {
    pub(super) name: String,
    pub(super) name_span: SourceSpan,
    pub(super) implemented_interfaces: Vec<ImplementedInterface>,
    pub(super) span: SourceSpan,
}

pub(crate) fn is_module_like_declaration_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('=') {
        return false;
    }

    let without_equals = trimmed.trim_start_matches('=').trim_start();
    let keyword = without_equals
        .chars()
        .take_while(|ch| is_identifier_continue(*ch))
        .collect::<String>();
    keyword.eq_ignore_ascii_case("module")
}

pub(super) fn parse_module_declaration(parser: &mut RuleParser<'_>) -> Option<ModuleDecl> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    let equals = parser.take_while(|ch| ch == '=')?;
    let equals_count = equals.chars().count();

    parser.skip_horizontal_whitespace();
    let keyword_span = parser.current_span();
    let keyword = parser.take_while(is_identifier_continue)?;
    if !keyword.eq_ignore_ascii_case("module") {
        return None;
    }

    if keyword != "module" {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            keyword_span,
            "Module declarations must use lowercase `module`",
        ));
        parser.skip_to_end();
        return None;
    }

    if equals_count != 3 {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            span,
            "Module declarations must use `=== module name ===`",
        ));
        parser.skip_to_end();
        return None;
    }

    parser.expect(
        "whitespace after 'module'",
        parse_horizontal_whitespace,
        |parser| parser.skip_to_end(),
    )?;

    let name_span = parser.current_span();
    let Some(name) = parser.take_while(|ch| !ch.is_whitespace() && ch != '=') else {
        parser.error("Expected module name but saw end of line");
        parser.skip_to_end();
        return None;
    };

    if name.contains('(') || name.contains(')') {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            name_span,
            "Module declarations do not accept parameters",
        ));
        parser.skip_to_end();
        return None;
    }

    if name.contains('.') {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            name_span,
            "Module names must be single identifiers; hierarchical module names are not supported",
        ));
        parser.skip_to_end();
        return None;
    }

    if !is_identifier(&name) {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            name_span,
            "Module name must be a single identifier",
        ));
        parser.skip_to_end();
        return None;
    }

    parser.skip_horizontal_whitespace();
    if parser.line_remainder().starts_with('(') {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            parser.current_span(),
            "Module declarations do not accept parameters",
        ));
        parser.skip_to_end();
        return None;
    }

    let implemented_interfaces = parse_optional_implementation_clause(parser)?;

    let suffix = parser.parse_rule(|parser| parser.take_while(|ch| ch == '='));
    if suffix.is_none_or(|suffix| suffix.chars().count() != 3) {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            parser.current_span(),
            "Module declarations must use `=== module name ===`",
        ));
        parser.skip_to_end();
        return None;
    }

    parser.skip_horizontal_whitespace();
    if !parser.line_remainder().is_empty() {
        parser.error(format!(
            "Expected end of line after module declaration but saw '{}'",
            parser.line_remainder()
        ));
        parser.skip_to_end();
        return None;
    }

    Some(ModuleDecl {
        name,
        name_span,
        implemented_interfaces,
        span,
    })
}

fn parse_optional_implementation_clause(
    parser: &mut RuleParser<'_>,
) -> Option<Vec<ImplementedInterface>> {
    parser.skip_horizontal_whitespace();
    if parser.line_remainder().starts_with('=') {
        return Some(Vec::new());
    }

    let keyword_span = parser.current_span();
    let Some(keyword) = parser.take_while(is_identifier_continue) else {
        return Some(Vec::new());
    };

    if !keyword.eq_ignore_ascii_case("implements") {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            keyword_span,
            format!("Expected `implements` or `===` after module name but saw '{keyword}'"),
        ));
        parser.skip_to_end();
        return None;
    }

    if keyword != "implements" {
        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            keyword_span,
            "Module implementation clauses must use lowercase `implements`",
        ));
        parser.skip_to_end();
        return None;
    }

    parser.expect(
        "whitespace after 'implements'",
        parse_horizontal_whitespace,
        |parser| parser.skip_to_end(),
    )?;

    parse_implemented_interfaces(parser)
}

fn parse_implemented_interfaces(parser: &mut RuleParser<'_>) -> Option<Vec<ImplementedInterface>> {
    let mut interfaces = Vec::new();
    let mut seen = BTreeSet::new();

    loop {
        parser.skip_horizontal_whitespace();
        let remainder = parser.line_remainder();
        if remainder.starts_with('=') {
            if interfaces.is_empty() {
                parser.diagnostic(crate::diagnostic::Diagnostic::error(
                    parser.current_span(),
                    "Module implementation clauses must include at least one interface name",
                ));
                parser.skip_to_end();
                return None;
            }
            return Some(interfaces);
        }

        if remainder.is_empty() {
            parser.diagnostic(crate::diagnostic::Diagnostic::error(
                parser.current_span(),
                "Module declarations must use `=== module name ===`",
            ));
            parser.skip_to_end();
            return None;
        }

        let interface_span = parser.current_span();
        let Some(name) = parser.take_while(|ch| !ch.is_whitespace() && ch != ',' && ch != '=')
        else {
            parser.diagnostic(crate::diagnostic::Diagnostic::error(
                parser.current_span(),
                "Module implementation clauses must include an interface name",
            ));
            parser.skip_to_end();
            return None;
        };

        if name.contains('(') || name.contains(')') {
            parser.diagnostic(crate::diagnostic::Diagnostic::error(
                interface_span,
                "Module implementation clauses do not accept interface parameters",
            ));
            parser.skip_to_end();
            return None;
        }

        if name.contains('.') {
            parser.diagnostic(crate::diagnostic::Diagnostic::error(
                interface_span,
                "Implemented interface names must be single identifiers; hierarchical interface names are not supported",
            ));
            parser.skip_to_end();
            return None;
        }

        if !is_identifier(&name) {
            parser.diagnostic(crate::diagnostic::Diagnostic::error(
                interface_span,
                "Implemented interface name must be a single identifier",
            ));
            parser.skip_to_end();
            return None;
        }

        if !seen.insert(name.clone()) {
            parser.diagnostic(crate::diagnostic::Diagnostic::error(
                interface_span,
                format!("Module implementation clauses must not repeat interface '{name}'"),
            ));
            parser.skip_to_end();
            return None;
        }

        interfaces.push(ImplementedInterface::new(name, interface_span));

        parser.skip_horizontal_whitespace();
        if parser.match_string(",").is_some() {
            parser.skip_horizontal_whitespace();
            if parser.line_remainder().is_empty() || parser.line_remainder().starts_with('=') {
                parser.diagnostic(crate::diagnostic::Diagnostic::error(
                    parser.current_span(),
                    "Module implementation clauses must include an interface name after ','",
                ));
                parser.skip_to_end();
                return None;
            }
            continue;
        }

        let remainder = parser.line_remainder();
        if remainder.starts_with('=') {
            return Some(interfaces);
        }

        parser.diagnostic(crate::diagnostic::Diagnostic::error(
            parser.current_span(),
            format!(
                "Expected ',' or `===` after implemented interface name but saw '{}'",
                remainder.trim()
            ),
        ));
        parser.skip_to_end();
        return None;
    }
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

    fn parse_module(source: &str) -> (Option<ModuleDecl>, Vec<crate::diagnostic::Diagnostic>) {
        let line = SourceLine {
            text: source.to_string(),
            span: SourceSpan::new(Some("module.ink".to_string()), 1, 1),
        };
        let mut parser = RuleParser::new(&line);
        let declaration = parser.parse_rule(parse_module_declaration);
        let diagnostics = parser.finish();
        (declaration, diagnostics)
    }

    #[test]
    fn parses_module_header() {
        let (declaration, diagnostics) = parse_module("=== module game ===");

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        let declaration = declaration.expect("expected module declaration");
        assert_eq!(declaration.name, "game");
        assert!(declaration.implemented_interfaces.is_empty());
        assert_eq!(
            declaration.name_span,
            SourceSpan::new(Some("module.ink".to_string()), 1, 12)
        );
    }

    #[test]
    fn parses_module_header_with_implemented_interfaces() {
        let (declaration, diagnostics) =
            parse_module("=== module left implements IItem, IOther ===");

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        let declaration = declaration.expect("expected module declaration");
        assert_eq!(declaration.name, "left");
        assert_eq!(
            declaration
                .implemented_interfaces
                .iter()
                .map(|interface| interface.name())
                .collect::<Vec<_>>(),
            vec!["IItem", "IOther"]
        );
        assert_eq!(
            declaration.implemented_interfaces[0].span(),
            &SourceSpan::new(Some("module.ink".to_string()), 1, 28)
        );
        assert_eq!(
            declaration.implemented_interfaces[1].span(),
            &SourceSpan::new(Some("module.ink".to_string()), 1, 35)
        );
    }

    #[test]
    fn rejects_invalid_module_header_forms() {
        let cases = [
            (
                "== module game ==",
                "Module declarations must use `=== module name ===`",
            ),
            (
                "= module game",
                "Module declarations must use `=== module name ===`",
            ),
            (
                "=== Module game ===",
                "Module declarations must use lowercase `module`",
            ),
            (
                "=== module game(seed) ===",
                "Module declarations do not accept parameters",
            ),
            (
                "=== module game.seed ===",
                "Module names must be single identifiers; hierarchical module names are not supported",
            ),
            (
                "=== module 123 ===",
                "Module name must be a single identifier",
            ),
            (
                "=== module game Implements IItem ===",
                "Module implementation clauses must use lowercase `implements`",
            ),
            (
                "=== module game implements ===",
                "Module implementation clauses must include at least one interface name",
            ),
            (
                "=== module game implements IItem, ===",
                "Module implementation clauses must include an interface name after ','",
            ),
            (
                "=== module game implements game.IItem ===",
                "Implemented interface names must be single identifiers; hierarchical interface names are not supported",
            ),
            (
                "=== module game implements 123 ===",
                "Implemented interface name must be a single identifier",
            ),
            (
                "=== module game implements IItem, IItem ===",
                "Module implementation clauses must not repeat interface 'IItem'",
            ),
        ];

        for (source, expected_message) in cases {
            let (declaration, diagnostics) = parse_module(source);

            assert!(declaration.is_none(), "{source}");
            assert_eq!(diagnostics.len(), 1, "{source}: {diagnostics:#?}");
            assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
            assert_eq!(diagnostics[0].message, expected_message);
        }
    }
}
