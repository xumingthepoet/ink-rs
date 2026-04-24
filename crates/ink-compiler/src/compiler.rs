use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    analysis::{self, CheckedStory},
    diagnostic::{Diagnostic, DiagnosticSeverity},
    emit,
    lower::{self, RuntimeProgram},
    parsed::Story as ParsedStory,
    source::{eliminate_comments, FileHandler, SourceInput, SourceSpan},
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
        let expanded = self.expand_includes(input);
        let mut diagnostics = expanded.diagnostics;
        if diagnostics_have_errors(&diagnostics) {
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        }
        let Some(input) = expanded.artifact else {
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

    fn expand_includes(&self, input: SourceInput) -> StageOutput<SourceInput> {
        let mut diagnostics = Vec::new();
        let mut open_files = Vec::new();
        let source_name = input
            .filename
            .clone()
            .or_else(|| self.options.source_filename.clone());
        let expanded = self.expand_include_source(
            &input.text,
            source_name.clone(),
            &mut open_files,
            &mut diagnostics,
        );

        StageOutput {
            artifact: (!diagnostics_have_errors(&diagnostics)).then(|| SourceInput {
                text: expanded.into_text(),
                filename: source_name,
            }),
            diagnostics,
        }
    }

    fn expand_include_source(
        &self,
        source: &str,
        source_name: Option<String>,
        open_files: &mut Vec<PathBuf>,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> ExpandedInclude {
        let mut root_lines = Vec::new();
        let mut flow_lines = Vec::new();
        let mut in_flow = false;
        let comment_eliminated = eliminate_comments(source);

        for (index, line) in comment_eliminated.lines().enumerate() {
            let line_number = index + 1;
            if let Some(include_name) = parse_include_name(line) {
                let Some(handler) = &self.options.file_handler else {
                    diagnostics.push(Diagnostic::error(
                        SourceSpan::new(source_name.clone(), line_number, 1),
                        "Failed to load include: no file handler configured",
                    ));
                    continue;
                };

                let resolved = handler.resolve_ink_filename(include_name);
                let include_key = include_path_key(&resolved);
                if open_files.contains(&include_key) {
                    diagnostics.push(Diagnostic::error(
                        SourceSpan::new(source_name.clone(), line_number, 1),
                        format!("Recursive INCLUDE detected: '{}'", include_key.display()),
                    ));
                    continue;
                }

                open_files.push(include_key);
                let included = match handler.load_ink_file_contents(&resolved) {
                    Ok(included_source) => Some(self.expand_include_source(
                        &included_source,
                        Some(include_name.to_string()),
                        open_files,
                        diagnostics,
                    )),
                    Err(_) => {
                        diagnostics.push(Diagnostic::error(
                            SourceSpan::new(source_name.clone(), line_number, 1),
                            format!("Failed to load: '{include_name}'"),
                        ));
                        None
                    }
                };
                open_files.pop();

                if let Some(included) = included {
                    if in_flow {
                        flow_lines.extend(included.root_lines);
                    } else {
                        root_lines.extend(included.root_lines);
                    }
                    flow_lines.extend(included.flow_lines);
                }
                continue;
            }

            if is_flow_declaration_line(line) {
                in_flow = true;
            }

            if in_flow {
                flow_lines.push(line.to_string());
            } else {
                root_lines.push(line.to_string());
            }
        }

        ExpandedInclude {
            root_lines,
            flow_lines,
        }
    }
}

#[derive(Debug, Default)]
struct ExpandedInclude {
    root_lines: Vec<String>,
    flow_lines: Vec<String>,
}

impl ExpandedInclude {
    fn into_text(self) -> String {
        self.root_lines
            .into_iter()
            .chain(self.flow_lines)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn parse_include_name(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("INCLUDE")?;
    if !rest.starts_with(|ch: char| ch.is_whitespace()) {
        return None;
    }
    let include_name = rest.trim();
    (!include_name.is_empty()).then_some(include_name)
}

fn is_flow_declaration_line(line: &str) -> bool {
    line.trim_start().starts_with('=')
}

fn include_path_key(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
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
