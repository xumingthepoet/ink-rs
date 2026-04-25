//! Typed memory model and JSON codec for the compiled Ink story format.
//!
//! This crate deliberately models the compiled story wire format, not compiler
//! lowering state and not runtime execution objects.

mod error;
mod json;
mod model;

pub use error::FormatError;
pub use model::{
    ChoicePoint, Container, ControlCommand, Divert, DivertKind, ListItemValue, ListValue,
    NamedContainer, NativeFunction, Object, Program, Value, VariableAssignment,
    VariableAssignmentKind, VariablePointer, VariableReference, VariableReferenceKind,
};

/// The current compiled story JSON format version.
pub const INK_VERSION_CURRENT: i32 = 21;

/// The minimum legacy compiled story JSON version accepted by the runtime.
pub const INK_VERSION_MINIMUM_COMPATIBLE: i32 = 18;
