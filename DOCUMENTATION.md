# ink-rs Documentation and Status

This file is the live project memory and audit log. Update it after every
implementation milestone.

## What ink-rs Is

`ink-rs` is a Rust compiler-layer port for ink. It uses the official C# compiler
implementation as the behavior reference and reuses the existing `blade-ink-rs`
runtime instead of rewriting runtime execution.

## Current Status

- Root Git repository exists.
- `ink-csharp/` and `blade-ink-rs/` are local ignored reference trees.
- Rust workspace exists with `crates/ink-compiler`.
- `ink-compiler` now exposes a stable API contract with structured diagnostics,
  parse/compile result types, and file handler abstractions.
- Parser, parsed hierarchy, reference resolution, and runtime export are still
  not implemented.
- Long-horizon project memory docs now exist.

## Current Milestone

Milestone 1 is complete. Next work starts at Milestone 2: String Parser
Foundation.

## Verification Checklist

- [x] `cargo fmt --all --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`

Last full verification: `2026-04-22`.

## How to Build and Check

```sh
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
```

The ignored `blade-ink-rs/` directory must be present because the workspace uses
`blade-ink-rs/lib` as a path dependency.

## Decisions

- The runtime is reused from `blade-ink-rs/lib` by path dependency.
- The compiler crate is named `ink-compiler` and lives at
  `crates/ink-compiler`.
- `ink-csharp/compiler` is the source of truth for compiler architecture,
  naming, and behavior.
- Official C# behavior takes precedence over idiomatic Rust redesign.
- Project memory follows the long-horizon Codex pattern: spec, plan, runbook,
  live status, and continuous validation.
- Completed green milestones should be committed in small reviewable commits
  unless the user says not to commit or unrelated user changes are present.

## Known Issues

- `Compiler::compile_json` currently returns structured unsupported diagnostics
  instead of exported JSON.
- `InkParser::parse` currently returns a structured unsupported diagnostic.
- The parsed hierarchy only has placeholder `Object` and `Story` structs.
- `CompilerOptions` now accepts an injectable file handler, but include parsing
  is not implemented yet.
- `cargo check --workspace` reports warnings from ignored dependency
  `blade-ink-rs/lib`; these are upstream/local reference warnings, not current
  compiler crate failures.

## Audit Log

### 2026-04-22

- Initialized the root Git repository.
- Added `.gitignore` rules for local reference trees and Rust build outputs.
- Added initial Rust workspace and `ink-compiler` scaffold.
- Added long-horizon project docs based on the Codex durable-memory workflow:
  `AGENTS.md`, `PLAN.md`, `IMPLEMENT.md`, and `DOCUMENTATION.md`.
- Added project-specific docs under `docs/`.
- Read the linked example Markdown files from the OpenAI article:
  `prompt.md`, `plans.md`, `implement.md`, and `documentation.md`.
- Updated the local docs to include the example's practical patterns:
  verification checklist, risk register, demo script, architecture overview,
  non-stop implementation loop, bug reproduction rule, and live status format.
- Merged the former standalone project specification into `AGENTS.md`, leaving
  `AGENTS.md` as the single project spec and agent rules entry point.
- Defined the Milestone 1 compiler API contract in `ink-compiler`:
  structured `Diagnostic`/`DiagnosticSeverity`, `ParseResult`,
  `CompileJsonResult`, `CompileResult`, `FileHandler`, and
  `DefaultFileHandler`.
- Wired `Compiler` and `InkParser` to return result objects instead of bare
  placeholder errors, and made `CompilerOptions` carry an injectable file
  handler.
- Added API contract tests covering diagnostic shape, file handler cloning, and
  the current unsupported parse/compile behavior.

Validation:

```sh
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted two existing warnings about
unnecessary parentheses around trait object types.

## Next Task

Port the string parser foundation for Milestone 2.

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
- `blade-ink-rs/`: ignored local Rust runtime dependency.

## Troubleshooting

- Missing `bladeink` path dependency:
  - Ensure ignored directory `blade-ink-rs/` exists at repository root.
- C# reference unavailable:
  - Ensure ignored directory `ink-csharp/` exists at repository root.
- Unexpected ignored files:
  - Run `git status --short --ignored` and confirm only reference/build
    directories are ignored.
- Runtime dependency warnings:
  - Warnings from `blade-ink-rs/lib` are tracked as local dependency warnings
    unless a compiler task requires changing runtime integration.
