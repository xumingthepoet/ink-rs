# Compiler Refactor Quality Review

Date: 2026-04-25

This review closes the compiler refactor plan against the runtime-level quality
target: easy to locate, easy to change, explicit module boundaries, and focused
validation before full-gate runs.

## Summary

The compiler is now much closer to the runtime layer in day-to-day
maintainability. The pipeline is explicit, public API boundaries are small, and
most features have predictable owners:

- Source ownership lives in `source.rs` and `source/preprocess.rs`.
- Syntax is split into focused rule modules under `syntax/`.
- Parsed syntax owns semantic data before analysis or lowering consume it.
- Analysis owns semantic diagnostics and lookup indexes.
- Lowering is split into IR, path, index, flow, weave, expression, sequence, and
  conditional modules.
- Emit is a narrow JSON writer.
- Focused tests exist for source preprocessing, parser rules, expression
  parsing, diagnostics, language divergence, public API, and C# compatibility.

The remaining quality gaps are concentrated and documented below. They should
not block the language-evolution phase, but they are the next places to improve
before adding large syntax or flow features.

## Runtime Comparison

What now matches the runtime quality bar:

- Both layers have concept-owned modules instead of one large mixed file.
- The compiler entry point is stable through `Compiler` and `StageOutput<T>`.
- Compiler internals are private by default; callers use stage methods and
  public artifact types.
- Tests can target a stage or feature before `make gate`.
- Runtime JSON compatibility is isolated to lowering IR plus emit.

What still falls short:

- `syntax/expression.rs` still mixes token definitions, tokenization, parser
  diagnostics, parser implementation, and tests.
- `lower/weave.rs` still owns choices, gathers, rejoin planning, choice inner
  containers, gather containers, and several path decisions.
- Runtime path strings are wrapped in parts of lowering, but analysis and some
  lower context code still pass dot-separated paths as plain strings.

## File Size Review

Largest compiler files after the refactor:

- `syntax/expression.rs`: 1171 lines
- `lower/weave.rs`: 881 lines
- `lower/indexes.rs`: 680 lines
- `syntax/parser.rs`: 602 lines
- `lower.rs`: 577 lines
- `lower/expression.rs`: 513 lines

The only compiler files over the rough 800 to 1000 line threshold are
`syntax/expression.rs` and `lower/weave.rs`. Both have known split targets and
are recorded in the follow-up backlog.

The runtime has one larger file, `story_state.rs`, but it is a cohesive owner of
story state serialization and restoration. The compiler hotspots are less
cohesive than that and should be split before major expansion.

## Raw Path And Symbol String Review

Wrapped or acceptable string usage:

- `lower/path.rs` centralizes runtime path compaction and semantic path
  canonicalization.
- `LabelIndex`, `LabelAlias`, and `RuntimePath` wrap label lookup and path
  targets inside lowering.
- `analysis/target_symbols.rs` centralizes target symbol collection and scoped
  target lookup.
- User-facing names, variable names, constant names, and external function names
  remain naturally string-backed.

Remaining risks:

- `analysis/context.rs` still stores flow paths as `String` keys.
- `analysis/targets.rs` still uses `contains('.')` and `split('.')` for some
  target/variable decisions.
- `lower/context.rs` still constructs many runtime container paths with
  `format!`.
- `lower/labels.rs`, `lower/flow.rs`, `lower/sequence.rs`, and `lower/weave.rs`
  still build some runtime paths manually.

These are acceptable for the current refactor because they are concentrated,
covered by path and C# tests, and already have some typed wrappers. The next
step is to introduce path builder types rather than letting the formatted
strings spread again.

## Feature Placement Drill

Reviewed feature: removed `LIST` declaration support.

Touch points:

- Parser ownership: `syntax/parser.rs` detects the removed declaration form.
- Diagnostic ownership: `diagnostic.rs` provides `DiagnosticCode::RemovedFeature`.
- Language tests: `crates/ink-test/tests/language.rs` asserts the stable code.
- Documentation: `docs/WritingWithInk.md` documents the divergence from
  upstream Ink.

Result: the feature is localized to parser diagnostics, tests, and docs. It
does not require parsed-model pollution, lowering special cases, emitted JSON
fragments, or fixture-specific branches.

## Changeability Drills

Expression operator drill:

- Expected touch points: `syntax/expression.rs`, `parsed/expression.rs`,
  `lower/expression.rs`, and focused expression tests.
- Result: behavior is testable, but `syntax/expression.rs` is still too large.

Choice syntax drill:

- Expected touch points: `syntax/choice.rs`, `syntax/text.rs`, `syntax/parser.rs`,
  `parsed/choice.rs`, `lower/weave.rs`, and choice tests.
- Result: ownership is clear at the stage level, but lowering changes still
  funnel through `lower/weave.rs`.

Removed-feature drill:

- Expected touch points: parser diagnostics, stable diagnostic codes, language
  tests, and user docs.
- Result: localized and suitable for future language removals.

Path resolution drill:

- Expected touch points: `analysis/target_symbols.rs`, `analysis/targets.rs`,
  `lower/path.rs`, `lower/context.rs`, `lower/labels.rs`, and path/C# tests.
- Result: path behavior is findable, but still string-heavy across analysis and
  lowering context.

Diagnostic drill:

- Expected touch points: `diagnostic.rs`, parser or analysis owner, and focused
  tests that assert `DiagnosticCode` where available.
- Result: stable enough for tools and tests.

## C# Reference Comments

No obsolete C# parity comments were found in `crates/ink-compiler/src`.
Remaining references in project docs and test harnesses describe upstream Ink as
a compatibility reference or identify the legacy C# test corpus. They do not
make C# parity the default goal for new language work.

## Transitional Code

The only explicit dead-code allowance found in the compiler was the unused
`parent_path` helper in `lower/path.rs`. It has been removed with its
test-only coverage.

## Follow-Up Backlog

1. Split `syntax/expression.rs`.
   Purpose: make expression changes as local as runtime command changes.
   Next step: move token definitions, tokenization, parser errors, and Pratt
   parsing into separate modules under `syntax/expression/`.

2. Split `lower/weave.rs`.
   Purpose: make choice, gather, and rejoin changes independently reviewable.
   Next step: extract choice lowering, gather planning, and rejoin container
   construction into focused lower modules.

3. Introduce typed analysis path keys.
   Purpose: reduce `String`/dot-split mistakes in target and variable lookup.
   Next step: add `FlowPath` or `TargetPath` wrappers shared by
   `analysis/context.rs`, `analysis/target_symbols.rs`, and `analysis/targets.rs`.

4. Introduce a runtime path builder for lowering.
   Purpose: keep runtime path formatting in one module.
   Next step: move common `format!("{parent}.{child}")`, `c-{index}`, and
   `g-{index}` construction behind `lower/path.rs` or a sibling builder module.

5. Add a compiler size fitness check.
   Purpose: prevent new god files from returning quietly.
   Next step: add a lightweight script or test that reports compiler files over
   the threshold, with explicit allowlist entries for known follow-ups.

