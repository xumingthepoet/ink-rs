use std::{
    fmt, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use bladeink::story::Story as RuntimeStory;

use crate::{error::Diagnostic, parsed};

pub trait FileHandler: fmt::Debug + Send + Sync {
    fn resolve_ink_filename(&self, include_name: &str) -> PathBuf;

    fn load_ink_file_contents(&self, full_filename: &Path) -> io::Result<String>;
}

#[derive(Debug, Clone, Default)]
pub struct DefaultFileHandler;

impl FileHandler for DefaultFileHandler {
    fn resolve_ink_filename(&self, include_name: &str) -> PathBuf {
        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        working_dir.join(include_name)
    }

    fn load_ink_file_contents(&self, full_filename: &Path) -> io::Result<String> {
        std::fs::read_to_string(full_filename)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ParseResult {
    pub parsed_story: Option<parsed::Story>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ParseResult {
    pub fn success(parsed_story: parsed::Story) -> Self {
        Self {
            parsed_story: Some(parsed_story),
            diagnostics: Vec::new(),
        }
    }

    pub fn failure(diagnostic: Diagnostic) -> Self {
        Self {
            parsed_story: None,
            diagnostics: vec![diagnostic],
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == crate::error::DiagnosticSeverity::Error)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompileJsonResult {
    pub json: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

impl CompileJsonResult {
    pub fn success(json: impl Into<String>) -> Self {
        Self {
            json: Some(json.into()),
            diagnostics: Vec::new(),
        }
    }

    pub fn failure(diagnostic: Diagnostic) -> Self {
        Self {
            json: None,
            diagnostics: vec![diagnostic],
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == crate::error::DiagnosticSeverity::Error)
    }
}

#[derive(Default)]
pub struct CompileResult {
    pub story: Option<RuntimeStory>,
    pub diagnostics: Vec<Diagnostic>,
}

impl CompileResult {
    pub fn success(story: RuntimeStory) -> Self {
        Self {
            story: Some(story),
            diagnostics: Vec::new(),
        }
    }

    pub fn failure(diagnostic: Diagnostic) -> Self {
        Self {
            story: None,
            diagnostics: vec![diagnostic],
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == crate::error::DiagnosticSeverity::Error)
    }
}

impl fmt::Debug for CompileResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompileResult")
            .field("story", &self.story.as_ref().map(|_| "<runtime story>"))
            .field("diagnostics", &self.diagnostics)
            .finish()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    pub source_filename: Option<String>,
    pub count_all_visits: bool,
    pub file_handler: Option<Arc<dyn FileHandler>>,
}
