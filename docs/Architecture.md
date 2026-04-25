# Architecture

## Overview

This repository has two main layers:

- `crates/ink-compiler`: parses Ink source, checks the parsed story, lowers it
  into runtime-shaped IR, and emits the JSON story format.
- `crates/ink-runtime`: loads and runs the JSON story format.

The compiler and runtime are Rust-native implementations. Upstream
`ink-csharp/` remains a behavioral reference for legacy-compatible features,
but new implementation work should follow the module boundaries and ownership
model documented here rather than copying the C# class structure.

The high-level compiler pipeline is:

```text
SourceInput
  -> source preprocessing
  -> syntax parsing
  -> parsed model
  -> analysis
  -> lowering IR
  -> JSON emit
  -> runtime Story
```

`Compiler::compile` in `crates/ink-compiler/src/compiler.rs` wires these stages
together. Each stage returns a `StageOutput<T>` with an optional artifact and a
list of `Diagnostic` values.

## Compiler Pipeline

### Source

Source ownership lives in `crates/ink-compiler/src/source.rs` and
`crates/ink-compiler/src/source/preprocess.rs`.

- `SourceInput` is the public source entry point.
- `FileHandler` resolves and loads `INCLUDE` files.
- `SourceFile` is the compiler-internal line model passed to syntax parsing.
- `SourceLine` carries normalized line text plus a `SourceSpan`.
- `preprocess_includes` expands `INCLUDE` statements before syntax parsing,
  preserves root/flow ordering, detects recursive includes, and keeps per-line
  source spans for included files.

Source preprocessing should stay limited to file-level concerns: comments,
includes, file names, and line spans. It should not parse language constructs.

### Diagnostics

Diagnostics are defined in `crates/ink-compiler/src/diagnostic.rs`.

`Diagnostic` carries:

- severity: error, warning, or author warning
- optional `DiagnosticCode`
- message
- source filename, line, and column

Messages are still important for users, but tests should prefer stable codes
when a code exists. New parser or analysis errors should get a code when the
category is likely to be asserted in tests or handled by tools.

### Syntax

Syntax parsing lives in `crates/ink-compiler/src/syntax/`.

The top-level parser is `syntax/parser.rs`. It consumes `SourceFile`, walks
`SourceLine` values, and dispatches to focused statement rules in a stable
order. The parser should own syntax decisions and diagnostics for malformed
source. Lowering should not compensate for syntax shapes that were parsed
ambiguously.

Important syntax modules:

- `rule.rs`: `RuleParser`, the local parser facade for one source line.
- `state.rs`: parser state and rule checkpoints.
- `scan.rs`: shared top-level scanning for strings, braces, parentheses, and
  escaped separators.
- `expression.rs`: tokenization and Pratt-style expression parsing.
- `choice.rs`, `gather.rs`, `weave.rs`: weave-point syntax.
- `conditional.rs`, `sequence.rs`: braced multiline and inline structures.
- `text.rs`: inline content, glue, tags, inline diverts, and braced content.
- `knot.rs`, `variable.rs`, `declaration.rs`, `logic.rs`, `divert.rs`:
  statement families.

Rules should fail by rewinding parser state unless they intentionally emit a
diagnostic. Use `RuleParser` and `scan` helpers rather than ad hoc cursor
management.

### Parsed Model

The parsed model lives in `crates/ink-compiler/src/parsed/`.

Parsed types represent high-level Ink concepts: `Story`, `Flow`, `Weave`,
`Choice`, `Gather`, `Divert`, `Expression`, `Conditional`, `Sequence`,
`VariableAssignment`, and related nodes. Parsed objects should preserve semantic
information that later stages need. Avoid sending raw source strings into
analysis or lowering when a typed parsed node can own the concept.

`parsed/visit.rs` provides traversal helpers used by analysis and tests. Add to
the parsed model first when a feature exposes structural data; lowering should
consume typed structure instead of rediscovering syntax.

### Analysis

Analysis lives in `crates/ink-compiler/src/analysis/`.

This stage checks story-wide semantic rules before lowering:

- `names.rs`: naming collisions and duplicate definitions
- `variables.rs`: global, temp, and argument indexing
- `targets.rs`: divert and call target validation
- `flow.rs`: termination and loose-end checks
- `constants.rs`: constant collection and redefinition behavior
- `warnings.rs`: author warnings
- `target_symbols.rs` and `variable_targets.rs`: typed lookup support
- `span.rs`: span helpers for diagnostics

Analysis owns semantic diagnostics. Syntax should not know about story-wide
symbol tables, and lowering should not be the first place a semantic error is
discovered when analysis can check it.

### Lowering

Lowering lives in `crates/ink-compiler/src/lower.rs` and
`crates/ink-compiler/src/lower/`.

The lowering stage converts the checked parsed model into runtime-shaped IR in
`lower/ir.rs`. It is split by responsibility:

- `context.rs`: lowering state, container stack, and scoped state
- `indexes.rs`: story-wide target, count, variable, and constant indexes
- `labels.rs`: label index construction
- `path.rs`: runtime path types and path compaction
- `flow.rs`: story, knot, stitch, and function containers
- `weave.rs`: choices, gathers, and weave-point lowering
- `conditional.rs`: conditional lowering
- `sequence.rs`: sequence lowering
- `expression.rs`: expression and command lowering

Lowering should be deterministic and mostly diagnostic-free. If lowering needs
to know whether a target, variable, or symbol exists, prefer adding a typed
analysis/index result rather than searching strings locally.

### Emit

`crates/ink-compiler/src/emit.rs` serializes the lowering IR into the runtime
JSON story format. It should remain a narrow JSON writer. It should not parse
Ink, validate semantics, or rewrite runtime paths beyond what the lowering IR
already describes.

## Public Compiler API

The public API is intentionally small and re-exported from `ink_compiler`:

- `Compiler`, `CompilerOptions`, `StageOutput<T>`, and `CompiledStory` are the
  main entry points.
- `SourceInput`, `SourceSpan`, `FileHandler`, and `eliminate_comments` cover
  source integration.
- `Diagnostic`, `DiagnosticSeverity`, and `DiagnosticCode` are the stable
  diagnostic surface.
- `ParsedStory` and parsed node types are exported for tools that inspect
  syntax output.
- `CheckedStory`, `RuntimeProgram`, `RuntimeContainer`, `RuntimeObject`, and
  `RuntimeControlCommand` are exported because stage methods and
  `CompiledStory::program` expose them.

Internal modules such as `syntax`, `analysis`, `lower`, and `emit` stay
private. External callers should use `Compiler` stages rather than depending on
module internals.

## Runtime

The runtime layer in `crates/ink-runtime` loads and executes the JSON format.
Important modules include:

- `story/`: story execution, navigation, choices, tags, flow, state, errors,
  external functions, and variable observers
- `container.rs`, `object.rs`, `path.rs`, `pointer.rs`: runtime object graph and
  addressing
- `json/`: JSON tokenizer, reader, and writer support
- `choice.rs`, `choice_point.rs`, `divert.rs`, `control_command.rs`,
  `native_function_call.rs`, `value.rs`: runtime instruction/value types

Compiler work should preserve runtime JSON compatibility unless the language
change explicitly requires a coordinated runtime change.

## Reference Implementation

`ink-csharp/` remains the behavioral reference for upstream Ink compatibility.
Use it when legacy behavior is unclear, especially for parser trial order,
weave structure, path compaction, and JSON shape.

The Rust compiler does not need to copy C# class structure. Prefer Rust-native
ownership and module boundaries as long as behavior remains compatible for
features that are still intentionally supported.

## Feature Design Workflow

New language features should start from the Rust architecture, not from
fixture-specific fixes:

- Define the intended syntax or behavior in `docs/WritingWithInk-updates.md`.
- Apply the resulting user-facing documentation to
  `docs/WritingWithInk-latest.md`.
- Add focused language tests in `crates/ink-test/tests/language.rs` and fixtures
  under `crates/ink-test/fixtures/language/` when useful.
- Add parsed-model types before analysis or lowering needs the data.
- Put semantic checks in `analysis/` and lowering-only runtime shape decisions
  in `lower/`.
- Keep runtime changes separate unless the JSON story format or runtime
  execution behavior genuinely needs to change.

## Testing And Validation

Use the smallest relevant validation first, then widen:

```text
cargo fmt --all --check
cargo check --workspace
cargo test -p ink-compiler <focused filter>
cargo test -p ink-test --features csharp-tests --test csharp_tests -- <focused C# test>
cargo test --workspace
make gate
```

`make gate` is the project-level gate and includes the legacy C# compatibility
tests. If a compiler change intentionally changes language behavior, update the
tests, `docs/WritingWithInk-updates.md`, and
`docs/WritingWithInk-latest.md` in the same change. Do not edit
`docs/WritingWithInk-origin.md`.

## Debugging Tips

- Start with the stage that owns the behavior: source, syntax, parsed model,
  analysis, lowering, emit, or runtime.
- For syntax failures, inspect `syntax/parser.rs` rule order and the focused
  rule module before changing lowering.
- For malformed inline content or expression behavior, prefer `scan.rs` and
  `expression.rs` helpers over new one-off string splitting.
- For target or path problems, inspect analysis indexes and `lower/path.rs`
  before changing emitted JSON.
- For runtime output differences, compare the lowering IR and then the emitted
  JSON. Avoid hardcoding JSON fragments to satisfy one fixture.
- When upstream behavior is unclear, inspect `ink-csharp/compiler` first and
  record only durable findings in `AGENTS.md` working notes.
