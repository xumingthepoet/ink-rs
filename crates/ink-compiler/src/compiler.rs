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
        let Some(input) = preprocessed.input else {
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        };

        let parsed = syntax::parse(input);
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
    use serde_json::json;

    use super::*;

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
}
