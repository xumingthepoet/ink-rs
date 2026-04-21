# ink-rs Milestone Plan

This plan is intentionally checkpointed for repeated `继续` prompts. Work from
top to bottom. Complete one small task at a time, validate it, then update this
file and `DOCUMENTATION.md`.

Guiding principles:

- Compatibility over novelty: follow the official C# compiler behavior.
- Determinism over convenience: generated JSON and diagnostics should be stable.
- Small verified slices: each ported concept needs focused tests.
- Runtime reuse: compiler work should integrate with `bladeink`, not duplicate
  runtime internals.
- Durable memory: decisions, status, and validation live in Markdown.

## Verification Checklist

Core commands to run after every completed milestone:

- [x] `cargo fmt --all --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`

Current last verified milestone: Milestone 7 list definitions and list values
slice (`2026-04-22`).

## Rules

- Pick the first unchecked task that is not blocked.
- Keep each diff scoped to the selected task.
- Run the validation listed for the task.
- If validation fails, repair it before moving to the next task.
- Update `DOCUMENTATION.md` with completed work, decisions, and next task.
- Do not mark a task complete unless its acceptance criteria are met.
- If a bug is discovered, add a failing test that reproduces it before fixing
  it whenever feasible.
- After a complete green milestone, make a small commit with a clear milestone
  message unless the user explicitly says not to commit or there are unrelated
  user changes that should not be included.

## Milestone 0: Project Memory and Guardrails

- [x] Initialize root Git repository.
- [x] Ignore local `ink-csharp/` and `blade-ink-rs/` reference trees.
- [x] Create initial Rust workspace and compiler crate scaffold.
- [x] Add durable project-memory docs for long-horizon Codex work.

Acceptance:

- `AGENTS.md`, `PLAN.md`, `IMPLEMENT.md`, and
  `DOCUMENTATION.md` exist.
- README links to the project-memory docs.
- Ignored reference trees remain untracked.

Validation:

```sh
cargo fmt --all --check
cargo check --workspace
git status --short --ignored
```

## Milestone 1: Compiler API Contract

- [x] Define stable public API for `Compiler`, `CompilerOptions`, parse result,
  compile result, diagnostics, and file handling.
- [x] Add API-level tests for unsupported current behavior and future expected
  behavior.
- [x] Add structured diagnostics with severity, source filename, line, column,
  and message.

Acceptance:

- Public API can represent parse errors, warnings, and runtime export errors.
- Tests document current expected failures without pretending features are
  implemented.
- API names remain close to C# `Compiler.cs`.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler
cargo check --workspace
```

## Milestone 2: String Parser Foundation

- [x] Port `CharacterSet`, `CharacterRange`, and low-level character helpers.
- [x] Port `StringParserState` stack behavior.
- [x] Port core `StringParser` cursor, rule, expectation, error, whitespace,
  line, and debug metadata helpers.
- [x] Add focused unit tests against small parsing rules.

Primary C# references:

- `ink-csharp/compiler/CharacterSet.cs`
- `ink-csharp/compiler/CharacterRange.cs`
- `ink-csharp/compiler/StringParser/StringParser.cs`
- `ink-csharp/compiler/StringParser/StringParserState.cs`

Acceptance:

- Parser state supports begin, fail, cancel, succeed, and rollback semantics.
- Errors include source location.
- Unit tests cover success, failure, rollback, and line tracking.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler string_parser
cargo check --workspace
```

## Milestone 3: Parsed Hierarchy Core

- [x] Port parsed `Object` ownership, parent/path model, debug metadata, and
  traversal helpers.
- [x] Port `Identifier`, `Path`, `INamedContent`, flow levels, and base traits.
- [x] Port `ContentList`, `Text`, `AuthorWarning`, `Tag`, `Wrap`, and basic
  leaf nodes.
- [x] Add tests for tree traversal and debug metadata propagation.

Primary C# references:

- `ink-csharp/compiler/ParsedHierarchy/Object.cs`
- `ink-csharp/compiler/ParsedHierarchy/Identifier.cs`
- `ink-csharp/compiler/ParsedHierarchy/Path.cs`
- `ink-csharp/compiler/ParsedHierarchy/ContentList.cs`
- `ink-csharp/compiler/ParsedHierarchy/Text.cs`

Acceptance:

- Parsed nodes can be composed and traversed without runtime export.
- Naming and debug metadata behavior matches C# concepts.
- Rust ownership model is documented in `docs/ARCHITECTURE.md`.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler parsed
cargo check --workspace
```

## Milestone 4: Minimal Ink Parsing

- [x] Port comment elimination and whitespace handling.
- [x] Parse plain text lines into parsed text/content nodes.
- [x] Parse basic knots and stitches.
- [x] Parse simple diverts.
- [x] Add golden parser tests for minimal `.ink` snippets.

Primary C# references:

- `ink-csharp/compiler/InkParser/CommentEliminator.cs`
- `ink-csharp/compiler/InkParser/InkParser.cs`
- `ink-csharp/compiler/InkParser/InkParser_Content.cs`
- `ink-csharp/compiler/InkParser/InkParser_Knot.cs`
- `ink-csharp/compiler/InkParser/InkParser_Divert.cs`

Acceptance:

- Basic story text parses into a structured parsed story.
- Knot/stitch names and divert targets are represented.
- Unsupported syntax returns diagnostics rather than panicking.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler parser
cargo check --workspace
```

## Milestone 5: Runtime Export Skeleton

- [x] Identify required public or internal runtime JSON structures in
  `blade-ink-rs/lib`.
- [x] Implement a compiler-owned JSON export path rather than copying runtime
  internals.
- [x] Export minimal plain text story JSON.
- [x] Verify exported JSON loads with `bladeink::story::Story::new`.

Primary C# references:

- `ink-csharp/compiler/ParsedHierarchy/Story.cs`
- `ink-csharp/compiler/ParsedHierarchy/FlowBase.cs`
- `ink-csharp/compiler/ParsedHierarchy/Object.cs`
- `blade-ink-rs/lib/src/json/`

Acceptance:

- `Compiler::compile_json` works for at least one plain text story.
- `Compiler::compile` returns a `bladeink::story::Story` for that story.
- A runtime smoke test can continue the story and observe expected output.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler runtime_export
cargo test --workspace
```

## Milestone 6: Flow, Weave, Choices, and Gathers

- [x] Port `FlowBase`, `Story`, `Knot`, and `Stitch` behavior.
- [x] Port `Weave`, `Choice`, `Gather`, and weave point naming.
- [x] Implement reference resolution for paths used by flow and weave.
- [x] Add runtime tests for choices, gathers, knots, stitches, and diverts.

Primary C# references:

- `ink-csharp/compiler/ParsedHierarchy/FlowBase.cs`
- `ink-csharp/compiler/ParsedHierarchy/Story.cs`
- `ink-csharp/compiler/ParsedHierarchy/Weave.cs`
- `ink-csharp/compiler/ParsedHierarchy/Choice.cs`
- `ink-csharp/compiler/ParsedHierarchy/Gather.cs`

Acceptance:

- Representative choice and divert stories compile and run.
- Reference resolution reports useful diagnostics for invalid paths.
- Visit-count behavior has documented coverage and gaps.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler flow
cargo test -p ink-compiler choices
cargo check --workspace
```

## Milestone 7: Expressions, Variables, Lists, and Logic

- [x] Port expression parsing and expression parsed hierarchy.
- [x] Port variable declarations, assignments, references, constants, and
  externals.
- [x] Port list definitions and list values.
- [ ] Port conditionals, sequences, function calls, returns, and tunnels.
- [ ] Add feature tests for arithmetic, variables, lists, conditions, functions,
  and sequences.

Primary C# references:

- `ink-csharp/compiler/InkParser/InkParser_Expressions.cs`
- `ink-csharp/compiler/InkParser/InkParser_Logic.cs`
- `ink-csharp/compiler/ParsedHierarchy/Expression.cs`
- `ink-csharp/compiler/ParsedHierarchy/VariableAssignment.cs`
- `ink-csharp/compiler/ParsedHierarchy/ListDefinition.cs`

Acceptance:

- Core ink language constructs compile to runtime JSON.
- Runtime behavior matches official compiler output for selected examples.
- Diagnostics cover common syntax and reference errors.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler expressions
cargo test -p ink-compiler variables
cargo test -p ink-compiler lists
cargo test --workspace
```

## Milestone 8: Includes, File Handling, Plugins, and CLI Readiness

- [ ] Port include handling and file handler abstraction.
- [ ] Decide plugin support scope and document any deliberate deferral.
- [ ] Add a small CLI or example if needed for manual compilation.
- [ ] Add tests for includes and source filename diagnostics.

Primary C# references:

- `ink-csharp/compiler/FileHandler.cs`
- `ink-csharp/compiler/ParsedHierarchy/IncludedFile.cs`
- `ink-csharp/compiler/Plugins/`

Acceptance:

- Includes work through an injectable file handler.
- Unsupported plugin behavior is explicit and documented.
- Manual compilation path is documented.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler includes
cargo check --workspace
```

## Milestone 9: Conformance and Hardening

- [ ] Build a conformance harness using local examples from `blade-ink-rs` and
  official `ink-csharp/tests`.
- [ ] Compare compiler JSON or runtime behavior against trusted fixtures.
- [ ] Add regression tests for every fixed bug.
- [ ] Document remaining incompatibilities.

Acceptance:

- A documented conformance subset passes.
- Known unsupported features are listed in `DOCUMENTATION.md`.
- The project has a clear path to broaden conformance coverage.

Validation:

```sh
cargo fmt --all --check
cargo test --workspace
```

## Next Task

Port conditionals, sequences, function calls, returns, and tunnels.

## Risk Register

1. Runtime JSON compatibility:
   - Risk: compiler output may not match the JSON shape expected by `bladeink`.
   - Mitigation: inspect `blade-ink-rs/lib/src/json` before export work, build
     small runtime smoke tests early, and normalize JSON golden tests.
2. C# inheritance to Rust ownership:
   - Risk: parent pointers, lazy `runtimeObject`, and virtual methods may be
     difficult to represent cleanly.
   - Mitigation: port parsed hierarchy core before grammar breadth, document
     ownership decisions, and test traversal/reference behavior in isolation.
3. Parser rollback correctness:
   - Risk: small differences in `StringParserState` rollback can cascade into
     incorrect grammar behavior.
   - Mitigation: port parser foundation first with focused state/rollback tests.
4. Reference resolution:
   - Risk: paths, weave points, functions, variables, and includes can fail late
     if name resolution is incomplete.
   - Mitigation: implement diagnostics and resolution tests alongside each
     feature rather than deferring all resolution work.
5. Scope creep during porting:
   - Risk: translating too many files at once creates unreviewable diffs and
     hidden regressions.
   - Mitigation: use milestone-sized patches, update docs every turn, and commit
     only green checkpoints.
6. Oracle drift:
   - Risk: tests may accidentally encode the Rust port's behavior rather than
     official compiler behavior.
   - Mitigation: compare against `ink-csharp` output or trusted fixtures for
     conformance milestones.

## Demo and Validation Script

This is the eventual developer demo for the completed compiler:

1. Build and test:
   - Run `cargo fmt --all --check`.
   - Run `cargo test --workspace`.
2. Compile a minimal story:
   - Input: plain text `.ink`.
   - Output: runtime JSON.
   - Load JSON with `bladeink::story::Story::new`.
   - Continue the story and show expected text output.
3. Compile a flow story:
   - Include knots, stitches, diverts, choices, and gathers.
   - Run through a choice path in `bladeink`.
4. Compile a logic story:
   - Include variables, expressions, conditions, and functions.
   - Verify runtime output and variable behavior.
5. Run conformance subset:
   - Use selected fixtures from `blade-ink-rs/conformance-tests/inkfiles`.
   - Document unsupported cases in `DOCUMENTATION.md`.

## Architecture Overview

The compiler pipeline is:

```text
.ink source
  -> StringParser
  -> InkParser
  -> parsed hierarchy
  -> reference resolution
  -> runtime JSON export
  -> bladeink::story::Story
```

Core module boundaries:

- `compiler`: public orchestration API and options.
- `parser::string_parser`: low-level rule stack, cursor, rollback, errors.
- `parser::ink_parser`: ink grammar translated from C# partial parser files.
- `parsed`: parsed hierarchy and runtime export behavior.
- `error`: diagnostics and compiler errors.
- tests: parser, parsed hierarchy, JSON export, runtime smoke, conformance.

Key determinism points:

- Diagnostics should be emitted in stable source order.
- Generated runtime JSON should use stable ordering.
- Golden tests should compare normalized JSON when object key ordering is not
  semantically important.
- Reference resolution should avoid hash-map iteration order in user-visible
  output.

## Implementation Notes

- Milestone 0:
  - Root workspace and `ink-compiler` crate were initialized.
  - Local reference trees are ignored but required for development.
  - Durable project-memory docs were added after reading the OpenAI Codex
    long-horizon task article and its linked example Markdown files.
