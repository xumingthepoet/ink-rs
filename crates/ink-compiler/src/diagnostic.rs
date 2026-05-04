use crate::source::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Author,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    InvalidChoiceSyntax,
    InvalidExpression,
    InvalidInlineSyntax,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: Option<DiagnosticCode>,
    pub message: String,
    pub source_filename: Option<String>,
    pub line: usize,
    pub column: usize,
}

/// Formats diagnostics as `file:line:column: severity: message` lines.
pub fn format_diagnostics(diagnostics: &[Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(format_diagnostic)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Formats one diagnostic as `file:line:column: severity: message`.
pub fn format_diagnostic(diagnostic: &Diagnostic) -> String {
    let source_filename = diagnostic.source_filename.as_deref().unwrap_or("<unknown>");

    format!(
        "{source_filename}:{}:{}: {}: {}",
        diagnostic.line,
        diagnostic.column,
        diagnostic.severity.as_str(),
        diagnostic.message
    )
}

impl DiagnosticSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Author => "author",
        }
    }
}

impl Diagnostic {
    pub fn error(span: SourceSpan, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code: None,
            message: message.into(),
            source_filename: span.source_name,
            line: span.line,
            column: span.column,
        }
    }

    pub fn warning(span: SourceSpan, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            code: None,
            message: message.into(),
            source_filename: span.source_name,
            line: span.line,
            column: span.column,
        }
    }

    pub fn author(span: SourceSpan, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Author,
            code: None,
            message: message.into(),
            source_filename: span.source_name,
            line: span.line,
            column: span.column,
        }
    }

    pub fn with_code(mut self, code: DiagnosticCode) -> Self {
        self.code = Some(code);
        self
    }
}
