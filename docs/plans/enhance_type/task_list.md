Progress: 0/64 steps complete

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

Acceptance method:

- No production behavior changes.
- `cargo fmt --all --check`
- `cargo check --workspace`
- `make gate`

Forbidden boundaries:

- Do not edit parser/lowering/runtime behavior in this step.
- Do not migrate fixtures yet.

Checklist:

- [ ] Inventory written in this step
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

### Step 02: Add Typed-Value Test Matrix Document

Implementation method:

- Add or extend a local test-planning note section in this file listing positive
  and negative fixture categories for typed values.
- Include primitives, structs, arrays, functions, externals, equality, runtime
  errors, and tail recursion.

Acceptance method:

- The matrix is detailed enough that later implementation steps can point to it.
- `cargo fmt --all --check`
- `make gate`

Forbidden boundaries:

- Do not add incomplete tests that fail.
- Do not modify language behavior.

Checklist:

- [ ] Test matrix added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Type enum/model added
- [ ] Type display/snapshot helpers tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Default value helper added
- [ ] Primitive defaults tested
- [ ] Array default tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Format value variants added
- [ ] JSON reader added
- [ ] JSON writer added
- [ ] Nested roundtrip tests added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Runtime array value storage added
- [ ] Runtime object value storage added
- [ ] Clone/value-copy test added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Runtime JSON reader handles arrays
- [ ] Runtime JSON reader handles objects
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Runtime JSON writer handles arrays
- [ ] Save state serializes arrays/objects
- [ ] Save state reloads arrays/objects
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Type parser added
- [ ] Nested array type tests added
- [ ] Invalid type syntax diagnostics tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Parsed variable declaration stores type
- [ ] Typed global parser tests added
- [ ] Existing fixtures still pass
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Typed temp parser added
- [ ] Omitted initializer parser test added
- [ ] `void` variable rejection tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Struct parsed model added
- [ ] Struct parser added
- [ ] Field syntax tests added
- [ ] Parse snapshot support added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Array literal expression node added
- [ ] Empty array parser test added
- [ ] Nested array parser test added
- [ ] Choice syntax regression test added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Struct literal expression node added
- [ ] Expression-position parsing tested
- [ ] Braced content regression tests added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Field access expression node added
- [ ] Parser tests added
- [ ] Divert path regression tests pass
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Index access expression node added
- [ ] Chained access tests added
- [ ] Choice bracket regression test added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Assignment target model added
- [ ] Field assignment parser test added
- [ ] Index assignment parser test added
- [ ] Simple assignment regression test added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Compound lvalue parser support added
- [ ] Field compound assignment test added
- [ ] Index compound assignment test added
- [ ] Existing inc/dec tests pass
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

### Step 19: Parse Typed Function Signatures

Implementation method:

- Extend function flow parser to accept typed args and `-> ReturnType`.
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

- [ ] Function signature parser updated
- [ ] Parsed flow stores typed signature
- [ ] Positive signature tests added
- [ ] Negative signature tests added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

### Step 20: Parse Typed `EXTERNAL` Signatures

Implementation method:

- Extend external declaration syntax to include typed args and return type.
- Store external function signature in parsed model.

Acceptance method:

- Parser tests cover `EXTERNAL name(a: int, b: string) -> bool`.
- Parser rejects missing arg or return types.
- `cargo test -p ink-compiler syntax::declaration`
- `make gate`

Forbidden boundaries:

- Do not change runtime external binding behavior yet.
- Do not allow untyped external declarations as final behavior.

Checklist:

- [ ] External signature parser updated
- [ ] Parsed external stores typed signature
- [ ] Positive external tests added
- [ ] Negative external tests added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Struct symbol index added
- [ ] Duplicate struct diagnostic tested
- [ ] Duplicate field diagnostic tested
- [ ] Unknown field type diagnostic tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Variable type scope index added
- [ ] Global type visibility tested
- [ ] Temp type visibility tested
- [ ] Function arg type visibility tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Primitive expression type inference added
- [ ] No implicit conversion tests added
- [ ] String concatenation test added
- [ ] Boolean operator tests added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Initializer type checks added
- [ ] Omitted default handling tested
- [ ] Invalid initializer diagnostics tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Simple assignment type checks added
- [ ] Compound assignment type checks added
- [ ] Invalid assignment diagnostics tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Struct literal type checking added
- [ ] Missing field defaults tested
- [ ] Unknown/duplicate field diagnostics tested
- [ ] Nested struct literal tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Array literal type checking added
- [ ] Empty expected-type test added
- [ ] Mixed element diagnostic tested
- [ ] Nested array test added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Field access type resolution added
- [ ] Unknown field diagnostic tested
- [ ] Non-struct access diagnostic tested
- [ ] Dotted divert regression tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Index access type resolution added
- [ ] Non-int index diagnostic tested
- [ ] Non-array index diagnostic tested
- [ ] Nested array indexing tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Complex lvalue type checking added
- [ ] Field assignment diagnostics tested
- [ ] Index assignment diagnostics tested
- [ ] Nested assignment tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Function call type checking added
- [ ] Wrong arg count diagnostic tested
- [ ] Wrong arg type diagnostic tested
- [ ] Composite type call tests added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Return type checking added
- [ ] Bare return tests added
- [ ] Wrong return diagnostics tested
- [ ] Composite return tests added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Condition type checks added
- [ ] Non-bool condition diagnostics tested
- [ ] Existing bool behavior tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] `LEN` type signature added
- [ ] Valid `LEN` tests added
- [ ] Invalid `LEN` diagnostics tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

### Step 35: Type-Check `ARRAY_REMOVE`

Implementation method:

- Add typed builtin signature `ARRAY_REMOVE(T[], int) -> void`.
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

- [ ] `ARRAY_REMOVE` type signature added
- [ ] Mutability/lvalue rule tested
- [ ] Invalid arg diagnostics tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Equality type rules added
- [ ] Inequality type rules added
- [ ] Nested equality tests added
- [ ] Ordered comparison rejection tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] External call type checking added
- [ ] External return type inference added
- [ ] Invalid external call diagnostics tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Global defaults lowered
- [ ] Temp defaults lowered
- [ ] Struct defaults lowered
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Array literal lowering added
- [ ] Nested array lowering tested
- [ ] Array of struct lowering tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Struct literal lowering added
- [ ] Missing field defaults emitted
- [ ] Nested struct lowering tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Runtime field read operation added
- [ ] Compiler lowering added
- [ ] Missing field runtime error tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Runtime index read operation added
- [ ] Compiler lowering added
- [ ] Out-of-bounds runtime error tested
- [ ] Zero-based behavior tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Runtime field write operation added
- [ ] Compiler lowering added
- [ ] Value-copy struct assignment tested
- [ ] Unknown field behavior tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Runtime index write operation added
- [ ] Compiler lowering added
- [ ] Value-copy array assignment tested
- [ ] Out-of-bounds write tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Compound field lowering added
- [ ] Compound index lowering added
- [ ] Single-evaluation behavior tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Runtime `LEN` added
- [ ] Compiler lowering connected
- [ ] Array length tests added
- [ ] Non-array runtime guard tested if reachable
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Runtime `ARRAY_REMOVE` added
- [ ] Compiler lowering connected
- [ ] In-place mutation tested
- [ ] Out-of-bounds runtime error tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Runtime recursive equality added
- [ ] Runtime inequality added
- [ ] Nested equality tests added
- [ ] Primitive regression tests pass
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] String concatenation runtime behavior verified
- [ ] Invalid mixed concat diagnostics tested
- [ ] Numeric addition regression tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Typed external lowering verified
- [ ] External return value tests added
- [ ] Existing external tests pass
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Tail-call detection added
- [ ] Positive detection test added
- [ ] Non-tail negative tests added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Stack-neutral lowering added
- [ ] Argument evaluation order tested
- [ ] Deep tail recursion test added
- [ ] Non-tail recursion regression tested
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Stack-depth or stress test added
- [ ] Test demonstrates stack-neutral behavior
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Primitive global fixtures migrated
- [ ] Parse snapshots updated
- [ ] JSON fixtures updated
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Primitive temp fixtures migrated
- [ ] Parse snapshots updated
- [ ] JSON fixtures updated
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

### Step 56: Migrate Function Signature Fixtures

Implementation method:

- Add explicit parameter and return types to function fixtures.
- Use `-> void` for functions without return value.

Acceptance method:

- Function conformance and csharp compatibility tests pass after migration.
- `cargo test -p ink-test --test compiler_conformance`
- `make gate`

Forbidden boundaries:

- Do not infer function return types in fixtures.
- Do not preserve old signatures in final fixture source.

Checklist:

- [ ] Function fixtures migrated
- [ ] External references updated
- [ ] Parse snapshots updated
- [ ] JSON fixtures updated
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] External fixtures migrated
- [ ] Host binding tests updated if needed
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Untyped global support removed
- [ ] Untyped temp support removed
- [ ] Untyped function signature support removed
- [ ] Untyped external signature support removed
- [ ] Missing-type diagnostics tested
- [ ] Fixture search clean
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Transitional parser paths removed
- [ ] Transitional analysis paths removed
- [ ] Transitional lowering paths removed
- [ ] Dead code search clean
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

## Phase 7: Documentation And Final Validation

### Step 60: Update WritingWithInk Updates

Implementation method:

- Add a dated entry to `docs/WritingWithInk-updates.md` describing typed
  variables, structs, arrays, typed functions, typed externals, and TCO.

Acceptance method:

- Documentation describes intentional divergence from upstream Ink.
- `cargo fmt --all --check`
- `make gate`

Forbidden boundaries:

- Do not edit `docs/WritingWithInk-origin.md`.
- Do not document behavior not implemented.

Checklist:

- [ ] Updates doc entry added
- [ ] Origin doc untouched
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

### Step 61: Update WritingWithInk Latest

Implementation method:

- Apply the syntax and semantic changes to `docs/WritingWithInk-latest.md`.
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

- [ ] Latest doc updated
- [ ] Examples covered by tests
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] JSON format docs updated
- [ ] Examples match tests
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Primitive typed fixture added
- [ ] Struct typed fixture added
- [ ] Array typed fixture added
- [ ] Nested typed fixture added
- [ ] Typed function fixture added
- [ ] Typed external fixture added
- [ ] TCO fixture added
- [ ] Focused validation passed
- [ ] `make gate` passed
- [ ] Committed immediately
- Commit:

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

- [ ] Stale typed-value TODOs reviewed
- [ ] Untyped fixture search clean
- [ ] Unsupported builtin search clean
- [ ] Full workspace tests passed
- [ ] `make gate` passed
- [ ] Final cleanup committed if needed
- Commit:
