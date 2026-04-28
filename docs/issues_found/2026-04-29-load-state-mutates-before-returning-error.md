# Failed load_state Can Partially Mutate Story State
Status: found
Found while: deep library-readiness and legacy-residue review
Scope: crates/ink-runtime/src/story_state.rs, crates/ink-runtime/src/flow.rs, crates/ink-runtime/src/variables_state.rs
Problem: `StoryState::load_json_obj` assigns `current_flow` and loads variables before validating later required fields such as `storySeed` and `previousRandom`. If a later check fails, `Story::load_state` returns an error after partially applying the bad save.
Why it matters: Host games generally rely on a failed load leaving the current story untouched. Partial mutation can corrupt the in-memory story after an `Err`, making recovery paths unreliable.
Suggested fix: Parse and validate the full save into temporary flow, variable, seed, and random values first. Only commit them to `self` after all required fields and nested structures have been validated.
Evidence: `crates/ink-runtime/src/story_state.rs:1053` assigns `self.current_flow`, `crates/ink-runtime/src/story_state.rs:1063` mutates `variables_state`, but `previousRandom` is not validated until `crates/ink-runtime/src/story_state.rs:1084`. A temporary runner removed `previousRandom` from a save where `game::x` was `7`; `load_state` returned `Err(BadJson("Missing previous random value"))`, but `get_variable("game::x")` then returned `int:7`.
