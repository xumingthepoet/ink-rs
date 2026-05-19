Progress: 4/20

# Typed Dict Values Task List

This plan adds `Dict<K, V>` values where `K` is `string` or `int`. Each task
must produce a reviewable repository change and pass its focused validation plus
`make gate` before the task can be marked waiting review.

Status key: `[ ]` pending, `[~]` in progress, `[>]` waiting review, `[x]`
complete, `[!]` blocked.

Global acceptance conditions:

- `Dict<string, V>` and `Dict<int, V>` work across compiler, format JSON,
  runtime execution, save/load, external values, tests, and docs.
- Existing array, struct, enum, interface, divert target, module, and legacy JSON
  behavior remains unchanged.
- Missing-key Dict reads produce a runtime error.
- Dict assignment inserts or replaces entries.
- `make gate` passes before any implementation task is marked `[>]`.

Global forbidden shortcuts:

- Do not encode int keys as plain JSON object field names.
- Do not special-case fixture names, fixture paths, expected outputs, or snapshot
  strings in production code.
- Do not weaken existing array or struct diagnostics to make Dict pass.
- Do not add ignored tests, skip filters, or compatibility shims for incomplete
  Dict behavior.
- Do not let runtime object graph types leak into compiler parsed model or the
  format crate.

## Milestone 1: Source Model And Parser

### [x] Task 01: Add Dict Type Names

Goal: Represent and parse `Dict<K, V>` type names with `string` and `int` keys.

Implementation method: Add a Dict variant to the parsed type model, display and
snapshot formatting, hashing/equality, and default-value metadata. Extend type
name parsing to parse `Dict<key, value>` with nested value types and emit clear
diagnostics for malformed syntax and unsupported key types.

Acceptance criteria: Parser unit tests cover valid string/int key Dicts, nested
Dict values, Dict arrays, missing comma/closing `>`, and invalid key types.

Forbidden shortcuts: Do not parse generic types with string splitting. Do not
accept arbitrary key types for later rejection if the parser has enough context
to reject them.

Modification boundaries: `crates/ink-compiler/src/parsed/type_name.rs`,
`crates/ink-compiler/src/syntax/type_name.rs`, and direct parser tests only.

Validation commands: `cargo test -p ink-compiler syntax::type_name`

Validation:

- `cargo test -p ink-compiler syntax::type_name` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.
- Review fix: replaced `PrimitiveType` Dict keys with a dedicated
  `DictKeyType` enum and reran `cargo test -p ink-compiler syntax::type_name`,
  `cargo fmt --all --check`, and `make gate`.

Commit record: implementation `d777e817` (`Add Dict type name parsing`); review
fix `3ffdc426` (`Constrain Dict key type model`)

### [x] Task 02: Add Dict Literal Parsed Nodes

Goal: Distinguish struct literals from Dict literals in the parsed expression
model.

Implementation method: Add parsed Dict literal key/value entries for string and
int keys plus an explicit empty-composite expression for `{}`. Extend expression
token parsing so quoted-string or integer keys in `{ key: value }` become Dict
literals, while identifier keys remain struct literal fields. Empty `{}` must
remain expected-type-driven so later analysis can accept it for structs or Dicts.

Acceptance criteria: Parser tests cover `{"a": 1}`, `{1: "a"}`, nested Dict
literals, empty `{}`, existing struct literals, and mixed syntax errors.

Forbidden shortcuts: Do not reinterpret identifier-key struct literals as Dicts.
Do not break inline conditional parsing.

Modification boundaries: parsed expression model, expression parser, expression
snapshots, visitor traversal.

Validation commands: `cargo test -p ink-compiler syntax::expression`

Validation:

- `cargo test -p ink-compiler syntax::expression` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.
- Review found no follow-up code fixes required; reran
  `cargo test -p ink-compiler syntax::expression`, `cargo fmt --all --check`,
  and `make gate`.

Commit record: implementation `80057918` (`Add Dict literal parsed nodes`)

## Milestone 2: Type Analysis

### [x] Task 03: Validate Dict Literal Types

Goal: Type-check Dict literals against expected `Dict<K, V>` declarations.

Implementation method: Add an analysis pass or extend the composite literal
checker so Dict literal keys match `K` and values match `V`, including nested
arrays, structs, enums, interfaces, and Dicts. Ensure empty `{}` is accepted
only under an expected Dict or struct type.

Acceptance criteria: Unit tests and fixture diagnostics cover wrong key type,
wrong value type, empty Dict declaration, and nested Dict value checking.

Forbidden shortcuts: Do not type-check Dict literals in lowering. Do not weaken
struct literal missing-field checks.

Modification boundaries: compiler analysis for composite literals and focused
diagnostic fixtures.

Validation commands: `cargo test -p ink-compiler analysis` and
`cargo test -p ink-test diagnostics`

Validation:

- `cargo test -p ink-compiler analysis` passed.
- `cargo test -p ink-test diagnostics` passed.
- `cargo fmt --all --check` passed.
- `make gate` passed.
- Review fix: covered Dict fields nested inside struct literal expected types,
  then reran `cargo test -p ink-compiler analysis`,
  `cargo test -p ink-test diagnostics`, `cargo fmt --all --check`, and
  `make gate`.

Commit record: implementation `d258c6b9` (`Validate Dict literal types`);
review fix `cbbb5d42` (`Cover Dict fields nested in structs`)

### [x] Task 04: Infer Dict Index Access

Goal: Make `dict[key]` infer the Dict value type while preserving array index
rules.

Implementation method: Extend expression type inference and index diagnostics so
arrays accept only `int`, `Dict<string, V>` accepts `string`, and
`Dict<int, V>` accepts `int`. Keep indexing non-array/non-Dict values rejected.

Acceptance criteria: Tests cover read type inference, wrong Dict key type,
array string indexing still rejected, and nested array/Dict chains.

Forbidden shortcuts: Do not special-case variable names. Do not let string
indexing apply to strings.

Modification boundaries: expression type inference and index access diagnostics.

Validation commands: `cargo test -p ink-compiler analysis::index_access`

Validation:

- `cargo test -p ink-compiler analysis::index_access` passed.
- `cargo fmt --all --check` passed.
- First `make gate` run reached the final `ink-test` doc-test stage but failed
  by the 120s wrapper timeout with no failing tests reported.
- Reran `make gate` after incremental compilation; passed.
- Review found no retained code fixes required; reran
  `cargo test -p ink-compiler analysis::index_access`, `cargo fmt --all --check`,
  and `make gate`. The first review `make gate` rerun hit the same final
  `ink-test` wrapper timeout with no failing tests reported; the hot rerun
  passed.

Commit record: implementation `f6d534a3` (`Infer Dict index access types`)

### [ ] Task 05: Validate Dict Assignment Targets

Goal: Type-check indexed Dict assignments and compound lvalue chains.

Implementation method: Extend assignment analysis to resolve Dict index path
components, expected inserted value type, and nested field/index chains such as
`players["ada"].hp = 5` and `table[1]["name"] = "Ada"`.

Acceptance criteria: Tests cover replacing values, inserting missing keys,
nested assignment, wrong key type, and wrong assigned value type.

Forbidden shortcuts: Do not move lvalue type checks into lowering. Do not
evaluate assignment indexes more than once.

Modification boundaries: assignment analysis and focused diagnostics.

Validation commands: `cargo test -p ink-compiler analysis`

Commit record: pending

### [ ] Task 06: Validate Dict Function And External Signatures

Goal: Allow Dict types in function, external, internal, constant, struct field,
and interface-relevant type positions.

Implementation method: Audit existing type-name validation and type comparison
paths so Dict values work anywhere arrays and structs already work, except where
an existing feature intentionally requires a different type.

Acceptance criteria: Tests cover Dict parameters, returns, constants, struct
fields, arrays of Dicts, and external signatures.

Forbidden shortcuts: Do not add runtime-only acceptance for types the compiler
cannot check.

Modification boundaries: compiler analysis paths that already support arrays and
structs.

Validation commands: `cargo test -p ink-compiler analysis`

Commit record: pending

## Milestone 3: Format JSON

### [ ] Task 07: Add Format Dict Wire Model And Document Encoding

Goal: Make `ink-story-json-format` own a reversible Dict value representation.

Implementation method: Add a Dict key enum and Dict value object model. Serialize
Dicts using a marker encoding that preserves key type and never conflicts with
existing struct/object values. Deserialize the same encoding recursively. Update
`docs/ink_JSON_runtime_format.md` in the same task with examples for string and
int keys and a compatibility note for existing object values.

Acceptance criteria: Format tests roundtrip string-key Dicts, int-key Dicts,
nested Dicts, and values containing arrays/objects. Runtime format docs describe
the marker shape, recursive values, and key-type preservation.

Forbidden shortcuts: Do not encode int keys as object field names. Do not break
existing object-value parsing.

Modification boundaries: `crates/ink-story-json-format` and
`docs/ink_JSON_runtime_format.md`.

Validation commands: `cargo test -p ink-story-json-format`

Commit record: pending

## Milestone 4: Lowering

### [ ] Task 08: Lower Dict Defaults And Literals

Goal: Emit format Dict values for defaults, constants, global initializers, temp
initializers, and nested literals.

Implementation method: Extend runtime default and literal lowering for
`Dict<K, V>`. Recursively lower values with the expected value type and preserve
key type in format objects.

Acceptance criteria: JSON sequence tests show empty default Dicts and literal
Dicts with string and int keys.

Forbidden shortcuts: Do not hardcode expected JSON fragments for individual
fixtures in compiler code.

Modification boundaries: compiler lowering value paths and focused tests.

Validation commands: `cargo test -p ink-test typed_values`

Commit record: pending

### [ ] Task 09: Lower Dict Reads

Goal: Emit runtime instructions for Dict index reads through existing expression
lowering.

Implementation method: Reuse the `INDEX` native token for Dict reads. Ensure
array read lowering remains unchanged and Dict keys are evaluated once.

Acceptance criteria: Runtime fixture output and JSON sequence tests cover
`dict["key"]` and `dict[1]`.

Forbidden shortcuts: Do not add a separate compiled token unless existing
`INDEX` cannot represent Dict reads.

Modification boundaries: expression lowering and tests.

Validation commands: `cargo test -p ink-test typed_values`

Commit record: pending

### [ ] Task 10: Lower Dict Writes

Goal: Emit runtime instructions for Dict indexed assignment and nested lvalue
updates.

Implementation method: Reuse `SET_INDEX` for Dict writes. Preserve current
single-evaluation behavior for complex indexes and rebuild nested copies through
the existing lvalue path.

Acceptance criteria: Fixtures cover insert, replace, nested Dict-in-struct, and
Dict-in-array assignment.

Forbidden shortcuts: Do not mutate shared values in place when existing
composite semantics copy on write.

Modification boundaries: assignment lowering and tests.

Validation commands: `cargo test -p ink-test typed_values`

Commit record: pending

## Milestone 5: Runtime Values

### [ ] Task 11: Add Runtime Dict Value Type

Goal: Represent Dict values in runtime-owned value data.

Implementation method: Add `ValueType::Dict` with key enum support, clone,
display, casts, truthiness errors, default handling, and value helper methods.

Acceptance criteria: Runtime unit tests cover clone/equality/display shape and
truthiness/cast rejection.

Forbidden shortcuts: Do not represent int-key Dicts as `BTreeMap<String, _>`.

Modification boundaries: runtime value modules.

Validation commands: `cargo test -p ink-runtime value_type value`

Commit record: pending

### [ ] Task 12: Runtime Dict INDEX And SET_INDEX

Goal: Execute Dict reads and writes with existing native index operations.

Implementation method: Extend native composite operations so `INDEX` accepts
arrays or Dicts and `SET_INDEX` writes arrays or Dicts. Dict missing-key reads
return `StoryError::InvalidStoryState`; writes insert or replace.

Acceptance criteria: Unit tests cover string keys, int keys, wrong key type,
missing read, insert, replace, and array behavior unchanged.

Forbidden shortcuts: Do not silently coerce key types.

Modification boundaries: runtime native function composite params and tests.

Validation commands: `cargo test -p ink-runtime native_function_call::composite`

Commit record: pending

### [ ] Task 13: Runtime Dict Equality

Goal: Compare Dict values recursively with `==` and `!=`.

Implementation method: Extend scalar equality helpers for Dict values and keep
cross-type comparisons false unless existing behavior says otherwise.

Acceptance criteria: Runtime unit tests cover equal Dicts, unequal values,
different key types, nested Dicts, and arrays/structs unaffected.

Forbidden shortcuts: Do not stringify Dicts for equality.

Modification boundaries: runtime scalar native functions.

Validation commands: `cargo test -p ink-runtime native_function_call::scalar`

Commit record: pending

### [ ] Task 14: Runtime JSON Load And Save

Goal: Load and save Dict values through the format crate.

Implementation method: Map format Dict objects to `ValueType::Dict` and back in
runtime JSON read/write. Ensure variable save-state omission compares Dict
defaults correctly.

Acceptance criteria: Runtime tests cover story JSON load, save JSON write/read,
and default Dict omission where applicable.

Forbidden shortcuts: Do not add a duplicate parser for the Dict marker outside
the format crate.

Modification boundaries: runtime JSON read/write and variable-state save paths.

Validation commands: `cargo test -p ink-runtime json variables_state`

Commit record: pending

### [ ] Task 15: Runtime API And Externals

Goal: Expose Dict values through host variable and external APIs.

Implementation method: Allow `ValueType::Dict` to be passed to and returned from
external bindings and variable get/set APIs. Extend runtime type checks used by
internal function signatures if needed.

Acceptance criteria: Integration tests cover external return/argument Dicts and
variable set/get with nested Dict values.

Forbidden shortcuts: Do not accept host objects that lose key type information.

Modification boundaries: runtime API tests and necessary runtime type helpers.

Validation commands: `cargo test -p ink-test runtime_api typed_values`

Commit record: pending

## Milestone 6: Integration Fixtures

### [ ] Task 16: Add Dict Runtime Fixtures And Author Docs

Goal: Cover end-to-end story behavior for core Dict operations and document the
supported author-facing feature.

Implementation method: Add typed fixtures and tests for defaults, literals,
reads, writes, insertion, replacement, nesting, equality, functions, constants,
and externals. Update `docs/SyntaxUpdates.md` and `docs/SyntaxReference.md` in
the same task to describe the completed V1 syntax and explicitly omit unsupported
collection APIs.

Acceptance criteria: `typed_values` tests assert output and key JSON sequences
for string and int key Dicts. Syntax docs mention `Dict<K, V>`, key
restrictions, literals, defaults, index read/write, equality, functions,
externals, and V1 non-goals.

Forbidden shortcuts: Do not weaken existing fixture expected output.

Modification boundaries: `crates/ink-test` fixtures/tests,
`docs/SyntaxUpdates.md`, and `docs/SyntaxReference.md`. Do not edit
`docs/WritingWithInk.md`.

Validation commands: `cargo test -p ink-test typed_values`

Commit record: pending

### [ ] Task 17: Add Dict Diagnostic Fixtures

Goal: Cover user-facing compiler failures for invalid Dict code.

Implementation method: Add diagnostics fixtures for unsupported key types,
wrong literal key type, wrong value type, wrong index key type, indexing
non-Dict/non-array, missing generic punctuation, and unsupported builtins.

Acceptance criteria: Diagnostics tests assert clear error fragments.

Forbidden shortcuts: Do not rely on parse panics or ambiguous fallback errors.

Modification boundaries: diagnostics fixtures and tests.

Validation commands: `cargo test -p ink-test diagnostics`

Commit record: pending

### [ ] Task 18: Add Parse Snapshots

Goal: Pin the visible parsed model for Dict source syntax.

Implementation method: Add `.ink.parse` snapshots for representative Dict type
declarations, literals, reads, writes, and nested expressions.

Acceptance criteria: Parse snapshot tests cover Dict and existing struct
literal syntax remains unchanged.

Forbidden shortcuts: Do not remove older snapshots to hide parser changes.

Modification boundaries: parse fixtures only.

Validation commands: `cargo test -p ink-test parse`

Commit record: pending

## Milestone 7: Gate And Closeout

### [ ] Task 19: Review Active Plan Records And Final Gate

Goal: Ensure task status, validation, commit records, and final workspace
validation are complete.

Implementation method: Review this task list for stale statuses, missing
validation records, missing commit hashes, and inconsistent progress count. Run
the full workspace validation and record the final results before closeout.

Acceptance criteria: Every implementation task has validation evidence and a
commit record. `cargo fmt --all --check`, `cargo check --workspace`,
`cargo test --workspace`, and `make gate` pass.

Forbidden shortcuts: Do not mark tasks complete without matching validation and
commit records.

Modification boundaries: active plan records only.

Validation commands: `cargo fmt --all --check`, `cargo check --workspace`,
`cargo test --workspace`, `make gate`

Commit record: pending

### [ ] Task 20: Move Dict Plan To Finished Plans

Goal: Close the active plan after implementation and validation are complete.

Implementation method: Move `docs/active_plan/dict-values` to
`docs/finished_plans/dict-values` after every task is complete.

Acceptance criteria: No Dict plan remains under `docs/active_plan`; finished
plan contains final records.

Forbidden shortcuts: Do not close the plan while implementation tasks are still
pending, blocked, or waiting review.

Modification boundaries: plan directory move only.

Validation commands: `cargo fmt --all --check`

Commit record: pending
