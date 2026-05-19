use crate::{
    diagnostic::Diagnostic,
    parsed::{ImportDeclaration, ImportedName},
    source::{SourceLine, SourceSpan},
};

use super::{is_identifier, is_identifier_continue, rule::RuleParser};

pub(super) fn is_import_like_declaration_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    let keyword = trimmed
        .chars()
        .take_while(|ch| is_identifier_continue(*ch))
        .collect::<String>();
    keyword.eq_ignore_ascii_case("FROM") || keyword.eq_ignore_ascii_case("IMPORT")
}

pub(super) fn parse_import_declaration(parser: &mut RuleParser<'_>) -> Option<ImportDeclaration> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    let keyword_span = parser.current_span();
    let keyword = parser.take_while(is_identifier_continue)?;

    if keyword.eq_ignore_ascii_case("IMPORT") {
        parser.diagnostic(obsolete_import_syntax_diagnostic(
            keyword_span,
            parser.line_remainder(),
        ));
        parser.skip_to_end();
        return None;
    }

    if !keyword.eq_ignore_ascii_case("FROM") {
        return None;
    }

    if keyword != "FROM" {
        parser.diagnostic(Diagnostic::error(
            keyword_span,
            "Import declarations must use uppercase `FROM`",
        ));
        parser.skip_to_end();
        return None;
    }

    if parser.line_remainder().is_empty() {
        parser.diagnostic(Diagnostic::error(
            parser.current_span(),
            "FROM declarations must include a source module name",
        ));
        parser.skip_to_end();
        return None;
    }
    parser.expect(
        "whitespace after 'FROM'",
        parse_horizontal_whitespace,
        |parser| parser.skip_to_end(),
    )?;
    parser.skip_horizontal_whitespace();

    let source_module_span = parser.current_span();
    let Some(source_module) = parser.take_while(|ch| !ch.is_whitespace()) else {
        parser.diagnostic(Diagnostic::error(
            parser.current_span(),
            "FROM declarations must include a source module name",
        ));
        parser.skip_to_end();
        return None;
    };

    if source_module.contains('.') {
        parser.diagnostic(Diagnostic::error(
            source_module_span,
            "Import source module must be a single identifier; hierarchical module names are not supported",
        ));
        parser.skip_to_end();
        return None;
    }

    if !is_identifier(&source_module) {
        parser.diagnostic(Diagnostic::error(
            source_module_span,
            "Import source module must be a single identifier",
        ));
        parser.skip_to_end();
        return None;
    }

    parser.skip_horizontal_whitespace();
    if parser.line_remainder().is_empty() {
        return Some(ImportDeclaration::module(
            source_module,
            source_module_span,
            span,
        ));
    }

    let import_keyword_span = parser.current_span();
    let Some(import_keyword) = parser.take_while(is_identifier_continue) else {
        parser.diagnostic(Diagnostic::error(
            parser.current_span(),
            format!(
                "Expected `IMPORT` or end of line after source module but saw '{}'",
                parser.line_remainder().trim()
            ),
        ));
        parser.skip_to_end();
        return None;
    };

    if !import_keyword.eq_ignore_ascii_case("IMPORT") {
        parser.diagnostic(Diagnostic::error(
            import_keyword_span,
            format!(
                "Expected `IMPORT` or end of line after source module but saw '{import_keyword}'"
            ),
        ));
        parser.skip_to_end();
        return None;
    }

    if import_keyword != "IMPORT" {
        parser.diagnostic(Diagnostic::error(
            import_keyword_span,
            "Import declarations must use uppercase `IMPORT`",
        ));
        parser.skip_to_end();
        return None;
    }

    if parser.line_remainder().is_empty() {
        parser.diagnostic(Diagnostic::error(
            parser.current_span(),
            "FROM ... IMPORT declarations must name at least one symbol",
        ));
        parser.skip_to_end();
        return None;
    }
    parser.expect(
        "whitespace after 'IMPORT'",
        parse_horizontal_whitespace,
        |parser| parser.skip_to_end(),
    )?;

    let imported_names = parse_imported_names(parser, &source_module)?;
    parser.skip_horizontal_whitespace();

    if !parser.line_remainder().is_empty() {
        parser.error(format!(
            "Expected end of line after FROM import declaration but saw '{}'",
            parser.line_remainder()
        ));
        parser.skip_to_end();
        return None;
    }

    Some(ImportDeclaration::symbols(
        imported_names,
        source_module,
        source_module_span,
        span,
    ))
}

pub(super) fn parse_import_declaration_lines(
    lines: &[SourceLine],
    index: usize,
) -> (Option<ImportDeclaration>, Vec<Diagnostic>, usize) {
    let line = &lines[index];
    let obsolete_multiline_import = is_obsolete_multiline_import_start(line);
    let mut line_parser = RuleParser::new(line);
    let declaration = line_parser.parse_rule(parse_import_declaration);
    let had_error = line_parser.had_error();
    let diagnostics = line_parser.finish();
    let next_index = if obsolete_multiline_import {
        consume_obsolete_multiline_import(lines, index)
    } else {
        index + 1
    };

    (declaration.filter(|_| !had_error), diagnostics, next_index)
}

fn is_obsolete_multiline_import_start(line: &SourceLine) -> bool {
    let trimmed = line.text.trim_start();
    let keyword = trimmed
        .chars()
        .take_while(|ch| is_identifier_continue(*ch))
        .collect::<String>();
    if !keyword.eq_ignore_ascii_case("IMPORT") {
        return false;
    }
    let rest = &trimmed[keyword.len()..];
    rest.starts_with([' ', '\t']) && rest.trim_start_matches([' ', '\t']).starts_with('{')
}

fn consume_obsolete_multiline_import(lines: &[SourceLine], start_index: usize) -> usize {
    let mut index = start_index + 1;
    while index < lines.len() {
        if lines[index].text.trim_start().starts_with('}') {
            return index + 1;
        }
        index += 1;
    }
    index
}

fn parse_imported_names(
    parser: &mut RuleParser<'_>,
    source_module: &str,
) -> Option<Vec<ImportedName>> {
    parser.skip_horizontal_whitespace();
    if parser.line_remainder().is_empty() {
        parser.diagnostic(Diagnostic::error(
            parser.current_span(),
            "FROM ... IMPORT declarations must name at least one symbol",
        ));
        parser.skip_to_end();
        return None;
    }

    let mut imported_names = Vec::new();
    loop {
        parser.skip_horizontal_whitespace();
        let name_span = parser.current_span();

        if parser.line_remainder().starts_with(',') {
            parser.diagnostic(Diagnostic::error(
                name_span,
                "IMPORT declarations must include an imported symbol name before ','",
            ));
            parser.skip_to_end();
            return None;
        }

        let Some(name) = parser.take_while(|ch| !ch.is_whitespace() && ch != ',') else {
            parser.diagnostic(Diagnostic::error(
                parser.current_span(),
                "IMPORT declarations must include an imported symbol name",
            ));
            parser.skip_to_end();
            return None;
        };

        if name.contains("::") || name.contains(':') {
            parser.diagnostic(Diagnostic::error(
                name_span,
                "IMPORT names must be unqualified identifiers",
            ));
            parser.skip_to_end();
            return None;
        }

        if !is_identifier(&name) {
            parser.diagnostic(Diagnostic::error(
                name_span,
                "IMPORT names must be single identifiers",
            ));
            parser.skip_to_end();
            return None;
        }

        if imported_names.is_empty() && name == source_module {
            parser.diagnostic(Diagnostic::error(
                name_span,
                format!(
                    "Module literals must be imported with `FROM {source_module}`, not `FROM {source_module} IMPORT {source_module}`"
                ),
            ));
            parser.skip_to_end();
            return None;
        }

        imported_names.push(ImportedName::new(name.clone(), name_span.clone()));
        parser.skip_horizontal_whitespace();

        if parser.match_string(",").is_some() {
            parser.skip_horizontal_whitespace();
            if parser.line_remainder().is_empty() {
                parser.diagnostic(Diagnostic::error(
                    parser.current_span(),
                    "IMPORT declarations must include an imported symbol name after ','",
                ));
                parser.skip_to_end();
                return None;
            }
            continue;
        }

        let remainder = parser.line_remainder();
        if remainder.is_empty() {
            break;
        }

        if remainder
            .strip_prefix("AS")
            .or_else(|| remainder.strip_prefix("as"))
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            parser.diagnostic(Diagnostic::error(
                parser.current_span(),
                "Import aliases are not supported in the first module-support phase",
            ));
            parser.skip_to_end();
            return None;
        }

        let next_token = remainder
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string();
        if is_import_kind_keyword(&name) && is_identifier(&next_token) {
            parser.diagnostic(Diagnostic::error(
                name_span,
                "IMPORT names do not include kind annotations",
            ));
            parser.skip_to_end();
            return None;
        }

        parser.diagnostic(Diagnostic::error(
            parser.current_span(),
            format!("Expected ',' or end of line after imported name but saw '{remainder}'"),
        ));
        parser.skip_to_end();
        return None;
    }

    Some(imported_names)
}

fn obsolete_import_syntax_diagnostic(span: SourceSpan, remainder: &str) -> Diagnostic {
    if let Some((names, source_module)) = obsolete_import_parts(remainder) {
        if names == source_module && is_identifier(names) {
            return Diagnostic::error(
                span,
                format!(
                    "Module literals must be imported with `FROM {source_module}`; `IMPORT {names} FROM {source_module}` is obsolete"
                ),
            );
        }

        return Diagnostic::error(
            span,
            format!(
                "Old import syntax `IMPORT {names} FROM {source_module}` has been replaced by `FROM {source_module} IMPORT {names}`"
            ),
        );
    }

    Diagnostic::error(
        span,
        "Old import syntax `IMPORT symbol FROM module` has been replaced by `FROM module IMPORT symbol`",
    )
}

fn obsolete_import_parts(remainder: &str) -> Option<(&str, &str)> {
    let trimmed = remainder.trim();
    let (names, source_module) = trimmed.split_once(" FROM ")?;
    let source_module = source_module.split_whitespace().next()?;
    Some((names.trim(), source_module.trim()))
}

fn is_import_kind_keyword(name: &str) -> bool {
    matches!(
        name,
        "knot" | "function" | "const" | "var" | "struct" | "external"
    )
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

    fn parse_import(
        source: &str,
    ) -> (
        Option<ImportDeclaration>,
        Vec<crate::diagnostic::Diagnostic>,
    ) {
        let line = SourceLine {
            text: source.to_string(),
            span: SourceSpan::new(Some("module.ink".to_string()), 1, 1),
        };
        let mut parser = RuleParser::new(&line);
        let declaration = parser.parse_rule(parse_import_declaration);
        let diagnostics = parser.finish();
        (declaration, diagnostics)
    }

    #[test]
    fn parses_symbol_import_declaration() {
        let (declaration, diagnostics) = parse_import("FROM items IMPORT sword, heal");

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        let declaration = declaration.expect("expected import declaration");
        assert!(declaration.is_symbol_import());
        assert!(!declaration.is_module_import());
        assert_eq!(declaration.source_module(), "items");
        assert_eq!(declaration.imported_names().len(), 2);
        assert_eq!(declaration.imported_names()[0].name(), "sword");
        assert_eq!(
            declaration.imported_names()[0].span(),
            &SourceSpan::new(Some("module.ink".to_string()), 1, 19)
        );
        assert_eq!(declaration.imported_names()[1].name(), "heal");
        assert_eq!(
            declaration.source_module_span(),
            &SourceSpan::new(Some("module.ink".to_string()), 1, 6)
        );
    }

    #[test]
    fn parses_module_import_declaration() {
        let (declaration, diagnostics) = parse_import("FROM items");

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        let declaration = declaration.expect("expected import declaration");
        assert!(declaration.is_module_import());
        assert!(!declaration.is_symbol_import());
        assert_eq!(declaration.source_module(), "items");
        assert!(declaration.imported_names().is_empty());
        assert_eq!(
            declaration.source_module_span(),
            &SourceSpan::new(Some("module.ink".to_string()), 1, 6)
        );
    }

    #[test]
    fn rejects_invalid_import_forms() {
        let cases = [
            (
                "from items IMPORT sword",
                "Import declarations must use uppercase `FROM`",
            ),
            (
                "FROM items import sword",
                "Import declarations must use uppercase `IMPORT`",
            ),
            (
                "FROM",
                "FROM declarations must include a source module name",
            ),
            (
                "FROM ",
                "FROM declarations must include a source module name",
            ),
            (
                "FROM item.weapons",
                "Import source module must be a single identifier; hierarchical module names are not supported",
            ),
            (
                "FROM 123",
                "Import source module must be a single identifier",
            ),
            (
                "FROM items IMPORT",
                "FROM ... IMPORT declarations must name at least one symbol",
            ),
            (
                "FROM items IMPORT ",
                "FROM ... IMPORT declarations must name at least one symbol",
            ),
            (
                "FROM items IMPORT sword AS blade",
                "Import aliases are not supported in the first module-support phase",
            ),
            (
                "FROM audio IMPORT function play",
                "IMPORT names do not include kind annotations",
            ),
            (
                "FROM items IMPORT sword,",
                "IMPORT declarations must include an imported symbol name after ','",
            ),
            (
                "FROM items IMPORT items::sword",
                "IMPORT names must be unqualified identifiers",
            ),
            (
                "FROM items IMPORT items",
                "Module literals must be imported with `FROM items`, not `FROM items IMPORT items`",
            ),
            (
                "IMPORT sword FROM items",
                "Old import syntax `IMPORT sword FROM items` has been replaced by `FROM items IMPORT sword`",
            ),
            (
                "IMPORT items FROM items",
                "Module literals must be imported with `FROM items`; `IMPORT items FROM items` is obsolete",
            ),
        ];

        for (source, expected_message) in cases {
            let (declaration, diagnostics) = parse_import(source);

            assert!(declaration.is_none(), "{source}");
            assert_eq!(diagnostics.len(), 1, "{source}: {diagnostics:#?}");
            assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
            assert_eq!(diagnostics[0].message, expected_message);
        }
    }
}
