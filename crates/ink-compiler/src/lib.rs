//! Compiler-layer port for ink.
//!
//! This crate intentionally depends on `bladeink` for the runtime layer and
//! focuses on porting the official C# compiler architecture.

pub mod compiler;
pub mod error;
pub mod parsed;
pub mod parser;
pub mod results;
mod runtime_export;

pub use compiler::Compiler;
pub use error::{CompilerError, Diagnostic, DiagnosticSeverity, Result};
pub use results::{
    CompileJsonResult, CompileResult, CompilerOptions, DefaultFileHandler, FileHandler, ParseResult,
};
