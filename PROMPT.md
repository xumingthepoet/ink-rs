# ink-rs Project Spec

## Purpose

Port the official ink compiler layer from the C# implementation to Rust while
reusing the existing `blade-ink-rs/lib` runtime.

The target result is a Rust compiler crate that can compile `.ink` source into
runtime JSON compatible with `bladeink::story::Story::new`.

## Background

The local `ink-csharp/` directory contains the official C# ink implementation.
The compiler source of truth is `ink-csharp/compiler`, especially:

- `Compiler.cs`
- `StringParser/`
- `InkParser/`
- `ParsedHierarchy/`

The local `blade-ink-rs/` directory contains an unofficial Rust runtime port.
It has runtime coverage only. The compiler crate must depend on it by path and
must not duplicate the runtime.

Both directories are intentionally ignored by Git. They must remain available
locally for development and verification.

## Goals

- Implement a Rust compiler layer under `crates/ink-compiler`.
- Preserve official C# compiler behavior wherever practical.
- Preserve recognizable type and concept names from the C# compiler.
- Compile ink source to runtime JSON accepted by `bladeink`.
- Add tests incrementally so each language feature has a small verification
  surface.
- Keep work resumable through `PLAN.md`, `IMPLEMENT.md`, and
  `DOCUMENTATION.md`.

## Non-Goals

- Do not reimplement the runtime layer.
- Do not vendor `ink-csharp/` or `blade-ink-rs/` into this Git repository.
- Do not redesign the ink language.
- Do not optimize before compatibility is established.
- Do not port all compiler files in one large unverified patch.

## Hard Constraints

- Runtime dependency remains `bladeink = { path = "blade-ink-rs/lib" }`.
- Compiler behavior follows `ink-csharp/compiler` before idiomatic Rust
  preferences.
- Each milestone must have clear acceptance criteria and validation commands.
- Failed validation blocks forward progress until repaired or explicitly
  documented as unrelated.
- Long-running work must keep status and decisions in Markdown, not only in chat
  history.
- Plan first, scaffold second, implement third. Do not expand compiler code
  beyond the current milestone before the durable plan is coherent.
- Prefer correctness, determinism, and compatibility over extra features.

## Deliverables

- Public compiler API:
  - `Compiler`
  - `CompilerOptions`
  - `compile_json`
  - `compile`
  - parser and parsed hierarchy modules
- Parser equivalent for official ink syntax.
- Parsed hierarchy equivalent for code generation and reference resolution.
- Runtime export that produces JSON compatible with `bladeink`.
- Tests for parser behavior, parsed hierarchy behavior, JSON export, and runtime
  smoke execution.
- Documentation describing architecture, porting rules, testing, status, and
  known gaps.

## Done When

- `cargo fmt --all --check` passes.
- `cargo check --workspace` passes.
- `cargo test --workspace` passes.
- A representative conformance subset compiles from `.ink` source to JSON and
  runs through `bladeink`.
- Public docs explain how to build, test, and continue development.
- `PLAN.md` has no unchecked required milestones.
- `DOCUMENTATION.md` records final status, decisions, and known limitations.

## Source of Truth

When project files conflict:

1. `PROMPT.md` defines the target.
2. `PLAN.md` defines next work.
3. `IMPLEMENT.md` defines execution behavior.
4. `DOCUMENTATION.md` records current status and decisions.
5. `AGENTS.md` gives agent-specific guardrails.

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
