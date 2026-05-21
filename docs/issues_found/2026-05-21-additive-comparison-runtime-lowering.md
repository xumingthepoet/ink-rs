# Additive Comparison Lowers To Invalid Runtime Expression
Status: found
Found while: adding `text-snake-10x10` under `crates/ink-experiments`
Scope: compiler expression lowering or runtime execution for comparison expressions containing additive subexpressions
Problem: The expression `x >= width + 2` compiles without diagnostics inside an ink function, but the generated story fails at runtime with `+ expected value parameters` when the condition is evaluated.
Why it matters: Game/grid code naturally compares coordinates against computed bounds such as `width + 2`, `x + dx`, or `limit - 1`. If these expressions compile but fail at runtime, experiment authors can accidentally ship broken stories instead of receiving a diagnostic or correct lowering.
Suggested fix: Add a focused compiler/runtime regression for comparison expressions whose operands contain additive expressions, inspect the parsed expression tree and lowered runtime instructions, and ensure `x >= (width + 2)` leaves both `+` operands on the evaluation stack before the comparison.
Evidence: `crates/ink-experiments/experiments/text-snake-10x10/story.ink:110` uses `{ if x >= width + 2: ... }`. Running `cargo test -p ink-experiments` fails in `playthrough_snapshots_match_when_present` with `RUNTIME ERROR: (game.border_row.10.b.10): + expected value parameters`.
