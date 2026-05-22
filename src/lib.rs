//! Unified library entry point for game projects using ink-rs, a
//! domain-specific language for narrative games.
//!
//! The lower-level crates remain available for projects that want tighter
//! dependency control. This facade keeps the common game integration path short:
//! use the default `runtime` feature to load compiled story JSON, and enable the
//! `compiler` feature when a tool, editor, build script, or mod pipeline needs
//! to compile ink-rs source.
//!
//! For current language syntax, see the repository
//! [`SyntaxReference.md`](https://github.com/xumingthepoet/ink-rs/blob/main/docs/SyntaxReference.md).

#[cfg(feature = "compiler")]
pub use ink_compiler::{
    format_diagnostic, format_diagnostics, CompileResult, CompiledStory, Compiler, CompilerOptions,
    Diagnostic, DiagnosticSeverity, DiagnosticsPolicy, SourceInput,
};

#[cfg(feature = "runtime")]
pub use ink_runtime::story::Story;

#[cfg(feature = "runtime")]
pub use ink_runtime::story_error::StoryError;

#[cfg(feature = "runtime")]
pub use ink_runtime::value_type::ValueType;

#[cfg(feature = "compiler")]
pub use ink_compiler as compiler;

#[cfg(feature = "runtime")]
pub use ink_runtime as runtime;

#[cfg(feature = "format")]
pub use ink_story_json_format as story_json_format;
