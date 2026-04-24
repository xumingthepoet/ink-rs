use crate::source::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source_filename: Option<String>,
    pub line: usize,
    pub column: usize,
}

impl Diagnostic {
    pub fn error(span: SourceSpan, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            source_filename: span.source_name,
            line: span.line,
            column: span.column,
        }
    }

    pub fn warning(span: SourceSpan, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            source_filename: span.source_name,
            line: span.line,
            column: span.column,
        }
    }

    pub fn unsupported(span: SourceSpan, feature: impl Into<String>) -> Self {
        Self::error(span, format!("unsupported syntax: {}", feature.into()))
    }
}
