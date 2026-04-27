Progress: 3/24

# Task List: Compiler And Runtime Refactor

Status key: `[ ]` pending, `[~]` in progress, `[x]` complete, `[!]` blocked.

This task list is an implementation plan, not a research log. Except for final
plan closeout, every task must edit production code, test code, fixtures, editor
assets, or another code-adjacent artifact. Maintained docs should be updated in
the same task as behavior changes, but this plan is intended to preserve
behavior.

Completion protocol for every task:

- Update the task checkbox from `[ ]` or `[~]` to `[x]`.
- Update `Progress: X/24`.
- Run the task's focused validation.
- Run `make gate`.
- Commit immediately after validation passes.
- Record the commit hash in that task's `Commit` field before starting another
  task.

Do not append notes, logs, or running commentary after this task list. Record
validation and commit metadata inside the relevant task section.

Global forbidden boundaries:

- Do not change Ink syntax, parser behavior, module semantics, runtime story
  execution behavior, or compiled story JSON wire compatibility.
- Do not add fixture-name checks, fixture-path checks, expected-output checks,
  test-order dependencies, ignored tests, skip filters, or compatibility shims.
- Do not move runtime execution objects into `ink-story-json-format`.
- Do not move runtime save-state ownership into `ink-story-json-format`.
- Do not make broad formatting churn outside files touched by a task.
- Do not change public APIs unless the task explicitly calls for it and tests
  the compatibility impact.

## Milestone 1: Lowering Context

### [x] Task 01: Introduce A Shared Lowering Context

Goal:

Add a small compiler lowering context type that groups the read-only indexes,
label maps, variable sets, constants, struct definitions, and current
`ChoicePathMode` currently passed through lowering helpers.

Implementation method:

- Add the context type under `crates/ink-compiler/src/lower/context.rs` or a
  nearby lowering module.
- Keep the first slice conservative: construct the context in root/module/flow
  lowering and use it in one narrow call chain.
- Preserve the existing `LoweringIndexes` builder and JSON output.
- Add or update focused lowering tests proving the selected call chain emits the
  same JSON.

Acceptance criteria:

- The context type is used by at least one production lowering path.
- No compiled story JSON snapshots or conformance output change.
- Existing lowering APIs remain callable until later migration tasks remove the
  old parameter bundles.

Forbidden shortcuts:

- Do not introduce global mutable lowering state.
- Do not use the context to smuggle fixture-specific behavior.

Modification boundary:

- `crates/ink-compiler/src/lower.rs`
- `crates/ink-compiler/src/lower/context.rs`
- focused compiler lowering tests

Validation:

- `cargo test -p ink-compiler lower::`
- `cargo test -p ink-test --test language`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler lower::` passed.
- `cargo test -p ink-test --test language` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: ab4791c9c3c39e07c2d1af8be69eec7546d5eaad

### [x] Task 02: Migrate Expression Lowering To The Context

Goal:

Replace the repeated expression-lowering parameter bundle with the shared
lowering context.

Implementation method:

- Update `lower/expression.rs` public-in-module helpers to accept the context
  rather than separate labels, variables, signatures, constants, struct
  definitions, and path mode.
- Keep the constant-recursion guard local to expression lowering.
- Update callers in conditional, sequence, weave, assignment, divert, and root
  lowering as needed.
- Add focused tests for function calls, constants, string content, field access,
  index access, and module-qualified references.

Acceptance criteria:

- Expression lowering no longer exposes 9-12 argument helper signatures.
- Current expression JSON output is unchanged.
- Recursive constant protection still works.

Forbidden shortcuts:

- Do not special-case individual operator tokens or fixtures.
- Do not change expression type analysis.

Modification boundary:

- `crates/ink-compiler/src/lower/expression.rs`
- direct lowering callers needed for compilation
- focused expression/lowering tests

Validation:

- `cargo test -p ink-compiler lower::`
- `cargo test -p ink-test --test language expression`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler lower::` passed.
- `cargo test -p ink-test --test language expression` passed with no matching
  tests; `cargo test -p ink-test --test language` passed as the effective
  language coverage.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 436fce33d3a68b6396078639b8c5eb90ec6ee339

### [x] Task 03: Move Assignment Lowering Behind The Context

Goal:

Extract assignment and increment/decrement lowering from the root lowering file
and make it use the shared lowering context.

Implementation method:

- Create `crates/ink-compiler/src/lower/assignment.rs`.
- Move assignment path collection, assignment path read/write, cached index,
  reassignment, variable assignment, and inc/dec lowering there.
- Replace raw parameter bundles with the context.
- Keep helper visibility restricted to the lowering module.
- Add focused tests for local assignment, global/module global assignment,
  compound assignment, field assignment, index assignment, and array removal
  assignment lowering.

Acceptance criteria:

- Assignment lowering code is no longer owned by `lower.rs`.
- Nested lvalue update order and cached index behavior are unchanged.
- Module-qualified variable writes still target the same runtime variable names.

Forbidden shortcuts:

- Do not change assignment diagnostics.
- Do not change field/index runtime operation order.

Modification boundary:

- `crates/ink-compiler/src/lower.rs`
- `crates/ink-compiler/src/lower/assignment.rs`
- `crates/ink-compiler/src/lower/expression.rs` if it calls assignment helpers
- focused compiler and language tests

Validation:

- `cargo test -p ink-compiler lower::`
- `cargo test -p ink-test --test language assignment`
- `cargo test -p ink-test --test language module`
- `cargo fmt --all --check`
- `make gate`

Validation status:

- `cargo test -p ink-compiler lower::` passed.
- `cargo test -p ink-test --test language assignment` passed.
- `cargo test -p ink-test --test language module` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.

Commit: 6fa7cba636d27708a223a88cd265858dc78518a1

### [ ] Task 04: Move Divert And Return Lowering Behind The Context

Goal:

Extract divert, dynamic divert, tunnel onwards, and tail-recursive return
lowering from `lower.rs` and make those paths use the shared context.

Implementation method:

- Create `crates/ink-compiler/src/lower/divert.rs` or split return lowering
  into a nearby file if that is clearer.
- Move static divert, dynamic divert, tunnel onwards, and tail-recursive return
  lowering helpers.
- Keep `runtime_divert` construction behavior unchanged.
- Add focused tests for static diverts, dynamic diverts, tunnels, tunnel
  onwards, function calls, external calls, and tail-recursive returns.

Acceptance criteria:

- Divert-related lowering code is separated from root object dispatch.
- Dynamic divert argument evaluation order is unchanged.
- Tail-recursive return optimization still emits the same runtime objects.

Forbidden shortcuts:

- Do not change source-level divert resolution semantics.
- Do not change runtime path compaction.

Modification boundary:

- `crates/ink-compiler/src/lower.rs`
- `crates/ink-compiler/src/lower/divert.rs`
- focused compiler/language tests

Validation:

- `cargo test -p ink-compiler lower::`
- `cargo test -p ink-test --test language divert`
- `cargo test -p ink-test --test language module`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 05: Finish Lowering Context Migration Across Weave, Sequence, And Conditional Code

Goal:

Complete context migration in the remaining lowering modules so broad lowering
helpers no longer carry repeated index bundles.

Implementation method:

- Update `lower/weave.rs`, `lower/sequence.rs`, and `lower/conditional.rs` to
  use the shared context.
- Keep current per-container derived path modes explicit through context cloning
  or scoped context helpers.
- Remove obsolete parameter-bundle helper overloads after all callers migrate.
- Add or update tests for choices, gathers, nested weaves, sequences, and
  multiline conditionals.

Acceptance criteria:

- Lowering helper signatures are substantially shorter across all lowering
  modules.
- Choice/gather path targets and count metadata remain unchanged.
- No old context-bundle wrapper remains unless it has a named compatibility
  purpose inside the lowering module.

Forbidden shortcuts:

- Do not collapse distinct path modes into one mutable global.
- Do not change sequence, gather, or choice target naming.

Modification boundary:

- `crates/ink-compiler/src/lower.rs`
- `crates/ink-compiler/src/lower/context.rs`
- `crates/ink-compiler/src/lower/weave.rs`
- `crates/ink-compiler/src/lower/sequence.rs`
- `crates/ink-compiler/src/lower/conditional.rs`
- focused compiler/language tests

Validation:

- `cargo test -p ink-compiler lower::`
- `cargo test -p ink-test --test language choice`
- `cargo test -p ink-test --test language sequence`
- `cargo test -p ink-test --test language module`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

## Milestone 2: Typed Native Function Tokens

### [ ] Task 06: Add Format-Crate Native Function Metadata

Goal:

Introduce a typed native function token model in `ink-story-json-format` without
changing the public `Object::NativeFunction` representation yet.

Implementation method:

- Add a `NativeFunction` enum or equivalent typed token model in the format
  crate.
- Implement `from_token`, `token`, and arity metadata where useful.
- Cover every token currently accepted by runtime `NativeFunctionCall`.
- Add unit tests for token round trips and unknown-token rejection at the typed
  metadata layer.

Acceptance criteria:

- The format crate owns the complete native function token list.
- Existing JSON serialization and deserialization still use the old object
  representation until the next task.
- Tests prove token coverage matches runtime-supported native functions.

Forbidden shortcuts:

- Do not duplicate token constants in compiler code.
- Do not drop tokens that are currently accepted by runtime loading.

Modification boundary:

- `crates/ink-story-json-format/src/model.rs`
- `crates/ink-story-json-format/src/json.rs`
- `crates/ink-story-json-format/src/lib.rs` if exports are needed
- format crate tests

Validation:

- `cargo test -p ink-story-json-format`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 07: Make Compiled Story Objects Use Typed Native Functions

Goal:

Change compiled-story format objects from raw native-function strings to the
typed native-function model while preserving the same JSON wire tokens.

Implementation method:

- Change `ink_story_json_format::Object::NativeFunction` to hold the typed
  native-function value.
- Update JSON parsing so known native function tokens parse to typed objects and
  unknown non-command string tokens produce clear `FormatError`s.
- Update JSON serialization to emit exactly the same token strings as before.
- Update format crate tests for native operation round trips and unknown tokens.

Acceptance criteria:

- Format object model no longer stores native function names as arbitrary
  strings.
- Existing compiled story JSON with supported native tokens still loads.
- Unsupported native function tokens fail with a clear format error.

Forbidden shortcuts:

- Do not change control-command token handling.
- Do not change text token handling for strings beginning with `^`.

Modification boundary:

- `crates/ink-story-json-format/src/model.rs`
- `crates/ink-story-json-format/src/json.rs`
- downstream compiler/runtime compile fixes required by the type change
- format crate tests

Validation:

- `cargo test -p ink-story-json-format`
- `cargo test -p ink-runtime json`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 08: Update Compiler Lowering To Emit Typed Native Functions

Goal:

Remove compiler-owned native token string construction from lowering and emit
typed format native function objects instead.

Implementation method:

- Replace raw native function string literals in lowering with typed
  `ink_story_json_format` native function values.
- Keep operator-to-token mapping centralized in the format crate or a compiler
  helper that returns typed values.
- Update tests for operators, field/index operations, `LEN`, `ARRAY_REMOVE`,
  sequence math, and conditional comparisons.

Acceptance criteria:

- Compiler lowering no longer constructs `RuntimeObject::NativeFunction` from
  ad hoc strings.
- Compiled JSON output remains wire-compatible.
- Operator lowering remains left-associative and preserves existing precedence.

Forbidden shortcuts:

- Do not hardcode fixture JSON fragments.
- Do not move expression operator parsing into the format crate.

Modification boundary:

- `crates/ink-compiler/src/lower`
- `crates/ink-compiler/src/analysis/expression_types.rs` only if builtin token
  lookup needs typed metadata
- focused compiler tests

Validation:

- `cargo test -p ink-compiler lower::`
- `cargo test -p ink-test --test language expression`
- `cargo test -p ink-test --test language module`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 09: Update Runtime Loading And Writing For Typed Native Functions

Goal:

Make runtime compiled-story loading and runtime JSON writing consume and produce
typed native function objects.

Implementation method:

- Update `json_read.rs` to map typed format native functions to runtime
  `NativeFunctionCall` operations.
- Update `json_write.rs` to convert runtime native functions back to typed
  format native functions without raw JSON round trips where practical.
- Keep save-state JSON behavior unchanged.
- Add focused runtime tests for native function load/write round trips.

Acceptance criteria:

- Runtime no longer depends on raw native function token strings from format
  objects.
- Unknown native function handling is explicit and tested.
- Existing compiled story JSON continues to load.

Forbidden shortcuts:

- Do not change runtime operation semantics.
- Do not conflate compiled-story JSON with save-state JSON.

Modification boundary:

- `crates/ink-runtime/src/json/json_read.rs`
- `crates/ink-runtime/src/json/json_write.rs`
- `crates/ink-runtime/src/native_function_call.rs`
- runtime JSON/native function tests

Validation:

- `cargo test -p ink-runtime json`
- `cargo test -p ink-runtime native_function_call`
- `cargo test -p ink-test --test conformance`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

## Milestone 3: Module Analysis Structure

### [ ] Task 10: Extract Module Symbol Indexing

Goal:

Move module symbol kinds, symbols, signatures, struct fields, symbol index
construction, and namespace collision diagnostics out of the monolithic module
analysis file.

Implementation method:

- Convert `analysis/modules.rs` into a module directory if needed.
- Add `analysis/modules/symbols.rs` for symbol data and symbol diagnostics.
- Preserve public exports used by `CheckedStory` and other analysis passes.
- Keep tests for symbol indexing and duplicate namespace diagnostics with the
  extracted module.

Acceptance criteria:

- Symbol indexing has a focused file and ownership boundary.
- `ModuleSymbolIndex` remains available where currently used.
- Namespace collision diagnostic messages and locations are unchanged.

Forbidden shortcuts:

- Do not relax duplicate-name checks.
- Do not hide module symbol types behind string maps that lose kind/type data.

Modification boundary:

- `crates/ink-compiler/src/analysis/modules.rs`
- `crates/ink-compiler/src/analysis/modules/symbols.rs`
- module analysis tests

Validation:

- `cargo test -p ink-compiler analysis::modules`
- `cargo test -p ink-test --test language module`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 11: Extract Dependency Graph And Reachability Analysis

Goal:

Move import dependency collection, cycle diagnostics, reachability, and
unreachable-module diagnostics into a focused module.

Implementation method:

- Add `analysis/modules/dependencies.rs` or `reachability.rs`.
- Move `ModuleDependencyGraph`, `ModuleReachability`, cycle discovery, cycle
  formatting, and unreachable module diagnostics.
- Preserve dependency ordering and diagnostic sorting.
- Keep focused tests for simple dependencies, cycles, duplicate cycles, and
  unreachable modules.

Acceptance criteria:

- Dependency and reachability code no longer lives in the same file as symbol
  indexing.
- Cycle and unreachable diagnostics are unchanged.
- Entry-module reachability behavior remains tied to the unique `main` module.

Forbidden shortcuts:

- Do not make imports transitive.
- Do not emit unreachable modules in compiled story JSON.

Modification boundary:

- `crates/ink-compiler/src/analysis/modules.rs`
- `crates/ink-compiler/src/analysis/modules/dependencies.rs`
- module analysis/language tests

Validation:

- `cargo test -p ink-compiler analysis::modules`
- `cargo test -p ink-test --test language module`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 12: Extract Import Use Checking

Goal:

Move module import validation, qualified-use collection, unused import warnings,
and import allow-list diagnostics into a focused module.

Implementation method:

- Add `analysis/modules/imports.rs`.
- Move `ModuleImportIndex`, qualified-use collection across objects,
  expressions, assignment targets, divert targets, and type names.
- Preserve sorted diagnostics.
- Add focused tests for unused imports, missing source modules, missing imported
  symbols, stitch import rejection, and qualified-use allow-list checks.

Acceptance criteria:

- Import validation has a focused file and does not rebuild unrelated indexes
  internally when the caller can provide them.
- Existing import diagnostics and warnings are unchanged.
- Qualified assignment and type-name uses remain covered.

Forbidden shortcuts:

- Do not introduce unqualified visibility for imported symbols.
- Do not make imports transitive.

Modification boundary:

- `crates/ink-compiler/src/analysis/modules.rs`
- `crates/ink-compiler/src/analysis/modules/imports.rs`
- module analysis tests

Validation:

- `cargo test -p ink-compiler analysis::modules`
- `cargo test -p ink-test --test language module`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 13: Extract Entry Point And Mixed Root Diagnostics

Goal:

Move module entry-point discovery, multiple-main diagnostics, missing-main
diagnostics, and mixed root/module diagnostics into a focused module.

Implementation method:

- Add `analysis/modules/entry_point.rs`.
- Move entry point and mixed root/module diagnostic helpers.
- Preserve `ModuleEntryPoint` ownership in `analysis/mod.rs` unless a cleaner
  public boundary is needed.
- Keep tests for no-main, multiple-main, valid single main, and root-content
  with modules.

Acceptance criteria:

- Entry-point diagnostics are isolated from dependency and import diagnostics.
- Diagnostic text and source spans remain unchanged.
- `CheckedStory.entry_point` behavior remains unchanged.

Forbidden shortcuts:

- Do not introduce library-only compilation in this refactor.
- Do not change `main` uniqueness rules.

Modification boundary:

- `crates/ink-compiler/src/analysis/mod.rs`
- `crates/ink-compiler/src/analysis/modules.rs`
- `crates/ink-compiler/src/analysis/modules/entry_point.rs`
- module analysis tests

Validation:

- `cargo test -p ink-compiler analysis::modules`
- `cargo test -p ink-test --test language module`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 14: Share Module Analysis Indexes Across Diagnostics And CheckedStory

Goal:

Avoid rebuilding module symbol, dependency, import, entry point, and reachability
indexes separately for diagnostics and `CheckedStory`.

Implementation method:

- Introduce an internal `ModuleAnalysis` or broader `AnalysisIndexes` value
  built once during analysis.
- Pass borrowed indexes into module diagnostic functions.
- Keep pass ordering in `run_analysis_passes` explicit.
- Update tests to confirm diagnostics remain stable and `CheckedStory` exposes
  the same data as before.

Acceptance criteria:

- `analyze` and `run_analysis_passes` no longer rebuild the same module indexes
  independently.
- Diagnostic ordering remains deterministic.
- `CheckedStory` public fields keep their current semantics.

Forbidden shortcuts:

- Do not store lowering-specific runtime path indexes in analysis indexes.
- Do not make diagnostics depend on mutation order.

Modification boundary:

- `crates/ink-compiler/src/analysis/mod.rs`
- `crates/ink-compiler/src/analysis/modules`
- analysis tests

Validation:

- `cargo test -p ink-compiler analysis::`
- `cargo test -p ink-test --test language module`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

## Milestone 4: Runtime JSON Robustness

### [ ] Task 15: Add Save-State JSON Read Helpers

Goal:

Add small helper functions for required strings, unsigned indexes, integer
fields, arrays, and objects in runtime-owned JSON reader code.

Implementation method:

- Add helpers inside `crates/ink-runtime/src/json/json_read.rs` or a private
  submodule.
- Convert one narrow save-state reader path to the helpers.
- Add tests for malformed fields returning `StoryError::BadJson`.

Acceptance criteria:

- At least one previously unchecked save-state JSON path returns an error
  instead of panicking.
- Helper error messages identify the missing or wrong field.
- Compiled story loading remains format-crate based.

Forbidden shortcuts:

- Do not change save-state field names.
- Do not broaden `BadJson` into generic invalid story state errors.

Modification boundary:

- `crates/ink-runtime/src/json/json_read.rs`
- runtime JSON tests

Validation:

- `cargo test -p ink-runtime json`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 16: Convert Choice And Tag Save-State Reading To Helpers

Goal:

Remove unchecked JSON access from choice and tag save-state loading.

Implementation method:

- Update `jobject_to_choice` and `jarray_to_tags` to use the new helper
  functions.
- Preserve the current choice JSON schema.
- Add malformed-choice and malformed-tags tests.

Acceptance criteria:

- Missing or wrong-type choice fields return `StoryError::BadJson`.
- Valid choice save-state JSON round trips unchanged.
- Tag arrays still load to the same runtime tag strings.

Forbidden shortcuts:

- Do not change choice serialization in this task.
- Do not silently default missing required fields.

Modification boundary:

- `crates/ink-runtime/src/json/json_read.rs`
- runtime save-state tests

Validation:

- `cargo test -p ink-runtime json`
- `cargo test -p ink-runtime story_state`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 17: Convert Runtime Value Dictionary Save-State Reading To Helpers

Goal:

Remove unchecked downcasts and JSON assumptions from runtime value dictionary
loading used by callstack and variable state save data.

Implementation method:

- Update `jobject_to_hashmap_values` to return `BadJson` when a token does not
  decode to a runtime `Value`.
- Update callers only as needed to propagate the error.
- Add tests for valid dictionary loading and non-value dictionary entries.

Acceptance criteria:

- Runtime value dictionaries no longer panic on non-value JSON entries.
- Valid save-state variable and temporary dictionaries remain unchanged.
- Error messages identify the failing key where practical.

Forbidden shortcuts:

- Do not change variable state save JSON names.
- Do not accept invalid JSON by coercing it to default values.

Modification boundary:

- `crates/ink-runtime/src/json/json_read.rs`
- `crates/ink-runtime/src/callstack.rs` if propagation changes are needed
- `crates/ink-runtime/src/variables_state.rs` if propagation changes are needed
- runtime save-state tests

Validation:

- `cargo test -p ink-runtime json`
- `cargo test -p ink-runtime story_state`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 18: Remove JSON Round Trips From Runtime Container Writing Where Practical

Goal:

Reduce runtime JSON writer dependence on serializing a format object to JSON and
immediately parsing it back into a format object.

Implementation method:

- Review `runtime_object_to_format_object` and `write_rtobject`.
- Convert safe branches to return typed format objects directly before calling
  `to_json_value`.
- Keep runtime save-state JSON output byte-compatible where ordering permits.
- Add round-trip tests for compiled story containers and save-state values.

Acceptance criteria:

- Runtime object-to-format conversion has fewer unnecessary JSON round trips.
- Existing valid runtime JSON output remains compatible.
- Unsupported runtime objects still produce clear `StoryError::BadJson`.

Forbidden shortcuts:

- Do not move save-state schema ownership to the format crate.
- Do not change named content or container name emission rules.

Modification boundary:

- `crates/ink-runtime/src/json/json_write.rs`
- runtime JSON tests

Validation:

- `cargo test -p ink-runtime json`
- `cargo test -p ink-test --test conformance`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

## Milestone 5: Runtime Native Function Organization

### [ ] Task 19: Extract Native Function Operation Metadata

Goal:

Move runtime native function operation name and arity metadata out of the main
operation implementation body.

Implementation method:

- Convert `native_function_call.rs` to a module directory if needed.
- Add an operation metadata submodule for `Op`, token mapping, and arity.
- Keep public runtime behavior and `NativeFunctionCall::new_from_name` /
  `get_name` compatibility.
- Add tests for every token and arity.

Acceptance criteria:

- Operation metadata is separated from operation implementation.
- Runtime native function tokens still match the format crate tokens.
- Existing native function tests pass unchanged or with only import-path
  updates.

Forbidden shortcuts:

- Do not change operation names.
- Do not remove currently supported operations.

Modification boundary:

- `crates/ink-runtime/src/native_function_call.rs`
- new files under `crates/ink-runtime/src/native_function_call/`
- native function tests

Validation:

- `cargo test -p ink-runtime native_function_call`
- `cargo test -p ink-story-json-format`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 20: Extract Composite Native Operations

Goal:

Move object field, array index, array length, and array remove native operation
implementation into a focused runtime module.

Implementation method:

- Extract `FIELD`, `INDEX`, `SET_FIELD`, `SET_INDEX`, `LEN`, and
  `ARRAY_REMOVE` helpers.
- Keep error text and value-copy behavior unchanged.
- Keep tests for missing fields, out-of-bounds indexes, nested arrays, and
  struct/object arrays close to the implementation.

Acceptance criteria:

- Composite native operations are separated from numeric/string/bool operations.
- All existing composite operation tests still pass.
- Error behavior for invalid composite operations is unchanged.

Forbidden shortcuts:

- Do not mutate arrays or objects in place if current semantics return copies.
- Do not loosen missing-field or bounds errors.

Modification boundary:

- `crates/ink-runtime/src/native_function_call.rs`
- new native function submodule files
- native function tests

Validation:

- `cargo test -p ink-runtime native_function_call`
- `cargo test -p ink-test --test language`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 21: Extract Numeric, String, And Boolean Native Operations

Goal:

Move arithmetic, comparison, logical, string, cast, min/max, and random-support
native operation helpers into focused runtime modules or helper groups.

Implementation method:

- Extract repeated int/float/string/bool operation patterns without changing
  coercion rules.
- Preserve string addition special handling and equality behavior for composite
  values.
- Keep existing operation tests and add coverage for any helper boundary that
  becomes nontrivial.

Acceptance criteria:

- Main native function dispatch is shorter and delegates to focused helpers.
- Numeric coercion, string addition, equality, and invalid-type errors remain
  unchanged.
- Operation tests still describe current semantics.

Forbidden shortcuts:

- Do not replace explicit runtime errors with panics.
- Do not change float/int coercion behavior.

Modification boundary:

- `crates/ink-runtime/src/native_function_call.rs`
- new native function submodule files
- native function tests

Validation:

- `cargo test -p ink-runtime native_function_call`
- `cargo test -p ink-test --test conformance`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 22: Centralize Runtime Native Value Parameter Helpers

Goal:

Reduce duplicate downcast and error handling in native function operation
implementation.

Implementation method:

- Add helper functions for required runtime `Value`, string field names, array
  indexes, and value type access where they remove duplication.
- Preserve current error messages unless a test documents an intentional
  clarification.
- Update native function tests to pin any clarified errors.

Acceptance criteria:

- Repeated `downcast_ref::<Value>` and index validation logic is centralized.
- Existing valid behavior is unchanged.
- Invalid operand tests still pass with the same or explicitly documented
  clearer messages.

Forbidden shortcuts:

- Do not hide invalid operand types by defaulting values.
- Do not broaden helper APIs beyond native function operation needs.

Modification boundary:

- `crates/ink-runtime/src/native_function_call.rs`
- native function submodules
- native function tests

Validation:

- `cargo test -p ink-runtime native_function_call`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

## Milestone 6: Mechanical Cleanup And Compatibility Checks

### [ ] Task 23: Fix Local Low-Risk Clippy Findings

Goal:

Clean up local Rust style issues that do not require semantic decisions and are
not already handled by the structural tasks.

Implementation method:

- Fix local findings such as `manual_ok_err`, `ptr_arg`, `get_first`,
  `unnecessary_unwrap`, `never_loop`, and simple `question_mark` suggestions
  only where the transformation is obviously behavior-preserving.
- Avoid touching large unrelated files just for style churn.
- Run focused tests for each touched crate.

Acceptance criteria:

- The selected local clippy warnings are removed.
- No behavior or public API changes are introduced.
- `cargo clippy --workspace --all-targets -- -W clippy::too_many_arguments -W clippy::type_complexity -W clippy::large_enum_variant` no longer reports the same local mechanical findings, even if intentional structural warnings remain for future work.

Forbidden shortcuts:

- Do not add `#[allow]` attributes for warnings that can be fixed locally.
- Do not chase unrelated lints outside the current touched surfaces.

Modification boundary:

- Small local edits in `crates/ink-compiler`
- Small local edits in `crates/ink-runtime`
- focused tests for touched files

Validation:

- focused crate tests for touched files
- `cargo clippy --workspace --all-targets -- -W clippy::too_many_arguments -W clippy::type_complexity -W clippy::large_enum_variant`
- `cargo fmt --all --check`
- `make gate`

Validation status: pending.

Commit: pending.

### [ ] Task 24: Final Compatibility Validation And Plan Closeout

Goal:

Verify the refactor set end to end and move this plan to finished plans after
all implementation tasks are complete.

Implementation method:

- Confirm every task has `[x]`, validation status, and a commit hash.
- Review `requirement_plan.md` against completed work and update only for actual
  clarified compatibility notes.
- Run broad compiler, runtime, format, conformance, and gate validation.
- Move `docs/current_plan/refactor_compiler_runtime/` to
  `docs/finished_plans/refactor_compiler_runtime/` only after validation
  passes.

Acceptance criteria:

- All refactor tasks are complete and recorded.
- Full validation passes.
- No active refactor plan remains under `docs/current_plan/`.

Forbidden shortcuts:

- Do not close the plan with uncommitted implementation changes.
- Do not move the plan if any task remains pending or blocked.

Modification boundary:

- plan directory movement
- closeout documentation updates

Validation:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo test -p ink-test --features csharp-tests --test csharp_tests`
- `make gate`

Validation status: pending.

Commit: pending.
