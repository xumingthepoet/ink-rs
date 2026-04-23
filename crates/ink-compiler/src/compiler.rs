use std::sync::Arc;

pub use crate::results::CompilerOptions;

use crate::{
    parser::InkParser,
    results::{CompileJsonResult, DefaultFileHandler, ParseResult},
};

#[derive(Debug)]
pub struct Compiler {
    input_string: String,
    options: CompilerOptions,
    parsed_story: Option<crate::parsed::Story>,
}

impl Compiler {
    pub fn new(ink_source: impl Into<String>, options: Option<CompilerOptions>) -> Self {
        let mut options = options.unwrap_or_default();
        if options.file_handler.is_none() {
            options.file_handler = Some(Arc::new(DefaultFileHandler));
        }

        Self {
            input_string: ink_source.into(),
            options,
            parsed_story: None,
        }
    }

    pub fn parsed_story(&self) -> Option<&crate::parsed::Story> {
        self.parsed_story.as_ref()
    }

    pub fn parse(&mut self) -> ParseResult {
        let source_filename = self.options.source_filename.as_deref();
        let file_handler = self.options.file_handler.clone();
        let mut parser = InkParser::new(&self.input_string, source_filename, file_handler);
        let parse_result = parser.parse();
        self.parsed_story = parse_result.parsed_story.clone();
        parse_result
    }

    pub fn compile_json(&mut self) -> CompileJsonResult {
        let parse_result = self.parse();
        let Some(parsed_story) = self.parsed_story.as_ref() else {
            return CompileJsonResult {
                json: None,
                diagnostics: parse_result.diagnostics,
            };
        };

        let mut diagnostics = parse_result.diagnostics;
        match crate::runtime_export::export_story_json(parsed_story) {
            Ok(json) => CompileJsonResult {
                json: Some(json),
                diagnostics,
            },
            Err(error) => {
                diagnostics.push(error.into_diagnostic());
                CompileJsonResult {
                    json: None,
                    diagnostics,
                }
            }
        }
    }
}
