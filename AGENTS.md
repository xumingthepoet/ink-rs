# AGENTS.md

This repository is a Rust implementation and language fork of Ink, originally
ported from inkle's official C# implementation.
The upstream reference lives in `ink-csharp/`.

## Project Goal

- Complete the compiled story JSON format refactor described in
  `CompiledStoryJsonFormatRefactor.md`.
- Add `crates/ink-story-json-format` as the single typed owner of compiled
  story JSON data structures, token names, memory-to-JSON serialization, and
  JSON-to-memory deserialization.
- Make both `crates/ink-compiler` and `crates/ink-runtime` depend directly on
  `ink-story-json-format`.
- Remove duplicate compiled-story JSON schemas from compiler lowering/emit code
  and runtime JSON reader/writer code.
- Delete migration adapters and wrappers before considering the refactor done:
  the compiler should lower directly into format crate data, and the runtime
  should load compiled story JSON through the format crate and consume that data
  directly when constructing its execution graph.
- Preserve compatibility with the existing JSON story format unless an explicit,
  documented runtime-format change is required.

## Current State

- The runtime layer has already been ported from a third-party implementation and is passing tests.
- The compiler currently lowers into compiler-owned runtime-shaped IR and then
  emits compiled story JSON.
- The runtime currently parses compiled story JSON into executable runtime
  objects with a separate JSON reader and token mapping.
- The main active project work is to introduce a shared compiled story JSON
  format crate and converge compiler output plus runtime loading onto it.
- Runtime execution objects should remain runtime-owned. The new format crate is
  only the wire-format memory model and JSON codec.
- Do not treat the current compiler `lower::ir` or runtime JSON reader shape as
  something to preserve at all costs. If rewriting the affected path is clearer
  and better supports the final format boundary, prefer it.

## Repository Layout

- `crates/ink-runtime`: runtime story engine
- `crates/ink-compiler`: parser, parsed model, and JSON export pipeline
- `crates/ink-story-json-format`: target shared compiled story JSON format
  crate for this refactor
- `crates/ink-test`: conformance and integration tests
- `ink-csharp/compiler`: historical C# compiler reference
- `ink-csharp/ink-engine-runtime`: historical C# runtime reference
- `ink-csharp/tests`: historical C# test corpus

## Working Rules

- Treat the project owner's requested language behavior as the source of truth.
- Use `ink-csharp/` as a compatibility reference for legacy behavior, not as a
  veto over intentional language changes.
- Prefer design notes and tests before broad language changes; avoid speculative
  refactors that are not tied to a concrete language goal.
- For the current format refactor, keep changes focused on the compiler JSON
  output path, the runtime compiled-story JSON loading path, and the new format
  crate. Avoid unrelated parser, language, or runtime execution changes.
- If the existing compiler architecture blocks progress, rewrite the affected area instead of extending a fragile partial port.
- For unchanged legacy features, preserve existing behavior unless there is a
  clear reason to change it.
- When behavior intentionally diverges from upstream Ink, update tests and
  documentation in the same change.
- Avoid broad unrelated edits when working on compiler or language behavior.
- Keep `CompiledStoryJsonFormatRefactor.md` synchronized with the active format
  refactor plan.
- Keep `docs/WritingWithInk-updates.md` synchronized with syntax and semantic
  changes, then apply those updates to `docs/WritingWithInk-latest.md`.
- Do not edit `docs/WritingWithInk-origin.md`; it is the upstream C# snapshot.

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
  going`, `next`, `继续`, `继续吧`, or similar without replacing the task,
  interpret it as: continue the compiled story JSON format refactor from
  `CompiledStoryJsonFormatRefactor.md` and validate it with the smallest
  relevant tests, then `make gate` when the change is ready.
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
