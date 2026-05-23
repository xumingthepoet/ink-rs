//! Reusable Dioxus-facing application layer for ink-rs games.
//!
//! Game crates provide ink-rs source files as [`InkSource`] values. This crate
//! compiles those sources, runs the story, interprets shared UI tags, and
//! exposes a Dioxus web shell behind the `web` feature.

pub mod app;
#[cfg(not(target_arch = "wasm32"))]
pub mod build;
pub mod runtime;
pub mod styled_text;
pub mod tags;
pub mod transcript;
#[cfg(feature = "web")]
pub mod web;

pub use app::{
    InkApp, InkAppInteraction, InkAppOptions, InkChoiceOption, InkChoicePrompt, InkToast,
};
pub use runtime::{
    InkError, InkGameSource, InkRuntime, InkSource, RuntimeChoice, RuntimePause, RuntimeStep,
    TextItem,
};
pub use transcript::TranscriptBuffer;
