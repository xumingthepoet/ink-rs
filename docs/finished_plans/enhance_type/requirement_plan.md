# Typed Value Syntax Extension Requirement Plan

Status: draft requirements captured from project owner on 2026-04-26.

This document records the planned language/runtime-format direction before
implementation. It should be updated as details are confirmed.

## Goal

Replace the current dynamic variable model with statically declared typed values.
The compiler should type-check variables, functions, structs, arrays, field
access, index access, and typed builtins before lowering. Runtime values may
still be stored dynamically in story JSON and save JSON.

## Type System

- Supported primitive types:
  - `int`
  - `float`
  - `bool`
  - `string`
- Supported composite types:
  - struct types declared in Ink source
  - array types written as `T[]`
  - arrays may contain primitive values or struct values
  - nested arrays such as `int[][]`
- No implicit type conversion is allowed.
  - `int` must not implicitly convert to `float`.
  - `float` must not assign to `int`.
  - `bool`, `string`, numeric values, arrays, and structs must not implicitly
    convert between each other.
- Type errors should be compiler diagnostics where enough static information is
  available.

## Variables

- Global declarations require explicit type syntax:

  ```ink
  VAR hp: int = 10
  ```

- Temporary declarations require explicit type syntax:

  ```ink
  ~ temp hp: int = 10
  ```

- The new language surface should migrate old untyped `VAR` and `temp`
  fixtures to typed declarations.
- The compiler should check initializer type compatibility against the declared
  type.
- The compiler should check later assignment type compatibility against the
  declared type.
- If a typed `VAR` or `temp` omits an initializer, it is allowed and uses the
  default value for the declared type:
  - `int`: `0`
  - `float`: `0.0`
  - `bool`: `false`
  - `string`: `""`
  - arrays: `[]`
  - structs: recursively default-initialized field values

## Structs

- Struct declaration syntax:

  ```ink
  STRUCT Player {
    hp: int
    name: string
  }
  ```

- Struct fields are declared one per line. Commas and semicolons between fields
  are not part of the supported syntax.
- Struct values use simple object literal syntax without repeating the type
  name:

  ```ink
  VAR player: Player = { hp: 10, name: "A" }
  ```

- Missing struct fields are filled from default values:
  - `int`: `0`
  - `float`: `0.0`
  - `bool`: `false`
  - `string`: `""`
  - arrays: `[]`
  - structs: recursively default-initialized field values
- Field access is supported:

  ```ink
  {state.hp}
  ```

- Field assignment is supported:

  ```ink
  ~ state.hp = 5
  ```

- Struct assignment uses value-copy semantics. After `p2 = p1`, later writes to
  `p2` do not affect `p1`.
- Structs may contain nested structs, arrays, arrays of structs, and nested
  arrays.
- Compound field assignment should be supported where practical:

  ```ink
  ~ state.hp += 1
  ```

  If compound assignment becomes too costly for the first implementation, plain
  assignment such as `~ state.hp = state.hp + 1` is acceptable as an interim
  path.
- The compiler should diagnose:
  - unknown struct types
  - unknown fields
  - duplicate field declarations
  - invalid field initializer types
  - invalid field assignment types

## Arrays

- Array type syntax is `T[]`:

  ```ink
  VAR scores: int[] = [1, 2, 3]
  VAR party: Player[] = []
  ```

- Array literals use bracket syntax:

  ```ink
  [1, 2, 3]
  ```

- Empty arrays are allowed when the expected type is known:

  ```ink
  VAR items: int[] = []
  ```

- Indexing is zero-based:

  ```ink
  {items[0]}
  ```

- Out-of-bounds index access is a runtime error.
- Indexed assignment is supported:

  ```ink
  ~ items[1] = 10
  ```

- Array assignment uses value-copy semantics. After `items2 = items1`, later
  writes to `items2` do not affect `items1`.
- Nested arrays are supported, for example `int[][]`.
- Every array element must match the array element type exactly. For `T[]`, all
  elements must be `T`.
- Arrays of arrays and arrays of structs are supported.
- The compiler should diagnose:
  - non-`int` index expressions
  - indexing a non-array value
  - assigning the wrong element type
  - array literal elements that do not match the expected element type

## Array Builtins

- `LEN(items)` returns an `int`.
- `ARRAY_REMOVE(list, n)` should be added.
  - It mutates the array in place.
  - It returns `void`.
  - Out-of-bounds removal is a runtime error.
- `PUSH`, `POP`, and `PEEK` are not required for this phase.
- Array reading is done with index access, e.g. `items[n]`.

## Functions

- Function arguments require explicit types.
- Function return type syntax:

  ```ink
  == function add(a: int, b: int) => int ==
    ~ return a + b
  ```

- Functions without a meaningful return value use `void`:

  ```ink
  == function log(message: string) => void ==
    ~ return
  ```

- Function parameters and return values support the new types, including arrays
  and structs.
- The compiler should check:
  - argument count
  - argument types
  - return expression type against the declared return type
  - missing return values in non-`void` functions, where statically detectable
- `EXTERNAL` declarations require typed argument and return signatures in this
  phase.

## Conditions And Operators

- Conditional expressions must have type `bool`.
- `string + string` is supported and concatenates strings.
- `string < string` and other ordered string comparisons are not supported.
- Array and struct equality is supported with recursive value comparison:
  - arrays are equal only when their lengths match and every corresponding
    element is equal
  - structs are equal only when every field value is equal
  - nested arrays and structs compare recursively
  - primitive fields compare by primitive equality
- Array and struct inequality is supported as logical negation of equality.

## Tail Recursion

- Direct self tail recursion must be optimized.
- Required case:

  ```ink
  ~ return f(...)
  ```

  where `f` is the current function.
- This form must not add another function callstack layer at runtime.
- Mutual recursion optimization is not required.
- Non-tail recursive calls may keep the current stack behavior.
- If a user writes recursion that is not in the supported tail-call shape, normal
  recursion behavior is acceptable.

## Story JSON And Save JSON

- Story JSON and save JSON do not need to carry static type metadata for this
  phase.
- Runtime value contents should be stored as dynamic JSON values:
  - existing `string`, `int`, `float`, `bool` representation remains unchanged
  - arrays should be stored as JSON arrays
  - struct values should be stored as JSON objects
- `ink-story-json-format`, compiler, and runtime should share a unified value
  representation for these runtime values, while the wire format remains dynamic.

## Compatibility And Migration

- This is a new syntax direction.
- Existing fixtures that use old untyped declarations should be migrated to the
  explicit typed syntax.
- Do not preserve old untyped `VAR`/`temp` syntax as the long-term source form
  unless the owner explicitly reintroduces a compatibility window.
- Tests and docs should be updated in the same change as implementation.

## Implementation Plan

1. Add typed AST/model concepts:
   - type names
   - array types
   - struct declarations
   - field/index assignment targets
   - typed function signatures
2. Extend parser syntax:
   - typed `VAR`
   - typed `temp`
   - `STRUCT`
   - struct literals
   - array literals
   - field access
   - index access
   - typed function signatures
3. Add type analysis:
   - symbol tables for structs, globals, temps, function args, return types
   - expression type inference/checking
   - assignment target checking
   - builtin signature checking for `LEN` and `ARRAY_REMOVE`
4. Update lowering and runtime value model:
   - array values
   - struct/object values
   - field/index read
   - field/index write
   - `LEN`
   - `ARRAY_REMOVE`
5. Add direct self tail-recursion optimization:
   - detect `return current_function(...)`
   - lower to a stack-neutral jump/update path
   - test that repeated tail recursion does not grow function callstack
6. Migrate fixtures and tests:
   - parser snapshots
   - compiler JSON fixtures
   - runtime conformance
   - language tests for type errors
7. Update documentation:
   - `docs/SyntaxUpdates.md`
   - `docs/SyntaxReference.md`
   - JSON runtime format docs if array/object value wire shape changes
8. Validate:
   - focused compiler tests
   - focused runtime tests
   - `cargo fmt --all --check`
   - `cargo check --workspace`
   - `cargo test --workspace`
   - `make gate`

## Open Questions

- None currently.
