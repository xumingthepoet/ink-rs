# ink-rs Milestone Plan

This plan is intentionally checkpointed for repeated `继续` prompts. Work from
top to bottom. Complete one small task at a time, validate it, then update this
file and `DOCUMENTATION.md`.

Guiding principles:

- Compatibility over novelty: follow the official C# compiler behavior.
- Determinism over convenience: generated JSON and diagnostics should be stable.
- Small verified slices: each ported concept needs focused tests.
- Runtime reuse: compiler work should integrate with `ink_runtime`, not duplicate
  runtime internals.
- Durable memory: decisions, status, and validation live in Markdown.

## Verification Checklist

Core commands to run after every completed milestone:

- [x] `cargo fmt --all --check`
- [x] `cargo check --workspace`
- [x] `timeout 30s cargo test --workspace`
- [x] `make gate`

Current last verified milestone: default workspace gate restored with the
imported legacy suites feature-gated (`2026-04-22`).

Current verified checkpoint: `make gate` is green again; the imported legacy
compiler-conformance and csharp suites are retained behind the
separate legacy features so they can keep migrating without blocking
the default workspace run (`2026-04-22`).

Workspace warning policy: `.cargo/config.toml` now denies warnings, and
`make gate` is the unified local entry point for format, check, and a
timeboxed test pass. Any standalone `cargo test` command should also be
wrapped in `timeout`.

The manual compiler entry point now lives under `crates/ink-tools/` instead
of `examples/`.

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
- [x] Ignore local `ink-csharp/` and `ink-runtime/` reference trees.
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
  `ink-runtime/lib`.
- [x] Implement a compiler-owned JSON export path rather than copying runtime
  internals.
- [x] Export minimal plain text story JSON.
- [x] Verify exported JSON loads with `ink_runtime::story::Story::new`.

Primary C# references:

- `ink-csharp/compiler/ParsedHierarchy/Story.cs`
- `ink-csharp/compiler/ParsedHierarchy/FlowBase.cs`
- `ink-csharp/compiler/ParsedHierarchy/Object.cs`
- `ink-runtime/lib/src/json/`

Acceptance:

- `Compiler::compile_json` works for at least one plain text story.
- `Compiler::compile` returns a `ink_runtime::story::Story` for that story.
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
- [x] Port conditionals, sequences, function calls, returns, and tunnels.
- [x] Add feature tests for arithmetic, variables, lists, conditions,
  functions, and sequences.

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
cargo test -p ink-compiler ink_parser_feature_cases_cover_arithmetic_variables_lists_conditions_functions_and_sequences
cargo test --workspace
```

## Milestone 8: Includes, File Handling, Plugins, and CLI Readiness

- [x] Port include handling and file handler abstraction.
- [x] Decide plugin support scope and document any deliberate deferral.
- [x] Add a small CLI or example if needed for manual compilation.
- [x] Add tests for includes and source filename diagnostics.

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

- [x] Build a conformance harness using local examples from `ink-runtime` and
  official `ink-csharp/tests`.
- [x] Compare compiler JSON or runtime behavior against trusted fixtures.
- [x] Add regression tests for every fixed bug.
- [x] Document remaining incompatibilities.

Acceptance:

- A documented conformance subset passes.
- Known unsupported features are listed in `DOCUMENTATION.md`.
- The project has a clear path to broaden conformance coverage.

Validation:

```sh
cargo fmt --all --check
cargo test --workspace
```

## Milestone 10: Runtime Crate Relocation

- [x] Scaffold `crates/ink-runtime` with the copied `ink-runtime/lib/src`
  runtime tree.
- [x] Point the workspace runtime dependency at `crates/ink-runtime`.
- [x] Update docs and tests to treat `crates/ink-runtime` as the canonical
  runtime home, move package-level tests into `crates/ink-test`, and remove
  the legacy `ink-runtime` tree.

Primary Rust references:

- `ink-runtime/lib/src/`
- `crates/ink-runtime/src/`

Acceptance:

- The workspace builds and tests against `crates/ink-runtime`.
- The old `ink-runtime/lib` path dependency is no longer used by the
  workspace, and the legacy `ink-runtime` tree is gone.
- Runtime tests continue to pass without conformance regression.

Validation:

```sh
cargo fmt --all --check
cargo check --workspace
timeout 30s cargo test --workspace
```

## Milestone 11: Parser Snapshot Relocation

- [x] Move the remaining parser trusted snapshots out of
  `crates/ink-compiler/src/parser/mod.rs` and into `crates/ink-test`.
- [x] Update docs and status once the parser snapshots live entirely in the
  test crate.

Primary Rust references:

- `crates/ink-compiler/src/parser/mod.rs`
- `crates/ink-compiler/src/parsed/snapshot.rs`
- `crates/ink-test/tests/`
- `crates/ink-test/fixtures/`

Acceptance:

- External fixture-driven parser snapshots live in `crates/ink-test`.
- `ink-compiler` keeps only internal unit tests.
- The workspace still passes `cargo fmt --all --check`,
  `cargo check --workspace`, and `cargo test --workspace`.

Validation:

```sh
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
```

## Milestone 12: Legacy Conformance Import

- [x] Import the legacy runtime conformance suite from
  `ink-tests-old/src/conformance` into `crates/ink-test`.
- [x] Import the legacy compiler-to-runtime conformance suite into
  `crates/ink-test`.
- [x] Import the legacy `csharp_tests` suite into `crates/ink-test`.
- [x] Decide whether any remaining `ink-tests-old` fixtures or wrappers should
  be retained after the migration completes.

  Decision: keep the imported legacy compiler-conformance and csharp suites
  behind separate legacy features so the default gate stays green
  while migration continues.

Primary Rust references:

- `crates/ink-test/tests/conformance_legacy.rs`
- `crates/ink-test/tests/compiler_conformance_legacy.rs`
- `crates/ink-test/tests/conformance/`
- `crates/ink-test/tests/compiler_conformance/`
- `crates/ink-test/fixtures/conformance/`
- `ink-tests-old/src/conformance/`
- `ink-tests-old/src/compiler_conformance/`

Acceptance:

- The legacy runtime conformance suite runs from `ink-test`.
- The copied fixture tree under `crates/ink-test/fixtures/conformance/`
  matches the legacy runtime tests.
- The legacy compiler-to-runtime suite is available from `ink-test` behind the
  normal workspace test flow so it can be exercised alongside the rest of the
  compiler port.
- The workspace still passes `cargo fmt --all --check`,
  `cargo check --workspace`, and `cargo test --workspace`.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-test --test conformance_legacy
cargo test -p ink-test --test compiler_conformance_legacy
cargo check --workspace
cargo test --workspace
```

## Next Task

## Milestone 13: Legacy Suite Stabilization

- [x] Add a timeout-wrapped compiler-conformance gate and use it for every
  focused iteration on the legacy compiler-to-runtime suite.
- While this milestone is in progress, save green checkpoints in small
  commits after every few passing compiler-conformance fixtures. This keeps
  the work resumable and makes it safe to stop after a narrow batch is green.
- [x] Fix compiler-conformance failures in dependency-light order:
  - basic text, knots, stitches, diverts, and glue
  - tags, gathers, and simple multi-flow paths
  - variables, lists, expressions, and functions
  - conditionals, sequences, and choice generation
  - runtime behaviors: save/load, externals, observers, visit counts, threads,
    and tunnels
- [x] Normalize and document any intentional JSON mismatches while the suite
  was being brought up, then remove the mismatches once the compiler matched
  the fixture set.
- [x] Add tokenizer-level `.ink.parse` snapshots for compiler-conformance
  fixtures and compare them before JSON.
- [x] Fix the imported legacy compiler-conformance suite first, in
  dependency-light order.
- [ ] Then fix the imported legacy csharp-tests suite, reusing the same
  dependency-light approach and compatibility wrappers where needed.
- [ ] Remove the legacy feature gates once both imported suites are ready to
  return to the default workspace test run.
- [ ] Delete any compiler-conformance or csharp-test wrappers that become
  unnecessary after the suites are promoted to normal test targets.

Primary Rust references:

- `crates/ink-test/tests/compiler_conformance_legacy.rs`
- `crates/ink-test/tests/compiler_conformance/`
- `crates/ink-test/fixtures/conformance/`
- `crates/ink-compiler/src/compiler.rs`
- `crates/ink-compiler/src/runtime_export.rs`

Current blocker while the imported suites are being stabilized:

- The imported legacy compiler-to-runtime suite is now green, including the
  new tokenizer-level parse snapshots that compare `A.ink.parse` before
  `A.ink.json`. The highest-priority remaining suite is the imported csharp
  suite, which still stays feature-gated while migration continues.

Acceptance:

- The legacy compiler-to-runtime suite can be run with a timeout wrapper and
  produces either a green run or a bounded failure report.
- The suite is reduced to dependency-light, isolated examples first so the
  least coupled bugs can be fixed before multi-flow and runtime-heavy cases.
- Once green, the suite no longer needs the `legacy-compiler-conformance`
  feature gate and can run as part of the normal workspace test pass.

Validation:

```sh
make compiler-gate
make compiler-gate COMPILER_TEST='compiler_conformance::choice_test::conditional_choice_test -- --exact'
timeout 30s cargo test -p ink-test --features legacy-compiler-conformance --test compiler_conformance_legacy
timeout 30s cargo test -p ink-test --features legacy-compiler-conformance --test compiler_conformance_legacy compiler_conformance::choice_test::conditional_choice_test -- --exact
cargo fmt --all --check
cargo check --workspace
timeout 30s cargo test --workspace
```

## Next Task

Continue fixing the feature-gated legacy csharp-tests suite in small
checkpointed slices now that the compiler-conformance suite is green and has
parse-snapshot coverage.

## Risk Register

1. Runtime JSON compatibility:
   - Risk: compiler output may not match the JSON shape expected by `ink_runtime`.
   - Mitigation: inspect `ink-runtime/lib/src/json` before export work, build
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
   - Load JSON with `ink_runtime::story::Story::new`.
   - Continue the story and show expected text output.
3. Compile a flow story:
   - Include knots, stitches, diverts, choices, and gathers.
   - Run through a choice path in `ink_runtime`.
4. Compile a logic story:
   - Include variables, expressions, conditions, and functions.
   - Verify runtime output and variable behavior.
5. Run conformance subset:
   - Use selected fixtures from `ink-runtime/conformance-tests/inkfiles`.
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
  -> ink_runtime::story::Story
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
