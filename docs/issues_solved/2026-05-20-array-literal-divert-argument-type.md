# Array Literal Divert Arguments Do Not Receive Expected Parameter Type

Status: solved

Found while: adding `ink-experiments` enemy dict battle initialization experiment

Scope: compiler analysis for divert or flow-call arguments, array literal type
checking, `crates/ink-experiments/experiments/enemy-dict-battle-init/story.ink`

Problem: Calling a typed knot with an inline array literal argument fails even
when the knot parameter type is explicit. The experiment calls
`-> start_battle([101, 204, 305])`, and `start_battle` declares
`enemy_ids: int[]`, but compilation reports `Array literal requires an expected
array type`.

Why it matters: Game systems should be able to pass encounter enemy ids,
skill ids, item ids, or other short runtime lists directly into reusable battle
or event flows. Requiring authors to hoist every list literal into a separately
typed `VAR` or `temp` would make common data-driven calls noisy and would hide
the intended shape of the battle setup.

Fix: Propagated declared parameter types into array literal analysis for typed
static divert arguments, typed function-call arguments, and dynamic interface
member arguments. Target-call diagnostics now let expected-type array literals
be validated by the array literal pass instead of treating them as primitive
type inference failures.

Validation: `cargo test -p ink-compiler array_literal`,
`cargo test -p ink-test --test typed_values`, `git diff --check`, and
`make gate`.

Original evidence: `cargo test -p ink-experiments` failed while compiling
`crates/ink-experiments/experiments/enemy-dict-battle-init/story.ink` with:
`<unknown>:1:1: error: Array literal requires an expected array type`.
