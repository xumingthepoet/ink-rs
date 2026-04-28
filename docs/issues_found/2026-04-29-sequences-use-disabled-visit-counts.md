# Source Sequences Use Disabled Visit Counts
Status: found
Found while: deep library-readiness and legacy-residue review
Scope: crates/ink-compiler/src/lower/sequence.rs, crates/ink-runtime/src/story/progress.rs, crates/ink-runtime/src/story_state.rs, crates/ink-test/tests/compiler_snapshots/fixtures.rs
Problem: Compiler-lowered source sequences still depend on the legacy `"visit"` control command and visit-count container flags, but the current runtime no longer records visits. `StoryState::visit_count_for_container` always returns `0`, and `Story::visit_container` is a no-op.
Why it matters: Authored sequences compile successfully but do not output their alternatives at runtime. This is a user-visible story execution bug and a legacy state dependency left behind after visit counts were removed.
Suggested fix: Rework sequence lowering to use maintained runtime state that is not the removed author-facing visit/turn count system, or lower sequences to explicit generated state. Add runtime tests for once, cycle, stopping, and shuffle sequences, including save/load interactions if state is persisted.
Evidence: `crates/ink-compiler/src/lower/sequence.rs:18` emits `ControlCommand::VisitIndex` and `flags: Some(5)`. `crates/ink-runtime/src/story/progress.rs:681` discards visit updates. `crates/ink-runtime/src/story_state.rs:357` returns `0` for every container visit count. A temporary runner using `First {one|two|three}. -> main` printed `sequence-0="First .\n"`, `sequence-1="First .\n"`, and `sequence-2="First .\n"`.
