#![allow(dead_code, unused_imports, unused_variables)]

//! Compiler-layer port for ink.
//!
//! This crate focuses on the compiler-facing JSON export pipeline and the
//! parsed hierarchy used to build it.

pub mod compiler;
pub mod error;
pub mod parsed;
pub mod parser;
mod runtime_export;
pub mod results;

pub use compiler::Compiler;
pub use error::{CompilerError, Diagnostic, DiagnosticSeverity, Result};
pub use results::{
    CompileJsonResult, CompilerOptions, DefaultFileHandler, FileHandler, ParseResult,
};
