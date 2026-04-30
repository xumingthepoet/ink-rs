use crate::diagnostic::Diagnostic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceInput {
    pub text: String,
    pub filename: Option<String>,
}

impl SourceInput {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            filename: None,
        }
    }

    pub fn named(text: impl Into<String>, filename: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            filename: Some(filename.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub source_name: Option<String>,
    pub line: usize,
    pub column: usize,
}

impl SourceSpan {
    pub fn new(source_name: Option<String>, line: usize, column: usize) -> Self {
        Self {
            source_name,
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceLine {
    pub text: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceFile {
    pub lines: Vec<SourceLine>,
}

pub(crate) struct SourceLoadOutput {
    pub(crate) source: Option<SourceFile>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

pub(crate) fn prepare_source_input(
    input: SourceInput,
    fallback_source_filename: Option<String>,
) -> SourceLoadOutput {
    let source_name = input.filename.clone().or(fallback_source_filename);
    let comment_eliminated = eliminate_comments(&input.text);
    let lines = comment_eliminated
        .lines()
        .enumerate()
        .map(|(index, raw_line)| SourceLine {
            text: raw_line.to_string(),
            span: SourceSpan::new(source_name.clone(), index + 1, 1),
        })
        .collect();
    let source = SourceFile::from_lines(lines);
    let mut diagnostics = diagnose_removed_includes(&source);
    diagnostics.extend(diagnose_removed_list_declarations(&source));
    let source = diagnostics.is_empty().then_some(source);

    SourceLoadOutput {
        source,
        diagnostics,
    }
}

impl SourceFile {
    #[cfg(test)]
    pub fn from_input(input: SourceInput) -> Self {
        prepare_source_input(input, None)
            .source
            .expect("test source should not contain removed source syntax")
    }

    pub(crate) fn from_lines(lines: Vec<SourceLine>) -> Self {
        let lines = lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| SourceLine {
                text: normalize_line_text(line.text, index),
                span: line.span,
            })
            .collect();

        Self { lines }
    }
}

fn diagnose_removed_includes(source: &SourceFile) -> Vec<Diagnostic> {
    source
        .lines
        .iter()
        .filter_map(|line| {
            removed_include_column(&line.text).map(|column| {
                Diagnostic::error(
                    SourceSpan::new(line.span.source_name.clone(), line.span.line, column),
                    "INCLUDE is no longer supported; use modules and IMPORT instead",
                )
            })
        })
        .collect()
}

fn diagnose_removed_list_declarations(source: &SourceFile) -> Vec<Diagnostic> {
    source
        .lines
        .iter()
        .filter_map(|line| {
            removed_list_declaration_column(&line.text).map(|column| {
                Diagnostic::error(
                    SourceSpan::new(line.span.source_name.clone(), line.span.line, column),
                    "LIST declarations are no longer supported; use variables, functions, or host-side data instead",
                )
            })
        })
        .collect()
}

fn removed_list_declaration_column(text: &str) -> Option<usize> {
    let leading_whitespace = text
        .chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .count();
    let trimmed = text.trim_start_matches([' ', '\t']);
    let rest = trimmed.strip_prefix("LIST")?;
    if rest.starts_with(char::is_whitespace) && rest.contains('=') {
        Some(leading_whitespace + 1)
    } else {
        None
    }
}

fn removed_include_column(text: &str) -> Option<usize> {
    let leading_whitespace = text
        .chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .count();
    let trimmed = text.trim_start_matches([' ', '\t']);
    let rest = trimmed.strip_prefix("INCLUDE")?;
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Some(leading_whitespace + 1)
    } else {
        None
    }
}

fn normalize_line_text(mut text: String, index: usize) -> String {
    if index == 0 {
        text = text.trim_start_matches('\u{feff}').to_string();
    }

    if text.trim_start().starts_with('*') {
        text
    } else {
        text.trim_end().to_string()
    }
}

pub fn eliminate_comments(input: &str) -> String {
    let mut output = String::new();
    let mut index = 0;
    let mut in_block_comment = false;

    while index < input.len() {
        let rest = &input[index..];
        let ch = rest.chars().next().expect("index is inside input");

        if in_block_comment {
            if rest.starts_with("*/") {
                index += "*/".len();
                in_block_comment = false;
                continue;
            }
            if rest.starts_with("\r\n") {
                output.push('\n');
                index += "\r\n".len();
                continue;
            }
            if ch == '\n' {
                output.push('\n');
            }
            index += ch.len_utf8();
            continue;
        }

        if rest.starts_with("//") {
            index += "//".len();
            while index < input.len() {
                let rest = &input[index..];
                if rest.starts_with("\r\n") || rest.starts_with('\n') {
                    break;
                }
                let ch = rest.chars().next().expect("index is inside input");
                index += ch.len_utf8();
            }
            continue;
        }

        if rest.starts_with("/*") {
            in_block_comment = true;
            index += "/*".len();
            continue;
        }

        if rest.starts_with("\r\n") {
            output.push('\n');
            index += "\r\n".len();
            continue;
        }

        output.push(ch);
        index += ch.len_utf8();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_line_comments() {
        let file = SourceFile::from_input(SourceInput::new("Line. // comment\nOther."));
        assert_eq!(file.lines[0].text, "Line.");
        assert_eq!(file.lines[1].text, "Other.");
        assert_eq!(
            eliminate_comments("Line. // comment\nOther."),
            "Line. \nOther."
        );
    }

    #[test]
    fn strips_block_comments() {
        let file = SourceFile::from_input(SourceInput::new("A /* comment\nstill comment */ B"));
        assert_eq!(file.lines[0].text, "A");
        assert_eq!(file.lines[1].text, " B");
        assert_eq!(
            eliminate_comments("A /* comment\nstill comment */ B"),
            "A \n B"
        );
    }
}
