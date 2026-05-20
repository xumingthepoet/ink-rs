# Experiment State Needs Array Append Helpers
Status: solved
Found while: reviewing `crates/ink-experiments` for workaround-shaped examples after the 100 experiment run
Scope: current ink-rs array builtins and experiment story-state patterns in `crates/ink-experiments/experiments/`
Problem: Several experiments model dynamic state by preallocating fixed-size arrays of empty structs and then filling or clearing slots with sentinel fields. This is not a compiler/runtime correctness bug, but it is an awkward authoring pattern that can hide real data-growth pressure behind fixed capacities.
Why it matters: Game systems commonly need growable logs, queues, result lists, pending event lists, temporary combat plans, and discovery ledgers. Without an append/push helper, authors either precompute exact capacity or add arbitrary spare slots. That makes examples less natural and can mask the point where the language should expose a dynamic collection operation.
Fix: Added typed array helpers `ARRAY_PUSH(array, value) => void` and `ARRAY_INSERT(array, index, value) => void`. Both require a mutable lvalue and preserve array element typing. `ARRAY_INSERT` inserts before the index, allows `index == LEN(array)`, and rejects indexes outside `0..=LEN(array)`.
Evidence: `job-031-enemy-ai-intention-stack` preallocates `intent_stack` with empty `Intention` values and tracks `stack_depth`; `job-042-cooking-meal-buffs` preallocates `buff_slots` and uses `member_index: -1`; `job-075-library-fines-membership` preallocates inactive `borrowed_books` slots; `job-083-inn-room-booking` preallocates empty `reservations`; `job-085-curse-item-identification` preallocates empty `clues`; `job-097-trading-card-deck-synergy` preallocates empty `deck_cards`.
Validation:
- `cargo test -p ink-story-json-format native_function`
- `cargo test -p ink-runtime native_function_call`
- `cargo test -p ink-compiler targets`
- `cargo test -p ink-compiler array_literals`
- `cargo test -p ink-test --test integration typed_values`
- `cargo test -p ink-experiments`
- `make gate`
