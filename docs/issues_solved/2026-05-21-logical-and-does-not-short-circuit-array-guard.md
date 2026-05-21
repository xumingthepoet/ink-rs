# Logical And Does Not Short-Circuit Array Guard
Status: solved
Found while: adding `text-2048-4x4` under `crates/ink-experiments`
Scope: compiler lowering or runtime evaluation for logical `&&` / `and`
Problem: A guarded array access using `index + 1 < LEN(scratch) && scratch[index + 1] == value` compiles, but the right-hand array access is still evaluated when `index + 1` equals the array length, causing a runtime out-of-bounds error.
Why it matters: Game logic commonly uses short-circuit guards before indexing arrays, checking Dict keys, or dereferencing state tables. If `&&` eagerly evaluates both operands, normal guard expressions can crash at runtime even though the source reads as safe and `SyntaxReference.md` describes `and` / `&&` as usual logical operators.
Suggested fix: Add focused tests for `false && out_of_bounds_read` and `true || out_of_bounds_read`, then update expression lowering/runtime evaluation so logical `and` and `or` short-circuit their right-hand side. If eager evaluation is intended instead, document that explicitly and add diagnostics or guidance for guard expressions.
Evidence: `crates/ink-experiments/experiments/text-2048-4x4/story.ink:151` uses `{ if index + 1 < LEN(scratch) && scratch[index + 1] == value: ... }`. Running `cargo test -p ink-experiments` fails in `playthrough_snapshots_match_when_present` with `RUNTIME ERROR: (game.merge_scratch.8.b.18): Array index out of bounds: 4`.
Fix: Explicit logical operators now lower to short-circuit control flow inside a single expression. Adjacent choice condition blocks remain independent and eager.
Validation: `cargo test -p ink-test --test integration expressions`; `cargo test -p ink-test --test integration choices`; `cargo test -p ink-experiments playthrough_snapshots_match_when_present -- --nocapture`; `make gate`.
