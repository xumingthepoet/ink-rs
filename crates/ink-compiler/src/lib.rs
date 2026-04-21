//! Compiler-layer port for ink.
//!
//! This crate intentionally depends on `bladeink` for the runtime layer and
//! focuses on porting the official C# compiler architecture.

pub mod compiler;
pub mod error;
pub mod parsed;
pub mod parser;

pub use compiler::{Compiler, CompilerOptions};
pub use error::{CompilerError, Result};
