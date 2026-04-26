use crate::{
    diagnostic::Diagnostic,
    parsed::{ImportDeclaration, ImportedName},
};

use super::{is_identifier, is_identifier_continue, rule::RuleParser};

pub(super) fn is_import_like_declaration_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    let keyword = trimmed
        .chars()
        .take_while(|ch| is_identifier_continue(*ch))
        .collect::<String>();
    keyword.eq_ignore_ascii_case("IMPORT")
}

pub(super) fn parse_import_declaration(parser: &mut RuleParser<'_>) -> Option<ImportDeclaration> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    let keyword_span = parser.current_span();
    let keyword = parser.take_while(is_identifier_continue)?;
    if !keyword.eq_ignore_ascii_case("IMPORT") {
        return None;
    }

    if keyword != "IMPORT" {
        parser.diagnostic(Diagnostic::error(
            keyword_span,
            "Import declarations must use uppercase `IMPORT`",
        ));
        parser.skip_to_end();
        return None;
    }

    parser.expect(
        "whitespace after 'IMPORT'",
        parse_horizontal_whitespace,
        |parser| parser.skip_to_end(),
    )?;
    parser.skip_horizontal_whitespace();

    let imported_names = parse_imported_names(parser)?;
    parser.skip_horizontal_whitespace();

    let from_span = parser.current_span();
    let Some(from_keyword) = parser.take_while(is_identifier_continue) else {
        parser.diagnostic(Diagnostic::error(
            parser.current_span(),
            "IMPORT declarations must include FROM moduleName",
        ));
        parser.skip_to_end();
        return None;
    };
    if !from_keyword.eq_ignore_ascii_case("FROM") {
        parser.diagnostic(Diagnostic::error(
            from_span,
            format!("Expected FROM in IMPORT declaration but saw '{from_keyword}'"),
        ));
        parser.skip_to_end();
        return None;
    }
    if from_keyword != "FROM" {
        parser.diagnostic(Diagnostic::error(
            from_span,
            "Import declarations must use uppercase `FROM`",
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
            "IMPORT declarations must include a source module name after FROM",
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
    if !parser.line_remainder().is_empty() {
        parser.error(format!(
            "Expected end of line after IMPORT declaration but saw '{}'",
            parser.line_remainder()
        ));
        parser.skip_to_end();
        return None;
    }

    Some(ImportDeclaration::new(
        imported_names,
        source_module,
        source_module_span,
        span,
    ))
}

fn parse_imported_names(parser: &mut RuleParser<'_>) -> Option<Vec<ImportedName>> {
    if parser
        .line_remainder()
        .strip_prefix("FROM")
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
    {
        parser.diagnostic(Diagnostic::error(
            parser.current_span(),
            "IMPORT declarations must name at least one symbol before FROM",
        ));
        parser.skip_to_end();
        return None;
    }

    let mut imported_names = Vec::new();
    loop {
        parser.skip_horizontal_whitespace();
        let name_span = parser.current_span();
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
        if remainder
            .strip_prefix("FROM")
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            break;
        }

        if remainder
            .strip_prefix("from")
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
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

        if remainder.is_empty() {
            parser.diagnostic(Diagnostic::error(
                parser.current_span(),
                "IMPORT declarations must include FROM moduleName",
            ));
        } else {
            parser.diagnostic(Diagnostic::error(
                parser.current_span(),
                format!("Expected ',' or FROM after imported name but saw '{remainder}'"),
            ));
        }
        parser.skip_to_end();
        return None;
    }

    Some(imported_names)
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
    fn parses_import_declaration() {
        let (declaration, diagnostics) = parse_import("IMPORT sword, heal FROM items");

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        let declaration = declaration.expect("expected import declaration");
        assert_eq!(declaration.source_module(), "items");
        assert_eq!(declaration.imported_names().len(), 2);
        assert_eq!(declaration.imported_names()[0].name(), "sword");
        assert_eq!(
            declaration.imported_names()[0].span(),
            &SourceSpan::new(Some("module.ink".to_string()), 1, 8)
        );
        assert_eq!(declaration.imported_names()[1].name(), "heal");
        assert_eq!(
            declaration.source_module_span(),
            &SourceSpan::new(Some("module.ink".to_string()), 1, 25)
        );
    }

    #[test]
    fn rejects_invalid_import_forms() {
        let cases = [
            (
                "import sword FROM items",
                "Import declarations must use uppercase `IMPORT`",
            ),
            (
                "IMPORT sword from items",
                "Import declarations must use uppercase `FROM`",
            ),
            (
                "IMPORT FROM items",
                "IMPORT declarations must name at least one symbol before FROM",
            ),
            (
                "IMPORT sword",
                "IMPORT declarations must include FROM moduleName",
            ),
            (
                "IMPORT sword AS blade FROM items",
                "Import aliases are not supported in the first module-support phase",
            ),
            (
                "IMPORT function play FROM audio",
                "IMPORT names do not include kind annotations",
            ),
            (
                "IMPORT sword,",
                "IMPORT declarations must include an imported symbol name after ','",
            ),
            (
                "IMPORT items::sword FROM items",
                "IMPORT names must be unqualified identifiers",
            ),
            (
                "IMPORT sword FROM item.weapons",
                "Import source module must be a single identifier; hierarchical module names are not supported",
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
