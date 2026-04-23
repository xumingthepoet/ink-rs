use std::{
    io,
    path::{Path, PathBuf},
};

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
    pub fn from_input(input: SourceInput) -> Self {
        let source_name = input.filename;
        let mut lines = Vec::new();

        for (index, raw_line) in input.text.lines().enumerate() {
            let mut text = raw_line.to_string();
            if index == 0 {
                text = text.trim_start_matches('\u{feff}').to_string();
            }

            let text = strip_line_comment(&text).trim_end().to_string();
            lines.push(SourceLine {
                text,
                span: SourceSpan::new(source_name.clone(), index + 1, 1),
            });
        }

        Self { lines }
    }
}

fn strip_line_comment(line: &str) -> &str {
    let mut previous = '\0';
    for (index, ch) in line.char_indices() {
        if previous == '/' && ch == '/' {
            let comment_start = index - previous.len_utf8();
            return &line[..comment_start];
        }
        previous = ch;
    }

    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_line_comments() {
        let file = SourceFile::from_input(SourceInput::new("Line. // comment\nOther."));
        assert_eq!(file.lines[0].text, "Line.");
        assert_eq!(file.lines[1].text, "Other.");
    }
}
