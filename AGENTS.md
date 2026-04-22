# AGENTS.md

## Purpose

Port the official ink compiler layer from the C# implementation to Rust while
reusing the existing `ink-runtime/lib` runtime.

The target result is a Rust compiler crate that can compile `.ink` source into
runtime JSON compatible with `ink_runtime::story::Story::new`.

## Operating Model

This project is intended to support long-horizon Codex work. Treat the
following files as durable project memory:

- `AGENTS.md`: project specification, goals, non-goals, constraints,
  deliverables, and agent rules.
- `PLAN.md`: ordered milestone plan with acceptance criteria, risk register,
  architecture notes, and validation commands.
- `IMPLEMENT.md`: runbook for how to execute each work loop.
- `DOCUMENTATION.md`: live status, decisions, audit log, known issues, and
  quickstart commands.

When the user says `继续`, read these files first, then continue with the first
unchecked task in `PLAN.md`. Keep the diff scoped to that task, run the
validation listed for the task, fix failures before moving on, and update
`PLAN.md` plus `DOCUMENTATION.md` before ending the turn.

## Background

The local `ink-csharp/` directory contains the official C# ink implementation.
The compiler source of truth is `ink-csharp/compiler`, especially:

- `Compiler.cs`
- `StringParser/`
- `InkParser/`
- `ParsedHierarchy/`

The local `ink-runtime/` directory contains an unofficial Rust runtime port.
It has runtime coverage only. The compiler crate must depend on it by path and
must not duplicate the runtime.

Both directories are intentionally ignored by Git. They must remain available
locally for development and verification.

## Goals

- Implement a Rust compiler layer under `crates/ink-compiler`.
- Preserve official C# compiler behavior wherever practical.
- Preserve recognizable type and concept names from the C# compiler.
- Compile ink source to runtime JSON accepted by `ink_runtime`.
- Add tests incrementally so each language feature has a small verification
  surface.
- Keep work resumable through `PLAN.md`, `IMPLEMENT.md`, and
  `DOCUMENTATION.md`.

## Non-Goals

- Do not reimplement the runtime layer.
- Do not vendor `ink-csharp/` or `ink-runtime/` into this Git repository.
- Do not redesign the ink language.
- Do not optimize before compatibility is established.
- Do not port all compiler files in one large unverified patch.

## Deliverables

- Public compiler API:
  - `Compiler`
  - `CompilerOptions`
  - `compile_json`
  - `compile`
  - parser and parsed hierarchy modules
- Parser equivalent for official ink syntax.
- Parsed hierarchy equivalent for code generation and reference resolution.
- Runtime export that produces JSON compatible with `ink_runtime`.
- Tests for parser behavior, parsed hierarchy behavior, JSON export, and runtime
  smoke execution.
- Documentation describing architecture, porting rules, testing, status, and
  known gaps.

## Done When

- `cargo fmt --all --check` passes.
- `cargo check --workspace` passes.
- `cargo test --workspace` passes.
- A representative conformance subset compiles from `.ink` source to JSON and
  runs through `ink_runtime`.
- Public docs explain how to build, test, and continue development.
- `PLAN.md` has no unchecked required milestones.
- `DOCUMENTATION.md` records final status, decisions, and known limitations.

## Architecture Direction

- Implement the Rust compiler layer under `crates/ink-compiler`.
- Keep compiler data structure names close to the C# compiler where practical:
  `Compiler`, `InkParser`, `StringParser`, `Parsed::Story`, `Parsed::Object`,
  `Choice`, `Divert`, `FlowBase`, `Weave`, and related parsed hierarchy types.
- Prefer Rust module naming conventions while preserving recognizable type
  names. For example, use `parsed::Story` and `parser::InkParser`.
- Treat `ink-csharp/compiler/ParsedHierarchy` as the source of truth for parsed
  AST structure and runtime export behavior.
- Treat `ink-csharp/compiler/InkParser` and `ink-csharp/compiler/StringParser`
  as the source of truth for parsing behavior.
- Export runtime JSON compatible with `ink_runtime::story::Story::new`.

## Constraints

- Runtime dependency remains `ink_runtime = { path = "ink-runtime/lib" }`.
- Do not vendor or track `ink-csharp/` or `ink-runtime/` in this repository.
- Do not duplicate runtime implementation from `ink-runtime/lib`.
- Keep initial ports small and testable. Port one compiler concept at a time
  instead of creating large unverified translations.
- Preserve behavior over idiomatic rewrites when porting from C#; idiomatic Rust
  is secondary to compatibility.
- Add tests with representative `.ink` snippets and expected compiled JSON or
  runtime behavior as soon as a feature is implemented.
- Each milestone must have clear acceptance criteria and validation commands.
- Failed validation blocks forward progress until repaired or explicitly
  documented as unrelated.
- Long-running work must keep status and decisions in Markdown, not only in chat
  history.
- Plan first, scaffold second, implement third. Do not expand compiler code
  beyond the current milestone before the durable plan is coherent.
- Prefer correctness, determinism, and compatibility over extra features.

## Process Requirements

1. Planning first:
   - Keep `PLAN.md` current with milestones, risk register, acceptance criteria,
     verification commands, and architecture notes.
   - Do not begin broad implementation work until the current milestone is
     specific enough to validate.
2. Scaffold second:
   - Keep the workspace, crate layout, diagnostics, and test structure ready for
     incremental feature work.
   - Ensure the local ignored reference trees remain available but untracked.
3. Implement third:
   - Implement one milestone at a time.
   - Run validation after each milestone.
   - Fix failures before moving on.
   - Keep diffs reviewable and avoid unstructured bulk translations.
   - Record decisions and tradeoffs in `DOCUMENTATION.md`.

## Source of Truth

When project files conflict:

1. `AGENTS.md` defines the target and agent rules.
2. `PLAN.md` defines next work.
3. `IMPLEMENT.md` defines execution behavior.
4. `DOCUMENTATION.md` records current status and decisions.

## Useful Commands

```sh
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

These commands require the ignored local `ink-runtime/` directory to be
present because `crates/ink-compiler` depends on `ink-runtime/lib` by path.

## Verification Rule

Every implementation turn should run the narrowest useful validation first, then
the broader workspace checks when feasible:

```sh
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
```

If validation fails, repair the failure in the same turn unless it is unrelated
to the current task and already documented in `DOCUMENTATION.md`.
