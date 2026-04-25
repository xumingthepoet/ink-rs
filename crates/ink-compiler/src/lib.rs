//! Incremental Ink compiler rewrite.
//!
//! The compiler is intentionally independent from `ink-runtime`: it parses Ink
//! source into compiler-owned structures and emits the official runtime JSON
//! wire format directly.

mod analysis;
mod compiler;
mod diagnostic;
mod emit;
mod lower;
mod parsed;
mod source;
mod syntax;

pub use compiler::{CompiledStory, Compiler, CompilerOptions, StageOutput};
pub use diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity};
pub use parsed::{
    Choice, ContentList, Divert, DivertTarget, Object, Story as ParsedStory, Text, Weave,
};
pub use source::{eliminate_comments, FileHandler, SourceInput, SourceSpan};
