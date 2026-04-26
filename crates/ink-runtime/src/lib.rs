//! This is a Rust implementation of ink-rs, a language fork of inkle's
//! [Ink](https://github.com/inkle/ink) for writing interactive narratives.
//! It intentionally diverges from upstream Ink in a few language and save-state
//! areas; see the repository documentation for the maintained language surface.
//!
//! To learn more about the Ink language, you can check [the official documentation](https://github.com/inkle/ink/blob/master/Documentation/WritingWithInk.md).
//!
//! Here is a quick example that uses basic features to play an Ink story using
//! the `ink_runtime` crate.
//!
//! ```
//! # use ink_runtime::{story::Story, story_error::StoryError};
//! # fn main() -> Result<(), StoryError> {
//! # let json_string = r##"{"inkVersion":1, "root":["done",null]}"##;
//! # let read_input = |_:&_| 0;
//! // story is the entry point of the `ink_runtime` lib.
//! // json_string is a string with all the contents of the .ink.json file.
//! let mut story = Story::new(json_string)?;
//!
//! loop {
//!     while story.can_continue() {
//!         let line = story.cont()?;
//!
//!         println!("{}", line);
//!     }
//!
//!     let choices = story.get_current_choices();
//!     if !choices.is_empty() {
//!         // read_input is a method that you should implement
//!         // to get the choice selected by the user.
//!         let choice_idx:usize = read_input(&choices);
//!         // set the option selected by the user
//!         story.choose_choice_index(choice_idx)?;
//!     } else {
//!         break;
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! The `ink_runtime` library supports the maintained ink-rs runtime features,
//! including threads, variable set/get from code, variable observing, external
//! functions, tags on choices, and v2 minimal save/load state. Upstream
//! multi-flow APIs and visit/turn-count state are intentionally not part of the
//! current runtime API.

mod callstack;
pub mod choice;
mod choice_point;
mod container;
mod control_command;
mod divert;
mod flow;
mod glue;
mod json;
mod native_function_call;
mod object;
mod path;
mod pointer;
mod push_pop;
mod search_result;
mod state_patch;
pub mod story;
pub mod story_error;
mod story_state;
mod tag;
mod value;
pub mod value_type;
mod variable_assigment;
mod variable_reference;
mod variables_state;
mod void;
