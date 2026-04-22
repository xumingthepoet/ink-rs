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
- `CharacterSet`, `CharacterRange`, and basic string parser character helpers
  are now ported.
- `StringParserState` stack behavior is now ported.
- Core `StringParser` cursor/rule helpers are now ported.
- Parsed hierarchy `Object` ownership, parent links, debug metadata
  inheritance, path primitives, and traversal helpers are now ported.
- Parsed hierarchy flow levels and base traits are now ported.
- Parsed hierarchy knot and stitch wrappers are now ported.
- Parsed hierarchy weave-point wrappers (`Weave`, `Choice`, and `Gather`)
  plus local weave-point naming lookup are now ported.
- Parsed hierarchy path resolution can now resolve flow names, nested stitches,
  and weave points from a parsed context.
- Parsed hierarchy expression wrappers (`Number`, `StringExpression`,
  `VariableReference`, `FunctionCall`, `DivertTarget`, `List`,
  `BinaryExpression`, `UnaryExpression`, `IncDecExpression`, and
  `MultipleConditionExpression`) are now ported.
- Parsed hierarchy variable statement wrappers (`VariableAssignment`,
  `ConstantDeclaration`, and `ExternalDeclaration`) are now ported.
- Parsed hierarchy list-definition wrappers (`ListDefinition` and
  `ListElementDefinition`) are now ported.
- Parsed hierarchy content nodes (`ContentList`, `Text`, `AuthorWarning`,
  `Tag`, and generic `Wrap<T>`) are now ported.
- Ink parser comment elimination and whitespace helpers are now ported.
- Ink parser can now parse plain text lines into structured parsed content
  nodes and rejects obvious unported structural syntax with diagnostics.
- Ink parser now recognizes basic knot and stitch headers and attaches their
  following content lines to `Knot` and `Stitch` flow nodes.
- Ink parser now also recognizes simple divert lines and models them as parsed
  divert nodes.
- Ink parser now also recognizes tunnel divert lines (`->->`) and models them
  as parsed tunnel diverts.
- Ink parser now also recognizes simple inline sequence lines such as
  `once: a | b` and models them as parsed sequence nodes.
- Ink parser now also recognizes simple inline conditional lines such as
  `{ x > 3: yes | no }` and models them as parsed conditional nodes.
- Ink parser now also recognizes brace-based multiline conditional and
  sequence blocks, including switch-style branch expressions.
- Ink parser now has golden parser tests for minimal plain-text, knot, and
  divert snippets, including tunnel diverts.
- Expression parser foundation is now ported with Pratt-style precedence
  parsing for literals, variable paths, function calls, lists, divert targets,
  unary operators, and binary operators.
- Ink parser now also recognizes `VAR`, `CONST`, `EXTERNAL`, and `~` logic
  lines for variable statements, temporary assignments, constants, and
  variable references.
- Ink parser now also recognizes `LIST` declarations and models them as parsed
  list-definition nodes.
- Ink parser now also recognizes `~ return` logic lines and models them as
  parsed return nodes.
- Runtime story behavior tests now cover choice selection, named knot/stitch
  and gather-like container paths, and explicit diverts against
  `bladeink::story::Story`.
- Runtime JSON shape for `bladeink::story::Story::new` has been identified
  with a minimal loading fixture.
- Compiler-owned runtime export now emits minimal plain-text story JSON plus
  `listDefs` metadata for parsed list declarations and loads successfully
  through `bladeink::story::Story::new`.
- Ink parser grammar is still pending, but Milestone 6 is now complete with
  flow base, knot/stitch behavior, weave-point wrappers, path resolution, and
  runtime behavior coverage.
- Plugin support is intentionally deferred for this Rust compiler port. The
  compiler crate does not implement dynamic plugin discovery or PreParse /
  PostParse / PostExport hooks, and that scope is documented as out of band
  until there is a concrete compatibility need.
- A minimal `ink_compile` example now provides a manual compile path for `.ink`
  files, rooted to the source file directory so relative includes resolve in
  the expected location.
- A conformance harness now compares compiler output against trusted local
  fixtures from `blade-ink-rs/conformance-tests` and compiler behavior against
  the official include examples from `ink-csharp/tests`.
- `CommentEliminator` now strips a leading UTF-8 BOM, and compiler-level
  regression tests cover both direct-source and include-file BOM handling.
- Remaining compiler/runtime incompatibilities are now documented in the
  Known Issues and Remaining Incompatibilities sections below.
- Long-horizon project memory docs now exist.

## Current Milestone

Milestone 1 is complete. Milestone 2 is complete. Milestone 3 is complete.
Milestone 4 is complete. Milestone 5 is complete. Milestone 6 is complete.
Milestone 7 task list is complete, and the expression, variable-statement,
list-definition, return-statement, tunnel-divert, inline-sequence,
inline-conditional, brace multiline conditional/sequence, and feature-test
slices are complete.
Milestone 8 is complete, including include handling, plugin-scope
documentation, and the minimal manual compilation example.
Milestone 9 is complete: the conformance harness, regression tests, and
remaining incompatibility notes are all documented.

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

For manual compilation, run:

```sh
cargo run -p ink-compiler --example ink_compile -- path/to/story.ink
```

Pass a second argument to write the JSON to a file instead of stdout.

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

- `Compiler::compile_json` now exports minimal plain-text story JSON, but the
  full compiler output path is still limited to plain-text stories.
- The parsed hierarchy currently has the object tree, path primitives, and a
  root-story wrapper with content-node leaf wrappers.
- Parsed `Weave`, `Choice`, and `Gather` wrappers now exist with indentation
  grouping and local weave-point naming lookup, but the parser/runtime wiring
  for choice stories is still pending.
- `InkParser::new` now stores an owned, comment-stripped input string so the
  preprocessor can normalize line endings before any grammar work begins.
- `InkParser::parse` currently accepts plain text stories and returns a parsed
  content tree, but the full grammar is still pending beyond the current plain
  text, knot/stitch, simple divert, and variable-statement slices.
- `InkParser::parse` now also recognizes basic knot and stitch declarations by
  line prefix and models them as parsed `Knot` and `Stitch` nodes.
- `InkParser::parse` now also recognizes simple divert lines and models them as
  parsed divert nodes, but the full divert grammar is still pending.
- `InkParser::parse` now also recognizes tunnel divert lines, but broader
  tunnel semantics and runtime export behavior remain limited.
- `InkParser::parse` now also recognizes simple inline sequence lines, but the
  broader multiline sequence grammar and runtime export behavior remain
  limited.
- `InkParser::parse` now also recognizes simple inline conditional lines, but
  the broader multiline conditional grammar and runtime export behavior remain
  limited.
- `InkParser::parse` now also recognizes brace-based multiline sequence and
  conditional blocks, including switch-style branch expressions, but nested
  brace logic and runtime export behavior remain limited.
- Parser feature coverage now includes arithmetic, variables, lists,
  conditions, functions, and sequences in a single stable snapshot test.
- Expression parsing currently covers a focused foundation subset: numeric and
  boolean literals, string expressions, variable paths, function calls, list
  expressions, divert targets, unary operators, and binary precedence.
- `InkParser::parse` now also recognizes `VAR`, `CONST`, `EXTERNAL`, and `~`
  logic lines for variable declarations, constants, externals, temporary
  assignments, and variable-reference expressions.
- `InkParser::parse` now also recognizes `LIST` declarations and models them as
  parsed list-definition nodes.
- The runtime JSON reader expects top-level `inkVersion`, `root`, and
  `listDefs` keys; the `root` value is a container array whose trailing entry
  is either named-content metadata or `null`.
- `Compiler::compile_json` now exports minimal plain-text story JSON plus
  `listDefs` metadata for parsed list declarations, and `Compiler::compile`
  can load that JSON through `bladeink`.
- `CompilerOptions` now accepts an injectable file handler, and `InkParser`
  now resolves `INCLUDE` lines through that abstraction with recursive include
  tracking and source-file diagnostics.
- Parsed list declarations populate runtime `listDefs`, but broader list-driven
  runtime semantics are still limited.
- Parsed return nodes are parsed, but compiler-side runtime export for
  function-return semantics is still pending.
- Parsed inline and brace multiline conditional/sequence nodes are parsed, but
  nested brace logic and runtime export behavior are still limited.
- The ink parser grammar is still pending for the remaining unported features.
- Feature coverage for arithmetic, variables, lists, conditions, functions, and
  sequences now exists as a stable parser snapshot test.
- The C# `Glue` and `LegacyTag` `Wrap<T>` aliases are not yet ported as
  dedicated compiler-side wrappers because the corresponding runtime modules in
  `blade-ink-rs` are private; the generic `Wrap<T>` helper is in place for
  future use.
- The `ink_compile` example is intentionally minimal and prints JSON to stdout
  or writes it to an optional output path; richer CLI ergonomics remain out of
  scope.
- The conformance harness currently covers a minimal plain-text baseline and a
  C# include story; broader conformance coverage is still pending.
- The leading UTF-8 BOM stripping fix is now covered by parser and API
  contract regression tests.
- `cargo check --workspace` reports warnings from ignored dependency
  `blade-ink-rs/lib`; these are upstream/local reference warnings, not current
  compiler crate failures.

## Remaining Incompatibilities

- `Compiler::compile_json` still emits only the minimal runtime JSON shape that
  the current `bladeink::story::Story::new` loader accepts, plus `listDefs`
  metadata for parsed list declarations.
- The parser and parsed hierarchy still only cover the ported slices listed in
  the current status above; the rest of the official C# grammar remains
  unported.
- Nested brace logic, full tunnel semantics, and broader runtime export
  behavior for conditionals and sequences remain limited to the ported slices.
- Plugin hooks and dynamic plugin discovery remain intentionally deferred.
- The conformance harness currently covers a small trusted subset, not the
  full official test suite.

## Audit Log

### 2026-04-22

- Completed the Milestone 9 documentation slice.
- Documented the remaining compiler/runtime incompatibilities so the current
  port boundary is explicit in the project memory.

Validation:

```sh
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
```

Result: all passed. The only remaining warnings are the two existing
`blade-ink-rs/lib/src/story_state.rs` parentheses warnings.

### 2026-04-22

- Completed the Milestone 9 regression-test slice.
- Added compiler-level regression tests that verify a leading UTF-8 BOM is
  stripped from both direct source text and included files before compilation.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler --test api_contract
cargo check --workspace
cargo test --workspace
```

Result: all passed. The only remaining warnings are the two existing
`blade-ink-rs/lib/src/story_state.rs` parentheses warnings.

### 2026-04-22

- Completed the Milestone 9 conformance harness slice.
- Added a reusable conformance harness that compares compiler output against
  trusted `blade-ink-rs` fixtures and compiles an official C# include example
  through the Rust compiler and runtime.
- Fixed BOM handling in the comment eliminator so included files from the C#
  reference set no longer preserve a leading BOM in text output.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler --test conformance_harness
cargo check --workspace
cargo test --workspace
```

Result: all passed. The only remaining warnings are the two existing
`blade-ink-rs/lib/src/story_state.rs` parentheses warnings.

### 2026-04-22

- Completed the Milestone 8 include-handling slice.
- Added parser include expansion through the compiler-owned file handler
  abstraction, including recursive include tracking and source filename
  propagation for included-file diagnostics.
- Added API contract tests covering include expansion through a custom file
  handler, file-handler call counts, and included-file diagnostic filenames.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler includes
cargo check --workspace
```

Result: all passed. The only remaining warnings are the two existing
`blade-ink-rs/lib/src/story_state.rs` parentheses warnings.

### 2026-04-22

- Completed the Milestone 8 manual compilation slice.
- Added a minimal `ink_compile` example and a smoke test that compiles a story
  with a relative include through the CLI path.
- Documented the manual compilation command in the project memory.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler cli_compiles_a_story_with_relative_includes
cargo check --workspace
cargo test --workspace
```

Result: all passed. The only remaining warnings are the two existing
`blade-ink-rs/lib/src/story_state.rs` parentheses warnings.

### 2026-04-22

- Completed the Milestone 8 plugin-scope decision slice.
- Documented the plugin-porting scope as intentionally deferred for now: the
  Rust compiler crate will not implement dynamic plugin loading or plugin
  hooks until a concrete compatibility need appears.

Validation:

```sh
cargo fmt --all --check
cargo check --workspace
```

Result: all passed. The only remaining warnings are the two existing
`blade-ink-rs/lib/src/story_state.rs` parentheses warnings.

### 2026-04-22

- Completed the Milestone 7 feature-tests slice.
- Added a parser feature snapshot test covering arithmetic, variables, lists,
  conditions, functions, and sequences in one compact fixture set.
- Confirmed the feature snapshot against the current parser tree rendering.

Validation:

```sh
cargo fmt --all
cargo test -p ink-compiler ink_parser_feature_cases_cover_arithmetic_variables_lists_conditions_functions_and_sequences
cargo test -p ink-compiler parser
cargo check --workspace
cargo test --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Continued Milestone 7 with the brace multiline conditional and sequence
  parsing slice.
- Added brace-block parsing for multiline sequences such as `{once: ... }` and
  multiline conditionals such as `{ x == 4: ... }`.
- Added support for switch-style conditional branches with own expressions in
  the parsed hierarchy.
- Added parser and golden tests covering multiline conditionals and sequences.

Validation:

```sh
cargo fmt --all
cargo test -p ink-compiler parser
cargo check --workspace
cargo test --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

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
- Ported `CharacterSet` and `CharacterRange` into `parser::character_set` and
  `parser::character_range`, plus basic `string_parser` character classification
  helpers.
- Added focused tests covering character set mutation, range caching, and the
  low-level character classification helpers.
- Ported `StringParserState` into `parser::string_parser::state` with push,
  pop, peek, squash, and error-scope behavior.
- Added unit tests covering stack initialization, push/pop/squash semantics,
  error scope marking, and mismatched rule ID panics.
- Ported the core `StringParser` cursor, rule, expectation, error, whitespace,
  line, and debug metadata helpers into `parser::string_parser`.
- Added unit tests covering cursor movement, newline tracking, character set
  parsing, rule rollback/commit behavior, diagnostics, and debug metadata.
- Ported parsed hierarchy ownership into reference-counted tree nodes with weak
  parent links, inherited debug metadata, depth-first traversal helpers, and
  basic `Identifier`/`Path` types.
- Reworked `parsed::Story` into a root object wrapper so top-level content uses
  the same ancestry model as C#.
- Added tests covering parent links, debug metadata inheritance, ancestry
  ordering, traversal order, and path formatting.
- Added parsed hierarchy flow primitives: `FlowLevel`, `NamedContent`,
  `FlowBase`, and `FlowArgument`.
- Reworked `Path` to carry base flow level metadata and formatting for weave
  point paths.
- Made `Story` implement the flow traits so it participates in the base flow
  model.
- Added tests for flow trait defaults, flow level ordering, and story-level
  trait behavior.
- Added parsed hierarchy content-node wrappers: `ContentList`, `Text`,
  `AuthorWarning`, `Tag`, and generic `Wrap<T>`.
- Added tests covering trailing-whitespace trimming, tag formatting,
  payload storage, wrapper passthrough, and parent-link preservation.
- Added a Rust `CommentEliminator` preprocessor that strips `//` and `/* */`
  comments while preserving line counts.
- Added whitespace helper functions that mirror the C# `InkParser_Whitespace`
  rules for newline, end-of-file, and spacing combinators.
- Made `InkParser::new` run the comment eliminator before storing the input
  string.
- Added tests for comment stripping, line-ending normalization, whitespace
  helpers, and the preprocessed parser input.
- Added a plain-text parser slice that turns lines into `ContentList` and
  `Text` parsed nodes and emits a diagnostic for obvious unported structure.
- Added parser tests covering plain-text story parsing and syntax rejection.
- Added a knot/stitch parser slice that creates parsed flow nodes for basic
  `== knot ==` and `= stitch` lines, with their body lines attached as nested
  content.
- Added parser tests covering basic knot/stitch parsing.
- Added a simple divert parser slice that recognizes `->` lines, empty
  diverts, and basic knot/stitch divert targets.
- Added parser tests covering simple divert parsing and tunnel-divert
  recognition.
- Added a sequence parser slice that recognizes simple inline `once: a | b`
  style lines and models them as parsed `Sequence` nodes.
- Added parser and parsed-hierarchy tests covering inline sequence parsing and
  rendering.
- Added an inline conditional parser slice that recognizes simple brace-based
  conditionals such as `{ x > 3: yes | no }` and models them as parsed
  `Conditional` nodes.
- Added parser and parsed-hierarchy tests covering inline conditional parsing
  and rendering.
- Added golden parser tests that snapshot minimal plain-text, knot, and
  simple-divert parsed trees as stable textual dumps.
- Added a runtime JSON smoke test that proves the minimal compiled story shape
  loads through `bladeink::story::Story::new`.
- Added a compiler-owned runtime export path that serializes minimal
  plain-text stories to runtime JSON and returns a runtime `Story`.
- Added compiler and API contract tests covering JSON export success,
  runtime loading, and rejection of non-text nodes.

Validation:

```sh
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted two existing warnings about
unnecessary parentheses around trait object types.

### 2026-04-22

- Continued Milestone 2 with the string parser foundation slice.
- Ported `CharacterSet` and `CharacterRange` as dedicated parser modules.
- Added basic string character helper functions for newline, whitespace,
  digits, letters, and identifier characters.
- Added unit tests for set mutation, range exclusion/cache behavior, and the
  helper classification functions.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler string_parser
cargo check --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Started the Milestone 7 return-statement slice.
- Added a parsed hierarchy `Return` wrapper and `~ return` parser support.
- Kept runtime export conservative by rejecting return nodes for now.
- Added parser and parsed-hierarchy tests for the new return node.

### 2026-04-22

- Continued Milestone 3 with the parsed hierarchy object slice.
- Reworked `parsed::Object` into `Rc<RefCell<_>>`-backed tree nodes with weak
  parent links and inherited debug metadata.
- Added `parsed::Identifier` and `parsed::Path` primitives.
- Reworked `parsed::Story` to own a root object that adopts top-level content.
- Added traversal helpers for depth-first search and ancestry collection.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler parsed
cargo check --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Continued Milestone 4 with the basic knot and stitch parsing slice.
- Added line-driven recognition for basic knot and stitch headers.
- Flow headers now become parsed flow nodes with nested body content lines.
- Added parser tests covering knot/stitch parsing alongside the existing
  plain-text and preprocessing coverage.

### 2026-04-22

- Continued Milestone 4 with the simple divert parsing slice.
- Added line-driven recognition for simple `->` divert lines.
- Diverts now parse into dedicated parsed `Divert` nodes, including empty
  diverts and basic knot/stitch targets.
- Added parser tests covering simple divert parsing and tunnel-divert
  recognition.

### 2026-04-22

- Continued Milestone 4 with the tunnel divert slice.
- Added parser support for `->->` tunnel diverts by reusing the parsed
  `Divert` wrapper with the tunnel flag set.
- Added parser and golden tests covering tunnel divert parsing.

### 2026-04-22

- Continued Milestone 7 with the inline sequence parsing slice.
- Added a parsed hierarchy `Sequence` wrapper and `SequenceType` enum.
- Added parser support for simple inline sequence lines such as `once: a | b`.
- Added parser and parsed-hierarchy tests covering inline sequence parsing.

### 2026-04-22

- Continued Milestone 7 with the inline conditional parsing slice.
- Added parsed hierarchy `Conditional` and `ConditionalSingleBranch`
  wrappers.
- Added parser support for simple inline brace conditionals such as
  `{ x > 3: yes | no }`.
- Added parser and parsed-hierarchy tests covering inline conditional parsing.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler tunnel_divert_marks_tunnel_flag
cargo test -p ink-compiler ink_parser_parses_tunnel_diverts
cargo test -p ink-compiler ink_parser_golden_cases_for_minimal_snippets
cargo test -p ink-compiler sequence
cargo test -p ink-compiler ink_parser_parses_sequences
cargo test -p ink-compiler conditional
cargo test -p ink-compiler ink_parser_parses_inline_conditionals
cargo check --workspace
cargo test --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Completed Milestone 4 with golden parser tests for minimal ink snippets.
- Added a data-driven golden parser test harness that renders parsed trees to
  stable textual snapshots.
- Covered plain-text, knot, simple-divert, and tunnel-divert snippets with
  golden expectations.

### 2026-04-22

- Started Milestone 5 with runtime JSON shape identification.
- Confirmed that `bladeink::story::Story::new` accepts a minimal JSON fixture
  with `inkVersion`, `root`, and `listDefs`.
- Recorded the top-level JSON contract and container terminator shape in the
  live documentation.

### 2026-04-22

- Completed Milestone 5 with a compiler-owned runtime export skeleton.
- Added a JSON exporter that turns plain-text parsed stories into runtime JSON
  without copying runtime internals.
- Verified that exported JSON loads through `bladeink::story::Story::new` and
  that `compile()` returns a runtime story for a plain-text input.

### 2026-04-22

- Continued Milestone 6 with the flow/weave path-resolution slice.
- Added `Path::resolve_from_context` and supporting search helpers for flow
  names, nested stitches, and weave points.
- Added path-resolution tests that resolve knot, stitch, and weave-point
  targets from a parsed tree.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler parsed::path
cargo check -p ink-compiler
cargo check --workspace
cargo test --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Completed Milestone 6 with runtime tests for choices, gathers, knots,
  stitches, and diverts.
- Added integration tests against `bladeink::story::Story` covering a choice
  fixture, nested knot/stitch and gather-like named container paths, and
  explicit diverts.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler --test runtime_story_behaviour
cargo check --workspace
cargo test --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Started Milestone 7 with the expression parsing and expression parsed
  hierarchy slice.
- Added expression wrappers for numbers, strings, variable references,
  function calls, divert targets, list expressions, binary and unary
  operators, increment/decrement, and multiple conditions.
- Added a standalone Pratt-style expression parser with tests for literals,
  variable paths, function calls, list expressions, divert targets, unary
  operators, and binary precedence.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler expression
cargo check --workspace
cargo test --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Continued Milestone 7 with the variable statement slice.
- Added parsed hierarchy wrappers for `VariableAssignment`,
  `ConstantDeclaration`, and `ExternalDeclaration`.
- Added line-driven parser support for `VAR`, `CONST`, `EXTERNAL`, and `~`
  logic statements with expression parsing for assignments and references.
- Added parser tests covering variable declarations, constants, externals,
  temporary assignments, increment/decrement assignments, and plain variable
  reference logic lines.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler variables
cargo check --workspace
cargo test --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Continued Milestone 6 with the weave-point parsed hierarchy slice.
- Added parsed `Weave`, `Choice`, and `Gather` wrappers plus a local naming
  table for weave points.
- Added indentation-based weave grouping and duplicate-label diagnostics for
  the parsed weave tree.
- Added tests covering weave indentation, weave-point naming, and choice /
  gather metadata accessors.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler parsed
cargo check --workspace
cargo test --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Continued Milestone 6 with the flow base and knot/stitch slice.
- Added dedicated `Knot` and `Stitch` parsed hierarchy wrappers built on the
  existing flow traits.
- Updated the parser to construct named knot/stitch flow nodes for line-based
  headers.
- Added tests covering knot/stitch flow metadata and parser recognition.

Validation:

```sh
cargo fmt --all --check
cargo test --workspace
cargo check --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Continued Milestone 4 with the plain-text parsing slice.
- Added a parser path that turns plain text lines into `ContentList` and `Text`
  parsed nodes.
- Added a structured diagnostic for obvious unported structural syntax such as
  knots and diverts.
- Added parser tests covering plain-text story parsing and syntax rejection.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler parser
cargo check --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Continued Milestone 4 with comment elimination and whitespace handling.
- Added a Rust `CommentEliminator` preprocessor for `//` and `/* */`
  comments, preserving line counts and normalizing line endings.
- Added whitespace helper functions mirroring `InkParser_Whitespace`:
  `newline`, `end_of_file`, `end_of_line`, `whitespace`,
  `multiline_whitespace`, `any_whitespace`, `spaced`, and `multi_spaced`.
- Made `InkParser::new` store an owned, comment-stripped input string.
- Added tests for comment stripping, whitespace helpers, and the parser input
  preprocessing step.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler parser
cargo check --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Continued Milestone 3 with the parsed hierarchy content-node slice.
- Added parsed hierarchy content-node wrappers: `ContentList`, `Text`,
  `AuthorWarning`, `Tag`, and generic `Wrap<T>`.
- `ContentList` now trims trailing whitespace from leaf text nodes and keeps
  tree parent links intact when adding or inserting content.
- Added tests covering trimming, payload storage, tag formatting, wrapper
  passthrough, and tree composition.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler parsed
cargo check --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Continued Milestone 3 with the parsed hierarchy flow trait slice.
- Added parsed hierarchy flow primitives: `FlowLevel`, `NamedContent`,
  `FlowBase`, and `FlowArgument`.
- Updated `Path` to track a base flow level and weave-point formatting.
- Implemented flow traits for `Story`.
- Added tests for flow-level ordering, flow trait defaults, and story-level
  flow behavior.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler parsed
cargo check --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Continued Milestone 2 with `StringParserState` stack behavior.
- Ported the parser state stack into `parser::string_parser::state`.
- Added tests for push/pop, squash, stack initialization, and error-scope
  propagation.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler string_parser
cargo check --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Completed Milestone 2 by porting the core `StringParser` helpers.
- Added a Rust `StringParser` implementation with rule stack helpers, cursor
  navigation, character parsing, expectation/error reporting, debug metadata
  creation, and newline handling.
- Added focused tests for cursor movement, `peek`/`parse_object` commit and
  rollback behavior, diagnostics, and debug metadata generation.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler string_parser
cargo check --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

### 2026-04-22

- Completed Milestone 7 list definitions and list values slice.
- Added parsed hierarchy `ListDefinition` and `ListElementDefinition`
  wrappers, plus parser support for `LIST` declarations.
- Extended runtime export to collect parsed list declarations into top-level
  `listDefs` metadata while preserving the existing plain-text export path.
- Added parser and runtime export tests for list-definition parsing and JSON
  loading.

Validation:

```sh
cargo fmt --all --check
cargo test -p ink-compiler lists
cargo check --workspace
cargo test --workspace
```

Result: all passed. `blade-ink-rs/lib` emitted the same two existing warnings
about unnecessary parentheses around trait object types.

## Next Task

Document remaining incompatibilities.

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
