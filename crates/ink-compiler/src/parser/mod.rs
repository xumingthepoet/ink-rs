pub mod character_range;
pub mod character_set;
mod comment_eliminator;
pub mod string_parser;
mod whitespace;

pub use comment_eliminator::CommentEliminator;
pub use string_parser::{Element, ParseSuccessStruct, StringParser, StringParserState};
pub use whitespace::{
    any_whitespace, end_of_file, end_of_line, multi_spaced, multiline_whitespace, newline, spaced,
    whitespace,
};

use std::sync::Arc;

use crate::{
    error::{Diagnostic, DiagnosticSeverity},
    parsed::{ContentList, Story as ParsedStory, Text},
    results::{FileHandler, ParseResult},
};

#[derive(Debug)]
pub struct InkParser<'source> {
    input_string: String,
    source_filename: Option<&'source str>,
    file_handler: Option<Arc<dyn FileHandler>>,
}

impl<'source> InkParser<'source> {
    pub fn new(
        input_string: &'source str,
        source_filename: Option<&'source str>,
        file_handler: Option<Arc<dyn FileHandler>>,
    ) -> Self {
        let input_string = CommentEliminator::process(input_string).unwrap_or_default();

        Self {
            input_string,
            source_filename,
            file_handler,
        }
    }

    pub fn input_string(&self) -> &str {
        &self.input_string
    }

    pub fn source_filename(&self) -> Option<&'source str> {
        self.source_filename
    }

    pub fn file_handler(&self) -> Option<&dyn FileHandler> {
        self.file_handler.as_deref()
    }

    pub fn parse(&mut self) -> ParseResult {
        match self.parse_plain_text_story() {
            Ok(parsed_story) => ParseResult::success(parsed_story),
            Err(diagnostic) => ParseResult::failure(diagnostic),
        }
    }

    fn parse_plain_text_story(&self) -> std::result::Result<ParsedStory, Diagnostic> {
        if let Some((line, column, marker)) = self.find_unsupported_syntax() {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                self.source_filename.map(str::to_string),
                line,
                column,
                format!(
                    "InkParser currently supports plain text lines only; found unsupported syntax starting with {marker:?}"
                ),
            ));
        }

        let mut top_level_content = Vec::new();

        if self.input_string.is_empty() {
            return Ok(ParsedStory::new(top_level_content, false));
        }

        for segment in self.input_string.split_inclusive('\n') {
            let had_newline = segment.ends_with('\n');
            let line_text = segment.strip_suffix('\n').unwrap_or(segment);

            let line = ContentList::new();
            if !line_text.is_empty() {
                line.add_content(Text::new(line_text).object());
            }
            line.trim_trailing_whitespace();

            if had_newline {
                line.add_content(Text::new("\n").object());
            }

            top_level_content.push(line.object());
        }

        Ok(ParsedStory::new(top_level_content, false))
    }

    fn find_unsupported_syntax(&self) -> Option<(usize, usize, &'static str)> {
        for (line_index, line) in self.input_string.lines().enumerate() {
            let trimmed = line.trim_start();
            let column = line.len().saturating_sub(trimmed.len()) + 1;

            for marker in ["===", "->", "~", "*", "-", "#", "{", "}"] {
                if trimmed.starts_with(marker) {
                    return Some((line_index + 1, column, marker));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::{CommentEliminator, InkParser};
    use crate::parsed::ObjectKind;

    #[test]
    fn ink_parser_preprocesses_comments_and_newlines() {
        let parser = InkParser::new("line1 // comment\r\nline2/*x\n y*/line3", None, None);

        assert_eq!(parser.input_string(), "line1 \nline2\nline3");
        assert_eq!(
            CommentEliminator::process("line1 // comment\r\nline2/*x\n y*/line3"),
            Some("line1 \nline2\nline3".to_string())
        );
    }

    #[test]
    fn ink_parser_parses_plain_text_into_content_nodes() {
        let mut parser = InkParser::new("Hello world\nSecond line", Some("story.ink"), None);
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 2);

        let first_line = content[0].borrow();
        assert!(matches!(first_line.kind(), ObjectKind::ContentList { .. }));
        assert_eq!(first_line.content().len(), 2);
        assert!(matches!(
            first_line.content()[0].borrow().kind(),
            ObjectKind::Text { text } if text == "Hello world"
        ));
        assert!(matches!(
            first_line.content()[1].borrow().kind(),
            ObjectKind::Text { text } if text == "\n"
        ));

        let second_line = content[1].borrow();
        assert!(matches!(second_line.kind(), ObjectKind::ContentList { .. }));
        assert_eq!(second_line.content().len(), 1);
        assert!(matches!(
            second_line.content()[0].borrow().kind(),
            ObjectKind::Text { text } if text == "Second line"
        ));
    }

    #[test]
    fn ink_parser_reports_unsupported_structural_syntax() {
        let mut parser = InkParser::new("-> knot", Some("story.ink"), None);
        let result = parser.parse();

        assert!(result.parsed_story.is_none());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].severity,
            crate::error::DiagnosticSeverity::Error
        );
        assert_eq!(
            result.diagnostics[0].source_filename.as_deref(),
            Some("story.ink")
        );
        assert_eq!(result.diagnostics[0].line, 1);
        assert_eq!(result.diagnostics[0].column, 1);
    }
}
