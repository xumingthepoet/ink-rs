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
  `9/126` after the latest choice slice.
- Green fixtures in the current checkpoint include `basictext/oneline`,
  `basictext/twolines`, `test1`, `choices/no-choice-text`, `choices/one`,
  `choices/single-choice`, `choices/divert-choice`, `choices/sticky-choice`,
  and `knot/single-line`.
- The next compiler-conformance fixture is `choices/multi-choice`; `choices/
  TheIntercept` stays in the last-pass section.
- When fixing any failing legacy fixture, read the source `.ink` and the
  matching assertions first.
- The audit log is a rolling five-entry window with minute timestamps.

## Audit Log

The audit log is a rolling window of the latest five entries. Timestamps use
`YYYY-MM-DD HH:MM`; older history is intentionally trimmed so this file stays
usable as prompt memory.

### 2026-04-23 02:49

- Fixed the `choices/single-choice` parse snapshot so both
  `single_choice1_test` and `single_choic2_test` now pass.
- Validated with the focused `single_choice1_test` and `single_choic2_test`
  compiler-conformance runs.

### 2026-04-23 02:49

- Kept the compiler-conformance queue in WritingWithInk order and continued
  the dependency-light choice slice.
- `choices/one` and `choices/no-choice-text` are green in the current
  checkpoint.

### 2026-04-23 02:49

- Added the rolling five-entry audit-log rule and minute-granularity
  timestamps to the project-memory docs.

### 2026-04-23 02:49

- Documented the fixture-first workflow: read the source `.ink` and the
  corresponding assertions before changing any failing legacy fixture.

### 2026-04-23 02:49

- Restored the default workspace gate, feature-gated the imported legacy
  suites, and kept `make gate` green while migration continues.

## Next Task

Keep shrinking the remaining `compiler_conformance_legacy` parse-snapshot
in dependency-light order. The next unchecked fixture after the current green
choice slice is `choices/multi-choice`. Once compiler-conformance is green
again, move to the imported `csharp_tests_legacy` suite. Both suites remain
feature-gated until they are ready for the default workspace gate.

Stabilize the legacy compiler-to-runtime conformance suite in `ink-test`.

## Repo Structure

- `crates/ink-compiler`: compiler crate under development.
- `docs/ARCHITECTURE.md`: module boundaries and pipeline architecture.
- `docs/PORTING_GUIDE.md`: C# to Rust porting rules.
- `docs/TESTING.md`: test layers and validation strategy.
- `AGENTS.md`: stable project spec and agent rules.
- `PLAN.md`: milestone plan, validation checklist, risks, and notes.
- `IMPLEMENT.md`: execution runbook for repeated `继续` prompts.
- `DOCUMENTATION.md`: this live status and audit log.
- `ink-csharp/`: ignored local official C# reference.
- `ink-runtime/`: ignored local Rust runtime dependency.

## Troubleshooting

- Missing `ink_runtime` path dependency:
  - Ensure ignored directory `ink-runtime/` exists at repository root.
- C# reference unavailable:
  - Ensure ignored directory `ink-csharp/` exists at repository root.
- Unexpected ignored files:
  - Run `git status --short --ignored` and confirm only reference/build
    directories are ignored.
- Runtime dependency warnings:
  - Warnings from `ink-runtime/lib` are tracked as local dependency warnings
    unless a compiler task requires changing runtime integration.
