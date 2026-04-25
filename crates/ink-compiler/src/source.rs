use std::{
    io,
    path::{Path, PathBuf},
};

pub(crate) mod preprocess;

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

pub trait FileHandler: Send + Sync {
    fn resolve_ink_filename(&self, include_name: &str) -> PathBuf;
    fn load_ink_file_contents(&self, full_filename: &Path) -> io::Result<String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl SourceFile {
    #[cfg(test)]
    pub fn from_input(input: SourceInput) -> Self {
        let source_name = input.filename;
        let comment_eliminated = eliminate_comments(&input.text);
        let lines = comment_eliminated
            .lines()
            .enumerate()
            .map(|(index, raw_line)| SourceLine {
                text: raw_line.to_string(),
                span: SourceSpan::new(source_name.clone(), index + 1, 1),
            })
            .collect();

        Self::from_lines(lines)
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

fn normalize_line_text(mut text: String, index: usize) -> String {
    if index == 0 {
        text = text.trim_start_matches('\u{feff}').to_string();
    }

    if text.trim_start().starts_with(|ch| matches!(ch, '*' | '+')) {
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
