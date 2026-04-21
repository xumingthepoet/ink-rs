use bladeink::story::Story as RuntimeStory;

use crate::{
    error::{CompilerError, Result},
    parsed,
    parser::InkParser,
};

#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    pub source_filename: Option<String>,
    pub count_all_visits: bool,
}

#[derive(Debug)]
pub struct Compiler {
    input_string: String,
    options: CompilerOptions,
    parsed_story: Option<parsed::Story>,
}

impl Compiler {
    pub fn new(ink_source: impl Into<String>, options: Option<CompilerOptions>) -> Self {
        Self {
            input_string: ink_source.into(),
            options: options.unwrap_or_default(),
            parsed_story: None,
        }
    }

    pub fn parsed_story(&self) -> Option<&parsed::Story> {
        self.parsed_story.as_ref()
    }

    pub fn parse(&mut self) -> Result<&parsed::Story> {
        let mut parser =
            InkParser::new(&self.input_string, self.options.source_filename.as_deref());
        let mut parsed_story = parser.parse()?;
        parsed_story.count_all_visits = self.options.count_all_visits;
        self.parsed_story = Some(parsed_story);

        Ok(self
            .parsed_story
            .as_ref()
            .expect("parsed_story was just populated"))
    }

    pub fn compile_json(&mut self) -> Result<String> {
        let _ = self.parse()?;

        Err(CompilerError::Unsupported(
            "runtime export has not been ported from ink-csharp/compiler yet",
        ))
    }

    pub fn compile(&mut self) -> Result<RuntimeStory> {
        let json = self.compile_json()?;
        RuntimeStory::new(&json).map_err(|err| CompilerError::Runtime(err.to_string()))
    }
}
