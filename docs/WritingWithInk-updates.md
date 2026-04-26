# Writing with Ink Updates

This file records intentional language documentation changes made by ink-rs.

`WritingWithInk-origin.md` is the immutable upstream C# documentation snapshot.
Do not edit it for ink-rs language changes.

`WritingWithInk-latest.md` is the current ink-rs documentation:

```text
WritingWithInk-latest.md = WritingWithInk-origin.md + WritingWithInk-updates.md
```

When syntax or semantics change, update this file first, then apply the same
change to `WritingWithInk-latest.md`.

Each entry should include:

- date
- status: experimental, supported, deprecated, or removed
- upstream behavior
- ink-rs behavior
- documentation effect
- rationale
- migration guidance
- tests

## 2026-04-26: CONST Declarations Require Explicit Types

- status: supported
- upstream behavior: upstream Ink constants are dynamically typed and declared
  with `CONST name = value`.
- ink-rs behavior: constants use `CONST name: Type = value`. Supported constant
  types are the same maintained value types as globals and temps: `int`,
  `float`, `bool`, `string`, user `STRUCT` types, arrays written as `T[]`, and
  nested arrays. Constant initializers are checked against the declared type,
  and struct defaults are applied when typed struct constants omit fields.
- documentation effect: `WritingWithInk-latest.md` updates the changed-from-
  upstream section and all constants examples to use explicit types.
- rationale: constants participate in expression type checking and lowering.
  Requiring a declared type keeps composite constants unambiguous, especially
  for empty arrays and partial struct literals.
- migration guidance: rewrite `CONST NAME = value` as `CONST NAME: Type =
  value`.
- tests: `typed_constants_support_struct_and_array_values` and
  `untyped_constant_declaration_reports_missing_type` in
  `crates/ink-test/tests/language.rs`, plus compiler parser, initializer,
  struct literal, and array literal unit tests.

## 2026-04-26: Global VAR Declarations Restricted To Story Top Level

- status: removed
- upstream behavior: upstream Ink allows global variables to be introduced with
  `VAR` anywhere in the parsed story, including inside knots, stitches,
  functions, choices, conditionals, and sequences.
- ink-rs behavior: `VAR` declarations are only accepted at the story top level,
  outside knots, stitches, functions, choices, conditionals, and sequences.
  Local executable state should use typed `temp` declarations instead.
- documentation effect: `WritingWithInk-latest.md` records the scope
  restriction in "Changed from upstream Ink" and in the global variable
  declaration section.
- rationale: hidden global declarations inside executable flow content make
  include expansion and future source organization rules ambiguous. Keeping
  globals in one story-level declaration area makes global state explicit before
  flow execution.
- migration guidance: move nested `VAR` declarations to the story top level. If
  the value is only needed inside a knot, stitch, function, choice, conditional,
  or sequence, replace it with a typed `temp` declaration.
- tests: `nested_global_var_declarations_report_removed_feature_diagnostic` in
  `crates/ink-test/tests/language.rs`,
  `global_var_declarations_inside_flows_report_removed_feature` in
  `crates/ink-compiler/src/syntax/parser.rs`, and
  `reports_global_var_declarations_outside_story_top_level` in
  `crates/ink-compiler/src/analysis/flow.rs`.

## 2026-04-26: Typed Values, Structs, Arrays, Functions, Externals, And Tail Calls

- status: supported
- upstream behavior: upstream Ink variables, temporary variables, function
  parameters, return values, and external declarations are dynamically typed by
  their runtime values. It does not provide source-level `STRUCT` declarations,
  typed array declarations with `T[]`, static field/index checks, or recursive
  structural equality for array and object values.
- ink-rs behavior: `VAR` and `temp` declarations require explicit types using
  `name: Type`, and omitted initializers are allowed for typed declarations.
  Supported value types are `int`, `float`, `bool`, `string`, user-declared
  structs, arrays written as `T[]`, and nested arrays. Structs are declared with
  `STRUCT Name { field: Type }`, constructed with object literals, and accessed
  or assigned through dotted fields. Arrays are constructed with `[a, b]`,
  indexed with `items[n]`, copied by value, and can contain primitives,
  structs, or nested arrays. Equality and inequality are checked statically and
  compare arrays and structs recursively at runtime.
- ink-rs behavior: functions require typed parameters and an explicit return
  type such as `== function add(a: int, b: int) -> int ==`; use `-> void` for
  functions that only perform effects. `EXTERNAL` declarations also require
  typed argument and return signatures. `LEN(array)` returns an `int`, and
  `ARRAY_REMOVE(array, index)` mutates the array and returns `void`.
- ink-rs behavior: direct self tail recursion in the shape
  `return current_function(...)` is lowered to parameter reassignment plus a
  jump back to the function body, avoiding growth of the Ink function callstack
  for that recursive step. Mutual recursion and non-tail recursion keep normal
  call behavior.
- documentation effect: `WritingWithInk-latest.md` must replace the upstream
  dynamic declaration examples with typed declarations, add struct and array
  syntax, document typed functions and externals, and document the array
  builtins and tail-call behavior as ink-rs language extensions.
- rationale: typed source declarations let the compiler reject mismatched
  assignments, bad function or external calls, non-bool conditions, invalid
  field/index access, and unsupported implicit conversions before runtime while
  preserving the existing dynamic story JSON execution model.
- migration guidance: add explicit type annotations to all `VAR` and `temp`
  declarations, all function parameters and returns, and all `EXTERNAL`
  signatures. Replace unsupported dynamic divert-target variables with direct
  diverts or typed values. Use `[]` only where the expected array type is known.
- tests: `typed_default_initializers_run_at_runtime`,
  `typed_default_initializers_are_lowered_to_json`, `array_literals_run_at_runtime`,
  `struct_literals_run_at_runtime`, `field_access_reads_struct_fields_at_runtime`,
  `index_access_reads_array_items_at_runtime`,
  `field_assignment_writes_struct_fields_at_runtime`,
  `index_assignment_writes_array_items_at_runtime`,
  `len_returns_array_length_at_runtime`,
  `array_remove_mutates_arrays_and_returns_void_at_runtime`,
  `typed_external_calls_keep_runtime_shape_and_return_values`, and
  `tail_recursion_rewrites_parameters_and_preserves_other_recursion` in
  `crates/ink-test/tests/language.rs`, plus the related compiler analysis,
  parser, runtime, and JSON format unit tests.

## 2026-04-25: LIST Declarations Removed

- status: removed
- upstream behavior: upstream Ink supports `LIST` declarations for named list
  origins and list items, documented in the upstream "Advanced State Tracking"
  list sections. It also exposes `LIST_*` builtin functions and serializes list
  definitions and list values in compiled story JSON.
- ink-rs behavior: `LIST` declarations produce a removed-feature diagnostic.
  Compiled story JSON no longer serializes `listDefs`, list value objects, or
  list-specific runtime tokens.
- documentation effect: `WritingWithInk-latest.md` removes the upstream list
  documentation from the main table of contents and body, and records the
  divergence in "Changed from upstream Ink".
- rationale: the Rust language surface is being reduced to features that are
  actively maintained and useful for the current project direction.
- migration guidance: use variables, functions, or host-side data for inventory
  and set-like game state until a replacement list design is added.
- tests: `removed_list_declaration_reports_removed_feature_diagnostic` in
  `crates/ink-test/tests/language.rs`.
