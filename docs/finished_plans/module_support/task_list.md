Progress: 30/30

# Task List: Module Support

Status key: `[ ]` pending, `[~]` in progress, `[x]` complete, `[!]` blocked.

This task list is an implementation plan, not a research log. Except for final
plan closeout, every task must edit production code, test code, fixtures, editor
assets, or another code-adjacent artifact. Maintained docs should be updated in
the same task as behavior changes, but pure prose-only tasks are not valid
implementation tasks. Necessary investigation belongs inside the implementation
method of the task that uses it.

Completion protocol for every task:

- Update the task checkbox from `[ ]` or `[~]` to `[x]`.
- Update `Progress: X/30`.
- Run the task's focused validation.
- Run `make gate`.
- Commit immediately after validation passes.
- Record the commit hash in the task's `Commit` field before starting another
  task.

Do not append notes, logs, or running commentary after this task list. Record
validation and commit metadata inside the relevant task section.

Global forbidden boundaries:

- Do not add fixture-name, fixture-path, or expected-output special cases.
- Do not hide failures with ignored tests, skipped fixtures, relaxed asserts, or
  compatibility shims for removed `INCLUDE` behavior.
- Do not preserve implicit root/global visibility as a fallback for module
  mistakes.
- Do not make imports transitive or re-export imported symbols by default.
- Do not change compiled story JSON schema unless `requirement_plan.md` is
  updated first with the explicit compatibility impact.
- Do not change unrelated parser, runtime execution, or language behavior while
  implementing module support.

## Milestone 1: Source Entry And Include Removal

### [x] Task 01: Add Explicit Multi-Source Compiler API

Goal:

Add the public explicit source-list API that all module compilation will use.

Implementation method:

- Inspect `SourceInput`, `Compiler::parse`, `Compiler::compile`, compiler API
  tests, and current diagnostics only as needed for this implementation.
- Add `Compiler::compile_sources(Vec<SourceInput>)`.
- Make `Compiler::compile(SourceInput)` delegate to the one-source
  `compile_sources` path.
- Preserve source filenames in diagnostics across all explicit source inputs.
- Reject empty source lists with a clear compiler diagnostic.
- Add public API tests for one source, multiple sources in different orders,
  empty input, and filename preservation.

Acceptance criteria:

- `compile_sources` is public and tested.
- `compile(SourceInput)` remains a tested convenience path.
- No directory scanning or implicit file discovery is introduced.
- Current non-module stories keep compiling through the compatibility path until
  parser migration tasks remove root story behavior.

Modification boundary:

- `crates/ink-compiler/src/compiler.rs`
- `crates/ink-compiler/src/source.rs` if needed
- `crates/ink-test/tests/compiler_api.rs`

Validation:

- `cargo test -p ink-test --test compiler_api`
- `cargo test -p ink-compiler compiler`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-test --test compiler_api` passed.
- `cargo test -p ink-compiler compiler` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 068edea

### [x] Task 02: Diagnose Removed INCLUDE And Retire Include Loading

Goal:

Remove active include expansion and make every `INCLUDE` form a clear removed
construct diagnostic.

Implementation method:

- Inspect `preprocess_includes`, `FileHandler`, include tests, C# include tests,
  and syntax fallback behavior only as needed.
- Stop running include expansion in the active parse/compile path.
- Diagnose `INCLUDE` at root level and indented legacy include positions.
- Remove or retire `CompilerOptions::file_handler` and public `FileHandler`
  behavior so no file loading remains reachable through compilation.
- Rewrite compiler and C# compatibility include tests to assert the intentional
  ink-rs divergence.
- Delete include-only helper code, empty shims, and obsolete include fixtures
  when no longer used.

Acceptance criteria:

- `INCLUDE` never loads another file.
- Removed include diagnostics are source-located and actionable.
- Include-specific compatibility tests are rewritten or removed intentionally,
  not skipped.
- No public compiler API presents include loading as current behavior.

Modification boundary:

- `crates/ink-compiler/src/compiler.rs`
- `crates/ink-compiler/src/source.rs`
- `crates/ink-compiler/src/source/preprocess.rs`
- `crates/ink-tools/src/main.rs` for public API compile-through after removing
  include file handlers
- `docs/Architecture.md` for current compiler source pipeline description
- compiler unit tests
- C# compatibility include tests and include fixtures

Validation:

- focused compiler include diagnostic tests
- `cargo test -p ink-test --features csharp-tests --test csharp_tests TestInclude`
- `cargo test -p ink-test --features csharp-tests --test csharp_tests TestNestedInclude`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler compiler` passed.
- `cargo test -p ink-test --test compiler_api` passed.
- `cargo test -p ink-test --features csharp-tests --test csharp_tests TestInclude` passed.
- `cargo test -p ink-test --features csharp-tests --test csharp_tests TestNestedInclude` passed.
- `cargo fmt --all --check` passed.
- `cargo check --workspace` passed.
- `make gate` passed.

Commit: 7db301f

### [x] Task 03: Update CLI Source Loading For Explicit Inputs

Goal:

Make `ink-tools` use explicit source inputs without file-handler include
support.

Implementation method:

- Update CLI source loading to pass explicit `SourceInput` values to the new
  compiler API.
- Remove CLI file-handler plumbing that only existed for `INCLUDE`.
- Add or update CLI tests if the crate has command-level coverage; otherwise
  add focused compiler/API coverage that proves the public path used by the CLI.
- Keep source filename reporting intact for CLI-loaded files.

Acceptance criteria:

- `ink-tools` compiles explicit input files without include file handlers.
- CLI-facing diagnostics still report filenames.
- The CLI does not scan directories for modules.

Modification boundary:

- `crates/ink-tools`
- compiler API tests needed to cover CLI-facing behavior

Validation:

- `cargo test -p ink-tools`
- `cargo test -p ink-test --test compiler_api`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-tools` passed.
- `cargo test -p ink-test --test compiler_api` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: f57855e

## Milestone 2: Parsed Module Syntax

### [x] Task 04: Add Parsed Module And Import Structures

Goal:

Represent modules and imports as first-class parsed model objects.

Implementation method:

- Add parsed `Module`, `ImportDeclaration`, and imported-name structures with
  spans for module names, imported symbols, source module names, and whole
  declaration lines.
- Extend parse snapshots so modules and imports are visible.
- Extend parsed visitor traversal and context with current module ownership.
- Add unit tests for construction, snapshot output, and visitor traversal.

Acceptance criteria:

- Parsed model can represent modules and imports without flow-name string
  encoding.
- Visitor traversal sees modules, imports, module declarations, knots,
  functions, and stitches in a stable order.

Modification boundary:

- `crates/ink-compiler/src/parsed`
- parsed-model tests

Validation:

- `cargo test -p ink-compiler parsed::`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler parsed::` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: aa09b09

### [x] Task 05: Parse Module Headers

Goal:

Parse valid `=== module name ===` declarations and reject invalid module header
forms.

Implementation method:

- Add a module-header parser that is distinct from knot/function/stitch parsing.
- Enforce semantic delimiter levels: modules use `===`, knots/functions use
  `==`, stitches use `=`.
- Enforce lowercase `module`, no parameters, and single-identifier module
  names.
- Reject dotted or hierarchical module names for this phase.
- Add parser tests for valid headers, wrong delimiter levels, keyword case,
  parameters, invalid names, and multiple modules in one source.

Acceptance criteria:

- `=== module name ===` starts a module.
- Invalid module-like headers produce clear diagnostics.
- Module headers do not fall through as text.

Modification boundary:

- `crates/ink-compiler/src/syntax`
- parser tests

Validation:

- focused module-header parser tests
- `cargo test -p ink-compiler syntax::parser`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler syntax::module` passed.
- `cargo test -p ink-compiler syntax::parser` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 349d64e

### [x] Task 06: Parse Import Declarations

Goal:

Parse first-phase single-line `IMPORT name, name FROM moduleName` declarations.

Implementation method:

- Add import declaration parsing inside active modules.
- Preserve imported item spans and source module spans.
- Enforce uppercase `IMPORT` and `FROM`.
- Reject aliases, kind annotations, empty lists, missing `FROM`, and multiline
  imports.
- Add parser tests for valid imports and invalid first-phase forms.

Acceptance criteria:

- Imports are parsed as module-local declarations.
- Unsupported import syntax fails in parsing with clear diagnostics.
- Import names do not carry explicit kind annotations.

Modification boundary:

- `crates/ink-compiler/src/syntax`
- `crates/ink-compiler/src/parsed`
- parser tests

Validation:

- focused import parser tests
- `cargo test -p ink-compiler syntax::parser`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler syntax::import` passed.
- `cargo test -p ink-compiler syntax::parser` passed.
- `cargo test -p ink-compiler parsed::module` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: a067438

### [x] Task 07: Reshape Story Parsing Around Explicit Modules

Goal:

Make `Story` own explicit modules and remove implicit root ownership from the
parsed language model.

Implementation method:

- Move module-scoped `CONST`, global `VAR`, `STRUCT`, `EXTERNAL`, knots, and
  functions under the active module.
- Assign ownership from a module header until the next module header or end of
  source.
- Support multiple modules in one source input.
- Diagnose content or module-scoped declarations before the first module header.
- Diagnose direct module-level story content and module-level tags.
- Keep stitches as child flows of their parent knot.
- Update snapshots and parser tests for module ownership.

Acceptance criteria:

- For sources that contain explicit module headers, there is no implicit root
  ownership.
- For sources that contain explicit module headers, all module-scoped
  declarations require an explicit active module.
- Direct module-level content and module-level tags are errors in explicit
  module sources.
- Stitches remain scoped to parent knots.
- Legacy no-module source remains a temporary compile path until module
  lowering and entry-point tasks replace it, so this task can still pass
  `make gate`.

Modification boundary:

- parser story/flow ownership code
- parsed story model
- parser snapshots and tests

Validation:

- focused module ownership parser tests
- focused parse snapshot tests
- `cargo test -p ink-compiler syntax::parser`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler syntax::parser` passed.
- `cargo test -p ink-compiler syntax::module` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 5f9a462

### [x] Task 08: Parse Multiple Explicit Source Inputs As One Compilation Unit

Goal:

Make parsing work across a list of explicit source inputs without text
concatenation.

Implementation method:

- Update the multi-source compiler path to parse each `SourceInput` while
  preserving source filenames and spans.
- Combine parsed modules into one parsed compilation unit without concatenating
  raw text.
- Add tests for modules spread across multiple sources in different orders.
- Add tests for one source containing multiple modules.

Acceptance criteria:

- Source order does not affect module ownership.
- Diagnostics still point at the original source input.
- No module names are inferred from filenames.

Modification boundary:

- `crates/ink-compiler/src/compiler.rs`
- parser source aggregation code
- compiler API/parser integration tests

Validation:

- `cargo test -p ink-test --test compiler_api`
- focused multi-source parser tests
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-test --test compiler_api` passed.
- `cargo test -p ink-test --test compiler_api public_parse_sources` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 685e763

## Milestone 3: Module Namespace And Import Analysis

### [x] Task 09: Build Module Symbol Index

Goal:

Create the semantic index for modules and per-module namespace symbols.

Implementation method:

- Add a module symbol index covering knots, functions, constants, global
  variables, structs, and externals.
- Track symbol kind, declaring module, source span, and available type or
  signature metadata.
- Exclude stitches, tags, labels, gathers, choices, temps, arguments, and local
  declarations from the module namespace.
- Add index unit tests for every importable declaration kind.

Acceptance criteria:

- Module symbol lookup can answer kind, module, span, and type/signature data.
- Non-module symbols are not present in the module namespace.

Modification boundary:

- `crates/ink-compiler/src/analysis`
- analysis tests

Validation:

- focused module symbol index tests
- `cargo test -p ink-compiler analysis::`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler analysis::modules` passed.
- `cargo test -p ink-compiler analysis::` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: b6fc00e

### [x] Task 10: Diagnose Duplicate Modules And Namespace Collisions

Goal:

Enforce duplicate module and per-module namespace rules.

Implementation method:

- Diagnose duplicate module declarations across all source inputs and within the
  same source.
- Diagnose collisions between knots, functions, constants, globals, structs,
  and externals within the same module.
- Allow the same symbol name in different modules.
- Preserve stitch name reuse in different knots.
- Add positive and negative diagnostics fixtures.

Acceptance criteria:

- Duplicate modules fail compilation.
- Same-module symbol reuse fails across declaration kinds.
- Same-name symbols in different modules pass unless another rule fails.
- Stitch names do not collide at module level.

Modification boundary:

- module/naming analysis
- diagnostic tests and fixtures

Validation:

- focused duplicate module/name diagnostics tests
- `cargo test -p ink-compiler analysis::names`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler analysis::modules` passed.
- `cargo test -p ink-compiler analysis::names` passed.
- `cargo test -p ink-compiler analysis::constants` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 5853d4b

### [x] Task 11: Enforce Main Entry Point Rules

Goal:

Require exactly one explicit module to define a knot named `main`.

Implementation method:

- Add analysis for missing `main` and multiple `main` knots.
- Identify the runnable entry point as `moduleName::main`.
- Make sure the main module receives no special cross-module visibility.
- Add tests for missing main, one valid main, and multiple mains across modules.

Acceptance criteria:

- Missing `main` is a compile error.
- Multiple `main` knots are compile errors.
- The checked story records the unique module-qualified entry point.

Modification boundary:

- module/flow analysis
- checked story metadata
- diagnostics tests

Validation:

- focused entry-point diagnostics tests
- `cargo test -p ink-compiler analysis::`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler analysis::modules` passed.
- `cargo test -p ink-compiler analysis::` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: bf3db0b

### [x] Task 12: Build Import Dependency Graph And Cycle Diagnostics

Goal:

Analyze module dependencies from imports independent of source order.

Implementation method:

- Build a directed graph from each module's imports.
- Detect direct and transitive import cycles.
- Report actionable cycle diagnostics with involved modules where practical.
- Add tests for acyclic imports, direct cycles, transitive cycles, and source
  order independence.

Acceptance criteria:

- Dependency analysis does not use text concatenation order.
- Cycles fail before lowering.
- Acyclic imports produce deterministic dependency metadata.

Modification boundary:

- module dependency analysis
- diagnostics tests

Validation:

- focused dependency/cycle tests
- `cargo test -p ink-compiler analysis::`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler analysis::modules` passed.
- `cargo test -p ink-compiler analysis::` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 275236a

### [x] Task 13: Validate Imports And Reachability Warnings

Goal:

Validate exact import allow-lists and compute modules reachable from main.

Implementation method:

- Validate missing imported modules and missing imported symbols.
- Reject unsupported imported kinds, including stitches and tags.
- Store direct import allow-lists for later qualified lookup.
- Compute modules reachable from the main module's transitive import path.
- Warn for unreachable modules while still semantically checking them.
- Add unused import warning support for currently observable qualified uses.
- Add tests for invalid imports, unused imports, unreachable modules, and
  unreachable modules with errors.

Acceptance criteria:

- Invalid imports fail before expression/lowering resolution.
- Unused imports warn without blocking successful compile.
- Unreachable modules warn, are checked, and are not treated as reachable.
- Errors in unreachable modules still fail compilation.

Modification boundary:

- import analysis
- warning diagnostics
- checked story metadata
- diagnostics tests

Validation:

- `cargo test -p ink-compiler analysis::modules` passed.
- `cargo test -p ink-compiler analysis::` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 17ec08a

## Milestone 4: Qualified Names And Resolution

### [x] Task 14: Parse Source-Level Qualified Names

Goal:

Parse `module::symbol` in every supported source position.

Implementation method:

- Add a parsed qualified-name representation.
- Parse qualified names in expressions, type names, assignment targets,
  function calls, external calls, diverts, tunnels, and dynamic divert targets.
- Keep `::` as source syntax only; do not treat it as a runtime path separator.
- Add parser tests for qualified vars, constants, structs, functions, externals,
  knots, and invalid `::` forms.

Acceptance criteria:

- Qualified names preserve module and symbol spans.
- Invalid qualified syntax fails with clear diagnostics.
- Existing dotted stitch/path syntax remains distinct from module qualification.

Modification boundary:

- syntax expression/type/target/divert parsing
- parsed expression/type/target models
- parser tests

Validation:

- `cargo test -p ink-compiler syntax::` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 2c02e30

### [x] Task 15: Make Unqualified Lookup Module-Local

Goal:

Remove story-global unqualified lookup for module-scoped symbols.

Implementation method:

- Update variable, constant, struct, flow, function, and external lookup to use
  the current module for unqualified names.
- Preserve local temps and arguments as closer scopes inside flows/functions.
- Add tests showing sibling module symbols are not visible unqualified.
- Add tests showing same-module symbols still resolve unqualified.

Acceptance criteria:

- Unqualified lookup never searches sibling modules.
- Imported symbols are not introduced unqualified.
- Existing local temp/argument shadowing rules still work inside the module.

Modification boundary:

- analysis variable/type/target/call resolution
- focused resolution tests

Validation:

- focused module-local lookup tests passed.
- `cargo test -p ink-compiler analysis::` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: fad4ec8

### [x] Task 16: Enforce Qualified Import Allow-Lists

Goal:

Require direct exact imports for cross-module `module::symbol` references.

Implementation method:

- Resolve qualified references against the current module's direct import
  allow-list.
- Reject missing direct imports and non-transitive import access.
- Decide and test same-module self-qualified behavior consistently with the
  implementation design.
- Add tests for direct imports, missing imports, transitive-only imports, and
  wrong-module qualified names.

Acceptance criteria:

- Cross-module qualified references require direct explicit imports.
- Imports are not transitive.
- The module containing `main` has no special visibility privileges.

Modification boundary:

- import-aware analysis resolution
- diagnostics tests

Validation:

- focused import allow-list tests passed.
- `cargo test -p ink-compiler analysis::` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: c1c4039

### [x] Task 17: Preserve Same-Module Stitch Lookup And Reject Cross-Module Stitches

Goal:

Keep stitch scoping unchanged inside a module while blocking direct
cross-module stitch access.

Implementation method:

- Preserve same-module `knot.stitch` addressing.
- Preserve relative stitch shorthand inside parent knots.
- Reject `otherModule::knot.stitch` and equivalent cross-module direct stitch
  access.
- Add positive same-module stitch tests and negative cross-module stitch tests.

Acceptance criteria:

- Stitches remain scoped to parent knots.
- Different knots in one module may reuse stitch names.
- Imported knots do not expose child stitches directly.

Modification boundary:

- target/flow analysis
- parser or target tests as needed

Validation:

- focused stitch resolution tests passed.
- `cargo test -p ink-compiler analysis::targets` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 041aae2

## Milestone 5: Imported Values, Types, Calls, And Assignments

### [x] Task 18: Resolve Qualified Constants And Structs

Goal:

Make qualified constants and structs participate in expression and type
analysis.

Implementation method:

- Resolve qualified constants in expressions and initializers.
- Resolve qualified struct types and struct literals where imported.
- Keep constants compile-time and struct values dynamic without runtime type
  metadata.
- Add positive and negative tests for imported constants, imported struct types,
  struct literals, defaults, and type errors.

Acceptance criteria:

- Qualified constants and structs type-check correctly.
- Missing imports and wrong types fail in analysis.
- No runtime schema fields are added for constant or struct type metadata.

Modification boundary:

- expression/type analysis
- struct/constant analysis tests

Validation:

- focused qualified const/struct tests passed.
- `cargo test -p ink-compiler analysis::` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 1ef7f4a

### [x] Task 19: Support Imported Global Variable Reads And Writes

Goal:

Allow imported module variables to be read and written through qualified names.

Implementation method:

- Resolve `module::var` reads to the declaring module variable.
- Allow assignment and supported compound assignment forms to imported globals.
- Reject writes to constants and unsupported qualified targets.
- Add runtime tests showing multiple modules share the same underlying variable.

Acceptance criteria:

- Imported variable reads and writes affect the declaring module's story-wide
  variable.
- Compound assignment follows existing local/global assignment semantics.
- Invalid imported writes fail in analysis.

Modification boundary:

- assignment/variable analysis
- lowering support needed for qualified global references
- runtime language tests

Validation:

- focused imported variable assignment tests
- `cargo test -p ink-test --test language module`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler analysis::` passed.
- `cargo test -p ink-compiler syntax::text::` passed.
- `cargo test -p ink-test --test language module` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 3996932

### [x] Task 20: Support Qualified Function And External Calls

Goal:

Resolve and type-check qualified Ink function calls and external calls.

Implementation method:

- Resolve qualified function calls through direct imports.
- Resolve qualified external calls through direct imports.
- Type-check arguments and return values using declaring module signatures.
- Add runtime tests for imported functions and external host bindings.

Acceptance criteria:

- Qualified Ink calls execute correctly.
- Qualified external calls require host bindings with source-qualified names
  such as `audio::play`.
- Imported calls are not visible unqualified.

Modification boundary:

- call/target analysis
- lowering support for qualified calls if needed
- runtime integration tests

Validation:

- focused qualified call tests
- `cargo test -p ink-test --test language module`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler analysis::targets::` passed.
- `cargo test -p ink-test --test language module` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 2e2bdcd

## Milestone 6: Module Lowering And Runtime Compatibility

### [x] Task 21: Carry Module Metadata Into Checked Story

Goal:

Give lowering a checked module view instead of recomputing module semantics from
raw parsed story data.

Implementation method:

- Extend checked-story metadata with main module, reachable modules, module
  symbol mappings, import metadata, and qualified-name resolution results.
- Keep analysis as the owner of semantic validation.
- Add tests proving lowering can access the checked module view.

Acceptance criteria:

- Lowering receives the unique module entry point and reachable module set from
  analysis.
- Lowering does not redo import validation or reachability analysis.

Modification boundary:

- `CheckedStory`
- analysis output metadata
- focused analysis/lowering support tests

Validation:

- focused checked-story metadata tests
- `cargo test -p ink-compiler analysis::`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler analysis::modules::tests::checked_story_records_module_symbol_metadata` passed.
- `cargo test -p ink-compiler lower::tests::module_lowering_uses_checked_entry_point_and_reachability` passed.
- `cargo test -p ink-compiler analysis::` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: d44e683

### [x] Task 22: Lower Module Containers And Root Entry

Goal:

Emit reachable module containers and start execution at `module::main`.

Implementation method:

- Emit root named content with one container per reachable module.
- Emit module knots/functions as child named containers.
- Emit root content that diverts to runtime path `module.main` and then
  completes normally.
- Add JSON snapshot tests for module containers and root entry divert.

Acceptance criteria:

- Runnable stories start at the unique `module::main`.
- Module containers do not collide for same-named flows in different modules.
- Emitted JSON uses existing format crate structures.

Modification boundary:

- `crates/ink-compiler/src/lower`
- compiler JSON snapshots/tests

Validation:

- focused module lowering snapshot tests
- `cargo test -p ink-test --test compiler_conformance`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler lower::tests::module_lowering_json_contains_root_entry_and_reachable_module_containers` passed.
- `cargo test -p ink-test --test compiler_conformance` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: fbbead9

### [x] Task 23: Map Qualified Flow Paths To Runtime Paths

Goal:

Map source `module::flow` semantics onto existing runtime dot paths.

Implementation method:

- Map qualified flow diverts to runtime `module.flow` paths.
- Update function calls, tunnels, dynamic divert-target values, and read/visit
  count paths where they refer to module flows.
- Preserve same-module relative stitch behavior.
- Add runtime navigation tests for qualified diverts, tunnels, and functions.

Acceptance criteria:

- Runtime loads and navigates module-qualified flow paths.
- `::` is not exposed as a runtime path separator.
- Existing same-module stitch runtime behavior remains intact.

Modification boundary:

- lower path logic
- runtime/compiler integration tests

Validation:

- focused qualified path runtime tests
- `cargo test -p ink-runtime`
- `cargo test -p ink-test --test language module`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-test --test language module` passed.
- `cargo test -p ink-runtime` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 8813842

### [x] Task 24: Lower Module Globals And Externals With Qualified Names

Goal:

Preserve runtime compatibility while avoiding cross-module global and external
name collisions.

Implementation method:

- Lower module globals as stable source-qualified names such as `items::count`.
- Update variable reference and assignment lowering for qualified globals.
- Lower module external declarations and calls using source-qualified host names
  such as `audio::play`.
- Add save/load or variable-state tests for module-qualified global names.
- Add external binding tests for module-qualified external names.

Acceptance criteria:

- Same-named globals in different modules do not collide.
- Imported writes affect the declaring module's variable.
- Host external binding uses names such as `audio::play`.
- Save-state behavior preserves module-qualified global names.

Modification boundary:

- lower variable/external logic
- runtime save/load tests if needed
- language/runtime integration tests

Validation:

- focused module global/external runtime tests
- `cargo test -p ink-test --test language module`
- `cargo test -p ink-runtime`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-test --test language module` passed.
- `cargo test -p ink-runtime` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 084aefa

### [x] Task 25: Exclude Unreachable Modules And Verify Format Roundtrips

Goal:

Emit only reachable modules while preserving compiled story JSON schema
compatibility.

Implementation method:

- Filter lowering to the modules reachable from main's import graph.
- Keep unreachable-module warnings and semantic errors from analysis intact.
- Add tests proving unreachable valid modules do not appear in emitted JSON.
- Add format crate roundtrip tests for module-shaped containers if existing
  coverage does not already prove this shape.

Acceptance criteria:

- Unreachable modules are not emitted into final runnable JSON.
- Unreachable module errors still fail compilation.
- Module-shaped JSON roundtrips through `ink-story-json-format`.
- No compiled story JSON schema change is introduced.

Modification boundary:

- lowering input selection
- format roundtrip tests
- compiler JSON tests

Validation:

- focused unreachable-module lowering tests
- focused format roundtrip tests
- `cargo test -p ink-story-json-format`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler lower::tests::module_` passed.
- `cargo test -p ink-story-json-format roundtrips_module_shaped_named_content_without_schema_changes` passed.
- `cargo test -p ink-story-json-format` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 8d2bdc9

## Milestone 7: Fixture, Compatibility, Docs, And Editor Migration

### [x] Task 26: Migrate Maintained Fixtures To Explicit Modules

Goal:

Update maintained language and conformance fixtures to the module language
model.

Implementation method:

- Migrate maintained `.ink` fixtures to explicit modules with exactly one
  `main` knot where runnable.
- Fix lowering paths only where moduleized maintained fixtures expose incorrect
  same-module runtime-name resolution for globals, constants, structs,
  functions, externals, or tail-recursive function lowering.
- Update parse and JSON expected fixtures only where module ownership, root
  entry lowering, module containers, or qualified names intentionally changed.
- Preserve existing story behavior unless the requirement plan explicitly
  removes it.

Acceptance criteria:

- Maintained fixtures compile under explicit module rules.
- Snapshot changes correspond to documented module behavior.
- No obsolete root direct content fixture remains as current language behavior.

Modification boundary:

- `crates/ink-test/fixtures`
- `crates/ink-test/tests/language.rs`
- `crates/ink-compiler/src/lower.rs`
- `crates/ink-compiler/src/lower/`
- compiler conformance generated expectations

Validation:

- `cargo test -p ink-test --test compiler_conformance`
- `cargo test -p ink-test --test conformance`
- `cargo test -p ink-test --test language`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-test --test compiler_conformance` passed.
- `cargo test -p ink-test --test conformance` passed.
- `cargo test -p ink-test --test language` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 078e58c

### [x] Task 27: Migrate Inline And C# Compatibility Tests

Goal:

Rewrite compatibility coverage to reflect intentional module-support
divergences without hiding unrelated legacy behavior.

Implementation method:

- Rewrite inline language and `inkling_examples` tests that depend on root
  direct content, unnamed roots, root/global tags, or story-global lookup.
- Rewrite or remove upstream C# compatibility tests for behavior intentionally
  removed by module support: includes, root direct content, global tags, and
  visibility leakage.
- Preserve unchanged C# compatibility behavior by migrating source examples to
  modules where appropriate.
- Add explicit diagnostics tests for removed behavior instead of skipped tests.

Acceptance criteria:

- C# compatibility divergence is intentional and visible.
- No compatibility test is skipped merely to hide module failures.
- Inline tests describe the new module language model.

Modification boundary:

- `crates/ink-test/tests`
- C# compatibility fixtures
- module diagnostics tests

Validation:

- `cargo test -p ink-test --test inkling_examples`
- `cargo test -p ink-test --features csharp-tests --test csharp_tests`
- `cargo test -p ink-test --test language`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-test --test inkling_examples` passed.
- `cargo test -p ink-test --features csharp-tests --test csharp_tests` passed.
- `cargo test -p ink-test --test language` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 6195e5c

### [x] Task 28: Update Maintained Docs With Tested Module Examples

Goal:

Make maintained writing and architecture docs describe modules as current
language behavior, with examples backed by fixtures or tests.

Implementation method:

- Update `docs/WritingWithInk-updates.md` first with module/import behavior and
  intentional upstream divergence.
- Apply the changes to `docs/WritingWithInk-latest.md`.
- Update `docs/Architecture.md` and `docs/ink_JSON_runtime_format.md` for
  source input, parsed modules, analysis, lowering, runtime path mapping, JSON
  compatibility, and save-state notes.
- Add or update small docs-example fixtures/tests for the primary module/import
  examples so this is not a prose-only task.
- Do not edit `WritingWithInk-origin.md`.

Acceptance criteria:

- Maintained docs no longer present `INCLUDE`, root direct content,
  module-level tags, or global visibility leaks as current behavior.
- Module examples in maintained docs are represented by passing tests or
  fixtures.
- Architecture docs match implemented code.

Modification boundary:

- maintained docs
- docs-example fixtures/tests

Validation:

- focused docs-example tests
- `cargo test -p ink-test --test language`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-test --test language docs_module_import_example_runs` passed.
- `cargo test -p ink-test --test language` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: b045079

### [x] Task 29: Update Editor Grammar For Module Syntax

Goal:

Update editor assets so module syntax is highlighted and removed syntax is not
promoted.

Implementation method:

- Highlight module headers, lowercase `module`, uppercase `IMPORT` and `FROM`,
  and `::` qualified names.
- Remove current-language `INCLUDE` highlighting or convert it to removed-syntax
  treatment if the grammar supports that distinction.
- Add or update editor grammar tests/examples if available.
- Keep unrelated editor grammar behavior unchanged.

Acceptance criteria:

- Editor assets recognize module declarations and imports.
- Editor assets do not present `INCLUDE` as current import syntax.
- Existing editor syntax coverage remains intact.

Modification boundary:

- editor grammar/assets
- editor tests/examples if present

Validation:

- focused editor validation command if present
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `node -e "const fs=require('fs'); for (const path of ['editor/vscode-ink-rs/package.json','editor/vscode-ink-rs/language-configuration.json','editor/vscode-ink-rs/syntaxes/ink.tmLanguage.json']) JSON.parse(fs.readFileSync(path,'utf8'));"` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 46a1d24

## Milestone 8: Closeout

### [x] Task 30: Final Validation And Plan Closeout

Goal:

Verify module support end to end and move the active plan to finished plans.

Implementation method:

- Review `requirement_plan.md` against implemented behavior and update only for
  actual clarified requirements or documented compatibility impact.
- Confirm every task has `[x]`, validation status, and a commit hash.
- Run focused parser, analysis, lowering, runtime, fixture, docs, and editor
  validation from prior tasks.
- Run broad validation.
- Move `docs/current_plan/module_support/` to
  `docs/finished_plans/module_support/` only after validation passes.

Acceptance criteria:

- Every requirement is implemented, explicitly deferred with owner approval, or
  documented as changed.
- Full validation passes.
- No active module-support plan remains under `docs/current_plan/`.

Modification boundary:

- plan directory movement
- closeout documentation updates

Validation:

- focused validation commands from Tasks 01-29
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `make gate`

Validation status:

- `cargo test -p ink-compiler syntax::module` passed.
- `cargo test -p ink-compiler analysis::modules` passed.
- `cargo test -p ink-compiler lower::tests::module_` passed.
- `cargo test -p ink-test --test language module` passed.
- `cargo test -p ink-test --test compiler_conformance` passed.
- `cargo test -p ink-test --test conformance` passed.
- `node -e "const fs=require('fs'); for (const path of ['editor/vscode-ink-rs/package.json','editor/vscode-ink-rs/language-configuration.json','editor/vscode-ink-rs/syntaxes/ink.tmLanguage.json']) JSON.parse(fs.readFileSync(path,'utf8'));"` passed.
- `cargo fmt --all --check` passed.
- `cargo check --workspace` passed.
- `cargo test --workspace` passed.
- `make gate` passed.

Commit: 0eb4bfe
