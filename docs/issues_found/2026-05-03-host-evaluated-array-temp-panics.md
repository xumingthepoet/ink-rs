# Host-Evaluated Function Composite Temp Assignment Panics
Status: found
Found while: implementing INTERNAL host-callable functions
Scope: crates/ink-runtime/src/story_state.rs, crates/ink-runtime/src/story/control_logic.rs, function evaluation from host
Problem: Calling an ink function from host with `evaluate_function`/`call_internal` can panic when the function assigns an array or struct literal to a typed temp before returning it. The runtime reaches a `VariableAssignment` with an empty evaluation stack and unwraps `None`.
Why it matters: Host-callable configuration helpers should be able to build composite values, and bad stack state should return `StoryError` instead of unwinding through the host application.
Suggested fix: Reproduce with a focused runtime fixture, inspect array literal lowering inside host function evaluation, and replace unchecked evaluation-stack pops with `StoryError` where malformed stack state can occur.
Evidence: Temporary INTERNAL fixture code `~ temp scores: int[] = [value, value + 1]` followed by `~ return scores`, and later `~ temp player: Player = { hp: hp }` followed by `~ return player`, panicked at `crates/ink-runtime/src/story_state.rs:752` through `crates/ink-runtime/src/story/control_logic.rs:528` during `cargo test -p ink-test internal`.
