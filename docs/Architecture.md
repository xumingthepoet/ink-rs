# Architecture

This document is the repository map for `ink-rs`. Its job is to route a bug,
feature request, or refactor to the likely owner without requiring a full code
read first. Keep it synchronized whenever crates, module boundaries, test
layout, fixture categories, or the compile/runtime data flow change.

## Fast Orientation

`ink-rs` is a new domain-specific language for narrative games, implemented in
Rust and inspired by ink by inkle. It does not aim to be compatible with the Ink
language. The implementation is split into a compiler, a shared compiled-story
JSON format crate, a runtime, an optional Dioxus game adapter, integration
tests, and a small CLI.

```text
ink-rs source
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

ink-dioxus -> ink-compiler + ink-runtime + ink-story-json-format
text-games-app -> ink-dioxus
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
- `crates/ink-dioxus` owns reusable Dioxus-facing game UI glue: embedded source
  list helpers, ink-rs runtime/app wrappers, shared UI tag interpretation,
  multi-game catalogs, text reveal, toast handling, and the optional web shell.
- `crates/text-games-app` owns the bundled playable text-game hub. It depends on
  `ink-dioxus` instead of depending directly on compiler, runtime, or format
  crates.

## Repository Tree

The tree below lists the directories and files that usually matter when routing
work. It intentionally omits build output and historical plan internals.

```text
.
|-- AGENTS.md
|   Stable repository instructions for agents.
|-- Notes.md
|   Ranked durable working notes. Maintenance rules live in docs/workflows/.
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
|   |       |   unsupported `INCLUDE` diagnostics.
|   |       |-- syntax/
|   |       |   Source-line parser and syntax diagnostics.
|   |       |-- parsed/
|   |       |   Typed parsed model for ink-rs language constructs.
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
|   |       |   Public format crate surface.
|   |       |-- model.rs
|   |       |   `Program`, `Container`, `NamedContainer`, `Object`,
|   |       |   `ControlCommand`, and `NativeFunction`.
|   |       |-- json.rs
|   |       |   JSON codec module root for model and `serde_json` conversion.
|   |       |-- json/
|   |       |   Program/metadata, container, object, dynamic value, and
|   |       |   scalar helper codec modules.
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
|   |       |   Current execution state, callstack, output stream, globals,
|   |       |   random state, errors, warnings, and save/load state.
|   |       |-- callstack.rs
|   |       |-- execution_state.rs
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
|   |-- ink-dioxus/
|   |   |-- Cargo.toml
|   |   |-- README.md
|   |   `-- src/
|   |       |-- lib.rs
|   |       |   Public Dioxus adapter surface.
|   |       |-- runtime.rs
|   |       |   Compile embedded `InkSource` values and run stories to pauses.
|   |       |-- app.rs
|   |       |   Choice prompts, title/prompt/toast control tags, and app state.
|   |       |-- tags.rs
|   |       |   Shared key/value, marker, and boolean ink-rs tag parsing.
|   |       |-- styled_text.rs
|   |       |   `[style ...]` markup and tag-derived style segments.
|   |       |-- transcript.rs
|   |       |   Transcript paragraph buffering.
|   |       |-- build.rs
|   |       |   Build-script helper for generated embedded ink-rs source lists.
|   |       |-- web.rs
|   |       |   Optional Dioxus web UI shell behind the `web` feature.
|   |       `-- web.css
|   |           Default web shell CSS embedded by `web.rs`.
|   |-- text-games-app/
|   |   |-- Cargo.toml
|   |   |-- Dioxus.toml
|   |   |-- build.rs
|   |   |   Generates the embedded multi-game catalog.
|   |   |-- src/main.rs
|   |   |   Launches the catalog web app through `ink-dioxus`.
|   |   |-- assets/ink/
|   |   |   Bundled playable game source directories.
|   |   `-- docs/
|   |       Supporting game design notes.
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
|   |-- LanguageOverview.md
|   |   Short entry point for current ink-rs source syntax.
|   |-- SyntaxUpdates.md
|   |   Historical language and semantic change log.
|   |-- SyntaxReference.md
|   |   Full current supported syntax reference.
|   |-- ink_JSON_runtime_format.md
|   |   Compiled story JSON notes.
|   |-- workflows/
|   |   Standing workflow rules for active plans, validation, issue capture,
|   |   and durable notes.
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

Individual public stages return `StageOutput<T>`, which contains an optional
artifact and accumulated diagnostics. Full compile entry points return
`CompileResult<CompiledStory>` so host projects can apply
`CompilerOptions::diagnostics_policy` through `CompileResult::failed()`. Later
stages should not run after diagnostics that fail the active compile policy.

### Source Preparation

Owner: `crates/ink-compiler/src/source.rs`

Use this area for file-level input concerns:

- public `SourceInput`
- internal `SourceFile` and `SourceLine`
- `SourceSpan` and source filename preservation
- comment elimination
- first-line byte-order-mark trimming
- normalized line endings and trailing whitespace
- unsupported `INCLUDE` diagnostics

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
`=== module name implements Interface ===` records explicit interface
implementation clauses; `FROM module` and `FROM module IMPORT ...` live at
module top level; knots/functions live inside modules; direct module-level
story content is a syntax error.

### Parsed Model

Owner: `crates/ink-compiler/src/parsed/`

Parsed nodes are the Rust-native representation of ink-rs concepts. Add or extend
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
about ink-rs syntax or semantic rules, the ownership boundary is probably wrong.

## Format Crate

Owner: `crates/ink-story-json-format/`

This crate is the typed owner of compiled-story JSON:

- `model.rs`: Rust model for the wire format.
- `json.rs`: codec module root with the crate-facing conversion surface.
- `json/`: conversion owners for programs/metadata, containers, objects,
  dynamic values/dicts, and scalar helpers.
- `error.rs`: format errors.
- `lib.rs`: public exports.

Use this crate when adding or changing a compiled-story JSON token, control
command, native function, object variant, or container field.
Compiler lowering should produce these types directly. Runtime loading should
parse JSON through these types before constructing runtime execution objects.

The format crate is current-format only. When a compiled-story JSON break is
approved, remove obsolete tokens from the format model and runtime execution
instead of keeping compatibility decoders.

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
- `story/callstack.rs`: public callstack reset behavior.
- `story/errors.rs`: runtime error handling.
- `story/external_functions.rs`: host function binding and fallback behavior.
- `story_state.rs`: callstack, output stream, generated choices, globals,
  evaluation stack, random state, current errors/warnings, and current
  save/load.
- `callstack.rs`, `execution_state.rs`, `variables_state.rs`,
  `state_patch.rs`: mutable execution state machinery.
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

Compiled-story JSON format changes require coordinated updates in the format
crate, compiler lowering/tests, runtime loading/tests, and
`docs/ink_JSON_runtime_format.md`.

### Runtime Save State

Runtime save JSON is not compiled-story JSON. Save/load is owned by
`story_state.rs` and runtime JSON helpers.

The save format stores only stable execution state:

- callstack frames
- global `variablesState`
- deterministic random state (`storySeed`, `previousRandom`)

It does not store generated choices, choice continuation snapshots, multi-flow
maps, mid-expression internals, current divert target internals, visit counts,
or turn indices. Saving is allowed only while visible choices are pending. The
serialized state is the stable execution snapshot from immediately before
choice generation; generated choices are regenerated by continuing after load.
Ordinary non-choice states are rejected as save points. Save-state JSON that
contains removed execution fields such as
`currentChoices`, `choiceThreads`, `flows`, `currentFlowName`, `evalStack`,
`currentDivertTarget`, `visitCounts`, `turnIndices`, or `resumeMode` are
rejected rather than migrated. Saving should fail clearly at unstable runtime
points rather than serializing incomplete execution internals.

## Tests And Fixtures

Owner: `crates/ink-test/`

Integration tests are normal Cargo integration targets grouped by behavior.
`crates/ink-test` compiles those behavior groups through one aggregate
`integration` test target so workspace validation does not rebuild shared test
dependencies once per file. There is no aggregate origin/example-suite target.
`make test` runs `cargo test --workspace --quiet` through
`tools/run-with-timeout`; `make gate` adds formatting and standalone workspace
checking around that test run.

```text
crates/ink-test/
|-- src/lib.rs
|   Workspace and fixture path helpers.
|-- tests/
|   |-- integration.rs
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
|   |-- choice_loops.rs
|   |-- tunnels.rs
|   |-- typed_values.rs
|   |-- variables.rs
|   `-- support/
|       |-- compiler.rs
|       |-- runtime.rs
|       `-- story_runner.rs
`-- fixtures/
    |-- choices/
    |-- choice_loops/
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
    |-- tunnels/
    |-- typed/
    `-- variables/
```

Test policy:

- Every `.ink` integration fixture should use explicit module syntax.
- Tests should load repository fixtures instead of constructing ink-rs source
  inline.
- `.ink.json` fixtures are allowed only as expected compiler-output snapshots
  beside the matching `.ink` source fixture; runtime integration tests compile
  source fixtures instead of loading compiled JSON directly.
- Tests and fixture paths should be grouped by behavior, not by historical
  origin.
- Shared helpers belong under `tests/support/` only when they are harness code,
  not behavior categories.
- `tests/integration_policy.rs` enforces fixture module syntax, no inline ink-rs
  source construction, and no banned origin/example-suite labels under
  `tests/` or `fixtures/`.

Use the smallest relevant test target first, for example:

```text
cargo test -p ink-test --test integration diagnostics
cargo test -p ink-test --test integration typed_values
cargo test -p ink-test --test integration runtime_api
cargo test -p ink-test --test integration integration_policy
cargo test -p ink-test
```

See `docs/workflows/validation.md` for the full validation policy, including
how Cargo filters package-level test commands and what `make gate` covers.

## Tools, Docs, And Reference Code

`crates/ink-tools/src/main.rs` provides the `ink_compile` binary. It currently
compiles one explicit source file and prints or writes compiled JSON. It uses
`Compiler::compile_sources` with one `SourceInput`; it does not discover imports
or sibling files.

`editor/vscode-ink-rs/` owns editor syntax/package assets. Update it when a
language syntax change should affect highlighting or editor examples.

`docs/LanguageOverview.md` is the short current-language entry point.
`docs/SyntaxReference.md` is the full maintained syntax reference.
`docs/SyntaxUpdates.md` is the historical language and semantic change log.
Keep `SyntaxReference.md` focused on current supported syntax. Historical
language changes belong in `SyntaxUpdates.md`, issue records, diagnostics
tests, or architecture notes.

Standing workflows live under `docs/workflows/`:

- `active_plan.md`: active plan task-list and continuation rules
- `validation.md`: focused validation and `make gate` policy
- `issues.md`: deferred issue capture and solved-issue movement
- `notes.md`: `Notes.md` ranking and maintenance rules

The current Rust implementation, maintained docs, and tests are the source of
truth for language behavior. External implementations are background context
only and should not override ink-rs design decisions.

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
| Missing or wrong JSON token mapping | `ink-story-json-format/src/model.rs`, `json/` | compiler emit and runtime json_read |
| Runtime cannot load compiled JSON | `ink-runtime/src/json/json_read.rs` | format crate codec |
| Runtime output differs after load | `story/progress.rs`, `story/control_logic.rs`, `story/navigation.rs` | lowered `Program` and runtime fixture |
| Choices appear, repeat, or save incorrectly | `story/choices.rs`, `choice_point.rs`, `story_state.rs` | `tests/choices.rs` |
| Tags wrong on lines or choices | `story/tags.rs`, `tag.rs`, `control_command.rs` | `tests/tags.rs` |
| External function behavior | `story/external_functions.rs`, `divert.rs` | runtime API fixtures |
| Native operation or typed runtime value wrong | `native_function_call/`, `value_type.rs`, `value.rs` | typed value tests |
| Variable get/set wrong | `variables_state.rs`, `story/state.rs` | runtime API and variables tests |
| Save/load bug | `story_state.rs`, `json/json_write.rs`, `execution_state.rs`, `callstack.rs` | choices/runtime save-load tests |
| Dioxus game UI, shared UI tags, embedded ink-rs source lists and catalogs | `crates/ink-dioxus/src/` | `cargo test -p ink-dioxus`, `cargo check -p ink-dioxus --features web` |
| Playable text game hub or bundled game sources | `crates/text-games-app/` | `cargo check -p text-games-app` |
| CLI compile behavior | `crates/ink-tools/src/main.rs` | compiler API tests |
| Test harness or fixture policy | `crates/ink-test/tests/support/`, `integration_policy.rs` | `cargo test -p ink-test --test integration integration_policy` |

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
9. Update `docs/LanguageOverview.md` when the short current-language entry point
   would otherwise become incomplete or misleading.
10. Update this `docs/Architecture.md` in the same change when the file tree,
    module ownership, pipeline, routing table, or validation expectations
    change.

Forbidden shortcuts:

- Do not add fixture-name, fixture-path, or expected-output special cases to
  compiler or runtime code.
- Do not duplicate compiled-story JSON schema knowledge outside
  `ink-story-json-format`.
- Do not hide unsupported behavior with ignored tests or weakened assertions.
- Do not preserve inactive compatibility shims after a feature is intentionally
  replaced.

## Validation

Start with the smallest relevant validation, then widen. `make gate` is the
single project-level gate:

```text
cargo fmt --all --check
cargo check --workspace
cargo test --workspace --quiet
```

See `docs/workflows/validation.md` for focused command examples, Cargo filter
pitfalls, timeout behavior, and doctest policy.

If a language behavior changes, update tests and maintained documentation in the
same change. If the change is only a code layout refactor, this architecture map
should still be updated when navigation would otherwise become stale.
