use std::sync::Arc;

use crate::{
    analysis::{self, CheckedStory},
    diagnostic::{Diagnostic, DiagnosticSeverity},
    emit,
    lower::{self, ir::RuntimeProgram},
    parsed::Story as ParsedStory,
    source::{
        preprocess::{preprocess_includes, PreprocessOptions},
        FileHandler, SourceInput,
    },
    syntax,
};

#[derive(Clone, Default)]
pub struct CompilerOptions {
    pub source_filename: Option<String>,
    pub count_all_visits: bool,
    pub file_handler: Option<Arc<dyn FileHandler>>,
}

#[derive(Default)]
pub struct Compiler {
    options: CompilerOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageOutput<T> {
    pub artifact: Option<T>,
    pub diagnostics: Vec<Diagnostic>,
}

impl<T> StageOutput<T> {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledStory {
    pub program: RuntimeProgram,
    pub json: String,
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_options(options: CompilerOptions) -> Self {
        Self { options }
    }

    pub fn parse(&self, input: SourceInput) -> StageOutput<ParsedStory> {
        let preprocessed = preprocess_includes(
            input,
            PreprocessOptions {
                source_filename: self.options.source_filename.clone(),
                file_handler: self.options.file_handler.as_deref(),
            },
        );
        let mut diagnostics = preprocessed.diagnostics;
        if diagnostics_have_errors(&diagnostics) {
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        }
        let Some(source) = preprocessed.source else {
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        };

        let parsed = syntax::parse_source(source);
        diagnostics.extend(parsed.diagnostics);
        StageOutput {
            artifact: parsed.artifact,
            diagnostics,
        }
    }

    pub fn analyze(&self, parsed: ParsedStory) -> StageOutput<CheckedStory> {
        analysis::analyze(parsed)
    }

    pub fn lower(&self, checked: &CheckedStory) -> StageOutput<RuntimeProgram> {
        lower::lower(checked, self.options.count_all_visits)
    }

    pub fn emit_json(&self, program: RuntimeProgram) -> StageOutput<String> {
        emit::emit_json(program)
    }

    pub fn compile(&self, input: SourceInput) -> StageOutput<CompiledStory> {
        let mut diagnostics = Vec::new();

        let parsed = self.parse(input);
        diagnostics.extend(parsed.diagnostics);
        if diagnostics_have_errors(&diagnostics) {
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        }
        let Some(parsed) = parsed.artifact else {
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        };

        let checked = self.analyze(parsed);
        diagnostics.extend(checked.diagnostics);
        if diagnostics_have_errors(&diagnostics) {
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        }
        let Some(checked) = checked.artifact else {
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        };

        let lowered = self.lower(&checked);
        diagnostics.extend(lowered.diagnostics);
        if diagnostics_have_errors(&diagnostics) {
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        }
        let Some(program) = lowered.artifact else {
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        };

        let emitted = self.emit_json(program.clone());
        let StageOutput {
            artifact: emitted_json,
            diagnostics: emitted_diagnostics,
        } = emitted;
        diagnostics.extend(emitted_diagnostics);
        let artifact = emitted_json.map(|json| CompiledStory { program, json });

        StageOutput {
            artifact,
            diagnostics,
        }
    }
}

fn diagnostics_have_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        io,
        path::{Path, PathBuf},
        sync::Arc,
    };

    use serde_json::json;

    use super::*;

    struct MemoryFileHandler {
        files: HashMap<PathBuf, String>,
    }

    impl MemoryFileHandler {
        fn new(files: &[(&str, &str)]) -> Self {
            Self {
                files: files
                    .iter()
                    .map(|(path, contents)| (PathBuf::from(path), contents.to_string()))
                    .collect(),
            }
        }
    }

    impl FileHandler for MemoryFileHandler {
        fn resolve_ink_filename(&self, include_name: &str) -> PathBuf {
            PathBuf::from(include_name)
        }

        fn load_ink_file_contents(&self, full_filename: &Path) -> io::Result<String> {
            self.files
                .get(full_filename)
                .cloned()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "include file not found"))
        }
    }

    #[test]
    fn compiles_plain_text_json() {
        let compiler = Compiler::default();
        let output = compiler.compile(SourceInput::new("Line."));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);

        let json: serde_json::Value =
            serde_json::from_str(&output.artifact.unwrap().json).expect("valid json");
        assert_eq!(
            json,
            json!({
                "inkVersion": 21,
                "root": [["^Line.", "\n", ["done", {"#n": "g-0"}], null], "done", null],
                "listDefs": {}
            })
        );
    }

    #[test]
    fn unsupported_syntax_blocks_compile() {
        let compiler = Compiler::default();
        let output = compiler.compile(SourceInput::new("LIST list = a"));
        assert!(output.artifact.is_none());
        assert_eq!(output.diagnostics.len(), 1);
    }

    #[test]
    fn diagnostics_from_included_content_use_include_filename() {
        let compiler = Compiler::with_options(CompilerOptions {
            file_handler: Some(Arc::new(MemoryFileHandler::new(&[(
                "inc.ink",
                "LIST list = ()",
            )]))),
            ..Default::default()
        });

        let output = compiler.parse(SourceInput::named("INCLUDE inc.ink", "main.ink"));

        assert_eq!(output.diagnostics.len(), 1);
        let diagnostic = &output.diagnostics[0];
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.source_filename.as_deref(), Some("inc.ink"));
        assert_eq!(diagnostic.line, 1);
        assert_eq!(diagnostic.column, 1);
        assert_eq!(diagnostic.message, "unsupported syntax: list declaration");
    }
}
