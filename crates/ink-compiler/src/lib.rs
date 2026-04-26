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

pub use analysis::{
    CheckedStory, ModuleDependencyGraph, ModuleEntryPoint, ModuleImportIndex, ModuleReachability,
};
pub use compiler::{CompiledStory, Compiler, CompilerOptions, StageOutput};
pub use diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity};
pub use ink_story_json_format::{
    Container as RuntimeContainer, ControlCommand as RuntimeControlCommand,
    NamedContainer as RuntimeNamedContainer, Object as RuntimeObject, Program as RuntimeProgram,
};
pub use parsed::{
    Choice, ContentList, DefaultValue, Divert, DivertTarget, ImportDeclaration, ImportedName,
    Module, Object, PrimitiveType, Story as ParsedStory, Text, TypeName, Weave,
};
pub use source::{eliminate_comments, SourceInput, SourceSpan};
