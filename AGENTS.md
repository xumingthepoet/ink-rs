# AGENTS.md

This repository is a Rust implementation and language fork of Ink, originally
ported from inkle's official C# implementation.
The upstream reference lives in `ink-csharp/`.

## Project Goal

- Continue the compiled story JSON format refactor tracked by the active plan
  under `docs/current_plan/`, when one exists.
- Keep `crates/ink-story-json-format` as the single typed owner of compiled
  story JSON data structures, token names, memory-to-JSON serialization, and
  JSON-to-memory deserialization.
- Keep both `crates/ink-compiler` and `crates/ink-runtime` depending directly on
  `ink-story-json-format`.
- Remove remaining duplicate compiled-story JSON schemas from compiler
  lowering/emit code and runtime JSON reader/writer code.
- Keep the compiler lowering directly into format crate data, and keep runtime
  compiled-story loading going through the format crate before constructing the
  runtime execution graph.
- Preserve compatibility with the existing JSON story format unless an explicit,
  documented runtime-format change is required.

## Current State

- The runtime layer has already been ported from a third-party implementation and is passing tests.
- The shared format crate exists and owns the compiled story JSON wire model and
  codec for `Program`, `Container`, and `Object`.
- The compiler lowers checked stories directly into
  `ink_story_json_format::Program`; JSON emission serializes that format model.
- The runtime loads compiled story JSON through `ink-story-json-format`, then
  converts the format model into runtime-owned execution objects.
- Runtime execution objects remain runtime-owned. The format crate is only the
  wire-format memory model and JSON codec, not the runtime object graph.
- Runtime save-state JSON still uses runtime-owned reader/writer support.

## Repository Layout

- `crates/ink-runtime`: runtime story engine
- `crates/ink-compiler`: parser, parsed model, and JSON export pipeline
- `crates/ink-story-json-format`: shared compiled story JSON format crate
- `crates/ink-test`: conformance and integration tests
- `ink-csharp/compiler`: historical C# compiler reference
- `ink-csharp/ink-engine-runtime`: historical C# runtime reference
- `ink-csharp/tests`: historical C# test corpus
- `docs/current_plan`: active implementation plan workspace
- `docs/finished_plans`: completed implementation plans
- `docs/WritingWithInk-updates.md`: ink-rs syntax and semantic change log
- `docs/WritingWithInk-latest.md`: current maintained writing guide
- `docs/WritingWithInk-origin.md`: upstream C# writing guide snapshot
- `docs/Architecture.md`: ink-rs architecture notes
- `docs/ink_JSON_runtime_format.md`: compiled story JSON format notes
- `Notes.md`: durable working notes that change more often than this file

## Working Rules

- Treat the project owner's requested language behavior as the source of truth.
- Use `ink-csharp/` as a compatibility reference for legacy behavior, not as a
  veto over intentional language changes.
- Prefer design notes and tests before broad language changes; avoid speculative
  refactors that are not tied to a concrete language goal.
- For the current format refactor, keep changes focused on the compiler JSON
  output path, the runtime compiled-story JSON loading path, and the format
  crate. Avoid unrelated parser, language, or runtime execution changes.
- If the existing compiler architecture blocks progress, rewrite the affected area instead of extending a fragile partial port.
- For unchanged legacy features, preserve existing behavior unless there is a
  clear reason to change it.
- When behavior intentionally diverges from upstream Ink, update tests and
  documentation in the same change.
- When a feature change makes old behavior obsolete or explicitly removed,
  delete code, tests, fixture hooks, empty macros, and compatibility shims that
  only served that old behavior. Do not leave unused `obsolete`, `removed
  behavior`, `dead_code`, ignored-test, or no-op compatibility paths behind.
- Avoid broad unrelated edits when working on compiler or language behavior.
- Do not automatically create or switch branches. Stay on the current branch
  unless the project owner explicitly asks for a branch change.
- Write repository-authored documentation in English. Avoid non-English prose in
  docs; describe localized user prompts generically unless exact text is
  required.
- Keep project terminology synchronized across `AGENTS.md`, `Notes.md`, `docs/`,
  tests, diagnostics, and code-facing comments. When renaming a concept,
  directory, or workflow term, update related documentation references in the
  same change.
- Keep active plan details in `docs/current_plan/`; do not add one-off plan file
  names to `AGENTS.md`.
- Keep `docs/WritingWithInk-updates.md` synchronized with syntax and semantic
  changes, then apply those updates to `docs/WritingWithInk-latest.md`.
- Store active implementation plans under `docs/current_plan/`. When a plan is
  complete, move it to `docs/finished_plans/`. Do not update `AGENTS.md` for
  each new plan unless the planning workflow itself changes.
- Do not edit `docs/WritingWithInk-origin.md`; it is the upstream C# snapshot.

## Active Plan Task Lists

- Task lists under `docs/current_plan/` must be implementation plans, not
  research logs. A task is not valid if it is only read-only inventory,
  planning, or context gathering.
- Put necessary inventory inside the implementation method of the first task
  that uses it. Do not create standalone read-only tasks whose only output is a
  note in the task list.
- Do not write task lists as one-line task tables. Each task must have its own
  section with enough detail for another implementer to execute it without
  guessing: goal, implementation method, acceptance criteria, forbidden
  shortcuts, modification boundaries, validation commands, and commit record.
- The first non-blank line of every task list must be a progress indicator in
  `Progress: X/N` form.
- Every task must be represented by a checkbox in its task heading, such as
  `### [ ] Task 01: ...`. Do not rely only on table status columns.
- Group related tasks under milestone sections so parser, analysis, lowering,
  runtime, fixture, and documentation work are easy to navigate.
- Prefer task lists with more than 20-25 tasks and fewer than 100 tasks. Split
  large milestones into reviewable implementation tasks, but do not split out
  read-only inventory or planning-only tasks.
- A valid task must produce a reviewable repository change that includes
  production code, test code, fixtures, editor assets, or another code-adjacent
  artifact. Maintained documentation should be updated in the same task when
  behavior changes, but pure prose-only tasks are not valid implementation
  tasks except for final plan closeout.
- Task scope should be large enough to make meaningful progress and small
  enough to review safely. Prefer a cohesive vertical slice that adds tests,
  implementation, diagnostics, docs, and migration updates together when those
  pieces are required for the behavior to be correct.
- Every active-plan task must pass focused validation relevant to the changed
  surface and then `make gate` before being marked complete. Do not add
  documentation-only or read-only exceptions to task completion, except for the
  final plan closeout task after implementation is already validated.
- Keep the progress counter, status key, acceptance conditions, forbidden
  shortcuts, and modification boundaries in the task list itself. Update task
  status and progress only after validation has passed.
- Record validation and commit metadata inside the relevant task section. Do not
  append separate notes, logs, journals, or running commentary after the task
  list.
- Commit after each completed task and record the commit hash in that task
  section. Do not start the next task with uncommitted changes from the prior
  task.

## User Assumption Checks

- When the user proposes a language or architecture change based on a claimed
  current behavior, verify the premise before changing code. The owner's desired
  behavior remains authoritative only after the premise is checked and any
  intended divergence is explicit.
- If the premise is wrong, say so directly. Explain the actual behavior with a
  minimal example, point out the inaccurate wording, and identify any ambiguous
  part of the request. Do not implement a code change just to match an
  incorrect premise.
- Example: `VAR smashingWindowItem: int = NONE` is not "default int
  initialization with NONE". It is explicit initialization from a constant
  reference, assuming `CONST NONE: int = 0` exists. Default initialization is the
  omitted-initializer form `VAR smashingWindowItem: int`.
- Failure mode to avoid: if the user says this form "must be disallowed" because
  they confused a constant reference with default initialization, do not
  accommodate that hallucinated premise by adding restrictions. The correct
  response is to stop, state that the premise is incorrect, explain that the
  request is ambiguous or misstated, and ask for confirmation only if they still
  want an intentional language change after the correction.

## Continuation Workflow

- If the user sends a continuation prompt such as `continue`, `go on`, `keep
  going`, `next`, or a localized equivalent without replacing the task,
  interpret it as: continue the active implementation plan in
  `docs/current_plan/` if one exists, otherwise continue the current project goal
  from the repository state and durable notes.
- Active plan directories under `docs/current_plan/` should contain the task
  instructions for that plan. They may also contain a progress file that must be
  kept synchronized while development proceeds; follow the plan's own files for
  the exact progress-tracking convention.
- When an active plan is complete, move its plan directory from
  `docs/current_plan/` to `docs/finished_plans/`.
- Validate continuation work with the smallest relevant tests first, then
  `make gate` when the change is ready.
- `make gate` is still the full project gate. If intentional language changes
  make legacy C# compatibility tests obsolete, update or replace those tests as
  part of the same language-change work rather than hiding failures.

## Language Evolution Rules

- New syntax or semantics should be represented in parser structures, parsed
  model nodes, lowering/export behavior, tests, and documentation as needed.
- Removing a language feature is allowed when requested, but the removal must be
  explicit: update diagnostics, docs, and tests so the new behavior is clear.
- Do not preserve awkward upstream behavior only for parity if it conflicts with
  the new language direction.
- Do not silently break JSON/runtime compatibility. If compatibility must
  change, document the new contract and update runtime tests.
- Keep migration impact visible. When changing or deleting old syntax, prefer
  clear diagnostics over ambiguous parse failures.

## Architecture Requirements

- Pass tests by implementing the intended language model, not by shaping code
  around individual fixtures.
- Prefer Rust-native representations: `enum`/`struct`/module boundaries,
  ownership-friendly APIs, explicit parser state, and typed parsed-model objects
  instead of C#-style inheritance.
- When parser behavior is added, prefer reusable parser rules, parser state transitions, and parsed-model nodes that can naturally support future fixtures.
- When legacy compiler behavior is unclear and still relevant, inspect the
  corresponding C# parser or parsed-hierarchy implementation before choosing a
  Rust-side design.

## Forbidden Shortcuts

- Do not add fixture-name checks, fixture-path checks, or expected-output checks
  in compiler code.
- Do not hardcode JSON fragments, runtime paths, container names, or snapshot
  strings purely to satisfy a specific fixture.
- Do not intentionally narrow accepted syntax to only the exact surface form
  used by the current fixture.
- Do not add "temporary" special cases, one-off branches, or test-order-dependent
  logic just to make a fixture pass.
- Do not add ignored tests, skip filters, fixture edits, or expected-output edits
  to hide failures. Test updates are appropriate only when they describe an
  intentional language change.
- In short: no "special-case", "narrowed scope", or other cheating-style test
  passes.

## Validation

Use the smallest relevant validation first, then widen coverage:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `make gate`

When touching compiler logic, favor focused test runs in `crates/ink-test` before running the full workspace.

For the current compiled-story format refactor, the minimum required validation
before marking a change done is:

- focused tests that cover the changed format data, JSON serialization, JSON
  deserialization, compiler output, or runtime loading behavior
- `make gate`

## Practical Guidance

- Compare against the official C# implementation when debugging legacy parser or
  export differences that are still meant to be compatible.
- Use conformance fixtures in `crates/ink-test/` to pin the intended language
  behavior.
- Keep diagnostics clear and actionable.
- Prefer small, reviewable changes that isolate parser, parsed-model, format,
  compiler output, and runtime loading logic.

## Working Notes

- Durable working notes live in `Notes.md` next to this file. Read that file
  before relying on notes during compiler, runtime, format-refactor, or
  language-evolution work.
- Update `Notes.md`, not `AGENTS.md`, when adding, deleting, merging, rewriting,
  liking, or downvoting notes. This keeps the stable agent instructions from
  changing just because the note set evolves.
- Follow the note rules in `Notes.md`.
