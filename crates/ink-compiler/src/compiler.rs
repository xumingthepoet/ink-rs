use std::sync::Arc;

use ink_runtime::story::Story as RuntimeStory;

pub use crate::results::CompilerOptions;

use crate::{
    error::CompilerError,
    parsed,
    parser::InkParser,
    results::{CompileJsonResult, CompileResult, DefaultFileHandler, ParseResult},
    runtime_export::export_story_json,
};

#[derive(Debug)]
pub struct Compiler {
    input_string: String,
    options: CompilerOptions,
    parsed_story: Option<parsed::Story>,
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

    pub fn parsed_story(&self) -> Option<&parsed::Story> {
        self.parsed_story.as_ref()
    }

    pub fn parse(&mut self) -> ParseResult {
        let mut parser = InkParser::new(
            &self.input_string,
            self.options.source_filename.as_deref(),
            self.options.file_handler.clone(),
        );
        let mut parse_result = parser.parse();

        if let Some(parsed_story) = parse_result.parsed_story.as_mut() {
            parsed_story.count_all_visits = self.options.count_all_visits;
            self.parsed_story = Some(parsed_story.clone());
        } else {
            self.parsed_story = None;
        }

        parse_result
    }

    pub fn compile_json(&mut self) -> CompileJsonResult {
        let parse_result = self.parse();
        if parse_result.parsed_story.is_none() {
            return CompileJsonResult {
                json: None,
                diagnostics: parse_result.diagnostics,
            };
        }

        let parsed_story = parse_result
            .parsed_story
            .expect("parsed_story checked to exist above");

        match export_story_json(&parsed_story) {
            Ok(json) => CompileJsonResult {
                json: Some(json),
                diagnostics: parse_result.diagnostics,
            },
            Err(err) => {
                let mut diagnostics = parse_result.diagnostics;
                diagnostics.push(
                    err.into_diagnostic()
                        .with_source_filename(self.options.source_filename.clone()),
                );

                CompileJsonResult {
                    json: None,
                    diagnostics,
                }
            }
        }
    }

    pub fn compile(&mut self) -> CompileResult {
        let compile_json_result = self.compile_json();

        let Some(json) = compile_json_result.json else {
            return CompileResult {
                story: None,
                diagnostics: compile_json_result.diagnostics,
            };
        };

        match RuntimeStory::new(&json) {
            Ok(story) => CompileResult {
                story: Some(story),
                diagnostics: compile_json_result.diagnostics,
            },
            Err(err) => {
                let mut diagnostics = compile_json_result.diagnostics;
                diagnostics.push(CompilerError::Runtime(err.to_string()).into_diagnostic());

                CompileResult {
                    story: None,
                    diagnostics,
                }
            }
        }
    }
}
