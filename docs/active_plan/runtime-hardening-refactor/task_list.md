Progress: 4/7

# Runtime Hardening And Lowering Refactor Plan

## Status Key

- `[ ]` pending
- `[~]` in progress
- `[>]` waiting review
- `[x]` complete
- `[!]` blocked

## Milestone 1: Tooling And Small Runtime Panics

### [x] Task 01: Make Gate Portable Without GNU Timeout

Goal: Let `make gate` run on macOS hosts that do not have GNU `timeout` or `gtimeout`, while preserving timeout behavior on hosts that do.

Implementation method:

- Add a small repository-owned timeout wrapper script that accepts the existing `TIMEOUT duration command...` shape used by the Makefile.
- Update `Makefile` so `make gate` uses that wrapper by default.
- Preserve existing behavior when either `timeout` or `gtimeout` is available.
- Fall back to executing the command without a timeout when neither command exists, instead of failing before tests start.

Acceptance criteria:

- `make gate` starts and completes in this macOS environment without a `TIMEOUT=...` override.
- `cargo fmt --all --check`, workspace check, and all gate tests still run from `make gate`.
- The wrapper remains simple shell code with no new external dependency.

Forbidden shortcuts:

- Do not delete the timeout argument shape from the Makefile test commands.
- Do not make `make gate` skip any existing validation command.
- Do not require developers to install GNU coreutils.

Modification boundaries:

- Allowed: `Makefile` and a small repository script under a tooling directory.
- Not allowed: Rust crate source, tests, or CI-only assumptions.

Validation commands:

- `make gate`

Commit record:

- Implementation commit: `63540c5b` (`Make gate timeout wrapper portable`)
- Focused validation: `make gate`
- Review/fix commits: none; review of `63540c5b` found no follow-up changes needed.
- Review validation: `make gate`
- Completion record commit: this commit

### [x] Task 02: Return Errors For Invalid String Numeric Casts

Goal: Replace string-to-int and string-to-float `unwrap()` casts with `StoryError::InvalidStoryState` so malformed runtime string values cannot unwind the host process.

Implementation method:

- Update `Value::cast` string numeric conversions to map parse failures to clear runtime errors.
- Add focused runtime unit tests for invalid string casts to int and float.
- Keep successful string numeric casts unchanged.

Acceptance criteria:

- Invalid string-to-int and string-to-float casts return `StoryError::InvalidStoryState`.
- Existing successful cast behavior and scalar native function behavior remain unchanged.

Forbidden shortcuts:

- Do not catch panics in tests as the implementation.
- Do not change numeric coercion semantics for valid inputs.
- Do not route cast errors through JSON errors.

Modification boundaries:

- Allowed: `crates/ink-runtime/src/value.rs` and focused runtime tests.
- Not allowed: compiler type checking, native function operator tables, or public API shape.

Validation commands:

- `cargo test -p ink-runtime invalid_string_cast`
- `cargo test -p ink-runtime`
- `make gate`

Commit record:

- Implementation commit: `cf8bbb55` (`Return errors for invalid string casts`)
- Focused validation: `cargo fmt --all --check`; `cargo test -p ink-runtime invalid_string_cast`; `cargo test -p ink-runtime`
- Full validation: `make gate`
- Review/fix commits: none; review of `cf8bbb55` found no follow-up changes needed.
- Review validation: `cargo test -p ink-runtime invalid_string_cast`; `make gate`
- Completion record commit: this commit

## Milestone 2: Runtime JSON And State Hardening

### [x] Task 03: Check Runtime JSON Write Preconditions

Goal: Replace save-state and runtime JSON writer `unwrap()` assumptions that can be reached from malformed runtime state with typed `StoryError` results.

Implementation method:

- Make runtime divert JSON conversion return `Result<format::Object, StoryError>` when a target name or target path is missing.
- Update flow choice-thread serialization so a choice without thread generation returns `StoryError::InvalidStoryState`.
- Update thread serialization so missing current or previous pointer targets return `StoryError::InvalidStoryState`.
- Add focused tests that construct malformed runtime objects or save-state data and assert errors instead of panics.

Acceptance criteria:

- Runtime JSON write paths for malformed divert and thread state return errors.
- Valid story save-state and compiled runtime object writing remain unchanged.
- Existing save/load tests continue to pass.

Forbidden shortcuts:

- Do not serialize placeholder paths or fake thread indexes for malformed state.
- Do not silently drop malformed choices, diverts, or callstack entries.
- Do not move runtime save-state JSON into `ink-story-json-format`.

Modification boundaries:

- Allowed: `crates/ink-runtime/src/json/json_write.rs`, `crates/ink-runtime/src/flow.rs`, `crates/ink-runtime/src/callstack.rs`, focused runtime tests.
- Not allowed: compiled story JSON schema changes or format crate ownership changes.

Validation commands:

- `cargo test -p ink-runtime malformed_json_write`
- `cargo test -p ink-runtime`
- `make gate`

Commit record:

- Implementation commit: `02256130` (`Check runtime JSON write preconditions`)
- Focused validation: `cargo fmt --all --check`; `cargo test -p ink-runtime malformed_json_write`; `cargo test -p ink-runtime json_write`; `cargo test -p ink-runtime`
- Full validation: `make gate`
- Review/fix commits: none; review of `02256130` found no follow-up changes needed.
- Review validation: `cargo test -p ink-runtime malformed_json_write`; `make gate`
- Completion record commit: this commit

### [x] Task 04: Harden Path Object Helpers

Goal: Remove avoidable panic points in runtime path/object helpers that can be reached by malformed runtime object graphs or malformed path math.

Implementation method:

- Make `Path` component string and append helpers use direct iteration and saturating bounds instead of indexing with `unwrap()`.
- Add a checked object-root lookup helper for callers that need to return errors.
- Adjust divert target resolution to return a null pointer or clear error for missing target path/object information instead of unwrapping where possible.
- Add focused tests for edge path append cases and malformed divert target objects.

Acceptance criteria:

- Path append and display helpers handle empty or parent-heavy paths without panics.
- Malformed divert target resolution no longer unwinds through save/runtime accessors.
- Existing path compacting and divert tests remain unchanged for valid stories.

Forbidden shortcuts:

- Do not change path string syntax.
- Do not hide broken target resolution by inventing a valid target.
- Do not change compiler emitted path format.

Modification boundaries:

- Allowed: `crates/ink-runtime/src/path.rs`, `crates/ink-runtime/src/object.rs`, `crates/ink-runtime/src/divert.rs`, focused runtime tests.
- Not allowed: compiler path lowering, format crate path tokens, or fixture snapshot updates.

Validation commands:

- `cargo test -p ink-runtime path_`
- `cargo test -p ink-runtime divert_`
- `cargo test -p ink-runtime`
- `make gate`

Commit record:

- Implementation commit: `ac2b30c1` (`Harden runtime path object helpers`)
- Focused validation: `cargo fmt --all --check`; `cargo test -p ink-runtime path_`; `cargo test -p ink-runtime divert_`; `cargo test -p ink-runtime`
- Full validation: `make gate`
- Review/fix commits: none; review of `ac2b30c1` found no follow-up changes needed.
- Review validation: `cargo fmt --all --check`; `cargo test -p ink-runtime path_`; `cargo test -p ink-runtime divert_`; `make gate`
- Completion record commit: this commit

### [ ] Task 05: Harden CallStack Context Access

Goal: Convert callstack context lookups that depend on save-state/runtime stack integrity into checked errors where the caller can recover.

Implementation method:

- Replace unchecked temporary-variable context indexing with checked access and `StoryError::InvalidStoryState`.
- Keep current-thread/current-element APIs that represent internal invariants, but add checked helpers for save-state and variable access paths that can receive malformed context indexes.
- Update callers to use checked helpers where they already return `Result`.
- Add malformed context-index tests that assert `StoryError` instead of panic.

Acceptance criteria:

- Invalid temporary-variable context indexes return `StoryError::InvalidStoryState`.
- Valid temporary variable reads and writes continue to work.
- Existing save/load and variable tests continue to pass.

Forbidden shortcuts:

- Do not catch unwind in tests.
- Do not silently treat invalid local contexts as globals.
- Do not rewrite the whole callstack object model.

Modification boundaries:

- Allowed: `crates/ink-runtime/src/callstack.rs`, direct callers in runtime state/variables code, focused runtime tests.
- Not allowed: compiler scoping rules, save-state JSON schema changes, or unrelated runtime execution behavior.

Validation commands:

- `cargo test -p ink-runtime temporary_variable`
- `cargo test -p ink-runtime`
- `make gate`

Commit record:

- Implementation commit: pending
- Review/fix commits: pending
- Completion record commit: pending

## Milestone 3: Compiler Lowering Structure

### [ ] Task 06: Extract Typed Composite Expression Lowering

Goal: Move typed dynamic composite literal lowering out of the general expression lowering flow so later compiler changes can reason about primitive expressions and composite construction separately.

Implementation method:

- Extract the expected-type array/struct literal lowering helpers into a dedicated lowering module.
- Keep constant composite literals on the compact value-literal path.
- Keep primitive strings and other non-composite expressions on the existing expression path.
- Add or keep focused tests that prove dynamic composite temp initializers, function arguments, and existing string expression snapshots remain unchanged.

Acceptance criteria:

- Compiler output snapshots are unchanged except for no intentional fixture updates.
- Dynamic array and struct literal assignments still run through runtime stack operations.
- The general expression lowering module has a smaller, clearer composite-literal boundary.

Forbidden shortcuts:

- Do not update snapshots to hide behavior drift.
- Do not special-case fixture names or expected output strings.
- Do not change parser or analysis semantics.

Modification boundaries:

- Allowed: `crates/ink-compiler/src/lower/expression.rs`, a new nearby lowering module, module declarations, focused tests if needed.
- Not allowed: parser, analysis, runtime execution, compiled-story JSON schema changes.

Validation commands:

- `cargo test -p ink-test --test runtime_api`
- `cargo test -p ink-test --test compiler_snapshots`
- `cargo test -p ink-test --test typed_values`
- `make gate`

Commit record:

- Implementation commit: pending
- Review/fix commits: pending
- Completion record commit: pending

## Milestone 4: Closeout

### [ ] Task 07: Archive The Completed Plan

Goal: Move this active plan to `docs/finished_plans/` after all implementation tasks are complete, reviewed, and validated.

Implementation method:

- Confirm Tasks 01-06 are marked complete with validation and commit records.
- Move this plan directory from `docs/active_plan/` to `docs/finished_plans/`.
- Mark this task complete only after the active directory no longer exists.

Acceptance criteria:

- No files remain under `docs/active_plan/runtime-hardening-refactor/`.
- The completed task list exists under `docs/finished_plans/runtime-hardening-refactor/`.
- `Progress: 7/7` and all task headings are marked `[x]`.

Forbidden shortcuts:

- Do not archive before Tasks 01-06 are complete.
- Do not leave duplicate active and finished copies of the plan.

Modification boundaries:

- Allowed: this plan directory only.
- Not allowed: unrelated documentation edits.

Validation commands:

- `test ! -e docs/active_plan/runtime-hardening-refactor`
- `test -f docs/finished_plans/runtime-hardening-refactor/task_list.md`

Commit record:

- Completion record commit: pending
