pub mod character_range;
pub mod character_set;
pub mod string_parser;

pub use string_parser::{Element, ParseSuccessStruct, StringParser, StringParserState};

use std::sync::Arc;

use crate::{
    error::CompilerError,
    results::{FileHandler, ParseResult},
};

#[derive(Debug)]
pub struct InkParser<'source> {
    input_string: &'source str,
    source_filename: Option<&'source str>,
    file_handler: Option<Arc<dyn FileHandler>>,
}

impl<'source> InkParser<'source> {
    pub fn new(
        input_string: &'source str,
        source_filename: Option<&'source str>,
        file_handler: Option<Arc<dyn FileHandler>>,
    ) -> Self {
        Self {
            input_string,
            source_filename,
            file_handler,
        }
    }

    pub fn input_string(&self) -> &'source str {
        self.input_string
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
