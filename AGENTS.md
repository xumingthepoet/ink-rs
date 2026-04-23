# AGENTS.md

This repository is a Rust port of inkle's official C# implementation of Ink.
The upstream reference lives in `ink-csharp/`.

## Project Goal

- Port the official Ink language implementation to Rust.
- Keep behavior aligned with the upstream C# code and test corpus.
- Preserve runtime compatibility with the existing JSON story format.

## Current State

- The runtime layer has already been ported from a third-party implementation and is passing tests.
- The compiler layer is the current bottleneck.
- The compiler rewrite is now the preferred path forward.
- Do not treat the current compiler code as something to preserve at all costs. If a clean rewrite is simpler and more correct, prefer it.

## Repository Layout

- `crates/ink-runtime`: runtime story engine
- `crates/ink-compiler`: parser, parsed model, and JSON export pipeline
- `crates/ink-test`: conformance and integration tests
- `ink-csharp/compiler`: official C# compiler reference
- `ink-csharp/ink-engine-runtime`: official C# runtime reference
- `ink-csharp/tests`: official C# test corpus

## Working Rules

- Use `ink-csharp/` as the behavioral source of truth when compiler behavior is unclear.
- Prefer test-driven changes over speculative refactors.
- Keep compiler work focused on the Rust compiler layer unless runtime changes are strictly required.
- If the existing compiler architecture blocks progress, rewrite the affected area instead of extending a fragile partial port.
- Match upstream behavior first, then improve Rust-side structure only where it does not change semantics.
- Avoid broad unrelated edits when working on compiler behavior.

## Continuation Workflow

- If the user sends a continuation prompt such as `continue`, `go on`, `keep going`, `next`, `继续`, `继续吧`, or similar without replacing the task, interpret it as: continue the compiler conformance campaign.
- `priority.md` is the authoritative queue for that campaign. Always take the first unchecked compiler-conformance fixture from `priority.md`, work in that order, and do not skip ahead.
- Keep `priority.md` synchronized with reality. When a fixture genuinely passes, update its checkbox and progress count before moving on. If the file has drifted from the actual passing set, fix the drift before resuming the queue.

## Compiler Conformance Campaign Rules

- The target suite is `crates/ink-test/tests/compiler_conformance.rs` and the corresponding `make compiler-gate` target.
- Every continuation cycle must end with at least one additional compiler-conformance fixture genuinely passing.
- After one fixture is genuinely passing, validated, and recorded in `priority.md`, create a git commit before starting the next unchecked fixture.
- After that commit, continue immediately to the next unchecked fixture in `priority.md`.
- Repeat until all 118 fixtures pass, unless the user interrupts or changes the task.

## Architecture Requirements

- Pass tests by moving the Rust compiler closer to the C# compiler model in `ink-csharp/compiler`, not by shaping the code around individual fixtures.
- Prefer Rust-native representations of the same compiler concepts: `enum`/`struct`/module boundaries, ownership-friendly APIs, explicit parser state, and typed parsed-model objects instead of C#-style inheritance.
- When parser behavior is added, prefer reusable parser rules, parser state transitions, and parsed-model nodes that can naturally support future fixtures.
- When compiler behavior is unclear, inspect the corresponding C# parser or parsed-hierarchy implementation before choosing a Rust-side design.

## Forbidden Shortcuts

- Do not add fixture-name checks, fixture-path checks, or expected-output checks in compiler code.
- Do not hardcode JSON fragments, runtime paths, container names, or snapshot strings purely to satisfy a specific fixture.
- Do not intentionally narrow accepted syntax to only the exact surface form used by the current fixture.
- Do not add “temporary” special cases, one-off branches, or test-order-dependent logic just to make a fixture pass.
- Do not modify tests, fixtures, or `priority.md` to hide failures instead of fixing compiler behavior.
- In short: no “special-case”, “narrowed scope”, or other cheating-style test passes.

## Validation

Use the smallest relevant validation first, then widen coverage:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `make compiler-gate`
- `make csharp-gate`

When touching compiler logic, favor focused test runs in `crates/ink-test` before running the full workspace.

For the continuation workflow above, the minimum required validation before marking a fixture done is:

- the focused compiler-conformance test for the current fixture
- `make gate`
- `make compiler-gate`

## Practical Guidance

- Compare against the official C# implementation when debugging parser or export differences.
- Use the conformance fixtures in `crates/ink-test/` to pin behavior.
- Keep diagnostics clear and actionable.
- Prefer small, reviewable changes that isolate parser, parsed-model, and export logic.

## Working Notes

- During fix work, only record notes that are general, reusable, and likely to help later compiler-conformance work.
- Do not record fixture-specific hacks, temporary observations, or narrow one-off facts.
- Keep this section to at most 10 entries total.
- If you want to add a new entry when the section already has 10, first delete the least important entry, delete outdated material, or merge overlapping entries.
- Keep each entry under 100 words.
- Keep entries high-signal and durable.
- Each note should carry both counts, such as `[👍0][👎0]`.
- Only increment a note's like count when, during the current work, that note provided real practical help.
- Only increment a note's downvote count when, during the current work, that note caused real confusion, wasted work, or pushed the fix in the wrong direction.
- Use likes and downvotes as signals when deciding what to keep, merge, rewrite, or delete, but still remove or rewrite notes that have become outdated.

## Working Note Entries

- [2026-04-24 00:02 CST] [👍12][👎0] First distinguish “queue maintenance” from real compiler work: if a priority fixture already passes under existing generic behavior, just enable it, validate it, update `priority.md`, and commit. Only change compiler code when the fixture exposes a genuine model or export gap.
- [2026-04-24 00:22 CST] [👍5][👎0] When a conformance fixture introduces a new structural node in the parse snapshot, add that node to the parsed model first, then lower JSON from that model. Keeping `Story`/`Flow` aligned with the C# compiler shape prevents flow semantics from leaking into ad hoc export logic and keeps parser, snapshot, and runtime output moving together.
- [2026-04-24 00:39 CST] [👍0][👎0] Choice lowering inside named flows needs its own path context. Root choices can use root-relative targets like `0.c-0`, but flow-local choices need compact relative targets such as `.^.c-0` or `.^.^.c-0`, while return labels still need stable absolute anchors inside the flow container.
