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
    RemovedFeature,
    UnsupportedSyntax,
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

    pub fn unsupported(span: SourceSpan, feature: impl Into<String>) -> Self {
        Self::error(span, format!("unsupported syntax: {}", feature.into()))
            .with_code(DiagnosticCode::UnsupportedSyntax)
    }

    pub fn removed_feature(
        span: SourceSpan,
        feature: impl Into<String>,
        guidance: impl Into<String>,
    ) -> Self {
        Self::error(
            span,
            format!("removed feature: {}. {}", feature.into(), guidance.into()),
        )
        .with_code(DiagnosticCode::RemovedFeature)
    }
}

#[cfg(test)]
mod tests {
    use crate::source::SourceSpan;

    use super::*;

    #[test]
    fn unsupported_syntax_diagnostic_has_stable_code() {
        let diagnostic = Diagnostic::unsupported(SourceSpan::new(None, 1, 1), "include");

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.code, Some(DiagnosticCode::UnsupportedSyntax));
        assert_eq!(diagnostic.message, "unsupported syntax: include");
    }

    #[test]
    fn removed_feature_diagnostic_has_stable_code() {
        let diagnostic = Diagnostic::removed_feature(
            SourceSpan::new(None, 1, 1),
            "LIST declarations",
            "Use variables, functions, or host data instead.",
        );

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.code, Some(DiagnosticCode::RemovedFeature));
        assert_eq!(
            diagnostic.message,
            "removed feature: LIST declarations. Use variables, functions, or host data instead."
        );
    }
}
