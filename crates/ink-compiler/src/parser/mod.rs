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
    error::CompilerError,
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
        ParseResult::failure(
            CompilerError::Unsupported(
                "InkParser has not been ported from ink-csharp/compiler/InkParser yet",
            )
            .into_diagnostic()
            .with_source_filename(self.source_filename.map(str::to_string)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CommentEliminator, InkParser};

    #[test]
    fn ink_parser_preprocesses_comments_and_newlines() {
        let parser = InkParser::new("line1 // comment\r\nline2/*x\n y*/line3", None, None);

        assert_eq!(parser.input_string(), "line1 \nline2\nline3");
        assert_eq!(
            CommentEliminator::process("line1 // comment\r\nline2/*x\n y*/line3"),
            Some("line1 \nline2\nline3".to_string())
        );
    }
}
