Progress: 0/2

# Runtime Evaluation Stack Error Handling Plan

## Status Key

- `[ ]` pending
- `[~]` in progress
- `[>]` waiting review
- `[x]` complete
- `[!]` blocked

## Milestone 1: Host-Callable Composite Values

### [>] Task 01: Harden Evaluation-Stack Consumption

Goal: Fix the host-evaluated INTERNAL function panic recorded in `docs/issues_found/2026-05-03-host-evaluated-array-temp-panics.md` by returning `StoryError` for malformed evaluation-stack state and by allowing valid array/struct temp assignment inside host-called INTERNAL functions.

Implementation method:

- Add focused regression coverage for INTERNAL functions that assign array and struct literals to typed temps, return those composite values, and are called through the host API.
- Lower non-constant array and struct literals into runtime stack operations instead of emitting an empty initializer when literal fields depend on variables or expressions.
- Replace the direct `pop().unwrap()` evaluation-stack API with a checked runtime API where malformed stack state can occur.
- Update variable-assignment execution to reject missing or non-`Value` assignment inputs with `StoryError::InvalidStoryState` instead of panicking.
- Update native-function parameter consumption to fail with `StoryError::InvalidStoryState` when too few stack values exist.
- Keep successful runtime behavior and compiled-story JSON output unchanged.
- Move the deferred issue record to `docs/issues_solved/` once the regression passes.

Acceptance criteria:

- Host `call_internal` returns array and object values produced through typed temporary assignments.
- Malformed evaluation-stack underflow and non-value assignment paths return `StoryError` rather than unwinding.
- Existing runtime API behavior remains compatible for valid stories.
- The issue record is moved from `docs/issues_found/` to `docs/issues_solved/` with solved status and validation evidence.

Forbidden shortcuts:

- Do not add fixture-name checks, expected-output checks, or special cases for the new regression.
- Do not bypass typed lowering or runtime execution to synthesize the returned array/object value.
- Do not hide failures with ignored tests, skip filters, or panic-catching wrappers.
- Do not move runtime save-state JSON into `ink-story-json-format`; this task is only runtime execution error handling.

Modification boundaries:

- Allowed: `crates/ink-runtime/src/story_state.rs`, runtime control logic that consumes the evaluation stack, compiler lowering for array/struct literals, focused runtime tests/fixtures, and the matching issue record.
- Allowed if needed: small helper methods in nearby runtime modules to preserve call-site clarity.
- Not allowed: compiler parser semantics, compiled-story JSON schema changes, runtime save-state schema changes, broad `Rc<dyn RTObject>` object-graph refactors.

Validation commands:

- `cargo test -p ink-test runtime_api`
- `cargo test -p ink-runtime`
- `cargo fmt --all --check`
- `make gate`

Commit record:

- Implementation commit: `c6247aee` (`Handle dynamic composite temp evaluation`)
- Focused validation: `cargo fmt --all --check`; `cargo test -p ink-test internal_host_calls_accept_typed_arguments_and_composite_returns -- --nocapture`; `cargo test -p ink-test --test runtime_api`; `cargo test -p ink-test --test compiler_snapshots`; `cargo test -p ink-runtime malformed_`; `cargo test -p ink-runtime`
- Full validation: `make gate TIMEOUT='bash -lc '\''shift; exec "$$@"'\'' bash'`
- Validation note: unmodified `make gate` cannot start in this macOS environment because the Makefile default `timeout` command is unavailable and `gtimeout` is not installed.
- Review/fix commits: pending
- Completion record commit: pending

## Milestone 2: Closeout

### [ ] Task 02: Archive The Completed Plan

Goal: Move this active plan to `docs/finished_plans/` after Task 01 is implemented, reviewed, validated, and marked complete.

Implementation method:

- Confirm Task 01 has a passing focused validation and `make gate`.
- Mark this task complete only after the plan directory has been moved out of `docs/active_plan/`.
- Keep the final plan document as the durable record of validation and commits.

Acceptance criteria:

- No files remain under `docs/active_plan/runtime-evaluation-stack-errors/`.
- The completed task list exists under `docs/finished_plans/runtime-evaluation-stack-errors/`.
- `Progress: 2/2` and both task headings are marked `[x]`.

Forbidden shortcuts:

- Do not archive the plan before Task 01 is complete.
- Do not leave duplicate active and finished copies of the same task list.

Modification boundaries:

- Allowed: this plan directory only.
- Not allowed: unrelated documentation edits.

Validation commands:

- `find docs/active_plan/runtime-evaluation-stack-errors -maxdepth 2 -type f`
- `test -f docs/finished_plans/runtime-evaluation-stack-errors/task_list.md`

Commit record:

- Completion record commit: pending
