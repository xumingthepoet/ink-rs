# Compiler And Runtime Refactor Requirement Plan

Status: completed on 2026-04-27 after full validation. Created on 2026-04-27
from current repository inspection after module support landed.

Keep detailed implementation sequencing in `task_lists.md`.

## Goal

Reduce maintenance risk in the compiler lowering, module analysis, format-token,
runtime JSON, and runtime native-function areas without changing Ink language
semantics or compiled story JSON compatibility.

This plan is a behavior-preserving refactor plan unless a task explicitly says
otherwise and documents the compatibility impact here first.

## Compatibility Boundaries

- Compiled story JSON wire output must remain compatible with existing stories.
- `ink-story-json-format` remains the single typed owner of compiled story JSON
  tokens, structures, serialization, and deserialization.
- Runtime execution objects remain runtime-owned and are not moved into the
  format crate.
- Runtime save-state JSON remains runtime-owned. Save-state reader refactors may
  improve error reporting, but must not silently change the save-state schema.
- No syntax, parser, diagnostics, or language behavior changes are part of this
  plan unless a task is updated with explicit owner approval.
- Module visibility, import, entry-point, and reachability semantics from the
  finished module-support plan remain unchanged.

## Refactor Targets

### Compiler Lowering Context

The compiler lowering path currently passes the same large bundle of indexes and
path context through many functions. Introduce a lowering context object that
owns or borrows these shared inputs and gives lowering helpers a stable API.

Expected result:

- Fewer long parameter lists in `crates/ink-compiler/src/lower`.
- No compiled JSON change.
- No fixture-specific lowering behavior.
- Easier follow-up changes to expression, assignment, divert, weave, sequence,
  and conditional lowering.

### Typed Native Function Tokens

`ControlCommand` is already typed in `ink-story-json-format`, but native
function tokens still use raw `String` values. Move native function token names
into a typed format enum while preserving the same wire tokens.

Expected result:

- `ink-story-json-format` owns native function token names.
- Compiler lowering constructs typed native function objects instead of raw
  token strings.
- Runtime loading maps typed native function tokens to runtime operations.
- Unknown native function tokens still produce clear JSON/runtime errors.

### Module Analysis Structure

Module support left `analysis/modules.rs` carrying several responsibilities:
symbol indexing, dependency analysis, import-use checking, reachability, entry
point diagnostics, and tests. Split this area along those boundaries and share
analysis indexes instead of rebuilding them repeatedly.

Expected result:

- Smaller module analysis files with clear ownership.
- Existing module diagnostics and warnings are preserved.
- Analysis pass ordering remains explicit.
- `CheckedStory` continues to expose stable indexes required by lowering.

### Runtime JSON Robustness

Compiled story JSON already flows through the format crate, while save-state and
runtime-token JSON still have direct `serde_json::Value` handling and several
unchecked unwraps. Refactor this path toward typed helpers and explicit
`StoryError::BadJson` failures.

Expected result:

- Malformed save-state JSON reports errors instead of panicking in covered paths.
- Save-state JSON shape remains unchanged.
- Compiled story loading still goes through `ink-story-json-format`.

### Native Function Runtime Organization

`native_function_call.rs` mixes token metadata, arity, dispatch, composite
operations, numeric operations, boolean operations, and tests. Split it into a
small module family without changing operation semantics.

Expected result:

- Operation metadata and operation implementations are easier to review.
- Composite array/object operations stay covered by focused tests.
- Numeric/string/bool operations keep current runtime behavior.

### Mechanical Cleanup

Low-risk Rust cleanups discovered while preparing the plan should be handled in
reviewable tasks after the structural changes they support. These cleanups must
not hide behavior changes.

Expected result:

- Targeted clippy warnings such as `manual_ok_err`, `ptr_arg`,
  `unnecessary_unwrap`, and `get_first` are removed where they are local and
  behavior-preserving.
- Broad style churn is avoided.

## Validation Policy

Each implementation task must run focused validation for the touched surface and
then `make gate` before being marked complete. Use the smallest relevant test
first, then widen coverage.

Minimum broad checks:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `make gate`

Focused checks should prefer:

- `cargo test -p ink-story-json-format`
- `cargo test -p ink-compiler lower::`
- `cargo test -p ink-compiler analysis::modules`
- `cargo test -p ink-test --test language module`
- `cargo test -p ink-runtime json`
- `cargo test -p ink-runtime native_function_call`
