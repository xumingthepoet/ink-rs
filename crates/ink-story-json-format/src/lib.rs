//! Typed memory model and JSON codec for the ink-rs compiled story format.
//!
//! This crate deliberately models the compiled story wire format, not compiler
//! lowering state and not runtime execution objects.

mod error;
mod json;
mod model;

pub use error::FormatError;
pub use model::{
    Container, ControlCommand, DictKey, DictKeyType, DictValue, InterfaceDefinition,
    InterfaceMemberKind, InternalFunction, NamedContainer, NativeFunction, Object, Program,
    DICT_KEY_TYPE_INT, DICT_KEY_TYPE_STRING, DICT_VALUE_MARKER, DYNAMIC_INTERFACE_ARGS_KEY,
    DYNAMIC_INTERFACE_FUNCTION_KEY, DYNAMIC_INTERFACE_NAME_KEY, DYNAMIC_INTERFACE_TARGET_KEY,
    INTERFACES_METADATA_KEY, INTERFACE_IMPLEMENTATIONS_KEY, INTERFACE_MEMBERS_KEY, TAG_END_TOKEN,
    TAG_START_TOKEN,
};
