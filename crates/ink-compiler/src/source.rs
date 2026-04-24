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
        let mut in_block_comment = false;

        for (index, raw_line) in input.text.lines().enumerate() {
            let mut text = raw_line.to_string();
            if index == 0 {
                text = text.trim_start_matches('\u{feff}').to_string();
            }

            let uncommented = strip_comments(&text, &mut in_block_comment);
            let is_choice_line = uncommented
                .trim_start()
                .starts_with(|ch| matches!(ch, '*' | '+'));
            let text = if uncommented.len() == text.len() && is_choice_line {
                uncommented
            } else {
                uncommented.trim_end().to_string()
            };
            lines.push(SourceLine {
                text,
                span: SourceSpan::new(source_name.clone(), index + 1, 1),
            });
        }

        Self { lines }
    }
}

fn strip_comments(line: &str, in_block_comment: &mut bool) -> String {
    let mut output = String::new();
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;

    while index < line.len() {
        let rest = &line[index..];

        if *in_block_comment {
            if let Some(end) = rest.find("*/") {
                index += end + "*/".len();
                *in_block_comment = false;
            } else {
                break;
            }
            continue;
        }

        if !in_string && rest.starts_with("//") {
            break;
        }

        if !in_string && rest.starts_with("/*") {
            *in_block_comment = true;
            index += "/*".len();
            continue;
        }

        let ch = rest.chars().next().expect("index is inside line");
        output.push(ch);
        index += ch.len_utf8();

        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            _ => {}
        }
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
    }

    #[test]
    fn strips_block_comments() {
        let file = SourceFile::from_input(SourceInput::new("A /* comment\nstill comment */ B"));
        assert_eq!(file.lines[0].text, "A");
        assert_eq!(file.lines[1].text, " B");
    }
}
