# ink-rs Documentation and Status

This file is the live project memory and audit log. Update it after every
implementation milestone.

## What ink-rs Is

`ink-rs` is a Rust compiler-layer port for ink. It uses the official C# compiler
implementation as the behavior reference and reuses the runtime port now copied
into `crates/ink-runtime` instead of rewriting runtime execution.

## Current Status

- Root repo exists; `ink-csharp/` and `ink-runtime/` remain local ignored
  reference trees.
- Workspace now contains `crates/ink-compiler`, `crates/ink-runtime`,
  `crates/ink-test`, and `crates/ink-tools`.
- `make gate` is the default local gate; warnings are denied; standalone test
  commands must use `timeout` (30 seconds by default).
- Imported legacy suites live in `ink-test` behind the separate
  `legacy-compiler-conformance` and `legacy-csharp-tests` features.
- Compiler-conformance compares `A.ink.parse` first and `A.ink.json` second;
  parse snapshots are generated locally from the Rust parsed tree.
- The compiler-conformance queue follows `WritingWithInk.md` order and is at
  `12/126` after the latest choice slice.
- Green fixtures in the current checkpoint include `basictext/oneline`,
  `basictext/twolines`, `test1`, `choices/no-choice-text`, `choices/one`,
  `choices/single-choice`, `choices/multi-choice`, `choices/suppress-choice`,
  `choices/label-scope`, `choices/divert-choice`, `choices/mixed-choice`,
  and `knot/single-line`.
- The next compiler-conformance fixture is `choices/varying-choice`; `choices/
  sticky-choice` is still under recovery because the exporter is still
  over-inserting a named-flow continuation.
- `choices/TheIntercept` stays in the last-pass section.
- When fixing any failing legacy fixture, read the source `.ink` and the
  matching assertions first.
- The audit log is a rolling five-entry window with minute timestamps.

## Audit Log

The audit log is a rolling window of the latest five entries. Timestamps use
`YYYY-MM-DD HH:MM`; older history is intentionally trimmed so this file stays
usable as prompt memory.

### 2026-04-23 03:28

- Fixed `choices/mixed-choice` in the legacy compiler-conformance queue.
- The choice tail now matches the official JSON, so the queue advances to
  `choices/varying-choice`.

### 2026-04-23 03:09

- Fixed the `choices/suppress-choice` compiler-conformance fixture and the
  `choices/label-scope` runtime JSON pathing so both pass again.
- `choices/sticky-choice` still has an extra named-flow continuation, and
  `choices/mixed-choice` still needs parse-snapshot alignment.
- Validated with focused `suppress_choice_test` and `label_scope_test` runs.

### 2026-04-23 02:55

- Fixed the `choices/multi-choice` compiler-conformance fixture by correcting
  the gather tail JSON shape and keeping the explicit gather target global.
- Validated with the focused `multi_choice_test` and a `no_choice_test`
  regression check.

### 2026-04-23 02:49

- Fixed the `choices/single-choice` parse snapshot so both
  `single_choice1_test` and `single_choic2_test` now pass.
- Validated with the focused `single_choice1_test` and `single_choic2_test`
  compiler-conformance runs.
### 2026-04-23 02:49

- Added the rolling five-entry audit-log rule and minute-granularity
  timestamps to the project-memory docs.

## Next Task

Keep shrinking the remaining `compiler_conformance_legacy` parse-snapshot
in dependency-light order. The next unchecked fixture after the current green
choice slice is `choices/mixed-choice`, while `choices/sticky-choice` remains
the current named-flow JSON blocker. Once compiler-conformance is green again,
move to the imported `csharp_tests_legacy` suite. Both suites remain
feature-gated until they are ready for the default workspace gate.

Stabilize the legacy compiler-to-runtime conformance suite in `ink-test`.
