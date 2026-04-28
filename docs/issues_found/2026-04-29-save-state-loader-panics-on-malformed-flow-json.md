# Save-State Loader Panics On Malformed Flow JSON
Status: found
Found while: deep library-readiness and legacy-residue review
Scope: crates/ink-runtime/src/flow.rs, crates/ink-runtime/src/callstack.rs, crates/ink-runtime/src/story_state.rs
Problem: `Story::load_state` returns `Result<(), StoryError>`, but malformed save-state fields can panic inside the flow and callstack loaders because several JSON shape checks use `unwrap()` after only checking field presence.
Why it matters: Save files are external input for games. Corrupt, user-edited, or stale save JSON should return `StoryError::BadJson`, not unwind through the host application.
Suggested fix: Replace save-load `unwrap()` calls with `ok_or_else` and type-specific `BadJson` messages. Add negative tests for wrong-type `callstack`, `currentChoices`, `threads`, `threadCounter`, `choiceThreads`, and malformed saved choice threads.
Evidence: `crates/ink-runtime/src/flow.rs:45`, `crates/ink-runtime/src/flow.rs:59`, and `crates/ink-runtime/src/flow.rs:131` use `unwrap()` on save JSON shapes. `crates/ink-runtime/src/callstack.rs:424` through `432` unwrap required fields and types. A temporary runner loaded `{"inkSaveVersion":2,"callstack":[],"currentChoices":[],"variablesState":{},"storySeed":0,"previousRandom":0}` and printed `malformed-save-panicked=true` after a panic at `crates/ink-runtime/src/flow.rs:60`.
