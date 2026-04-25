use std::path::{Path, PathBuf};

use crate::diagnostic::{Diagnostic, DiagnosticSeverity};

use super::{eliminate_comments, FileHandler, SourceInput, SourceSpan};

pub(crate) struct PreprocessOptions<'a> {
    pub(crate) source_filename: Option<String>,
    pub(crate) file_handler: Option<&'a dyn FileHandler>,
}

pub(crate) struct PreprocessOutput {
    pub(crate) input: Option<SourceInput>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

pub(crate) fn preprocess_includes(
    input: SourceInput,
    options: PreprocessOptions<'_>,
) -> PreprocessOutput {
    let mut diagnostics = Vec::new();
    let mut open_files = Vec::new();
    let source_name = input.filename.clone().or(options.source_filename);
    let expanded = expand_include_source(
        &input.text,
        source_name.clone(),
        options.file_handler,
        &mut open_files,
        &mut diagnostics,
    );

    PreprocessOutput {
        input: (!diagnostics_have_errors(&diagnostics)).then(|| SourceInput {
            text: expanded.into_text(),
            filename: source_name,
        }),
        diagnostics,
    }
}

fn expand_include_source(
    source: &str,
    source_name: Option<String>,
    file_handler: Option<&dyn FileHandler>,
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
            let Some(handler) = file_handler else {
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
                Ok(included_source) => Some(expand_include_source(
                    &included_source,
                    Some(include_name.to_string()),
                    file_handler,
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
