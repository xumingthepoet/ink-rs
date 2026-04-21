use std::{error, fmt};

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
