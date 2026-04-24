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
- Keep `docs/WritingWithInk.md` synchronized with language syntax changes.

## Continuation Workflow

- If the user sends a continuation prompt such as `continue`, `go on`, `keep going`, `next`, `继续`, `继续吧`, or similar without replacing the task, interpret it as: continue the C# tests campaign.
- The current campaign target is `make csharp-gate`.
- Use the ignored tests in `crates/ink-test/tests/csharp_tests/mod.rs` as the remaining queue. Take the first ignored non-LIST C# test in file order, remove its ignore marker, then fix compiler/runtime behavior until it genuinely passes.

## C# Tests Campaign Rules

- The target suite is `crates/ink-test/tests/csharp_tests.rs`, run through `make csharp-gate`.
- Every continuation cycle must end with at least one additional ignored C# test unignored and genuinely passing.
- After one C# test is genuinely passing and validated, create a git commit before starting the next ignored test.
- After that commit, continue immediately to the next ignored test in file order.
- Repeat until all non-LIST C# tests run by default and pass, unless the user interrupts or changes the task.

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
- Do not add new ignored tests, skip filters, fixture edits, or expected-output edits to hide failures instead of fixing behavior.
- In short: no “special-case”, “narrowed scope”, or other cheating-style test passes.

## Validation

Use the smallest relevant validation first, then widen coverage:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `make gate`
- `make csharp-gate`

When touching compiler logic, favor focused test runs in `crates/ink-test` before running the full workspace.

For the continuation workflow above, the minimum required validation before marking a fixture done is:

- the focused C# test for the current case
- `make csharp-gate`
- `make gate` when compiler or runtime logic changed

## Practical Guidance

- Compare against the official C# implementation when debugging parser or export differences.
- Use the conformance fixtures in `crates/ink-test/` to pin behavior.
- Keep diagnostics clear and actionable.
- Prefer small, reviewable changes that isolate parser, parsed-model, and export logic.

## Working Notes

- During fix work, only record notes that are general, reusable, and likely to help later C# tests campaign work.
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

- [2026-04-25 00:00 CST] [👍5][👎0] The C# tests campaign queue is the ignored non-LIST tests in `crates/ink-test/tests/csharp_tests/mod.rs`. First remove the ignore marker for the next test in file order and run it focused. If failure sits in ported Rust scaffolding, compare upstream C# API overload defaults and parser sentinels like `ParseSuccess` before changing compiler behavior.
- [2026-04-24 00:22 CST] [👍12][👎0] When a fixture exposes structural data, add it to the parsed model first, then lower JSON from that model. For example, `Divert` carries call arguments while `Flow` carries parameters and emits entry `temp=` assignments. This prevents flow semantics from leaking into ad hoc export logic.
- [2026-04-24 08:50 CST] [👍8][👎0] Function calls and string expressions are expressions: lower call arguments before the runtime command/user function token. External declarations only register signatures; matching calls lower to `x()` with `exArgs`. Divert-target arguments used as values count visits and turns unless direct `TURNS_SINCE`/`READ_COUNT` gives a narrower purpose. String expressions lower as `str ... /str`.
- [2026-04-24 09:34 CST] [👍10][👎0] Weave lowering needs a current runtime-container model. After gathers, content and choices stay in that gather; inside flows, linear weave objects must carry flow/stitch container paths. Branch rejoin diverts should use C#-style compact paths, choosing relative only when shorter than absolute.
- [2026-04-24 08:31 CST] [👍6][👎0] Braced inline content is not always a sequence. Try explicit sequence annotations first, then top-level `condition: content` inline conditionals, then expression/default sequence forms. Top-level delimiter scans must respect escaped separators like `\|` before splitting. Keep expression content as `ev ... out /ev`.
- [2026-04-25 00:35 CST] [👍5][👎0] Diagnostics belong over parsed-model semantics. Parse invalid-or-warning C# forms such as function stitches, empty divert statements, and empty choices into nodes with spans, then report diagnostics. Use flow/label indexes for target restrictions and missing direct diverts, but treat declared variables/args as possible variable divert targets.
- [2026-04-24 09:06 CST] [👍9][👎0] Preserve C# trial order/type shape in syntax. Comment elimination is a source prepass and must be tested before runtime whitespace cleanup. Braced logic tries sequence annotations, then conditionals, then expression/default sequence. Divert targets parse before unary prefix. Choice conditions can continue across newline; dash-brace in conditional branches is nested content. Choice parsing separates visible content before trailing diverts; divert-only choices are invisible defaults even with targets.
- [2026-04-24 09:31 CST] [👍3][👎0] Read-count resolution needs a story-level target index for flow and weave labels, but divert resolution must still run through path-mode compaction when the index maps a name to itself; otherwise self/child diverts lose relative C# paths. Local labels still win for same-weave `CNT?` paths.
- [2026-04-24 08:38 CST] [👍3][👎0] Expression operators should be parsed into `Expression::Binary` and lowered by emitting operands followed by the runtime native function token. Split only at top-level operators, respecting quoted strings, so text literals do not accidentally drive expression structure.
- [2026-04-24 09:23 CST] [👍4][👎0] `VAR` declarations are story-scope wherever parsed, including inside functions/flows, and lower into root `global decl`. Story-scope `temp` still registers a variable declaration and can force an empty `global decl`. Parse declarations before normal statements so root-scope diverts resolve correctly. Flow auto-divert only applies when parent flow content lacks a terminator.
