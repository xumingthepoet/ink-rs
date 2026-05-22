# AGENTS.md

This repository is the Rust implementation of ink-rs, a new domain-specific
language for narrative games. ink-rs is inspired by ink by inkle, but Ink
language compatibility is not a project goal; current language behavior is
defined by this repository.

## Source Of Truth

- Treat the project owner's requested language behavior as authoritative after
  verifying any claim about current behavior.
- Keep repository-authored documentation in English.
- Keep terminology synchronized across `AGENTS.md`, `Notes.md`, `docs/`, tests,
  diagnostics, and code-facing comments.

## Project Goal

- Continue the compiled story JSON format refactor tracked by an active plan
  under `docs/active_plan/`, when one exists.
- Keep `crates/ink-story-json-format` as the single typed owner of compiled
  story JSON data structures, token names, memory-to-JSON serialization, and
  JSON-to-memory deserialization.
- Keep both `crates/ink-compiler` and `crates/ink-runtime` depending directly on
  `ink-story-json-format`.
- Remove remaining duplicate compiled-story JSON schemas from compiler
  lowering/emit code and runtime JSON reader/writer code.
- Keep compiler lowering directly into format crate data, and keep runtime
  compiled-story loading going through the format crate before constructing the
  runtime execution graph.
- Preserve compatibility with the existing compiled-story JSON format unless an
  explicit, documented runtime-format change is required.

## Current Architecture

- `crates/ink-story-json-format` owns the compiled-story JSON wire model and
  codec for `Program`, `Container`, and `Object`.
- `crates/ink-compiler` owns source preparation, parsing, parsed model,
  semantic analysis, lowering into the format model, and JSON emission.
- `crates/ink-runtime` owns runtime execution objects, story state, public
  runtime APIs, external functions, and runtime save-state JSON.
- Runtime execution objects remain runtime-owned. The format crate is only the
  wire-format memory model and JSON codec, not the runtime object graph.
- Runtime save-state JSON still uses runtime-owned reader/writer support.
- For a fuller map, read `docs/Architecture.md`.

## Repository Map

- `crates/ink-runtime`: runtime story engine
- `crates/ink-compiler`: parser, parsed model, analysis, lowering, and JSON
  export pipeline
- `crates/ink-story-json-format`: shared compiled-story JSON format crate
- `crates/ink-dioxus`: Dioxus-facing game engine and web shell
- `crates/text-games-app`: playable text game hub built on `ink-dioxus`
- `crates/ink-test`: conformance and integration tests
- `docs/LanguageOverview.md`: short current-language entry point
- `docs/SyntaxReference.md`: current syntax reference for the latest language
- `docs/SyntaxUpdates.md`: syntax and semantic change log
- `docs/Architecture.md`: architecture and routing notes
- `docs/ink_JSON_runtime_format.md`: compiled-story JSON format notes
- `docs/workflows/`: standing workflow rules for active plans, validation,
  issue capture, and durable notes
- `docs/active_plan/`: active implementation plan workspace
- `docs/finished_plans/`: completed implementation plans
- `docs/issues_found/`: deferred issues discovered during implementation
- `docs/issues_solved/`: issue records moved here after their fixes land
- `Notes.md`: ranked durable working notes that change more often than this file

## Standing Rules

- Prefer design notes and tests before broad language changes; avoid
  speculative refactors that are not tied to a concrete language goal.
- For the current format refactor, keep changes focused on compiler JSON output,
  runtime compiled-story JSON loading, and the format crate. Avoid unrelated
  parser, language, or runtime execution changes.
- If the existing compiler architecture blocks progress, rewrite the affected
  area instead of extending a fragile partial port.
- For existing supported features, preserve current behavior unless there is a
  clear reason to change it.
- When behavior intentionally changes, update tests and maintained
  documentation in the same change.
- When a feature change intentionally replaces previous behavior,
  delete code, tests, fixture hooks, empty macros, and compatibility shims that
  only served that behavior. Do not leave unused `dead_code`, ignored-test, or
  no-op compatibility paths behind.
- Avoid broad unrelated edits when working on compiler or language behavior.
- Do not automatically create or switch branches. Stay on the current branch
  unless the project owner explicitly asks for a branch change.
- When asked to implement an approved plan, active plan task, or issue fix
  outside an active plan, commit the successful, validated change by default.
  Stage only related files, preserve unrelated edits, and skip the commit if
  validation fails or the owner explicitly asks not to commit.
- During an active plan, use the active plan task list as the durable progress
  ledger. Let the implementer decide when optional checkpoint commits are useful
  to avoid an oversized temporary diff or preserve a recovery point. Before the
  active plan is pushed or considered landed, combine active-plan commits into
  one final commit unless the owner asks for a different history shape.
- Use `active plan` terminology consistently. Do not create or reference stale
  aliases for the active-plan directory or concept.
- Store active implementation plans under `docs/active_plan/`. When a plan is
  complete, move it to `docs/finished_plans/`.
- Keep `docs/SyntaxUpdates.md` synchronized with syntax and semantic changes,
  then apply those updates to `docs/SyntaxReference.md` and
  `docs/LanguageOverview.md` when the short overview would otherwise become
  stale.
- Keep `docs/SyntaxReference.md` focused only on the latest supported ink-rs
  syntax. Put historical language changes in `docs/SyntaxUpdates.md`, issue
  records, diagnostics tests, or architecture notes instead.
- When working under `crates/text-games-app/`, read and follow
  `crates/text-games-app/README.md`.

## Workflows

- Active plan task rules and continuation behavior live in
  `docs/workflows/active_plan.md`.
- Validation command policy and `make gate` expectations live in
  `docs/workflows/validation.md`.
- Deferred issue capture rules live in `docs/workflows/issues.md`.
- Durable note maintenance rules live in `docs/workflows/notes.md`.
- Read `Notes.md` before relying on durable compiler, runtime, format-refactor,
  or language-evolution notes. Update `Notes.md`, not this file, when adding,
  deleting, merging, rewriting, liking, or downvoting notes.

## User Assumption Checks

- When the user proposes a language or architecture change based on a claimed
  current behavior, verify the premise before changing code.
- If the premise is wrong, say so directly. Explain the actual behavior with a
  minimal example, point out the inaccurate wording, and identify any ambiguous
  part of the request.
- Example: `VAR smashingWindowItem: int = NONE` is not default int
  initialization with `NONE`; it is explicit initialization from a constant
  reference, assuming `CONST NONE: int = 0` exists. Default initialization is
  the omitted-initializer form `VAR smashingWindowItem: int`.
- Do not implement restrictions just to match an incorrect premise.

## Language And Architecture Requirements

- New syntax or semantics should be represented in parser structures, parsed
  model nodes, lowering/export behavior, tests, and documentation as needed.
- Removing a language feature is allowed when requested, but the removal must be
  explicit: update diagnostics, docs, and tests so the new behavior is clear.
- Do not silently break JSON/runtime compatibility. If compatibility must
  change, document the new contract and update runtime tests.
- Keep source impact visible. When changing or deleting syntax, prefer clear
  diagnostics over ambiguous parse failures.
- Pass tests by implementing the intended language model, not by shaping code
  around individual fixtures.
- Prefer Rust-native representations: `enum`/`struct`/module boundaries,
  ownership-friendly APIs, explicit parser state, and typed parsed-model
  objects.
- When parser behavior is added, prefer reusable parser rules, parser state
  transitions, and parsed-model nodes that can naturally support future
  fixtures.
- When compiler behavior is unclear, inspect the current Rust parser, parsed
  model, lowering, diagnostics, and tests before choosing a design.

## Forbidden Shortcuts

- Do not add fixture-name checks, fixture-path checks, or expected-output checks
  in compiler code.
- Do not hardcode JSON fragments, runtime paths, container names, or snapshot
  strings purely to satisfy a specific fixture.
- Do not intentionally narrow accepted syntax to only the exact surface form
  used by the current fixture.
- Do not add temporary special cases, one-off branches, or
  test-order-dependent logic just to make a fixture pass.
- Do not add ignored tests, skip filters, fixture edits, or expected-output
  edits to hide failures. Test updates are appropriate only when they describe
  an intentional language change.

## Practical Guidance

- If `.codegraph/` exists, use `codegraph context "<task>"`,
  `codegraph query "<symbol>"`, or `codegraph files --filter <dir>` before
  broad project exploration or cross-crate file hunting.
- Treat CodeGraph as a navigation index only; verify behavior in source and
  tests. Run CodeGraph commands sequentially to avoid SQLite lock noise.
- The repo-local Git hooks refresh CodeGraph when configured via
  `core.hooksPath=.githooks`; run `.githooks/update-codegraph-index` manually
  if the index seems stale.
- Use conformance fixtures in `crates/ink-test/` to pin the intended language
  behavior.
- Keep diagnostics clear and actionable.
- Prefer small, reviewable changes that isolate parser, parsed-model, format,
  compiler output, and runtime loading logic.
