//! Compiler for ink-rs, a domain-specific language for narrative games.
//!
//! The compiler is intentionally independent from `ink-runtime`: it parses
//! ink-rs source into compiler-owned structures and emits compiled story JSON
//! directly.

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
pub use compiler::{
    CompileResult, CompiledStory, Compiler, CompilerOptions, DiagnosticsPolicy, StageOutput,
};
pub use diagnostic::{
    format_diagnostic, format_diagnostics, Diagnostic, DiagnosticCode, DiagnosticSeverity,
};
pub use ink_story_json_format::{
    Container as RuntimeContainer, ControlCommand as RuntimeControlCommand,
    NamedContainer as RuntimeNamedContainer, Object as RuntimeObject, Program as RuntimeProgram,
};
pub use parsed::{
    Choice, ContentList, DefaultValue, Divert, DivertTarget, ImplementedInterface,
    ImportDeclaration, ImportedName, InterfaceDeclaration, InterfaceMemberKind,
    InterfaceMemberSignature, Module, Object, PrimitiveType, Story as ParsedStory, Text, TypeName,
    Weave,
};
pub use source::{eliminate_comments, SourceInput, SourceSpan};
