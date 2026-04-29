Progress: 64/64 steps complete

# Typed Value Implementation Task List

This task list breaks down `requirement_plan.md` into small, gateable commits.
After each step is complete, run the focused validation listed for that step,
then run `make gate`. If `make gate` passes, commit immediately and record the
commit hash in that step. Do not continue implementing the next step in the
same uncommitted change.

Completion protocol for every step:

- Update the step checklist from `[ ]` to `[x]`.
- Update `Progress: X/64 steps complete`.
- Run the step's focused validation.
- Run `make gate`.
- Commit immediately after `make gate` passes.
- Record the commit hash in the step.

Global forbidden boundaries:

- Do not add fixture-name, fixture-path, or expected-output special cases.
- Do not hide failures with ignored tests, skipped fixtures, or relaxed asserts.
- Do not keep final compatibility code for old untyped declarations unless
  explicitly added to `requirement_plan.md`.
- Do not let compiler lowering perform type checking that belongs in analysis.
- Do not let runtime execution objects leak back into compiler parsed model.
- Do not change unrelated language behavior while implementing typed values.

## Phase 0: Baseline And Guardrails

### Step 01: Inventory Current Variable And Function Pipeline

Implementation method:

- Trace current parser nodes for `VAR`, `temp`, function arguments, returns,
  expressions, and external declarations.
- Trace lowering from parsed values into `ink-story-json-format`.
- Trace runtime JSON read/write for primitive values and variable state.
- Write a short implementation note in this step summarizing the exact files
  that must change.

Implementation note:

- Current parsed model:
  - `crates/ink-compiler/src/parsed/variable_assignment.rs` stores only
    `name`, required `expression`, `is_global`, `is_temporary`, and `span`.
    Typed declarations and omitted initializers must extend this model.
  - `crates/ink-compiler/src/parsed/flow.rs` stores `FlowArgument` as
    name/ref/divert-target metadata and `Flow` as `is_function` only. Typed
    parameters and function return types must be added here.
  - `crates/ink-compiler/src/parsed/external_declaration.rs` stores only
    external argument names. Typed external arg and return signatures must be
    added here.
  - `crates/ink-compiler/src/parsed/expression.rs` currently supports strings,
    primitive numeric/bool literals, divert targets, variable refs, function
    calls, binary/unary expressions, and multiple conditions. Array literals,
    struct literals, field access, and index access must be added here.
  - `crates/ink-compiler/src/parsed/mod.rs` and
    `crates/ink-compiler/src/parsed/visit.rs` must be updated whenever new
    parsed nodes or expression variants are added.
- Current syntax pipeline:
  - `crates/ink-compiler/src/syntax/variable.rs` parses untyped `VAR name =
    expr`, `~ temp name = expr`, simple reassignment, and variable-only
    inc/dec. It must parse explicit types, omitted typed initializers, and
    complex lvalue assignments.
  - `crates/ink-compiler/src/syntax/knot.rs` parses untyped function/knot
    arguments and no return type. It must parse typed parameters and `->`
    return types.
  - `crates/ink-compiler/src/syntax/declaration.rs` parses `EXTERNAL name(args)`
    without types. It must parse typed external signatures.
  - `crates/ink-compiler/src/syntax/expression.rs` tokenizes only identifiers,
    literals, calls, operators, parentheses, commas, and divert arrows. It must
    add reusable type parsing plus array/object literal and access syntax.
  - `crates/ink-compiler/src/syntax/parser.rs` must route new top-level
    `STRUCT` declarations and keep parse diagnostics/rule order coherent.
  - `crates/ink-compiler/src/syntax/logic.rs` must continue wrapping
    function-call-containing return/assignment expressions after new expression
    variants are added.
- Current analysis pipeline:
  - `crates/ink-compiler/src/analysis/mod.rs` only runs constants, warnings,
    naming, flow, and target diagnostics. Typed symbol and expression analysis
    must be inserted here before lowering.
  - `crates/ink-compiler/src/analysis/context.rs`,
    `crates/ink-compiler/src/analysis/variables.rs`, and
    `crates/ink-compiler/src/analysis/targets.rs` currently index names and
    scopes without types. They must carry declared types, function signatures,
    external signatures, and expression type results.
  - `crates/ink-compiler/src/analysis/flow.rs` currently checks structural
    function restrictions and return placement, not return types or condition
    types. It must participate in typed return and condition diagnostics.
  - `crates/ink-compiler/src/analysis/names.rs`,
    `crates/ink-compiler/src/analysis/span.rs`,
    `crates/ink-compiler/src/analysis/test_support.rs`, and
    `crates/ink-compiler/src/diagnostic.rs` must be updated for struct names,
    new spans, tests, and typed diagnostics as needed.
- Current lowering/emission pipeline:
  - `crates/ink-compiler/src/lower.rs` lowers global declarations and
    assignments directly from `VariableAssignment.expression()` into
    `ink_story_json_format::Object`. It must use typed default initializers and
    complex lvalue operations.
  - `crates/ink-compiler/src/lower/expression.rs` lowers only current
    expression variants and primitive literals. It must lower arrays, objects,
    field/index access, typed builtins, and typed external/function calls.
  - `crates/ink-compiler/src/lower/flow.rs` lowers untyped parameters as temp
    variable assignments. It must preserve typed parameter metadata through
    analysis while keeping story JSON dynamic.
  - `crates/ink-compiler/src/lower/indexes.rs` currently collects variable
    declarations and external/Ink call arity signatures. It must collect typed
    signatures and declarations for lowering support after analysis owns type
    checking.
  - `crates/ink-compiler/src/lower/conditional.rs`,
    `crates/ink-compiler/src/lower/weave.rs`, and
    `crates/ink-compiler/src/lower/sequence.rs` must be adjusted when new
    expression/lvalue forms appear inside conditions, choices, sequences, and
    content lists.
  - `crates/ink-compiler/src/emit.rs` already delegates JSON writing to
    `ink-story-json-format`; it only needs changes if format serialization
    APIs change.
- Current format/runtime value pipeline:
  - `crates/ink-story-json-format/src/model.rs` and
    `crates/ink-story-json-format/src/json.rs` currently model primitive
    runtime values plus containers, diverts, variable commands, native
    functions, and void. They must add dynamic array/object value variants and
    roundtrip JSON support without static type metadata.
  - `crates/ink-runtime/src/value_type.rs` and
    `crates/ink-runtime/src/value.rs` currently store bool/int/float/string,
    divert targets, and variable pointers. They must add array and object
    storage with clone/value-copy behavior.
  - `crates/ink-runtime/src/json/json_read.rs` converts
    `ink-story-json-format` objects into runtime objects and save values. It
    must map dynamic arrays/objects into runtime values.
  - `crates/ink-runtime/src/json/json_write.rs` writes runtime values through
    `ink-story-json-format`. It must serialize dynamic arrays/objects for story
    JSON/save JSON.
  - `crates/ink-runtime/src/variables_state.rs`,
    `crates/ink-runtime/src/story_state.rs`, and
    `crates/ink-runtime/src/state_patch.rs` currently save/load/copy primitive
    variable state. They must preserve array/object values and compare them
    structurally for default-value elision and equality.
  - `crates/ink-runtime/src/native_function_call.rs` currently performs legacy
    runtime coercion and primitive native operations. It must add typed array
    builtins, recursive equality, and string concatenation behavior required by
    the new typed language.
  - `crates/ink-runtime/src/story/control_logic.rs` is the execution point for
    variable assignment/reference, native calls, eval stack operations, and
    should receive new field/index read/write operations.
- Test and documentation files that must change across later steps include
  `crates/ink-test` fixtures/snapshots, compiler/runtime crate unit tests,
  `docs/SyntaxUpdates.md`, `docs/SyntaxReference.md`, and any
  JSON format docs added for dynamic arrays/objects.

Acceptance method:

- No production behavior changes.
- `cargo fmt --all --check`
- `cargo check --workspace`
- `make gate`

Forbidden boundaries:

- Do not edit parser/lowering/runtime behavior in this step.
- Do not migrate fixtures yet.

Checklist:

- [x] Inventory written in this step
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 7c4b9a1

### Step 02: Add Typed-Value Test Matrix Document

Implementation method:

- Add or extend a local test-planning note section in this file listing positive
  and negative fixture categories for typed values.
- Include primitives, structs, arrays, functions, externals, equality, runtime
  errors, and tail recursion.

Typed-value test matrix:

- Primitive typed variables:
  - Positive: typed global `VAR` declarations for `int`, `float`, `bool`, and
    `string`; typed `temp` declarations in root and flow scopes; omitted
    initializers using default values; valid later reassignment with the exact
    declared type.
  - Negative: missing type annotations after migration, initializer type
    mismatch, reassignment type mismatch, `int` to `float` assignment, `float`
    to `int` assignment, assigning `bool`/`string` to numeric variables, and
    `void` variables.
- Primitive expressions and conditions:
  - Positive: valid numeric arithmetic with matching numeric types, bool
    operators on bool values, string concatenation with `string + string`, and
    bool conditions in choices, conditionals, sequences, and conditional
    diverts.
  - Negative: implicit numeric widening, mixed primitive arithmetic, ordered
    string comparison, non-bool conditions, `string + int`, `int + string`, and
    use of `void` expression results where a value is required.
- Struct declarations and struct values:
  - Positive: top-level `STRUCT` declarations, primitive fields, nested struct
    fields, array fields, full object literals, partial object literals with
    defaulted missing fields, field reads, field writes, nested field access,
    and value-copy assignment between struct variables.
  - Negative: duplicate struct names, duplicate field names, unknown field
    types, unknown literal fields, duplicate literal fields, wrong field value
    types, field access on non-struct values, unknown field reads/writes, and
    assigning the wrong value type to a field.
- Arrays:
  - Positive: empty arrays with expected type, primitive arrays, struct arrays,
    nested arrays such as `int[][]`, index reads, index writes, nested
    field/index chains like `party[0].hp`, and value-copy assignment between
    array variables.
  - Negative: empty array without expected type, mixed element types,
    non-`int` index expressions, indexing non-array values, assigning wrong
    element types, indexed assignment to non-lvalues, and out-of-bounds read or
    write runtime errors.
- Typed functions:
  - Positive: typed parameters for primitives, arrays, structs, and nested
    arrays; declared primitive/composite return types; `void` functions with
    bare `return`; valid function calls in expressions and logic lines; valid
    by-reference behavior where still supported.
  - Negative: missing parameter types, `void` parameters, missing return type
    after migration, wrong argument count, wrong argument type, value return in
    `void` functions, bare return in non-`void` functions, and wrong return
    expression type.
- Typed externals:
  - Positive: `EXTERNAL` signatures with typed primitive/composite arguments
    and typed return values; valid calls; return values used in typed
    expressions; existing host binding behavior for primitive values.
  - Negative: missing argument types, missing return type, wrong call arity,
    wrong call argument type, and use of unsupported host-return shapes if the
    runtime API cannot provide them yet.
- Builtins:
  - Positive: `LEN(T[]) -> int` for empty arrays, primitive arrays, struct
    arrays, and nested arrays; `ARRAY_REMOVE(T[], int) => void` removing from
    beginning, middle, and end.
  - Negative: `LEN` on non-arrays, wrong `LEN` arity, `ARRAY_REMOVE` on
    non-arrays, non-`int` removal index, wrong `ARRAY_REMOVE` arity,
    non-lvalue array targets if required by implementation, and out-of-bounds
    remove runtime errors.
- Equality and inequality:
  - Positive: primitive equality with identical types, array equality with
    equal lengths and recursive element equality, struct equality with all
    fields equal, nested array/struct equality, and inequality as logical
    negation.
  - Negative: equality across unrelated types, equality between `int` and
    `float`, ordered comparisons for arrays/structs/strings, arrays with
    unequal lengths, objects with missing or unequal fields, and nested
    mismatches.
- Runtime JSON and save JSON:
  - Positive: `ink-story-json-format` roundtrips primitive values, arrays,
    nested arrays, objects, nested object/array combinations, and dynamic
    values inside containers; runtime story loading converts those values;
    save/load preserves global array/object values and evaluation stack values
    where reachable.
  - Negative: malformed array/object JSON, unsupported object tokens that look
    like compiled story commands, non-value JSON where a runtime value is
    expected, and save-state reload failures for invalid dynamic value shapes.
- Tail recursion:
  - Positive: direct self tail return `return f(...)` updates parameters and
    jumps without adding another function callstack layer; deep self-recursive
    stress case; argument evaluation order; existing non-recursive and normal
    recursive functions remain correct.
  - Negative: mutual recursion, non-tail self calls such as `return 1 + f(...)`,
    recursive calls inside larger expressions, calls not in actual return-tail
    position, and tests that would pass merely by increasing runtime stack
    limits.
- Migration and compatibility:
  - Positive: migrated primitive `VAR`, `temp`, function, and `EXTERNAL`
    fixtures preserve story output and JSON behavior except for intentional
    typed syntax changes; docs examples compile or mirror language tests.
  - Negative: untyped declarations after migration, skipped/ignored legacy
    fixtures, hidden compatibility branches, and diagnostics that collapse into
    ambiguous parse failures instead of explicit missing-type messages.

Acceptance method:

- The matrix is detailed enough that later implementation steps can point to it.
- `cargo fmt --all --check`
- `make gate`

Forbidden boundaries:

- Do not add incomplete tests that fail.
- Do not modify language behavior.

Checklist:

- [x] Test matrix added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: cc06fc8

## Phase 1: Type Model Foundations

### Step 03: Add Parsed Type Representation

Implementation method:

- Add a parsed type model for primitive types, named struct types, `void`, and
  array types.
- Support nested arrays in the model, e.g. `int[][]`.
- Add display/snapshot helpers for type names.

Acceptance method:

- Unit tests cover parsing-independent construction/display of all type shapes.
- Existing parser snapshots are unchanged.
- `cargo test -p ink-compiler parsed::`
- `make gate`

Forbidden boundaries:

- Do not parse new syntax yet.
- Do not change runtime JSON.

Checklist:

- [x] Type enum/model added
- [x] Type display/snapshot helpers tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 738bfd9

### Step 04: Add Default Value Model In Compiler

Implementation method:

- Add compiler-side helpers that describe default values for `int`, `float`,
  `bool`, `string`, arrays, and structs.
- Keep struct defaults abstract until struct declarations exist.

Acceptance method:

- Unit tests cover primitive and array default construction metadata.
- `cargo test -p ink-compiler`
- `make gate`

Forbidden boundaries:

- Do not emit default values into JSON yet.
- Do not assume unknown struct fields.

Checklist:

- [x] Default value helper added
- [x] Primitive defaults tested
- [x] Array default tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 6547db5

### Step 05: Add Runtime Format Value Variants For Arrays And Objects

Implementation method:

- Extend `ink-story-json-format` object/value model with array values and
  object/struct values represented as dynamic JSON-compatible values.
- Preserve existing primitive representation unchanged.
- Add `mem -> JSON` and `JSON -> mem` roundtrip tests.

Acceptance method:

- `ink-story-json-format` roundtrips primitives, arrays, nested arrays, objects,
  and nested object/array combinations.
- `cargo test -p ink-story-json-format`
- `make gate`

Forbidden boundaries:

- Do not add runtime execution semantics yet.
- Do not add type metadata to story JSON or save JSON.

Checklist:

- [x] Format value variants added
- [x] JSON reader added
- [x] JSON writer added
- [x] Nested roundtrip tests added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 2d86f3a

### Step 06: Add Runtime Value Storage For Arrays And Objects

Implementation method:

- Extend runtime `Value` to store arrays and object maps if not already covered.
- Ensure clone/copy behavior supports value semantics.
- Keep primitive behavior unchanged.

Acceptance method:

- Runtime unit tests can create, clone, compare basic array/object values.
- Existing runtime tests pass.
- `cargo test -p ink-runtime`
- `make gate`

Forbidden boundaries:

- Do not implement field/index read/write yet.
- Do not alter primitive coercion behavior beyond typed-value needs.

Checklist:

- [x] Runtime array value storage added
- [x] Runtime object value storage added
- [x] Clone/value-copy test added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 1a25523

### Step 07: Wire Format Arrays And Objects Into Runtime JSON Reader

Implementation method:

- Convert format array/object values into runtime `Value`.
- Keep existing primitive conversion unchanged.
- Add tests loading a minimal story JSON containing array/object constants in
  eval paths if possible.

Acceptance method:

- Runtime JSON reader accepts new value shapes.
- `cargo test -p ink-runtime json::`
- `make gate`

Forbidden boundaries:

- Do not implement compiler emission of these values yet.
- Do not add type metadata.

Checklist:

- [x] Runtime JSON reader handles arrays
- [x] Runtime JSON reader handles objects
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 10d9242

### Step 08: Wire Runtime Arrays And Objects Into JSON Writer And Save State

Implementation method:

- Ensure runtime JSON writer can serialize array/object values.
- Ensure save state can persist and reload dynamic array/object values.
- Keep save state version policy unchanged unless required by the owner.

Acceptance method:

- Save/load tests cover array and object values in variable state.
- `cargo test -p ink-runtime story_state`
- `make gate`

Forbidden boundaries:

- Do not add static type schema to save JSON.
- Do not change save-state version unless explicitly required.

Checklist:

- [x] Runtime JSON writer handles arrays
- [x] Save state serializes arrays/objects
- [x] Save state reloads arrays/objects
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 6669781

## Phase 2: Type Syntax Parsing

### Step 09: Parse Type Names

Implementation method:

- Add parser support for primitive type names, named struct type identifiers,
  `void`, and nested `[]` suffixes.
- Keep it as a reusable parser rule.

Acceptance method:

- Unit tests parse `int`, `float`, `bool`, `string`, `void`, `Player`,
  `int[]`, and `Player[][]`.
- `cargo test -p ink-compiler syntax::`
- `make gate`

Forbidden boundaries:

- Do not attach types to declarations yet.
- Do not accept `void` for variables.

Checklist:

- [x] Type parser added
- [x] Nested array type tests added
- [x] Invalid type syntax diagnostics tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: c1f5ae3

### Step 10: Parse Typed Global `VAR`

Implementation method:

- Extend global variable declaration parser to accept `VAR name: Type = expr`.
- Preserve old untyped syntax temporarily only as an intentional migration
  bridge until fixture migration steps remove it.
- Store declared type on parsed variable assignment.

Acceptance method:

- Parser unit tests cover typed globals with primitive and array types.
- Existing tests still pass.
- `cargo test -p ink-compiler syntax::variable`
- `make gate`

Forbidden boundaries:

- Do not silently infer type for new typed declarations.
- Do not remove old fixture support before migration.

Checklist:

- [x] Parsed variable declaration stores type
- [x] Typed global parser tests added
- [x] Existing fixtures still pass
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: a230391

### Step 11: Parse Typed `temp`

Implementation method:

- Extend temp declaration parser to accept `~ temp name: Type = expr`.
- Store declared type on parsed temp declaration.
- Allow omitted initializer in syntax, e.g. `~ temp hp: int`.

Acceptance method:

- Parser tests cover typed temp with and without initializer.
- `cargo test -p ink-compiler syntax::variable`
- `make gate`

Forbidden boundaries:

- Do not allow `void` temp variables.
- Do not infer omitted initializer type from later assignment.

Checklist:

- [x] Typed temp parser added
- [x] Omitted initializer parser test added
- [x] `void` variable rejection tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 38f4509

### Step 12: Parse `STRUCT` Declarations

Implementation method:

- Add parsed model for top-level `STRUCT Name { field: Type }`.
- Support one field per line.
- Reject comma/semicolon field separators.

Acceptance method:

- Parser tests cover valid struct declarations.
- Parser tests reject comma/semicolon separators.
- Parse snapshots include struct declarations.
- `cargo test -p ink-compiler syntax::parser`
- `make gate`

Forbidden boundaries:

- Do not allow nested `STRUCT` declarations inside flows.
- Do not assign field defaults in parser.

Checklist:

- [x] Struct parsed model added
- [x] Struct parser added
- [x] Field syntax tests added
- [x] Parse snapshot support added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 450e449

### Step 13: Parse Array Literals

Implementation method:

- Add expression syntax for `[expr, expr]` and `[]`.
- Ensure parser handles nested arrays and expressions inside arrays.

Acceptance method:

- Parser tests cover empty, primitive, nested, and struct-containing arrays.
- `cargo test -p ink-compiler syntax::expression`
- `make gate`

Forbidden boundaries:

- Do not type-check element homogeneity in parser.
- Do not confuse choice bracket syntax with expression array literals.

Checklist:

- [x] Array literal expression node added
- [x] Empty array parser test added
- [x] Nested array parser test added
- [x] Choice syntax regression test added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: e40290b

### Step 14: Parse Struct Literals

Implementation method:

- Add expression syntax for `{ field: expr, field2: expr }` when used in
  expression position.
- Preserve existing conditional/sequence braced content parsing trial order.

Acceptance method:

- Parser tests cover struct literal initializers.
- Regression tests cover existing braced conditionals/sequences.
- `cargo test -p ink-compiler syntax::expression`
- `make gate`

Forbidden boundaries:

- Do not treat all braced content as struct literals.
- Do not require type name in the literal.

Checklist:

- [x] Struct literal expression node added
- [x] Expression-position parsing tested
- [x] Braced content regression tests added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: b9de2e8

### Step 15: Parse Field Access Expressions

Implementation method:

- Add expression node for `base.field`.
- Preserve dotted path behavior for divert/read-count resolution where it is not
  an expression field access.

Acceptance method:

- Parser tests cover `state.hp`, nested `state.stats.hp`, and field access in
  output expressions.
- Existing divert path tests pass.
- `cargo test -p ink-compiler syntax::expression`
- `make gate`

Forbidden boundaries:

- Do not reinterpret divert targets as struct field access.
- Do not resolve field names in parser.

Checklist:

- [x] Field access expression node added
- [x] Parser tests added
- [x] Divert path regression tests pass
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: e09904d

### Step 16: Parse Index Access Expressions

Implementation method:

- Add expression node for `base[index]`.
- Support chaining with fields, e.g. `items[0].hp`.

Acceptance method:

- Parser tests cover `items[0]`, `items[i]`, `party[0].hp`, and nested arrays.
- Choice bracket syntax remains unchanged.
- `cargo test -p ink-compiler syntax::expression`
- `make gate`

Forbidden boundaries:

- Do not type-check index expression in parser.
- Do not parse choice text brackets as array indexes.

Checklist:

- [x] Index access expression node added
- [x] Chained access tests added
- [x] Choice bracket regression test added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 8d00f17

### Step 17: Parse Field And Index Assignment Targets

Implementation method:

- Extend logic-line assignment target parsing for `state.hp = value` and
  `items[1] = value`.
- Preserve existing simple variable reassignment.

Acceptance method:

- Parser tests cover field assignment, index assignment, nested field/index
  assignment, and simple variable assignment regression.
- `cargo test -p ink-compiler syntax::variable`
- `make gate`

Forbidden boundaries:

- Do not allow assignment to arbitrary non-lvalue expressions.
- Do not type-check assignment target yet.

Checklist:

- [x] Assignment target model added
- [x] Field assignment parser test added
- [x] Index assignment parser test added
- [x] Simple assignment regression test added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 2b11969

### Step 18: Parse Compound Assignment For Fields And Indexes

Implementation method:

- Extend `+=`, `-=`, and existing inc/dec handling where applicable to complex
  lvalues.
- At minimum support `+=` for fields and indexes.

Acceptance method:

- Parser tests cover `state.hp += 1` and `items[0] += 1`.
- Existing postfix increment tests pass.
- `cargo test -p ink-compiler syntax::variable`
- `make gate`

Forbidden boundaries:

- Do not lower compound assignment by duplicating side-effectful index
  expressions incorrectly.
- Do not implement only fixture-specific compound forms.

Checklist:

- [x] Compound lvalue parser support added
- [x] Field compound assignment test added
- [x] Index compound assignment test added
- [x] Existing inc/dec tests pass
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 247cbd0

### Step 19: Parse Typed Function Signatures

Implementation method:

- Extend function flow parser to accept typed args and `=> ReturnType`.
- Store parameter types and return type on parsed `Flow`.
- Use `void` for no meaningful return.

Acceptance method:

- Parser tests cover primitive, array, struct, nested array return types.
- Parser rejects missing parameter types for new typed functions.
- `cargo test -p ink-compiler syntax::knot`
- `make gate`

Forbidden boundaries:

- Do not infer argument or return types.
- Do not allow `void` parameters.

Checklist:

- [x] Function signature parser updated
- [x] Parsed flow stores typed signature
- [x] Positive signature tests added
- [x] Negative signature tests added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: ee443bd

### Step 20: Parse Typed `EXTERNAL` Signatures

Implementation method:

- Extend external declaration syntax to include typed args and return type.
- Store external function signature in parsed model.

Acceptance method:

- Parser tests cover `EXTERNAL name(a: int, b: string) => bool`.
- Parser rejects missing arg or return types.
- `cargo test -p ink-compiler syntax::declaration`
- `make gate`

Forbidden boundaries:

- Do not change runtime external binding behavior yet.
- Do not allow untyped external declarations as final behavior.

Checklist:

- [x] External signature parser updated
- [x] Parsed external stores typed signature
- [x] Positive external tests added
- [x] Negative external tests added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: b87ed13

## Phase 3: Type Analysis

### Step 21: Build Struct Type Symbol Index

Implementation method:

- Add analysis pass that collects struct declarations.
- Detect duplicate struct names and duplicate fields.
- Resolve field type names and nested arrays.

Acceptance method:

- Unit tests cover duplicates, unknown field types, recursive references if
  unsupported, and valid nested structures.
- `cargo test -p ink-compiler analysis::`
- `make gate`

Forbidden boundaries:

- Do not lower struct declarations.
- Do not use runtime values for type symbols.

Checklist:

- [x] Struct symbol index added
- [x] Duplicate struct diagnostic tested
- [x] Duplicate field diagnostic tested
- [x] Unknown field type diagnostic tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: c1fc3bc

### Step 22: Build Typed Variable Scope Index

Implementation method:

- Extend variable scope analysis with declared types for globals, temps, and
  function parameters.
- Preserve current visibility rules.

Acceptance method:

- Unit tests cover global, temp, argument, shadowing, and flow boundaries.
- `cargo test -p ink-compiler analysis::variables`
- `make gate`

Forbidden boundaries:

- Do not change variable visibility semantics.
- Do not infer untyped declarations as final behavior.

Checklist:

- [x] Variable type scope index added
- [x] Global type visibility tested
- [x] Temp type visibility tested
- [x] Function arg type visibility tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: afd9c72

### Step 23: Add Expression Type Inference For Primitives

Implementation method:

- Add analysis helper that infers primitive expression types.
- Cover literals, variables, unary operators, numeric operators, boolean
  operators, and string concatenation.

Acceptance method:

- Unit tests cover valid primitive expressions and invalid no-conversion cases.
- `cargo test -p ink-compiler analysis::`
- `make gate`

Forbidden boundaries:

- Do not allow implicit numeric widening.
- Do not let runtime coercion define compiler type rules.

Checklist:

- [x] Primitive expression type inference added
- [x] No implicit conversion tests added
- [x] String concatenation test added
- [x] Boolean operator tests added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 82db26d

### Step 24: Type-Check Variable Initializers

Implementation method:

- Check `VAR` and `temp` initializer expressions against declared types.
- Insert default initializer metadata for omitted initializer cases.

Acceptance method:

- Tests cover valid and invalid primitive initializers.
- Tests cover omitted initializer defaults.
- `cargo test -p ink-compiler analysis::`
- `make gate`

Forbidden boundaries:

- Do not silently coerce initializer types.
- Do not emit default values in analysis if lowering is responsible for output.

Checklist:

- [x] Initializer type checks added
- [x] Omitted default handling tested
- [x] Invalid initializer diagnostics tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 673935c

### Step 25: Type-Check Simple Assignment

Implementation method:

- Check reassignment `x = expr` against variable declared type.
- Check compound assignment type rules for simple variables.

Acceptance method:

- Tests cover valid assignment and invalid reassignment.
- Tests cover `+=` on numeric and string values where supported.
- `cargo test -p ink-compiler analysis::`
- `make gate`

Forbidden boundaries:

- Do not permit assigning undeclared variables.
- Do not allow `float` assigned to `int` or `int` assigned to `float`.

Checklist:

- [x] Simple assignment type checks added
- [x] Compound assignment type checks added
- [x] Invalid assignment diagnostics tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: c6ae76a

### Step 26: Type-Check Struct Literals

Implementation method:

- Determine expected struct type from variable/function context.
- Check provided fields exist and values match field types.
- Apply defaults for missing fields.

Acceptance method:

- Tests cover full literal, partial literal, unknown field, wrong field type,
  nested struct, and array field defaults.
- `cargo test -p ink-compiler analysis::`
- `make gate`

Forbidden boundaries:

- Do not infer struct type from field names alone.
- Do not accept duplicate fields in one literal.

Checklist:

- [x] Struct literal type checking added
- [x] Missing field defaults tested
- [x] Unknown/duplicate field diagnostics tested
- [x] Nested struct literal tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: e63adf5

### Step 27: Type-Check Array Literals

Implementation method:

- Check array literals against expected `T[]`.
- Ensure every element is exactly `T`.
- Support nested arrays.

Acceptance method:

- Tests cover primitive arrays, struct arrays, nested arrays, empty arrays, and
  mixed-type diagnostics.
- `cargo test -p ink-compiler analysis::`
- `make gate`

Forbidden boundaries:

- Do not infer heterogeneous union types.
- Do not accept empty array without expected type.

Checklist:

- [x] Array literal type checking added
- [x] Empty expected-type test added
- [x] Mixed element diagnostic tested
- [x] Nested array test added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 2554f18

### Step 28: Type-Check Field Access

Implementation method:

- Resolve base expression type.
- Ensure base is a struct type.
- Resolve field type.

Acceptance method:

- Tests cover valid field read, nested field read, unknown field, and field
  access on non-struct.
- `cargo test -p ink-compiler analysis::`
- `make gate`

Forbidden boundaries:

- Do not allow field access on dynamic object values without a struct type.
- Do not resolve dotted divert paths as struct fields.

Checklist:

- [x] Field access type resolution added
- [x] Unknown field diagnostic tested
- [x] Non-struct access diagnostic tested
- [x] Dotted divert regression tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 36ef83d

### Step 29: Type-Check Index Access

Implementation method:

- Resolve base expression type.
- Ensure base is an array.
- Ensure index expression is `int`.
- Return element type.

Acceptance method:

- Tests cover primitive arrays, struct arrays, nested arrays, non-int index,
  and indexing non-array values.
- `cargo test -p ink-compiler analysis::`
- `make gate`

Forbidden boundaries:

- Do not check runtime bounds statically except for obvious constant cases if
  already available.
- Do not allow string indexing unless separately requested.

Checklist:

- [x] Index access type resolution added
- [x] Non-int index diagnostic tested
- [x] Non-array index diagnostic tested
- [x] Nested array indexing tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: fb514a1

### Step 30: Type-Check Field And Index Assignment

Implementation method:

- Resolve assignment target type for field/index lvalues.
- Check assigned expression type exactly matches target type.
- Apply compound assignment type rules.

Acceptance method:

- Tests cover valid field/index assignment and invalid type assignments.
- Tests cover nested lvalue assignments.
- `cargo test -p ink-compiler analysis::`
- `make gate`

Forbidden boundaries:

- Do not evaluate index expressions more than once in later lowering design.
- Do not allow assigning to non-lvalue expressions.

Checklist:

- [x] Complex lvalue type checking added
- [x] Field assignment diagnostics tested
- [x] Index assignment diagnostics tested
- [x] Nested assignment tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 34dac55

### Step 31: Type-Check Function Calls

Implementation method:

- Use typed function signatures for argument count and argument type checks.
- Return the declared return type for function call expressions.

Acceptance method:

- Tests cover valid calls, wrong arg count, wrong arg type, struct args, array
  args, and return type use.
- `cargo test -p ink-compiler analysis::targets`
- `make gate`

Forbidden boundaries:

- Do not rely on runtime arity errors for statically known Ink function calls.
- Do not allow untyped function args in final behavior.

Checklist:

- [x] Function call type checking added
- [x] Wrong arg count diagnostic tested
- [x] Wrong arg type diagnostic tested
- [x] Composite type call tests added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 7608795

### Step 32: Type-Check Function Returns

Implementation method:

- Check `return expr` against current function declared return type.
- Check bare `return` only in `void` functions.
- Diagnose value return in `void` functions.

Acceptance method:

- Tests cover valid primitive/composite returns, missing value, extra value, and
  wrong return type.
- `cargo test -p ink-compiler analysis::flow`
- `make gate`

Forbidden boundaries:

- Do not attempt full control-flow proof beyond statically detectable cases.
- Do not let runtime function return errors replace compiler diagnostics.

Checklist:

- [x] Return type checking added
- [x] Bare return tests added
- [x] Wrong return diagnostics tested
- [x] Composite return tests added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: edfe7c5

### Step 33: Type-Check Conditions

Implementation method:

- Require conditional expressions to be `bool` in choices, conditionals,
  sequences if relevant, and conditional diverts.
- Remove legacy truthiness for typed language paths.

Acceptance method:

- Tests cover boolean conditions and reject `int`, `float`, `string`, array,
  and struct conditions.
- `cargo test -p ink-compiler analysis::flow`
- `make gate`

Forbidden boundaries:

- Do not preserve numeric truthiness in typed source.
- Do not change unrelated output text behavior.

Checklist:

- [x] Condition type checks added
- [x] Non-bool condition diagnostics tested
- [x] Existing bool behavior tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 11af20b

### Step 34: Type-Check `LEN`

Implementation method:

- Add typed builtin signature for `LEN(T[]) -> int`.
- Reject non-array arguments.

Acceptance method:

- Tests cover `LEN(int[])`, `LEN(Player[])`, `LEN(int[][])`, and non-array
  diagnostics.
- `cargo test -p ink-compiler analysis::`
- `make gate`

Forbidden boundaries:

- Do not implement runtime behavior in analysis.
- Do not accept multiple args.

Checklist:

- [x] `LEN` type signature added
- [x] Valid `LEN` tests added
- [x] Invalid `LEN` diagnostics tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 7ab65ef

### Step 35: Type-Check `ARRAY_REMOVE`

Implementation method:

- Add typed builtin signature `ARRAY_REMOVE(T[], int) => void`.
- Ensure first arg is mutable lvalue if the language requires in-place mutation.

Acceptance method:

- Tests cover valid primitive, struct, nested array removal.
- Tests reject non-array, non-int index, wrong arity, and non-lvalue array
  target if required.
- `cargo test -p ink-compiler analysis::`
- `make gate`

Forbidden boundaries:

- Do not add `PUSH`, `POP`, or `PEEK`.
- Do not make `ARRAY_REMOVE` return the removed value.

Checklist:

- [x] `ARRAY_REMOVE` type signature added
- [x] Mutability/lvalue rule tested
- [x] Invalid arg diagnostics tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 387c7b6

### Step 36: Type-Check Equality And Inequality

Implementation method:

- Implement equality type rules for primitives, arrays, and structs.
- Implement `!=` as logical negation of equality for arrays and structs.
- Reject equality between unrelated types.

Acceptance method:

- Tests cover primitive equality, array equality, struct equality, nested
  equality, inequality, and mismatched type diagnostics.
- `cargo test -p ink-compiler analysis::`
- `make gate`

Forbidden boundaries:

- Do not implement ordered comparisons for arrays, structs, or strings.
- Do not allow implicit numeric conversion in equality.

Checklist:

- [x] Equality type rules added
- [x] Inequality type rules added
- [x] Nested equality tests added
- [x] Ordered comparison rejection tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: d5b7763

### Step 37: Type-Check `EXTERNAL` Calls

Implementation method:

- Use typed external signatures for compiler-side call validation.
- Ensure return type is available to expression type inference.

Acceptance method:

- Tests cover valid typed external calls, wrong arg count, wrong arg type, and
  using external return type in expressions.
- `cargo test -p ink-compiler analysis::targets`
- `make gate`

Forbidden boundaries:

- Do not change host binding API behavior yet unless needed.
- Do not allow untyped external signatures in final behavior.

Checklist:

- [x] External call type checking added
- [x] External return type inference added
- [x] Invalid external call diagnostics tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 3fef442

## Phase 4: Lowering And Runtime Execution

### Step 38: Lower Default Initializers

Implementation method:

- Lower omitted initializer defaults for typed globals and temps.
- Use compiler default value model and struct symbol data.

Acceptance method:

- Compiler JSON tests show defaults for primitive, array, and struct variables.
- Runtime smoke tests output default values.
- `cargo test -p ink-test --test language`
- `make gate`

Forbidden boundaries:

- Do not put static type metadata in story JSON.
- Do not duplicate default logic ad hoc in emit tests.

Checklist:

- [x] Global defaults lowered
- [x] Temp defaults lowered
- [x] Struct defaults lowered
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: e9e1b4b

### Step 39: Lower Array Literals

Implementation method:

- Lower parsed array literals into format/runtime array values.
- Support nested arrays and arrays of struct values.

Acceptance method:

- Compiler conformance fixtures cover array literals.
- Runtime language tests output indexed array contents.
- `cargo test -p ink-test --test compiler_conformance`
- `make gate`

Forbidden boundaries:

- Do not emit type metadata.
- Do not rely on runtime to fix compiler type errors.

Checklist:

- [x] Array literal lowering added
- [x] Nested array lowering tested
- [x] Array of struct lowering tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 262a72e

### Step 40: Lower Struct Literals

Implementation method:

- Lower struct literals into format/runtime object values.
- Fill missing fields with defaults before emission.

Acceptance method:

- Compiler JSON fixtures cover full, partial, and nested struct values.
- Runtime language tests output struct fields.
- `cargo test -p ink-test --test compiler_conformance`
- `make gate`

Forbidden boundaries:

- Do not emit struct type tags unless requirement changes.
- Do not preserve source field order as semantic behavior unless documented.

Checklist:

- [x] Struct literal lowering added
- [x] Missing field defaults emitted
- [x] Nested struct lowering tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 0ac3de9

### Step 41: Runtime Field Read

Implementation method:

- Add runtime operation/value access for reading `object.field`.
- Lower field access expressions to that operation.
- Return runtime error for missing fields.

Acceptance method:

- Runtime tests cover valid field read and missing field runtime error.
- Compiler language tests cover `{state.hp}`.
- `cargo test -p ink-runtime`
- `make gate`

Forbidden boundaries:

- Do not perform field type checks in runtime.
- Do not silently return void for missing fields.

Checklist:

- [x] Runtime field read operation added
- [x] Compiler lowering added
- [x] Missing field runtime error tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 57ef1d0

### Step 42: Runtime Index Read

Implementation method:

- Add runtime operation/value access for reading `array[index]`.
- Lower index access expressions to that operation.
- Runtime error on out-of-bounds.

Acceptance method:

- Runtime tests cover valid index read and out-of-bounds error.
- Compiler language tests cover `{items[0]}`.
- `cargo test -p ink-runtime`
- `make gate`

Forbidden boundaries:

- Do not use 1-based indexing.
- Do not silently clamp or return void on out-of-bounds.

Checklist:

- [x] Runtime index read operation added
- [x] Compiler lowering added
- [x] Out-of-bounds runtime error tested
- [x] Zero-based behavior tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: cd26e6d

### Step 43: Runtime Field Write

Implementation method:

- Add runtime operation for writing `object.field`.
- Ensure value-copy semantics for variables containing objects.
- Lower field assignment to the operation.

Acceptance method:

- Tests cover `state.hp = 5`.
- Tests cover `p2 = p1; p2.hp = 1` leaves `p1.hp` unchanged.
- `cargo test -p ink-runtime`
- `make gate`

Forbidden boundaries:

- Do not mutate shared object aliases.
- Do not silently create unknown fields unless compiler/runtime design
  explicitly requires it.

Checklist:

- [x] Runtime field write operation added
- [x] Compiler lowering added
- [x] Value-copy struct assignment tested
- [x] Unknown field behavior tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 51024fa

### Step 44: Runtime Index Write

Implementation method:

- Add runtime operation for writing `array[index]`.
- Ensure value-copy semantics for variables containing arrays.
- Lower index assignment to the operation.

Acceptance method:

- Tests cover `items[1] = 10`.
- Tests cover `items2 = items1; items2[0] = 9` leaves `items1[0]` unchanged.
- Tests cover out-of-bounds write runtime error.
- `cargo test -p ink-runtime`
- `make gate`

Forbidden boundaries:

- Do not auto-grow arrays on indexed assignment.
- Do not mutate shared array aliases.

Checklist:

- [x] Runtime index write operation added
- [x] Compiler lowering added
- [x] Value-copy array assignment tested
- [x] Out-of-bounds write tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 431f582

### Step 45: Lower Compound Field And Index Assignment

Implementation method:

- Lower `target += expr` into read, operation, and write while evaluating target
  side effects once.
- Cover fields and indexes.

Acceptance method:

- Tests cover `state.hp += 1`, `items[0] += 1`, and string concatenation
  compound assignment if supported.
- `cargo test -p ink-test --test language`
- `make gate`

Forbidden boundaries:

- Do not duplicate index expressions with side effects.
- Do not implement compound assignment only for constant indexes.

Checklist:

- [x] Compound field lowering added
- [x] Compound index lowering added
- [x] Single-evaluation behavior tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 9cf6953

### Step 46: Runtime `LEN`

Implementation method:

- Add native/runtime function for `LEN`.
- Return array length as `int`.

Acceptance method:

- Runtime tests cover empty, primitive, struct, and nested arrays.
- Compiler language tests cover `{LEN(items)}`.
- `cargo test -p ink-runtime`
- `make gate`

Forbidden boundaries:

- Do not make `LEN` accept strings unless requirement changes.
- Do not return float.

Checklist:

- [x] Runtime `LEN` added
- [x] Compiler lowering connected
- [x] Array length tests added
- [x] Non-array runtime guard tested if reachable
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 5ffd2d0

### Step 47: Runtime `ARRAY_REMOVE`

Implementation method:

- Add runtime function for `ARRAY_REMOVE`.
- Mutate array in place and return `void`.
- Runtime error on out-of-bounds.

Acceptance method:

- Tests cover removal from beginning, middle, end, and nested/struct arrays.
- Tests cover out-of-bounds error.
- Compiler language tests verify returned value is `void`.
- `cargo test -p ink-runtime`
- `make gate`

Forbidden boundaries:

- Do not return removed element.
- Do not implement `POP`, `PUSH`, or `PEEK`.
- Do not no-op on out-of-bounds.

Checklist:

- [x] Runtime `ARRAY_REMOVE` added
- [x] Compiler lowering connected
- [x] In-place mutation tested
- [x] Out-of-bounds runtime error tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 83b1ad8

### Step 48: Runtime Struct And Array Equality

Implementation method:

- Implement recursive equality for runtime arrays and objects.
- Implement inequality as negation.
- Preserve primitive equality behavior.

Acceptance method:

- Runtime tests cover nested arrays, nested structs, arrays of structs, unequal
  lengths, missing fields, and primitive fields.
- `cargo test -p ink-runtime`
- `make gate`

Forbidden boundaries:

- Do not add ordered comparisons.
- Do not treat different numeric types as equal.

Checklist:

- [x] Runtime recursive equality added
- [x] Runtime inequality added
- [x] Nested equality tests added
- [x] Primitive regression tests pass
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 43ab0b5

### Step 49: Runtime String Concatenation

Implementation method:

- Ensure `string + string` concatenates strings in runtime native operation.
- Keep numeric addition unchanged.

Acceptance method:

- Runtime and language tests cover string concatenation.
- Type analysis still rejects `string + int` and `int + string`.
- `cargo test -p ink-runtime`
- `make gate`

Forbidden boundaries:

- Do not add implicit string conversion.
- Do not change numeric addition.

Checklist:

- [x] String concatenation runtime behavior verified
- [x] Invalid mixed concat diagnostics tested
- [x] Numeric addition regression tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 005e69c

### Step 50: Lower Typed `EXTERNAL` Calls

Implementation method:

- Preserve runtime external call representation while compiler carries typed
  metadata.
- Ensure return value handling works for primitives, arrays, and structs if host
  binding can provide them.

Acceptance method:

- Compiler tests cover typed external JSON.
- Runtime tests cover typed external calls returning supported values where
  current API permits.
- `cargo test -p ink-test --test language`
- `make gate`

Forbidden boundaries:

- Do not add type metadata to story JSON.
- Do not break existing external binding API without explicit migration.

Checklist:

- [x] Typed external lowering verified
- [x] External return value tests added
- [x] Existing external tests pass
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 9427bc0

## Phase 5: Tail Recursion

### Step 51: Detect Direct Self Tail Return

Implementation method:

- Add compiler detection for `return current_function(...)`.
- Only classify calls in actual return-tail position.
- Keep non-tail recursive calls unchanged.

Acceptance method:

- Compiler unit tests classify direct self tail return and reject non-tail forms
  such as `return 1 + f(...)`.
- `cargo test -p ink-compiler`
- `make gate`

Forbidden boundaries:

- Do not optimize mutual recursion.
- Do not optimize calls inside larger expressions.

Checklist:

- [x] Tail-call detection added
- [x] Positive detection test added
- [x] Non-tail negative tests added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 592c5f7

### Step 52: Lower Tail Recursion To Stack-Neutral Jump

Implementation method:

- Lower direct self tail return by updating parameters and jumping to function
  body start without pushing a new function stack frame.
- Preserve evaluation order of arguments.

Acceptance method:

- Runtime test proves repeated tail recursion completes for a depth that would
  otherwise grow stack heavily.
- Existing recursive factorial remains correct.
- `cargo test -p ink-test --test language`
- `make gate`

Forbidden boundaries:

- Do not change semantics of non-tail recursion.
- Do not evaluate tail-call arguments after overwriting parameters.

Checklist:

- [x] Stack-neutral lowering added
- [x] Argument evaluation order tested
- [x] Deep tail recursion test added
- [x] Non-tail recursion regression tested
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: ce37938

### Step 53: Prove Tail Recursion Does Not Grow Function Callstack

Implementation method:

- Add a focused runtime/compiler test with observable callstack depth or a
  stress case that fails without TCO.
- Use direct self recursion only.

Acceptance method:

- Test fails without TCO and passes with TCO.
- `cargo test -p ink-test --test language`
- `make gate`

Forbidden boundaries:

- Do not weaken the test to only check output.
- Do not increase runtime stack limits to hide missing TCO.

Checklist:

- [x] Stack-depth or stress test added
- [x] Test demonstrates stack-neutral behavior
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 298986d

## Phase 6: Fixture Migration And Compatibility Cleanup

### Step 54: Migrate Primitive Global `VAR` Fixtures

Implementation method:

- Update conformance and compiler fixtures with primitive global declarations
  to explicit types.
- Update parse snapshots and JSON expectations.

Acceptance method:

- `cargo test -p ink-test --test compiler_conformance`
- `cargo test -p ink-test --test conformance`
- `make gate`

Forbidden boundaries:

- Do not change fixture story behavior.
- Do not edit expected output except where typed syntax intentionally changes
  JSON shape.

Checklist:

- [x] Primitive global fixtures migrated
- [x] Parse snapshots updated
- [x] JSON fixtures updated
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: cda184c

### Step 55: Migrate Primitive `temp` Fixtures

Implementation method:

- Update temp declarations to explicit typed syntax.
- Update snapshots and JSON expectations.

Acceptance method:

- `cargo test -p ink-test --test compiler_conformance`
- `cargo test -p ink-test --test conformance`
- `make gate`

Forbidden boundaries:

- Do not change variable lifetime or scope behavior.
- Do not use overly broad fixture rewrites without reviewing diffs.

Checklist:

- [x] Primitive temp fixtures migrated
- [x] Parse snapshots updated
- [x] JSON fixtures updated
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: d1bc14b

### Step 56: Migrate Function Signature Fixtures

Implementation method:

- Add explicit parameter and return types to function fixtures.
- Use `=> void` for functions without return value.

Acceptance method:

- Function conformance and csharp compatibility tests pass after migration.
- `cargo test -p ink-test --test compiler_conformance`
- `make gate`

Forbidden boundaries:

- Do not infer function return types in fixtures.
- Do not preserve old signatures in final fixture source.

Checklist:

- [x] Function fixtures migrated
- [x] External references updated
- [x] Parse snapshots updated
- [x] JSON fixtures updated
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: fa06b32

### Step 57: Migrate `EXTERNAL` Fixtures

Implementation method:

- Add typed argument and return signatures to external declarations.
- Update tests for host bindings if needed.

Acceptance method:

- External runtime tests pass.
- `cargo test -p ink-test --test conformance`
- `cargo test -p ink-test --test compiler_conformance`
- `make gate`

Forbidden boundaries:

- Do not break host binding behavior.
- Do not leave untyped external fixtures behind.

Checklist:

- [x] External fixtures migrated
- [x] Host binding tests updated if needed
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 7dceb3e

### Step 58: Remove Transitional Untyped Declaration Support

Implementation method:

- Remove parser compatibility for untyped `VAR`, `temp`, function args, function
  returns, and external declarations.
- Add diagnostics for missing explicit types.

Acceptance method:

- Negative language tests cover each missing-type form.
- Search confirms no untyped fixture declarations remain.
- `cargo test -p ink-test --test language`
- `make gate`

Forbidden boundaries:

- Do not remove the existing removed-feature diagnostic for `LIST` declarations.
- Do not silently infer missing types.

Checklist:

- [x] Untyped global support removed
- [x] Untyped temp support removed
- [x] Untyped function signature support removed
- [x] Untyped external signature support removed
- [x] Missing-type diagnostics tested
- [x] Fixture search clean
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 1ac9ba8

### Step 59: Remove Transitional Analysis/Lowering Paths

Implementation method:

- Delete any temporary compatibility branches introduced for staged migration.
- Simplify analysis and lowering around explicit types only.

Acceptance method:

- `rg` shows no compatibility TODOs for untyped declarations.
- `cargo test --workspace`
- `make gate`

Forbidden boundaries:

- Do not keep dead code for old untyped source syntax.
- Do not remove public APIs still needed by typed implementation.

Checklist:

- [x] Transitional parser paths removed
- [x] Transitional analysis paths removed
- [x] Transitional lowering paths removed
- [x] Dead code search clean
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: f340b6d

## Phase 7: Documentation And Final Validation

### Step 60: Update Syntax Updates

Implementation method:

- Add a dated entry to `docs/SyntaxUpdates.md` describing typed
  variables, structs, arrays, typed functions, typed externals, and TCO.

Acceptance method:

- Documentation describes intentional divergence from upstream Ink.
- `cargo fmt --all --check`
- `make gate`

Forbidden boundaries:

- Do not edit `docs/WritingWithInk.md`.
- Do not document behavior not implemented.

Checklist:

- [x] Updates doc entry added
- [x] Origin doc untouched
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 4ca1de0

### Step 61: Update Syntax Reference

Implementation method:

- Apply the syntax and semantic changes to `docs/SyntaxReference.md`.
- Include examples for typed variables, structs, arrays, functions, externals,
  and array builtins.

Acceptance method:

- Examples compile in dedicated doc/example tests or are mirrored by language
  tests.
- `make gate`

Forbidden boundaries:

- Do not copy upstream list language back in.
- Do not include syntax that lacks tests.

Checklist:

- [x] Latest doc updated
- [x] Examples covered by tests
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: 57a512d

### Step 62: Update JSON Runtime Format Documentation

Implementation method:

- Document dynamic JSON representation of arrays and struct/object values.
- Clarify that static type metadata is not serialized in this phase.

Acceptance method:

- Documentation examples match `ink-story-json-format` tests.
- `cargo test -p ink-story-json-format`
- `make gate`

Forbidden boundaries:

- Do not document type metadata in story/save JSON.
- Do not mention unsupported `PUSH`/`POP`/`PEEK`.

Checklist:

- [x] JSON format docs updated
- [x] Examples match tests
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: c3845da

### Step 63: Add End-To-End Typed Language Fixtures

Implementation method:

- Add small language fixtures covering the full typed feature set:
  primitives, structs, arrays, nested arrays, arrays of structs, typed
  functions, typed externals, equality, `LEN`, `ARRAY_REMOVE`, and TCO.

Acceptance method:

- `cargo test -p ink-test --test language`
- `cargo test -p ink-test --test compiler_conformance`
- `cargo test -p ink-test --test conformance`
- `make gate`

Forbidden boundaries:

- Do not rely on one giant fixture only.
- Do not use fixture-specific code paths.

Checklist:

- [x] Primitive typed fixture added
- [x] Struct typed fixture added
- [x] Array typed fixture added
- [x] Nested typed fixture added
- [x] Typed function fixture added
- [x] Typed external fixture added
- [x] TCO fixture added
- [x] Focused validation passed
- [x] `make gate` passed
- [x] Committed immediately
- Commit: a18ee23

### Step 64: Final Cleanup And Release Gate

Implementation method:

- Search for stale TODOs, transitional comments, untyped syntax fixtures, and
  incomplete docs.
- Run full validation.
- Commit only if cleanup changes are needed.

Acceptance method:

- `rg "TODO|temporary|compat|untyped|PUSH|POP|PEEK"` reviewed for typed-value
  leftovers.
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `make gate`

Forbidden boundaries:

- Do not make broad unrelated refactors.
- Do not skip gate because focused tests passed.

Checklist:

- [x] Stale typed-value TODOs reviewed
- [x] Untyped fixture search clean
- [x] Unsupported builtin search clean
- [x] Full workspace tests passed
- [x] `make gate` passed
- [x] Final cleanup committed if needed
- Commit: N/A (no cleanup changes)
