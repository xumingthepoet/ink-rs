# ink-rs

`ink-rs` aims to port the official C# ink compiler layer to Rust.

The runtime layer is reused from the local `ink-runtime/lib` crate. The local
`ink-csharp/` tree is the architecture and naming reference for the compiler
port, especially `ink-csharp/compiler`.

Both reference trees are intentionally ignored by this repository. Keep them
next to this project when building locally.

## Layout

- `crates/ink-compiler`: Rust compiler layer under development.
- `ink-csharp/`: local official C# reference implementation, ignored by Git.
- `ink-runtime/`: local Rust runtime implementation, ignored by Git.

## Long-Horizon Workflow

This repository keeps project memory in Markdown so Codex can continue work
across many prompts without losing the target:

- `AGENTS.md`: project specification, definition of done, and agent rules.
- `PLAN.md`: milestone checklist and validations.
- `IMPLEMENT.md`: execution runbook for each "continue" loop.
- `DOCUMENTATION.md`: live status, decisions, audit log, and known issues.

Type `继续` in the prompt to continue from the next unchecked item in
`PLAN.md`.

Additional project docs:

- `docs/ARCHITECTURE.md`: compiler architecture and module boundaries.
- `docs/PORTING_GUIDE.md`: C# to Rust porting rules and class mapping.
- `docs/TESTING.md`: verification strategy and golden/oracle test plan.

## Current Status

The project currently contains the initial workspace and compiler crate
scaffold. Parser, parsed hierarchy, reference resolution, and runtime export are
to be ported from the C# compiler.
