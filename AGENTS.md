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

- During current format-refactor or language-evolution work, only record notes
  that are general, reusable, and likely to help future compiler, runtime, or
  format-design work.
- Do not record fixture-specific hacks, temporary observations, or narrow one-off
  facts.
- Keep this section to at most 10 entries total.
- If you want to add a new entry when the section already has 10, first delete the least important entry, delete outdated material, or merge overlapping entries.
- Keep each entry under 100 words.
- Keep entries high-signal and durable.
- Each note should carry both counts, such as `[👍0][👎0]`.
- Only increment a note's like count when, during the current work, that note provided real practical help.
- Only increment a note's downvote count when, during the current work, that note caused real confusion, wasted work, or pushed the fix in the wrong direction.
- Use likes and downvotes as signals when deciding what to keep, merge, rewrite, or delete, but still remove or rewrite notes that have become outdated.

## Working Note Entries

- [2026-04-25 00:00 CST] [👍27][👎0] Legacy C# tests now run through `make gate`. Keep them passing for unchanged legacy behavior, but when the language intentionally changes, update or replace obsolete compatibility tests instead of preserving C# parity by default.
- [2026-04-24 00:22 CST] [👍14][👎0] When a fixture exposes structural data, add it to the parsed model first, then lower JSON from that model. `Divert` carries call arguments, `Flow` carries parameters, and `TunnelOnwards` preserves override targets plus arguments. This prevents flow semantics from leaking into ad hoc export logic.
- [2026-04-24 08:50 CST] [👍9][👎0] Function calls and string expressions are expressions: lower call arguments before the runtime command/user function token. External declarations only register signatures; matching calls lower to `x()` with `exArgs`. Divert-target arguments used as values count visits and turns unless direct `TURNS_SINCE`/`READ_COUNT` gives a narrower purpose. String expressions lower as `str ... /str` and can contain nested mixed text/logic.
- [2026-04-24 09:34 CST] [👍13][👎0] Weave handling needs a current runtime-container model. Grouping derives base indentation from the first weave point, not always depth 1. Gathers are structural weave points even without choices. Root weave participates in weave-point naming before flow weaves. After gathers, content and choices stay in that gather; inside flows, linear weave objects carry flow/stitch container paths. Branch rejoin diverts use C#-style compact paths. Named content metadata must stay as the container tail.
- [2026-04-24 08:31 CST] [👍7][👎0] Braced inline or multiline content is not always a sequence. Try sequence annotations, then `condition: content` conditionals, then expression/default sequences. A multiline conditional closing brace can have same-line suffix; parse suffix as inline content and nested multiline logic before adding newline. Keep expression content as `ev ... out /ev`.
- [2026-04-25 00:35 CST] [👍15][👎0] Diagnostics and compiler-pipeline behavior belong over parsed-model semantics. Story-wide `CONST` redefinition only errors when values change; constant refs expand during expression lowering before path indexes are planned. Flow args used as variable divert targets must be marked `->`, but expression divert targets that resolve to variables must omit `->`; direct divert statements still use `-> var`. Sealed conditional/sequence loose choices are analysis errors. `TODO:` is an AuthorWarning node and is ignored by loose-end termination. `INCLUDE` inserts non-flow content at the include site and appends included flows to story end.
- [2026-04-24 09:06 CST] [👍14][👎0] Preserve C# trial order/type shape in syntax. Route parser alternatives through `RuleParser` or the multiline rule checkpoint so failed trials rewind cursor/index and diagnostics. Comment elimination is a source prepass. Identifiers may start with digits but cannot be all digits. Braced logic tries sequence annotations, then conditionals, then expression/default sequence. Inline content tokenization pauses on glue, braces, tags, and both divert arrows (`->`, `<-`). Choice parsing separates visible content before trailing diverts.
- [2026-04-24 09:31 CST] [👍5][👎0] Read/turn-count resolution needs a story-level target index for flow and weave labels, but metadata pre-scans must still carry path-mode context so sibling stitches get counted. Count metadata belongs on the target's container-for-counting, including choice inner containers; local labels still win for same-weave `CNT?` paths.
- [2026-04-24 08:38 CST] [👍5][👎0] Expression operators should be parsed into `Expression::Binary` and lowered by emitting operands followed by the runtime native function token. Split only at top-level operators, respecting quoted strings. Operators with the same C# precedence split as one group to preserve left associativity. Contains operators `?`/`has` and `!?`/`hasnt` lower to native `?`/`!?`.
- [2026-04-24 09:23 CST] [👍8][👎0] `VAR` declarations are story-scope wherever parsed, including inside functions/flows, and lower into root `global decl`. Story-scope `temp` registers a variable declaration and can force an empty `global decl`. Variable refs resolve like C#: closest flow args/temps, then story globals; parent knot locals do not leak into child stitches. Temp names collide with current flow arguments. Flow auto-divert only applies when parent flow content lacks a terminator.
