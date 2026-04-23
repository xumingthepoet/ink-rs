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

## Validation

Use the smallest relevant validation first, then widen coverage:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `make compiler-gate`
- `make csharp-gate`

When touching compiler logic, favor focused test runs in `crates/ink-test` before running the full workspace.

## Practical Guidance

- Compare against the official C# implementation when debugging parser or export differences.
- Use the conformance fixtures in `crates/ink-test/` to pin behavior.
- Keep diagnostics clear and actionable.
- Prefer small, reviewable changes that isolate parser, parsed-model, and export logic.

