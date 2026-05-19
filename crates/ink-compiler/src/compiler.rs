use ink_story_json_format::Program as RuntimeProgram;

use crate::{
    analysis::{self, CheckedStory},
    diagnostic::{Diagnostic, DiagnosticSeverity},
    emit, lower,
    parsed::Story as ParsedStory,
    source::{prepare_source_input, SourceFile, SourceInput, SourceSpan},
    syntax,
};

/// Options that control source attribution and full compile failure policy.
#[derive(Clone, Default)]
pub struct CompilerOptions {
    pub source_filename: Option<String>,
    /// Controls whether warning diagnostics make full compile results fail.
    pub diagnostics_policy: DiagnosticsPolicy,
}

#[derive(Default)]
pub struct Compiler {
    options: CompilerOptions,
}

/// Controls how `CompileResult::failed` treats non-error diagnostics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DiagnosticsPolicy {
    /// Only error diagnostics make a compile result fail.
    #[default]
    AllowWarnings,
    /// Error, warning, and author diagnostics make a compile result fail.
    DenyWarnings,
}

/// Output from an individual public compiler pipeline stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageOutput<T> {
    pub artifact: Option<T>,
    pub diagnostics: Vec<Diagnostic>,
}

impl<T> StageOutput<T> {
    pub fn has_errors(&self) -> bool {
        diagnostics_have_errors(&self.diagnostics)
    }

    pub fn has_warnings(&self) -> bool {
        diagnostics_have_warnings(&self.diagnostics)
    }
}

/// Output from `Compiler::compile` and `Compiler::compile_sources`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileResult<T> {
    pub artifact: Option<T>,
    pub diagnostics: Vec<Diagnostic>,
    diagnostics_policy: DiagnosticsPolicy,
}

impl<T> CompileResult<T> {
    fn new(
        artifact: Option<T>,
        diagnostics: Vec<Diagnostic>,
        diagnostics_policy: DiagnosticsPolicy,
    ) -> Self {
        Self {
            artifact,
            diagnostics,
            diagnostics_policy,
        }
    }

    pub fn has_errors(&self) -> bool {
        diagnostics_have_errors(&self.diagnostics)
    }

    pub fn has_warnings(&self) -> bool {
        diagnostics_have_warnings(&self.diagnostics)
    }

    pub fn failed(&self) -> bool {
        diagnostics_failed(&self.diagnostics, self.diagnostics_policy)
    }
}

impl DiagnosticsPolicy {
    fn denies_warnings(self) -> bool {
        matches!(self, DiagnosticsPolicy::DenyWarnings)
    }
}

#[derive(Debug, Clone, PartialEq)]
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
        let source_load = prepare_source_input(input, self.options.source_filename.clone());
        let mut diagnostics = source_load.diagnostics;
        if diagnostics_have_errors(&diagnostics) {
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        }
        let Some(source) = source_load.source else {
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        };
        if let Some(diagnostic) = explicit_module_diagnostic(&source) {
            diagnostics.push(diagnostic);
            return StageOutput {
                artifact: None,
                diagnostics,
            };
        }

        let parsed = syntax::parse_source(source);
        diagnostics.extend(parsed.diagnostics);
        StageOutput {
            artifact: parsed.artifact,
            diagnostics,
        }
    }

    fn parse_source_inputs(
        &self,
        inputs: Vec<SourceInput>,
        api_name: &str,
    ) -> StageOutput<ParsedStory> {
        if inputs.is_empty() {
            return StageOutput {
                artifact: None,
                diagnostics: vec![Diagnostic::error(
                    SourceSpan::new(None, 1, 1),
                    format!("{api_name} requires at least one source input"),
                )],
            };
        }

        let mut diagnostics = Vec::new();
        let mut stories = Vec::new();

        for input in inputs {
            let parsed = self.parse(input);
            diagnostics.extend(parsed.diagnostics);
            if let Some(story) = parsed.artifact {
                stories.push(story);
            }
        }

        let artifact = if diagnostics_have_errors(&diagnostics) || stories.is_empty() {
            None
        } else {
            Some(merge_parsed_stories(stories))
        };

        StageOutput {
            artifact,
            diagnostics,
        }
    }

    pub fn analyze(&self, parsed: ParsedStory) -> StageOutput<CheckedStory> {
        if parsed.modules().is_empty() {
            return StageOutput {
                artifact: None,
                diagnostics: vec![Diagnostic::error(
                    SourceSpan::new(None, 1, 1),
                    "Cannot analyze a story without explicit modules",
                )],
            };
        }

        analysis::analyze(parsed)
    }

    pub fn lower(&self, checked: &CheckedStory) -> StageOutput<RuntimeProgram> {
        lower::lower(checked)
    }

    pub fn emit_json(&self, program: RuntimeProgram) -> StageOutput<String> {
        emit::emit_json(program)
    }

    pub fn compile(&self, input: SourceInput) -> CompileResult<CompiledStory> {
        self.compile_sources(vec![input])
    }

    pub fn parse_sources(&self, inputs: Vec<SourceInput>) -> StageOutput<ParsedStory> {
        self.parse_source_inputs(inputs, "Compiler::parse_sources")
    }

    pub fn compile_sources(&self, inputs: Vec<SourceInput>) -> CompileResult<CompiledStory> {
        let mut diagnostics = Vec::new();

        let parsed = self.parse_source_inputs(inputs, "Compiler::compile_sources");
        diagnostics.extend(parsed.diagnostics);
        if self.compile_failed(&diagnostics) {
            return self.compile_result(None, diagnostics);
        }
        let Some(parsed) = parsed.artifact else {
            return self.compile_result(None, diagnostics);
        };

        let checked = self.analyze(parsed);
        diagnostics.extend(checked.diagnostics);
        if self.compile_failed(&diagnostics) {
            return self.compile_result(None, diagnostics);
        }
        let Some(checked) = checked.artifact else {
            return self.compile_result(None, diagnostics);
        };

        let lowered = self.lower(&checked);
        diagnostics.extend(lowered.diagnostics);
        if self.compile_failed(&diagnostics) {
            return self.compile_result(None, diagnostics);
        }
        let Some(program) = lowered.artifact else {
            return self.compile_result(None, diagnostics);
        };

        let emitted = self.emit_json(program.clone());
        let StageOutput {
            artifact: emitted_json,
            diagnostics: emitted_diagnostics,
        } = emitted;
        diagnostics.extend(emitted_diagnostics);
        let artifact = if self.compile_failed(&diagnostics) {
            None
        } else {
            emitted_json.map(|json| CompiledStory { program, json })
        };

        self.compile_result(artifact, diagnostics)
    }

    fn compile_failed(&self, diagnostics: &[Diagnostic]) -> bool {
        diagnostics_failed(diagnostics, self.options.diagnostics_policy)
    }

    fn compile_result<T>(
        &self,
        artifact: Option<T>,
        diagnostics: Vec<Diagnostic>,
    ) -> CompileResult<T> {
        CompileResult::new(artifact, diagnostics, self.options.diagnostics_policy)
    }
}

fn merge_parsed_stories(stories: Vec<ParsedStory>) -> ParsedStory {
    let mut root_content = Vec::new();
    let mut flows = Vec::new();
    let mut interfaces = Vec::new();
    let mut modules = Vec::new();

    for story in stories {
        root_content.extend(story.root_weave().content().iter().cloned());
        flows.extend(story.flows().iter().cloned());
        interfaces.extend(story.interfaces().iter().cloned());
        modules.extend(story.modules().iter().cloned());
    }

    ParsedStory::new_with_modules_and_interfaces(root_content, flows, modules, interfaces)
}

fn diagnostics_have_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
}

fn diagnostics_have_warnings(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.severity,
            DiagnosticSeverity::Warning | DiagnosticSeverity::Author
        )
    })
}

fn diagnostics_failed(diagnostics: &[Diagnostic], policy: DiagnosticsPolicy) -> bool {
    diagnostics_have_errors(diagnostics)
        || (policy.denies_warnings() && diagnostics_have_warnings(diagnostics))
}

fn explicit_module_diagnostic(source: &SourceFile) -> Option<Diagnostic> {
    const MESSAGE: &str = "Source files must start with an explicit module or interface declaration (`=== module name ===` or `=== interface name ===`)";

    let Some(first_content) = source
        .lines
        .iter()
        .find(|line| !line.text.trim().is_empty())
    else {
        return Some(Diagnostic::error(SourceSpan::new(None, 1, 1), MESSAGE));
    };

    (!syntax::is_module_like_declaration_line(&first_content.text)
        && !syntax::is_interface_like_declaration_line(&first_content.text))
    .then(|| Diagnostic::error(first_content.span.clone(), MESSAGE))
}

#[cfg(test)]
mod tests {
    use crate::parsed::{Object, Text};

    use super::*;

    #[test]
    fn rejects_plain_text_without_explicit_module() {
        let compiler = Compiler::default();
        let output = compiler.compile(SourceInput::new("Line."));

        assert!(output.artifact.is_none());
        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].message,
            "Source files must start with an explicit module or interface declaration (`=== module name ===` or `=== interface name ===`)"
        );
    }

    #[test]
    fn current_syntax_errors_block_compile() {
        let compiler = Compiler::default();
        let output = compiler.compile(SourceInput::new(
            "=== module game ===\nVAR score = 1\n== main ==\n-> DONE",
        ));
        assert!(output.artifact.is_none());
        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].message,
            "Variable 'score' is missing a type"
        );
    }

    #[test]
    fn public_analyze_rejects_root_story_artifacts() {
        let story = ParsedStory::new(
            vec![Object::Text(Text::new(
                "Line.",
                SourceSpan::new(None, 1, 1),
            ))],
            Vec::new(),
        );
        let output = Compiler::default().analyze(story);

        assert!(output.artifact.is_none());
        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].message,
            "Cannot analyze a story without explicit modules"
        );
    }

    #[test]
    fn include_statement_is_removed_diagnostic() {
        let compiler = Compiler::default();
        let output = compiler.parse(SourceInput::named(
            "Line.\n  INCLUDE inc.ink\nAfter.",
            "main.ink",
        ));

        assert_eq!(output.diagnostics.len(), 1);
        let diagnostic = &output.diagnostics[0];
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.code, None);
        assert_eq!(diagnostic.source_filename.as_deref(), Some("main.ink"));
        assert_eq!(diagnostic.line, 2);
        assert_eq!(diagnostic.column, 3);
        assert_eq!(
            diagnostic.message,
            "INCLUDE is no longer supported; use modules and IMPORT instead"
        );
        assert!(output.artifact.is_none());
    }

    #[test]
    fn list_declaration_is_removed_diagnostic_before_root_module_fallback() {
        let compiler = Compiler::default();
        let output = compiler.compile(SourceInput::named(
            "  LIST colors = red, blue\n",
            "main.ink",
        ));

        assert_eq!(output.diagnostics.len(), 1);
        let diagnostic = &output.diagnostics[0];
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.code, None);
        assert_eq!(diagnostic.source_filename.as_deref(), Some("main.ink"));
        assert_eq!(diagnostic.line, 1);
        assert_eq!(diagnostic.column, 3);
        assert_eq!(
            diagnostic.message,
            "LIST declarations are no longer supported; use variables, functions, or host-side data instead"
        );
        assert!(output.artifact.is_none());
    }

    #[test]
    fn list_declaration_is_removed_diagnostic_before_module_content_fallback() {
        let compiler = Compiler::default();
        let output = compiler.compile(SourceInput::named(
            "=== module game ===\n  LIST colors = red, blue\n== main ==\n-> END",
            "main.ink",
        ));

        assert_eq!(output.diagnostics.len(), 1);
        let diagnostic = &output.diagnostics[0];
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.line, 2);
        assert_eq!(diagnostic.column, 3);
        assert_eq!(
            diagnostic.message,
            "LIST declarations are no longer supported; use variables, functions, or host-side data instead"
        );
        assert!(output.artifact.is_none());
    }
}
