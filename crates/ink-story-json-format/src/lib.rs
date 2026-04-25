//! Typed memory model and JSON codec for the compiled Ink story format.
//!
//! This crate deliberately models the compiled story wire format, not compiler
//! lowering state and not runtime execution objects.

mod error;
mod json;
mod model;

pub use error::FormatError;
pub use model::{
    native_function_name_from_token, native_function_token, Container, ControlCommand,
    ListItemValue, ListValue, NamedContainer, Object, Program,
};

/// The current compiled story JSON format version.
pub const INK_VERSION_CURRENT: i32 = 1;
