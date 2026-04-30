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

pub(super) fn parse_import_declaration_lines(
    lines: &[SourceLine],
    index: usize,
) -> (Option<ImportDeclaration>, Vec<Diagnostic>, usize) {
    let line = &lines[index];
    if is_multiline_import_declaration_start(line) {
        parse_multiline_import_declaration(lines, index)
    } else {
        let mut line_parser = RuleParser::new(line);
        let declaration = line_parser.parse_rule(parse_import_declaration);
        let had_error = line_parser.had_error();
        let diagnostics = line_parser.finish();

        (declaration.filter(|_| !had_error), diagnostics, index + 1)
    }
}

fn is_multiline_import_declaration_start(line: &SourceLine) -> bool {
    let trimmed = line.text.trim_start();
    let keyword = trimmed
        .chars()
        .take_while(|ch| is_identifier_continue(*ch))
        .collect::<String>();
    if !keyword.eq_ignore_ascii_case("IMPORT") {
        return false;
    }
    let rest = &trimmed[keyword.len()..];
    if !rest.starts_with([' ', '\t']) {
        return false;
    }
    rest.trim_start_matches([' ', '\t']).starts_with('{')
}

fn parse_multiline_import_declaration(
    lines: &[SourceLine],
    start_index: usize,
) -> (Option<ImportDeclaration>, Vec<Diagnostic>, usize) {
    let start_line = &lines[start_index];
    let mut diagnostics = Vec::new();
    let mut had_error = false;

    let Some(opening) = parse_multiline_import_opening(start_line, &mut diagnostics) else {
        return (None, diagnostics, start_index + 1);
    };
    if !opening.trailing.trim().is_empty() {
        diagnostics.push(Diagnostic::error(
            span_at(start_line, opening.trailing_byte_index),
            format!(
                "Expected end of line after multiline IMPORT opening but saw '{}'",
                opening.trailing.trim()
            ),
        ));
        return (None, diagnostics, start_index + 1);
    }

    let mut imported_names = Vec::new();
    let mut index = start_index + 1;
    while index < lines.len() {
        let line = &lines[index];
        if line.text.trim_start().starts_with('}') {
            if let Some(source_module) = parse_multiline_import_closing(line, &mut diagnostics) {
                if imported_names.is_empty() {
                    diagnostics.push(Diagnostic::error(
                        opening.open_brace_span,
                        "IMPORT declarations must name at least one symbol before FROM",
                    ));
                    had_error = true;
                }
                let declaration = (!had_error).then(|| {
                    ImportDeclaration::new(
                        imported_names,
                        source_module.name,
                        source_module.span,
                        opening.import_span,
                    )
                });
                return (declaration, diagnostics, index + 1);
            }
            return (None, diagnostics, index + 1);
        }

        if line.text.trim().is_empty() {
            index += 1;
            continue;
        }

        if !parse_multiline_import_names_line(line, &mut imported_names, &mut diagnostics) {
            had_error = true;
        }
        index += 1;
    }

    diagnostics.push(Diagnostic::error(
        opening.import_span,
        "Multiline IMPORT declarations must close with `} FROM moduleName`",
    ));
    (None, diagnostics, index)
}

struct MultilineImportOpening<'a> {
    import_span: SourceSpan,
    open_brace_span: SourceSpan,
    trailing: &'a str,
    trailing_byte_index: usize,
}

fn parse_multiline_import_opening<'a>(
    line: &'a SourceLine,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<MultilineImportOpening<'a>> {
    let trimmed = line.text.trim_start();
    let leading_bytes = line.text.len() - trimmed.len();
    let keyword = trimmed
        .chars()
        .take_while(|ch| is_identifier_continue(*ch))
        .collect::<String>();
    let import_span = span_at(line, leading_bytes);

    if keyword != "IMPORT" {
        diagnostics.push(Diagnostic::error(
            import_span,
            "Import declarations must use uppercase `IMPORT`",
        ));
        return None;
    }

    let rest_byte_index = leading_bytes + keyword.len();
    let rest = &line.text[rest_byte_index..];
    if !rest.starts_with([' ', '\t']) {
        diagnostics.push(Diagnostic::error(
            span_at(line, rest_byte_index),
            "Expected whitespace after 'IMPORT'",
        ));
        return None;
    }

    let after_whitespace = rest.trim_start_matches([' ', '\t']);
    let open_brace_byte_index = line.text.len() - after_whitespace.len();
    let after_open = after_whitespace.strip_prefix('{')?;
    let trailing_byte_index = open_brace_byte_index + 1;

    Some(MultilineImportOpening {
        import_span,
        open_brace_span: span_at(line, open_brace_byte_index),
        trailing: after_open,
        trailing_byte_index,
    })
}

struct SourceModuleName {
    name: String,
    span: SourceSpan,
}

fn parse_multiline_import_closing(
    line: &SourceLine,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SourceModuleName> {
    let trimmed = line.text.trim_start();
    let leading_bytes = line.text.len() - trimmed.len();
    let after_close = trimmed.strip_prefix('}')?;
    let after_close_byte_index = leading_bytes + 1;
    let after_whitespace = after_close.trim_start_matches([' ', '\t']);
    let from_byte_index = line.text.len() - after_whitespace.len();

    let from_keyword = after_whitespace
        .chars()
        .take_while(|ch| is_identifier_continue(*ch))
        .collect::<String>();
    if from_keyword.is_empty() {
        diagnostics.push(Diagnostic::error(
            span_at(line, after_close_byte_index),
            "IMPORT declarations must include FROM moduleName",
        ));
        return None;
    }

    let from_span = span_at(line, from_byte_index);
    if !from_keyword.eq_ignore_ascii_case("FROM") {
        diagnostics.push(Diagnostic::error(
            from_span,
            format!("Expected FROM in IMPORT declaration but saw '{from_keyword}'"),
        ));
        return None;
    }
    if from_keyword != "FROM" {
        diagnostics.push(Diagnostic::error(
            from_span,
            "Import declarations must use uppercase `FROM`",
        ));
        return None;
    }

    let after_from_byte_index = from_byte_index + from_keyword.len();
    let after_from = &line.text[after_from_byte_index..];
    if !after_from.starts_with([' ', '\t']) {
        diagnostics.push(Diagnostic::error(
            span_at(line, after_from_byte_index),
            "Expected whitespace after 'FROM'",
        ));
        return None;
    }

    let source_module_text = after_from.trim_start_matches([' ', '\t']);
    let source_module_byte_index = line.text.len() - source_module_text.len();
    let Some(source_module) = source_module_text.split_whitespace().next() else {
        diagnostics.push(Diagnostic::error(
            span_at(line, source_module_byte_index),
            "IMPORT declarations must include a source module name after FROM",
        ));
        return None;
    };
    let source_module_span = span_at(line, source_module_byte_index);
    let trailing = &source_module_text[source_module.len()..];
    if !trailing.trim().is_empty() {
        diagnostics.push(Diagnostic::error(
            span_at(line, source_module_byte_index + source_module.len()),
            format!(
                "Expected end of line after IMPORT declaration but saw '{}'",
                trailing.trim()
            ),
        ));
        return None;
    }

    if source_module.contains('.') {
        diagnostics.push(Diagnostic::error(
            source_module_span,
            "Import source module must be a single identifier; hierarchical module names are not supported",
        ));
        return None;
    }

    if !is_identifier(source_module) {
        diagnostics.push(Diagnostic::error(
            source_module_span,
            "Import source module must be a single identifier",
        ));
        return None;
    }

    Some(SourceModuleName {
        name: source_module.to_string(),
        span: source_module_span,
    })
}

fn parse_multiline_import_names_line(
    line: &SourceLine,
    imported_names: &mut Vec<ImportedName>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    let mut had_error = false;
    let mut cursor = 0;
    let mut expect_name = true;

    while cursor < line.text.len() {
        let remainder = &line.text[cursor..];
        let after_whitespace = remainder.trim_start_matches([' ', '\t']);
        cursor = line.text.len() - after_whitespace.len();
        if cursor >= line.text.len() {
            break;
        }

        if line.text[cursor..].starts_with(',') {
            if expect_name {
                diagnostics.push(Diagnostic::error(
                    span_at(line, cursor),
                    "IMPORT declarations must include an imported symbol name before ','",
                ));
                had_error = true;
            }
            cursor += 1;
            expect_name = true;
            continue;
        }

        let name_start = cursor;
        let name = line.text[cursor..]
            .chars()
            .take_while(|ch| !ch.is_whitespace() && *ch != ',')
            .collect::<String>();
        cursor += name.len();
        let name_span = span_at(line, name_start);

        if name.contains("::") || name.contains(':') {
            diagnostics.push(Diagnostic::error(
                name_span,
                "IMPORT names must be unqualified identifiers",
            ));
            return false;
        }

        if !is_identifier(&name) {
            diagnostics.push(Diagnostic::error(
                name_span,
                "IMPORT names must be single identifiers",
            ));
            return false;
        }

        let after_name = &line.text[cursor..];
        let after_name_trimmed = after_name.trim_start_matches([' ', '\t']);
        if after_name_trimmed
            .strip_prefix("AS")
            .or_else(|| after_name_trimmed.strip_prefix("as"))
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            diagnostics.push(Diagnostic::error(
                span_at(line, line.text.len() - after_name_trimmed.len()),
                "Import aliases are not supported in the first module-support phase",
            ));
            return false;
        }

        let next_token = after_name_trimmed
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string();
        if is_import_kind_keyword(&name) && is_identifier(&next_token) {
            diagnostics.push(Diagnostic::error(
                name_span,
                "IMPORT names do not include kind annotations",
            ));
            return false;
        }

        imported_names.push(ImportedName::new(name, name_span));
        expect_name = false;

        let after_whitespace = after_name.trim_start_matches([' ', '\t']);
        cursor = line.text.len() - after_whitespace.len();
        if cursor >= line.text.len() {
            break;
        }
        if line.text[cursor..].starts_with(',') {
            continue;
        }

        diagnostics.push(Diagnostic::error(
            span_at(line, cursor),
            format!(
                "Expected ',' or end of line after imported name but saw '{}'",
                line.text[cursor..].trim()
            ),
        ));
        had_error = true;
        break;
    }

    !had_error
}

fn span_at(line: &SourceLine, byte_index: usize) -> SourceSpan {
    SourceSpan::new(
        line.span.source_name.clone(),
        line.span.line,
        line.span.column + line.text[..byte_index].chars().count(),
    )
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
