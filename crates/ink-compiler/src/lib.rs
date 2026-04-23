//! Incremental Ink compiler rewrite.
//!
//! The compiler is intentionally independent from `ink-runtime`: it parses Ink
//! source into compiler-owned structures and emits the official runtime JSON
//! wire format directly.

mod analysis;
mod ast;
mod compiler;
mod diagnostic;
mod emit;
mod lower;
mod source;
mod syntax;

pub use ast::{AstNode, ParsedStory};
pub use compiler::{CompiledStory, Compiler, CompilerOptions, StageOutput};
pub use diagnostic::{Diagnostic, DiagnosticSeverity};
pub use source::{FileHandler, SourceInput, SourceSpan};
