Progress: 0/5

# v0.1.1 ENUM Feature Active Plan

## Status Key

- `[ ]` pending
- `[~]` in progress
- `[>]` implementation committed and waiting review
- `[x]` complete
- `[!]` blocked

## Milestone 1: Parser And Parsed Model

### [>] Task 01: Add ENUM Declarations To Syntax And Parsed Model

Goal:
Add payload-free `ENUM` declarations as root/module-level parsed objects without changing type checking or lowering yet.

Implementation method:
Add parsed enum declaration structures, expose them through `Object`, parse snapshots, and visitor traversal. Extend the structure parser and top-level/module parser paths to accept inline and multiline `ENUM` declarations using no comma or semicolon separators. Reject explicit member values at parse time and keep empty-enum validation for the semantic pass in Task 02.

Acceptance criteria:
Inline and multiline enum declarations parse at root and inside modules. Enum declarations appear in parse snapshots. Existing `STRUCT`, module, flow, and variable parsing behavior remains unchanged.

Forbidden shortcuts:
Do not encode enum declarations as `STRUCT`, `CONST`, or text objects. Do not special-case fixture names or skip parse errors by silently dropping enum declarations.

Modification boundaries:
Parser, parsed model, parse snapshots, and parser unit tests only. Do not add enum type checking, runtime lowering, or JSON fixture changes in this task.

Validation commands:
`cargo test -p ink-compiler syntax::parser`
`cargo test -p ink-compiler parsed`
`make gate`

Commit record:
Implementation commit: `f071fe50` Add enum declarations to parser
Review validation: pending

## Milestone 2: Type Model And Module Symbols

### [ ] Task 02: Add Enum Types, Indexes, And Module Import Integration

Goal:
Represent enum types as first-class nominal types and make enum declarations participate in module symbols, imports, name conflicts, and type existence diagnostics.

Implementation method:
Extend `TypeName` with root and qualified enum variants, default metadata, display names, and array support. Add an enum type index analogous to struct indexing, with diagnostics for empty enums and duplicate members. Register enums in module symbols and import-use collection so `IMPORT State FROM items` authorizes `items::State` and `items::State.Member`. Update name collision analysis so enum names share the module element namespace with structs, variables, constants, flows/functions, and externals.

Acceptance criteria:
Enum types are accepted in every existing type position syntactically. Unknown enum types and invalid declarations produce explicit diagnostics. Existing struct import and namespace diagnostics continue to pass.

Forbidden shortcuts:
Do not treat all unknown named types as enums. Do not weaken existing struct diagnostics or import checks. Do not change current import syntax in this release.

Modification boundaries:
Type model, enum/type analysis, module symbol/import analysis, naming analysis, and focused unit tests. Do not lower enum values or add runtime fixtures in this task.

Validation commands:
`cargo test -p ink-compiler analysis`
`cargo test -p ink-compiler syntax::type_name`
`make gate`

Commit record:
Implementation commit: pending
Review validation: pending

## Milestone 3: Expressions And Lowering

### [ ] Task 03: Resolve Enum Members And Lower Enum Values

Goal:
Make `Enum.Member` and `module::Enum.Member` valid enum member expressions with type-safe equality, assignment, defaults, and lowering to runtime strings.

Implementation method:
Resolve field-access-shaped expressions whose base names an enum type into enum member values during analysis/lowering. Preserve normal struct field access when the base is a value expression. Apply first-member defaults for omitted enum initializers. Type-check enum assignment, constants, function returns, parameters, struct fields, arrays, switch cases, and `==`/`!=`; reject string interop, ordering, arithmetic, and unknown members. Lower enum member values and enum defaults to strings using `Enum.Member` for root enums and `module::Enum.Member` for module enums.

Acceptance criteria:
Enum values compile and run through existing runtime value support. Enum declarations remain absent from story JSON. Invalid enum operations produce clear compiler diagnostics.

Forbidden shortcuts:
Do not add runtime enum value variants or JSON schema changes. Do not lower enum members through ad hoc string matching on source text. Do not allow enum/string assignment just because runtime values are strings.

Modification boundaries:
Expression analysis, initializer/type checks, defaults, lowering indexes/value lowering, and focused compiler/runtime tests. Do not update public docs in this task except test fixtures that require comments.

Validation commands:
`cargo test -p ink-compiler analysis::expression_types`
`cargo test -p ink-compiler analysis::initializers`
`cargo test -p ink-test typed_values`
`make gate`

Commit record:
Implementation commit: pending
Review validation: pending

## Milestone 4: Fixtures And Documentation

### [ ] Task 04: Add Public Fixtures, Diagnostics, And Syntax Documentation

Goal:
Pin the complete v0.1.1 enum language behavior in integration fixtures and user-facing syntax documentation.

Implementation method:
Add compiler snapshot fixtures and runtime fixtures covering root enums, module enums, defaults, equality, switch with `- else:`, function parameters/returns, struct fields, arrays, text output, and JSON string lowering. Add diagnostic fixtures for empty enums, duplicate members, explicit member values, unknown members, string interop, and invalid operators. Update `docs/SyntaxUpdates.md` and `docs/SyntaxReference.md` with the latest enum syntax only.

Acceptance criteria:
The docs describe the supported v0.1.1 enum syntax and do not describe rejected alternatives as supported syntax. Fixtures demonstrate both successful behavior and common failures.

Forbidden shortcuts:
Do not edit `docs/WritingWithInk.md`. Do not add ignored tests, fixture-specific compiler branches, or expected-output-only changes that hide compiler defects.

Modification boundaries:
Integration fixtures, snapshot registrations, diagnostics tests, and maintained docs. Keep production-code edits limited to defects discovered while wiring the fixtures.

Validation commands:
`cargo test -p ink-test compiler_snapshots typed_values diagnostics`
`cargo test --workspace`
`make gate`

Commit record:
Implementation commit: pending
Review validation: pending

## Milestone 5: Closeout

### [ ] Task 05: Archive Completed ENUM Plan

Goal:
Close the active plan after all implementation tasks are reviewed and validated.

Implementation method:
Move this plan directory from `docs/active_plan/v0.1.1-enum` to `docs/finished_plans/v0.1.1-enum` and verify no active enum plan remains.

Acceptance criteria:
The plan is archived only after Tasks 01-04 are complete. The repository has no stale active-plan enum files.

Forbidden shortcuts:
Do not archive while implementation tasks are still `[>]`, `[~]`, `[!]`, or `[ ]`.

Modification boundaries:
Plan files only.

Validation commands:
`git status --short`

Commit record:
Implementation commit: pending
Review validation: pending
