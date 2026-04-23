use std::sync::Arc;

pub use crate::results::CompilerOptions;

use crate::{
    error::{CompilerError, Diagnostic},
    results::{
        CompileJsonResult, CompileResult, DefaultFileHandler, ParseResult,
    },
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
        let _ = &self.input_string;
        let _ = &self.options;
        self.parsed_story = None;
        ParseResult::failure(CompilerError::Unsupported(
            "ink-compiler implementation is temporarily removed",
        )
        .into_diagnostic())
    }

    pub fn compile_json(&mut self) -> CompileJsonResult {
        let _ = self.parse();
        CompileJsonResult::failure(Diagnostic::error(
            "ink-compiler implementation is temporarily removed",
        ))
    }

    pub fn compile(&mut self) -> CompileResult {
        let _ = self.compile_json();
        CompileResult::failure(Diagnostic::error(
            "ink-compiler implementation is temporarily removed",
        ))
    }
}
