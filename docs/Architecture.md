# Architecture

This document is the repository map for `ink-rs`. Its job is to route a bug,
feature request, or refactor to the likely owner without requiring a full code
read first. Keep it synchronized whenever crates, module boundaries, test
layout, fixture categories, or the compile/runtime data flow change.

## Fast Orientation

`ink-rs` is a Rust implementation and language fork of Ink. The implementation
is split into a compiler, a shared compiled-story JSON format crate, a runtime,
integration tests, and a small CLI.

```text
Ink source
  -> ink-compiler
     -> ParsedStory
     -> CheckedStory
     -> ink_story_json_format::Program
     -> JSON string
  -> ink-runtime
     -> ink_story_json_format::Program
     -> runtime object graph
     -> Story execution and save/load state
```

Crate dependencies are intentionally narrow:

```text
ink-compiler ----------+
                       |
                       v
              ink-story-json-format
                       ^
                       |
ink-runtime -----------+

ink-test  -> ink-compiler + ink-runtime
ink-tools -> ink-compiler
```

Ownership rules:

- `crates/ink-story-json-format` owns compiled-story JSON wire structures,
  token names, JSON serialization, and JSON deserialization.
- `crates/ink-compiler` owns source preparation, syntax parsing, parsed model,
  semantic analysis, lowering into the format crate model, and compile-time
  diagnostics.
- `crates/ink-runtime` owns runtime execution objects, story state, public
  runtime APIs, external functions, and runtime save-state JSON.
- Runtime execution objects are not the shared wire model. The runtime loads
  compiled story JSON through the format crate, then builds runtime-owned
  containers and objects.
- Runtime save-state JSON is separate from compiled-story JSON and remains
  runtime-owned.

## Repository Tree

The tree below lists the directories and files that usually matter when routing
work. It intentionally omits build output and most upstream reference internals.

```text
.
|-- AGENTS.md
|   Stable repository instructions for agents.
|-- Notes.md
|   Ranked durable working notes for compiler/runtime/language work.
|-- Makefile
|   Project gates: fmt, check, test, gate.
|-- Cargo.toml
|   Workspace members.
|-- crates/
|   |-- ink-compiler/
|   |   |-- Cargo.toml
|   |   `-- src/
|   |       |-- lib.rs
|   |       |   Public compiler re-exports.
|   |       |-- compiler.rs
|   |       |   `Compiler`, stage orchestration, `StageOutput`, `CompiledStory`.
|   |       |-- diagnostic.rs
|   |       |   Diagnostic severity, codes, messages, and spans.
|   |       |-- source.rs
|   |       |   `SourceInput`, `SourceFile`, line spans, comment elimination,
|   |       |   removed `INCLUDE` diagnostics.
|   |       |-- syntax/
|   |       |   Source-line parser and syntax diagnostics.
|   |       |-- parsed/
|   |       |   Typed parsed model for Ink language constructs.
|   |       |-- analysis/
|   |       |   Semantic checks and indexes over the parsed model.
|   |       |-- lower.rs
|   |       |   Lowering entry point from `CheckedStory` to format `Program`.
|   |       |-- lower/
|   |       |   Lowering helpers by behavior: flow, weave, expressions,
|   |       |   assignments, diverts, paths, labels, indexes, values.
|   |       `-- emit.rs
|   |           JSON emission through `ink-story-json-format`.
|   |-- ink-story-json-format/
|   |   |-- Cargo.toml
|   |   `-- src/
|   |       |-- lib.rs
|   |       |   Public format crate surface and `INK_VERSION_CURRENT`.
|   |       |-- model.rs
|   |       |   `Program`, `Container`, `NamedContainer`, `Object`,
|   |       |   `ControlCommand`, and `NativeFunction`.
|   |       |-- json.rs
|   |       |   JSON codec for compiled story programs, containers, and objects.
|   |       `-- error.rs
|   |           Format parse/serialization errors.
|   |-- ink-runtime/
|   |   |-- Cargo.toml
|   |   `-- src/
|   |       |-- lib.rs
|   |       |   Runtime crate surface.
|   |       |-- story/
|   |       |   Public `Story` behavior split by execution concern.
|   |       |-- story_state.rs
|   |       |   Current flow, callstack, output stream, globals, random state,
|   |       |   errors, warnings, and save/load state.
|   |       |-- callstack.rs
|   |       |-- flow.rs
|   |       |-- variables_state.rs
|   |       |-- state_patch.rs
|   |       |   Runtime state machinery.
|   |       |-- container.rs
|   |       |-- object.rs
|   |       |-- path.rs
|   |       |-- pointer.rs
|   |       |-- search_result.rs
|   |       |   Runtime object graph and addressing.
|   |       |-- json/
|   |       |   Compiled story loading plus runtime JSON writing helpers.
|   |       |-- native_function_call/
|   |       |   Native operations, including scalar and composite typed-value ops.
|   |       |-- choice.rs
|   |       |-- choice_point.rs
|   |       |-- divert.rs
|   |       |-- control_command.rs
|   |       |-- push_pop.rs
|   |       |-- tag.rs
|   |       |-- glue.rs
|   |       |-- value.rs
|   |       |-- value_type.rs
|   |       |-- variable_assigment.rs
|   |       |-- variable_reference.rs
|   |       `-- void.rs
|   |-- ink-test/
|   |   |-- Cargo.toml
|   |   |-- src/lib.rs
|   |   |   Shared fixture path helpers for integration tests.
|   |   |-- tests/
|   |   |   Root integration test targets grouped by behavior.
|   |   |-- tests/support/
|   |   |   Shared compile/runtime/story-runner helpers only.
|   |   `-- fixtures/
|   |       Functional `.ink`, `.ink.json`, and `.ink.parse` fixtures.
|   `-- ink-tools/
|       |-- Cargo.toml
|       `-- src/main.rs
|           `ink_compile` CLI for compiling one source file to JSON.
|-- docs/
|   |-- Architecture.md
|   |   This repository map. Update it with structural changes.
|   |-- SyntaxUpdates.md
|   |   Language and semantic changes from upstream.
|   |-- SyntaxReference.md
|   |   Current supported syntax reference.
|   |-- WritingWithInk.md
|   |   Upstream C# guide snapshot. Do not edit.
|   |-- ink_JSON_runtime_format.md
|   |   Compiled story JSON notes.
|   |-- active_plan/
|   |   Current implementation plan workspace.
|   |-- finished_plans/
|   |   Completed plan records.
|   |-- issues_found/
|   |   Deferred issue records.
|   `-- issues_solved/
|       Solved issue records.
|-- editor/
|   `-- vscode-ink-rs/
|       VS Code syntax/package assets and examples.
`-- target/
    Local Cargo build output. Never use it as source truth.
```

## Compiler Pipeline

`Compiler::compile_sources` in `crates/ink-compiler/src/compiler.rs` is the
canonical compile path. It calls:

```text
SourceInput
  -> prepare_source_input
  -> syntax::parse_source
  -> analysis::analyze
  -> lower::lower
  -> emit::emit_json
  -> CompiledStory { program, json }
```

Every public stage returns `StageOutput<T>`, which contains an optional artifact
and accumulated diagnostics. Later stages should not run after error diagnostics
from earlier stages.

### Source Preparation

Owner: `crates/ink-compiler/src/source.rs`

Use this area for file-level input concerns:

- public `SourceInput`
- internal `SourceFile` and `SourceLine`
- `SourceSpan` and source filename preservation
- comment elimination
- first-line byte-order-mark trimming
- normalized line endings and trailing whitespace
- removed `INCLUDE` diagnostics

Do not put language parsing here. If logic needs to know about declarations,
expressions, modules, or flow structure, it belongs in syntax or later.

### Syntax Parsing

Owner: `crates/ink-compiler/src/syntax/`

Syntax turns source lines into the parsed model and emits syntax-local
diagnostics. The parser is line-oriented and trial-order sensitive.

Important files:

- `parser.rs`: top-level parser, module/root routing, flow parsing, statement
  trial order.
- `rule.rs`: `RuleParser`, checkpoints, diagnostics, and cursor operations for
  one source line.
- `state.rs`: parser state snapshots and rewinds.
- `scan.rs`: balanced scanning helpers for strings, braces, parentheses, and
  separators.
- `expression.rs`: expression tokenizer/parser and top-level argument splitting.
- `text.rs`: text lines, inline content, glue, tags, inline diverts, braced
  content.
- `choice.rs`, `gather.rs`, `weave.rs`: weave-point syntax.
- `conditional.rs`, `structure.rs`: braced conditionals and structured
  literals.
- `knot.rs`, `module.rs`, `import.rs`: story structure.
- `declaration.rs`, `variable.rs`, `logic.rs`, `divert.rs`, `type_name.rs`:
  statement families.
- `error.rs`: syntax error helpers.

Parser rules should either fail cleanly and rewind or intentionally emit a
diagnostic. Avoid ad hoc string splitting when `scan.rs`, `RuleParser`, or
`expression.rs` can represent the syntax boundary.

Explicit module syntax is parsed here. `=== module name ===` opens a module;
`IMPORT ... FROM ...` lives at module top level; knots/functions live inside
modules; direct module-level story content is a syntax error.

### Parsed Model

Owner: `crates/ink-compiler/src/parsed/`

Parsed nodes are the Rust-native representation of Ink concepts. Add or extend
these types before analysis or lowering needs the data.

Key files:

- `story.rs`, `module.rs`, `flow.rs`, `weave.rs`, `content_list.rs`: story
  structure.
- `choice.rs`, `gather.rs`, `divert.rs`, `tunnel_onwards.rs`: flow and weave
  control.
- `expression.rs`, `type_name.rs`, `struct_declaration.rs`,
  `conditional.rs`: typed values and expressions.
- `variable_assignment.rs`, `constant_declaration.rs`,
  `external_declaration.rs`, `return_node.rs`, `inc_dec.rs`: logic statements.
- `text.rs`, `glue.rs`, `tag.rs`, `author_warning.rs`: content markers.
- `qualified_name.rs`: module/path-aware names.
- `visit.rs`: traversal helpers used by analysis and tests.

Do not send raw source strings downstream when a parsed type can carry the
structure. Bugs caused by "lowering rediscovered syntax" usually need a parsed
model addition.

### Analysis

Owner: `crates/ink-compiler/src/analysis/`

Analysis owns semantic diagnostics and story-wide indexes. It should not import
lowering or emit internals.

Important files:

- `mod.rs`: pass ordering and `CheckedStory`.
- `context.rs`: shared semantic context.
- `modules/`: module symbols, imports, dependencies, reachability, and entry
  point selection.
- `names.rs`: duplicate definitions and naming collisions.
- `flow.rs`: loose ends, returns, and function/control-flow shape.
- `targets.rs`, `target_symbols.rs`: divert/call target validation and lookup.
- `variables.rs`: globals, temps, args, and variable visibility support.
- `constants.rs`, `initializers.rs`: constants and variable initializer rules.
- `structs.rs`, `struct_literals.rs`, `array_literals.rs`: typed composite
  declarations and literals.
- `expression_types.rs`: expression type reasoning.
- `assignments.rs`, `field_access.rs`, `index_access.rs`: assignment and typed
  access checks.
- `warnings.rs`: author warning diagnostics.
- `span.rs`: diagnostic span helpers.
- `test_support.rs`: analysis-only test helpers.

Add semantic checks here when a problem depends on names, targets, imports,
types, variables, module reachability, or cross-line story structure. Lowering
should not be the first stage to discover semantic invalidity.

### Lowering

Owners:

- entry point: `crates/ink-compiler/src/lower.rs`
- helpers: `crates/ink-compiler/src/lower/`

Lowering converts `CheckedStory` into
`ink_story_json_format::Program`. It decides runtime container shape and command
sequences but should remain mostly diagnostic-free.

Important files:

- `lower.rs`: root/module lowering orchestration and global declarations.
- `context.rs`: scoped lowering state, path mode, variables, externals,
  constants, and struct definitions visible to expression/content lowering.
- `indexes.rs`: lowering-time indexes for targets, labels, variables,
  constants, externals, structs, and runtime length estimation.
- `labels.rs`: label collection.
- `path.rs`: runtime path construction and path compaction.
- `flow.rs`: root weave, knots, stitches, functions, and module flows.
- `weave.rs`: content lists, choices, gathers, weave containers.
- `expression.rs`: expression, command, string-expression, and logic lowering.
- `assignment.rs`: global/temp/field/index assignment lowering.
- `divert.rs`: diverts, tunnels, function calls, external calls, tail recursion.
- `conditional.rs`: conditional lowering.
- `value.rs`: compile-time values used by constants/defaults/composites.

For explicit module stories, lowering emits a root container that auto-diverts
to the selected `module.knot` runtime path. Root named content contains
reachable module containers, and module containers own their lowered flows.
Source-qualified runtime state and host names such as `state::score` and
`audio::play` stay source-qualified; container paths use dot-separated runtime
paths such as `state.main`.

### Emit

Owner: `crates/ink-compiler/src/emit.rs`

Emit serializes the lowered format `Program` using
`ink-story-json-format`. It should stay a narrow writer. If emit needs to know
about Ink syntax or semantic rules, the ownership boundary is probably wrong.

## Format Crate

Owner: `crates/ink-story-json-format/`

This crate is the typed owner of compiled-story JSON:

- `model.rs`: Rust model for the wire format.
- `json.rs`: conversion between model values and `serde_json`.
- `error.rs`: format errors.
- `lib.rs`: public exports and `INK_VERSION_CURRENT`.

Use this crate when adding or changing a compiled-story JSON token, control
command, native function, object variant, container field, or version rule.
Compiler lowering should produce these types directly. Runtime loading should
parse JSON through these types before constructing runtime execution objects.

Do not add duplicate compiled-story JSON schemas in compiler emit or runtime
JSON read/write code. If compiler and runtime both need to understand a
compiled-story JSON shape, it belongs here.

## Runtime

Owner: `crates/ink-runtime/`

`ink_runtime::story::Story` is the public runtime entry point. `Story::new`
loads compiled JSON through `json/json_read.rs`, which first parses
`ink_story_json_format::Program` and then converts it into runtime-owned
containers and objects.

Key areas:

- `story/mod.rs`: `Story`, construction, public narrative loop behavior, and
  module-local submodules.
- `story/choices.rs`: choice generation and choice selection.
- `story/control_logic.rs`: command execution, expression evaluation, diverts,
  function/tunnel stack behavior, random commands.
- `story/navigation.rs`: path and pointer movement.
- `story/progress.rs`: continue loop and output production.
- `story/state.rs`: public accessors and state APIs.
- `story/tags.rs`: current tags and choice tags.
- `story/flow.rs`: flow-level runtime behavior.
- `story/errors.rs`: runtime error handling.
- `story/external_functions.rs`: host function binding and fallback behavior.
- `story_state.rs`: callstack, output stream, generated choices, globals,
  evaluation stack, random state, current errors/warnings, and v2 save/load.
- `callstack.rs`, `flow.rs`, `variables_state.rs`, `state_patch.rs`: mutable
  execution state machinery.
- `container.rs`, `object.rs`, `path.rs`, `pointer.rs`, `search_result.rs`:
  object graph and addressing.
- `json/json_read.rs`: compiled-story JSON load path through the format crate.
- `json/json_write.rs`: runtime object JSON writing helpers used by save state
  and debug/runtime serialization paths.
- `native_function_call/`: built-in operations. `scalar.rs` handles primitive
  ops; `composite.rs` handles object/array field/index operations; `metadata.rs`
  maps runtime ops to format native functions; `params.rs` handles argument
  coercion.
- `value.rs`, `value_type.rs`, `void.rs`: runtime values.
- `choice.rs`, `choice_point.rs`, `divert.rs`, `control_command.rs`,
  `push_pop.rs`, `tag.rs`, `glue.rs`, `variable_assigment.rs`,
  `variable_reference.rs`: runtime instruction and marker objects.

Compiled-story JSON compatibility changes require coordinated updates in the
format crate, compiler lowering/tests, runtime loading/tests, and
`docs/ink_JSON_runtime_format.md`.

### Runtime Save State

Runtime save JSON is not compiled-story JSON. Save/load is owned by
`story_state.rs` and runtime JSON helpers. The current save-state version is
`INK_SAVE_STATE_VERSION = 2`.

The v2 save format stores only stable pause-point state:

- callstack frames
- current generated choices
- choice thread snapshots when choices require them
- global `variablesState`
- deterministic random state (`storySeed`, `previousRandom`)
- `inkSaveVersion` and compiled-story `inkFormatVersion`

It does not store old upstream multi-flow maps, mid-expression internals,
current divert target internals, visit counts, turn indices, or old version 1
state. Saving should fail clearly at unstable runtime points rather than
serializing incomplete execution internals.

## Tests And Fixtures

Owner: `crates/ink-test/`

Integration tests are normal Cargo integration targets grouped by behavior.
There is no aggregate origin/example-suite target. `make test` runs all
workspace tests except `ink-test`, then `integration_policy`, then the whole
`ink-test` package.

```text
crates/ink-test/
|-- src/lib.rs
|   Workspace and fixture path helpers.
|-- tests/
|   |-- choices.rs
|   |-- compiler_api.rs
|   |-- compiler_snapshots.rs
|   |-- compiler_snapshots/
|   |   |-- common.rs
|   |   `-- fixtures.rs
|   |-- conditionals.rs
|   |-- diagnostics.rs
|   |-- diverts.rs
|   |-- expressions.rs
|   |-- flow.rs
|   |-- functions.rs
|   |-- gathers.rs
|   |-- glue.rs
|   |-- integration_policy.rs
|   |-- knots.rs
|   |-- misc.rs
|   |-- modules.rs
|   |-- runtime_api.rs
|   |-- stitches.rs
|   |-- tags.rs
|   |-- text.rs
|   |-- threads.rs
|   |-- tunnels.rs
|   |-- typed_values.rs
|   |-- variables.rs
|   `-- support/
|       |-- compiler.rs
|       |-- runtime.rs
|       `-- story_runner.rs
`-- fixtures/
    |-- choices/
    |-- compiler_api/
    |-- conditionals/
    |-- diagnostics/
    |-- diverts/
    |-- expressions/
    |-- flow/
    |-- functions/
    |-- gather/
    |-- glue/
    |-- knots/
    |-- misc/
    |-- modules/
    |-- runtime_api/
    |-- stitches/
    |-- tags/
    |-- text/
    |-- threads/
    |-- tunnels/
    |-- typed/
    `-- variables/
```

Test policy:

- Every `.ink` integration fixture should use explicit module syntax.
- Tests should load repository fixtures instead of constructing Ink source
  inline.
- `.ink.json` fixtures are allowed only as expected compiler-output snapshots
  beside the matching `.ink` source fixture; runtime integration tests compile
  source fixtures instead of loading compiled JSON directly.
- Tests and fixture paths should be grouped by behavior, not by historical
  origin.
- Shared helpers belong under `tests/support/` only when they are harness code,
  not behavior categories.
- `tests/integration_policy.rs` enforces fixture module syntax, no inline Ink
  source construction, and no banned origin/example-suite labels under
  `tests/` or `fixtures/`.

Use the smallest relevant test target first, for example:

```text
cargo test -p ink-test --test diagnostics
cargo test -p ink-test --test typed_values
cargo test -p ink-test --test runtime_api
cargo test -p ink-test --test integration_policy
cargo test -p ink-test
```

## Tools, Docs, And Reference Code

`crates/ink-tools/src/main.rs` provides the `ink_compile` binary. It currently
compiles one explicit source file and prints or writes compiled JSON. It uses
`Compiler::compile_sources` with one `SourceInput`; it does not discover imports
or sibling files.

`editor/vscode-ink-rs/` owns editor syntax/package assets. Update it when a
language syntax change should affect highlighting or editor examples.

`docs/SyntaxUpdates.md` records language and semantic changes from
upstream. `docs/SyntaxReference.md` is the maintained syntax reference users
should read. Keep `SyntaxReference.md` focused on current supported syntax;
removed syntax, migration notes, and compatibility explanations belong in
`SyntaxUpdates.md`, issue records, diagnostics tests, or architecture notes. Do
not edit `docs/WritingWithInk.md`.

The upstream C# implementation remains an external reference for
legacy-compatible behavior questions, especially parser trial order, weave
grouping, path compaction, and upstream JSON shape. Do not copy C# class
structure into Rust when the Rust module boundary already has a clearer owner.

## Change Routing

Use this table to decide where to start.

| Symptom or request | Start here | Then check |
| --- | --- | --- |
| Comments, filenames, line numbers, `INCLUDE` removal | `ink-compiler/src/source.rs` | diagnostics tests |
| Syntax is accepted/rejected incorrectly | `ink-compiler/src/syntax/parser.rs`, focused syntax module | parsed snapshots and behavior fixture |
| Inline text, glue, tags, braced content | `syntax/text.rs`, `syntax/scan.rs` | `tests/text.rs`, `tests/glue.rs`, `tests/tags.rs` |
| Expression parse or operator precedence | `syntax/expression.rs` | `parsed/expression.rs`, `lower/expression.rs`, expression fixtures |
| Struct, array, field, index syntax | `syntax/structure.rs`, `syntax/expression.rs` | analysis typed checks, `tests/typed_values.rs` |
| Module/import behavior | `syntax/module.rs`, `syntax/import.rs`, `analysis/modules/` | `lower.rs`, `tests/modules.rs` |
| Duplicate names or symbol visibility | `analysis/names.rs`, `analysis/modules/symbols.rs` | target/variable indexes |
| Divert/call target diagnostics | `analysis/targets.rs`, `analysis/target_symbols.rs` | `lower/divert.rs`, diverts diagnostics |
| Variable scope, globals, temps, args | `analysis/variables.rs`, `analysis/initializers.rs` | `lower/indexes.rs`, `tests/variables.rs` |
| Type errors for composites or assignments | `analysis/expression_types.rs`, `analysis/assignments.rs`, `analysis/field_access.rs`, `analysis/index_access.rs` | typed fixtures |
| Wrong compiled JSON shape | `lower/` owner for the construct | `ink-story-json-format/src/model.rs`, snapshots |
| Missing or wrong JSON token mapping | `ink-story-json-format/src/model.rs`, `json.rs` | compiler emit and runtime json_read |
| Runtime cannot load compiled JSON | `ink-runtime/src/json/json_read.rs` | format crate codec |
| Runtime output differs after load | `story/progress.rs`, `story/control_logic.rs`, `story/navigation.rs` | lowered `Program` and runtime fixture |
| Choices appear, repeat, or save incorrectly | `story/choices.rs`, `choice_point.rs`, `story_state.rs` | `tests/choices.rs` |
| Tags wrong on lines or choices | `story/tags.rs`, `tag.rs`, `control_command.rs` | `tests/tags.rs` |
| External function behavior | `story/external_functions.rs`, `divert.rs` | runtime API fixtures |
| Native operation or typed runtime value wrong | `native_function_call/`, `value_type.rs`, `value.rs` | typed value tests |
| Variable get/set wrong | `variables_state.rs`, `story/state.rs` | runtime API and variables tests |
| Save/load bug | `story_state.rs`, `json/json_write.rs`, `flow.rs`, `callstack.rs` | choices/runtime save-load tests |
| CLI compile behavior | `crates/ink-tools/src/main.rs` | compiler API tests |
| Test harness or fixture policy | `crates/ink-test/tests/support/`, `integration_policy.rs` | `cargo test -p ink-test --test integration_policy` |

## Feature Workflow

For language or runtime behavior changes:

1. Verify the current behavior with a minimal fixture or focused test before
   changing code.
2. Put the new syntax shape in `syntax/` and the typed meaning in `parsed/`.
3. Put cross-line, cross-symbol, type, target, module, and scope checks in
   `analysis/`.
4. Lower checked parsed data into `ink_story_json_format::Program` in `lower/`.
5. Change `ink-story-json-format` only when the compiled-story JSON wire model
   actually changes.
6. Change `ink-runtime` only when loading, execution, public runtime APIs, or
   save/load behavior changes.
7. Add or update behavior-grouped fixtures and tests under `crates/ink-test`.
8. Update `docs/SyntaxUpdates.md` and
   `docs/SyntaxReference.md` for language or semantic changes.
9. Update this `docs/Architecture.md` in the same change when the file tree,
   module ownership, pipeline, routing table, or validation expectations change.

Forbidden shortcuts:

- Do not add fixture-name, fixture-path, or expected-output special cases to
  compiler or runtime code.
- Do not duplicate compiled-story JSON schema knowledge outside
  `ink-story-json-format`.
- Do not hide unsupported behavior with ignored tests or weakened assertions.
- Do not preserve obsolete compatibility shims after a feature is explicitly
  removed.

## Validation

Start with the smallest relevant validation, then widen:

```text
cargo fmt --all --check
cargo check --workspace
cargo test -p ink-compiler <focused filter>
cargo test -p ink-runtime <focused filter>
cargo test -p ink-test --test <focused_target>
cargo test -p ink-test --test integration_policy
cargo test -p ink-test
cargo test --workspace
make gate
```

`make gate` is the project-level gate:

```text
cargo fmt --all --check
cargo check --workspace
cargo test --workspace --exclude ink-test
cargo test -p ink-test --test integration_policy
cargo test -p ink-test
```

If a change intentionally diverges from upstream Ink, update tests and
maintained documentation in the same change. If the change is only a code
layout refactor, this architecture map should still be updated when navigation
would otherwise become stale.
