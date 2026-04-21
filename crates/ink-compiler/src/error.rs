use std::{error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub source_filename: Option<String>,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl Diagnostic {
    pub fn new(
        severity: DiagnosticSeverity,
        source_filename: Option<String>,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            source_filename,
            line,
            column,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Error, None, 0, 0, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Warning, None, 0, 0, message)
    }

    pub fn with_source_filename(mut self, source_filename: Option<String>) -> Self {
        self.source_filename = source_filename;
        self
    }
}

pub type Result<T> = std::result::Result<T, CompilerError>;

#[derive(Debug)]
pub enum CompilerError {
    Unsupported(&'static str),
    Parse {
        message: String,
        source_filename: Option<String>,
        line: usize,
        column: usize,
    },
    Runtime(String),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(message) => f.write_str(message),
            Self::Parse {
                message,
                source_filename,
                line,
                column,
            } => {
                if let Some(source_filename) = source_filename {
                    write!(f, "{source_filename}:{line}:{column}: {message}")
                } else {
                    write!(f, "{line}:{column}: {message}")
                }
            }
            Self::Runtime(message) => f.write_str(message),
        }
    }
}

impl error::Error for CompilerError {}

impl From<CompilerError> for Diagnostic {
    fn from(value: CompilerError) -> Self {
        match value {
            CompilerError::Unsupported(message) => {
                Diagnostic::new(DiagnosticSeverity::Error, None, 0, 0, message)
            }
            CompilerError::Parse {
                message,
                source_filename,
                line,
                column,
            } => Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line,
                column,
                message,
            ),
            CompilerError::Runtime(message) => {
                Diagnostic::new(DiagnosticSeverity::Error, None, 0, 0, message)
            }
        }
    }
}

impl CompilerError {
    pub fn into_diagnostic(self) -> Diagnostic {
        self.into()
    }
}
