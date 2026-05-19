Progress: 10/23

# Interface-Typed Dynamic Module Access Active Plan

## Status Key

- `[ ]` pending
- `[~]` in progress
- `[>]` implementation committed and waiting review
- `[x]` complete
- `[!]` blocked

## Milestone 1: Syntax And Parsed Model

### [x] Task 01: Parse Interface Declarations

Goal: Add top-level `=== interface Name ===` declarations as parsed model objects without adding type checking or lowering yet.

Implementation method:

- Add parsed interface declaration structures that own the interface name, source span, and member signature list.
- Extend the structure/module-entry parser so interface declarations are top-level peers of `=== module ... ===`, not module content.
- Reject interface declarations nested inside modules, knots, stitches, functions, choices, conditionals, or weave content.
- Add parse snapshot output and parsed visitor traversal for interface declarations.

Acceptance criteria:

- `=== interface IItem ===` parses and appears in parse snapshots.
- Existing explicit module entry requirements remain unchanged for runnable stories.
- Interface declarations do not generate compiled story JSON or runtime content in this task.

Forbidden shortcuts:

- Do not encode interfaces as modules, structs, text nodes, or comments.
- Do not special-case fixture names or silently drop invalid nested interface declarations.
- Do not add dynamic dispatch, type checking, or runtime behavior in this task.

Modification boundaries:

- Allowed: syntax parser, parsed model, parse snapshots, visitor traversal, parser unit tests, compiler snapshot test registration.
- Not allowed: analysis semantics, lowering, format crate, runtime, maintained syntax docs.

Validation commands:

- `cargo test -p ink-compiler syntax::parser`
- `cargo test -p ink-compiler parsed`
- `cargo test -p ink-test --test compiler_snapshots`
- `make gate`

Commit record:

- Implementation commit: `3eef7024 Parse interface declarations`
- Focused validation: `cargo test -p ink-compiler syntax::parser`; `cargo test -p ink-compiler parsed`; `cargo test -p ink-test --test compiler_snapshots`
- Full validation: `make gate`
- Review/fix commits: reviewed; no fix commits required
- Completion record commit: this task-list commit

### [x] Task 02: Parse Interface Member Signatures

Goal: Let interface bodies declare knot and function signatures without executable content.

Implementation method:

- Reuse the existing flow signature parser for `== target(args) ==` and `== function score(args) => Type ==` inside interface declarations.
- Store interface members as signature-only parsed nodes with member kind, name, parameters, return type, typed-signature flag, and source spans.
- Reject story content, tags, choices, gathers, diverts, logic lines, variables, constants, structs, enums, externals, and nested stitches inside interface bodies with explicit diagnostics.
- Preserve existing module-owned flow parsing behavior outside interface bodies.

Acceptance criteria:

- Interface knot and function signatures parse and appear in snapshots.
- Interface members never create runtime containers or executable weave.
- Invalid interface body content produces clear parser diagnostics.

Forbidden shortcuts:

- Do not parse interface members by ad hoc string splitting when existing flow signature parsing can be reused.
- Do not accept executable content inside interfaces as a temporary no-op.
- Do not weaken normal knot/function parser diagnostics.

Modification boundaries:

- Allowed: parser state, parsed interface member model, parse snapshots, parser diagnostics tests.
- Not allowed: module implementation validation, dynamic access expressions, lowering, runtime.

Validation commands:

- `cargo test -p ink-compiler syntax::parser`
- `cargo test -p ink-compiler parsed`
- `cargo test -p ink-test --test diagnostics interface`
- `make gate`

Commit record:

- Implementation commit: `d436460f Parse interface member signatures`
- Focused validation: `cargo test -p ink-compiler syntax::parser`; `cargo test -p ink-compiler parsed`; `cargo test -p ink-test --test diagnostics interface`
- Full validation: `make gate`
- Review/fix commits: reviewed; no fix commits required
- Completion record commit: this task-list commit

### [x] Task 03: Parse Module Implementation Clauses And New Import Syntax

Goal: Support explicit module implementation clauses such as `=== module left implements IItem, IOther ===` and the new `FROM` import syntax.

Implementation method:

- Extend module header parsing to capture a comma-separated interface name list after `implements`.
- Store implemented interface names and spans on parsed `Module`.
- Reject duplicate interface names in the same module header with a parser or early analysis diagnostic.
- Keep `=== module name ===` unchanged for modules that implement no interfaces.
- Replace the old symbol import syntax `IMPORT target FROM left` with `FROM left IMPORT target`, including comma-separated names.
- Extend import parsing and the parsed import model to represent bare module imports separately from symbol imports.
- Parse `FROM left` as a module import that authorizes the `left` module literal and creates a dependency.
- Reject missing module names, invalid module identifiers, missing symbol names after `IMPORT`, and the redundant `IMPORT left FROM left` workaround.
- Add a migration diagnostic for old `IMPORT target FROM left` syntax if the parser can do so without weakening existing errors.

Acceptance criteria:

- Module parse snapshots show implemented interface names.
- Import parse snapshots distinguish `FROM left` from `FROM left IMPORT target`.
- Multiple implemented interfaces are preserved in source order.
- Existing flow, variable, enum, and struct parsing behavior is unchanged.

Forbidden shortcuts:

- Do not introduce a second module declaration form with `== module`.
- Do not treat `implements` as a normal module name suffix.
- Do not infer implementations structurally without an explicit header clause.
- Do not represent module imports as fake symbol imports such as `IMPORT left FROM left`.
- Do not keep the old `IMPORT symbol FROM module` form as supported current syntax.
- Do not let bare module imports authorize static `module::symbol` access; `FROM module IMPORT symbol` owns that rule.

Modification boundaries:

- Allowed: module syntax parsing, import syntax parsing, parsed `Module`/import model, parse snapshots, parser tests, import syntax fixtures.
- Not allowed: checking interface existence or member implementation completeness in this task.

Validation commands:

- `cargo test -p ink-compiler syntax::module`
- `cargo test -p ink-compiler syntax::import`
- `cargo test -p ink-compiler parsed`
- `cargo test -p ink-test --test compiler_snapshots modules`
- `cargo test -p ink-test --test diagnostics imports`
- `make gate`

Commit record:

- Implementation commit: `8ceae72f Parse module implements and FROM imports`
- Focused validation: `cargo test -p ink-compiler syntax::module`; `cargo test -p ink-compiler syntax::import`; `cargo test -p ink-compiler syntax::parser`; `cargo test -p ink-compiler parsed`; `cargo test -p ink-test --test compiler_snapshots modules` (matched 0 tests); `cargo test -p ink-test --test compiler_snapshots`; `cargo test -p ink-test --test diagnostics imports`
- Full validation: `make gate`
- Review/fix commits: `7ff86ab3 Reject redundant module symbol imports`; post-review validation: `cargo test -p ink-compiler syntax::import`; `cargo test -p ink-compiler syntax::parser`; `cargo test -p ink-test --test diagnostics imports`; `make gate`
- Completion record commit: this task-list commit

### [x] Task 04: Parse Interface Type Names

Goal: Add source type syntax `interface<IItem>` and make it available anywhere the later tasks explicitly support interface values.

Implementation method:

- Extend `TypeName` with an interface variant that stores the interface name and source spans.
- Extend type-name parsing for `interface<Name>`, including clear diagnostics for missing `<`, missing `>`, and invalid names.
- Add display and snapshot names for interface types.
- Keep primitive, array, struct, qualified struct, and void parsing unchanged.

Acceptance criteria:

- `interface<IItem>` parses in type positions and displays as `interface<IItem>`.
- Existing named type and `module::Struct` parsing remains unchanged.
- `interface<>`, `interface<123>`, and unterminated interface types report explicit diagnostics.

Forbidden shortcuts:

- Do not encode interface types as struct names or strings.
- Do not make every unknown named type an interface.
- Do not add default values for interface types in this task.

Modification boundaries:

- Allowed: `TypeName`, type-name parser, type display helpers, unit tests.
- Not allowed: variable initializer semantics, dynamic access expressions, runtime value support.

Validation commands:

- `cargo test -p ink-compiler syntax::type_name`
- `cargo test -p ink-compiler parsed::type_name`
- `cargo test -p ink-compiler analysis`
- `make gate`

Commit record:

- Implementation commit: `43891621 Parse interface type names`
- Focused validation: `cargo test -p ink-compiler syntax::type_name`; `cargo test -p ink-compiler parsed::type_name`; `cargo test -p ink-compiler analysis`
- Full validation: `make gate`
- Review/fix commits: reviewed; no fix commits required; post-review validation: `cargo test -p ink-compiler syntax::type_name`; `cargo test -p ink-compiler parsed::type_name`; `cargo test -p ink-compiler analysis`; `make gate`
- Completion record commit: this task-list commit

### [x] Task 05: Parse Dynamic Interface Member Expressions

Goal: Parse `{expr}::member` and `{expr}::member(args)` as expression forms for dynamic interface access.

Implementation method:

- Add parsed expression variants for dynamic interface target/member access and dynamic interface function calls.
- Extend the expression parser so a braced module/interface expression followed by `::` becomes dynamic interface access, while ordinary `{ ... }` struct literals remain unchanged.
- Support the nested source form used by dynamic diverts: `-> {{route}::target}`.
- Preserve existing static `module::symbol`, field access, struct literals, and dynamic divert parsing.

Acceptance criteria:

- `{route}::target` parses as a dynamic target/member expression, not as a struct literal.
- `{route}::score(3)` parses as a dynamic function call with arguments.
- Existing `{ hp: 1 }`, `{route.field}`, `module::symbol`, and `-> {next}` behavior remains unchanged.

Forbidden shortcuts:

- Do not parse the new syntax only in divert source; it must be a real expression form.
- Do not break struct literal parsing or braced content parsing.
- Do not lower dynamic access directly from source text.

Modification boundaries:

- Allowed: expression lexer/parser, parsed expression model, parse snapshots, parser unit tests.
- Not allowed: type checking, lowering, format tokens, runtime execution.

Validation commands:

- `cargo test -p ink-compiler syntax::expression`
- `cargo test -p ink-compiler parsed`
- `cargo test -p ink-test --test compiler_snapshots`
- `make gate`

Commit record:

- Implementation commit: `01af07c0 Parse dynamic interface expressions`
- Focused validation: `cargo fmt --all --check`; `cargo test -p ink-compiler syntax::expression`; `cargo test -p ink-compiler syntax::divert`; `cargo test -p ink-compiler parsed`; `cargo test -p ink-compiler analysis`; `cargo test -p ink-test --test compiler_snapshots`
- Full validation: `make gate`
- Review/fix commits: `093052f9 Avoid duplicate dynamic interface diagnostics`; post-review validation: `cargo fmt --all --check`; `cargo test -p ink-compiler syntax::expression`; `cargo test -p ink-compiler syntax::divert`; `cargo test -p ink-compiler parsed`; `cargo test -p ink-compiler analysis`; `cargo test -p ink-test --test compiler_snapshots`; `make gate`
- Completion record commit: this task-list commit

## Milestone 2: Interface Analysis And Type Semantics

### [x] Task 06: Index Interface Declarations And Names

Goal: Make interface declarations participate in compiler analysis with explicit namespace and duplicate diagnostics.

Implementation method:

- Build an interface declaration index keyed by interface name.
- Add diagnostics for duplicate interfaces and collisions with modules, structs, enums, globals, constants, flows, functions, and externals where the existing namespace rules require uniqueness.
- Make `interface<IItem>` type references resolve through the interface index.
- Extend module import analysis so `FROM left` validates that `left` exists, warns when unused, and is tracked separately from symbol import allow-lists.
- Extend symbol import analysis so `FROM left IMPORT target` replaces old `IMPORT target FROM left` for static `left::target` authorization.
- Extend dependency/reachability analysis so bare module imports and symbol imports create dependency edges and imported implementation modules are included in compiled story JSON when reachable from the entry module or host-callable root.
- Keep static `module::symbol` behavior unchanged apart from the new import syntax; bare module imports authorize module literals only.

Acceptance criteria:

- Unknown interface types produce explicit diagnostics.
- Duplicate interface declarations and conflicting names are reported deterministically.
- Unknown and unused bare module imports produce diagnostics consistent with existing import diagnostics.
- Bare module imports affect dependency and reachability analysis without authorizing static symbol access.
- Static symbol access requires `FROM module IMPORT symbol`; old `IMPORT symbol FROM module` is rejected or diagnosed as obsolete.
- Existing struct/enum/module name diagnostics continue to pass.

Forbidden shortcuts:

- Do not treat interfaces as structs or enums in the type index.
- Do not weaken existing module symbol collision checks.
- Do not require importing interfaces in this task unless the implementation explicitly adds module-scoped interface declarations, which this plan does not.
- Do not model `FROM left` as `IMPORT left FROM left`.
- Do not let bare module imports re-export symbols or satisfy `FROM module IMPORT symbol` rules.
- Do not keep old `IMPORT symbol FROM module` syntax as current supported syntax.

Modification boundaries:

- Allowed: analysis indexes, naming diagnostics, type-reference diagnostics, module import/dependency/reachability analysis, focused analysis tests and import diagnostics fixtures.
- Not allowed: lowering, runtime, JSON format, dynamic dispatch execution.

Validation commands:

- `cargo test -p ink-compiler analysis::names`
- `cargo test -p ink-compiler analysis`
- `cargo test -p ink-test --test compiler_snapshots modules`
- `cargo test -p ink-test --test diagnostics interface`
- `cargo test -p ink-test --test diagnostics imports`
- `make gate`

Commit record:

- Implementation commit: `ec81ead8 Index interface declarations and module imports`
- Focused validation: `cargo fmt --all --check`; `cargo test -p ink-compiler analysis::interfaces`; `cargo test -p ink-compiler analysis::modules`; `cargo test -p ink-compiler analysis::names`; `cargo test -p ink-compiler analysis`; `cargo test -p ink-test --test compiler_snapshots modules`; `cargo test -p ink-test --test diagnostics interface`; `cargo test -p ink-test --test diagnostics imports`
- Full validation: `make gate`
- Review/fix commits: `f214cb49 Cover bare module import static access`; post-review validation: `cargo fmt --all --check`; `cargo test -p ink-compiler analysis::interfaces`; `cargo test -p ink-compiler analysis::modules`; `cargo test -p ink-compiler analysis::names`; `cargo test -p ink-compiler analysis`; `cargo test -p ink-test --test compiler_snapshots modules`; `cargo test -p ink-test --test diagnostics interface`; `cargo test -p ink-test --test diagnostics imports`; `make gate`
- Completion record commit: this task-list commit

### [x] Task 07: Validate Module Interface Implementations

Goal: Check that every `implements` clause references existing interfaces and that modules implement all required members.

Implementation method:

- Add an analysis pass that compares each module's explicit implementations with interface member signatures.
- Require matching member kind, name, parameter count, parameter types, return type, and function-vs-knot shape.
- Permit module members to contain normal executable content; only signatures must match the interface.
- Reject `EXTERNAL` declarations as implementations of interface function members in the first version.
- Report missing members, wrong member kind, and signature mismatches at useful source spans.

Acceptance criteria:

- `=== module left implements IItem ===` fails if `left` omits any `IItem` member.
- Signature mismatches are reported with expected and actual types.
- Modules may implement multiple interfaces, and overlapping identical member requirements are accepted.
- `EXTERNAL score(...)` does not satisfy `== function score(...) => T ==` in an interface and produces a clear diagnostic.

Forbidden shortcuts:

- Do not infer implementation from structural matching without `implements`.
- Do not ignore function return types or knot parameter types.
- Do not allow dynamic EXTERNAL dispatch or EXTERNAL-backed interface members in this version.
- Do not require interface members to appear in any particular source order inside the module.

Modification boundaries:

- Allowed: interface implementation analysis, target/flow signature indexes, diagnostics tests.
- Not allowed: dynamic expression lowering, runtime behavior, JSON format.

Validation commands:

- `cargo test -p ink-compiler analysis`
- `cargo test -p ink-test --test diagnostics interface`
- `make gate`

Commit record:

- Implementation commit: `20be2aa5 Validate module interface implementations`
- Focused validation: `cargo fmt --all --check`; `cargo test -p ink-compiler analysis::interfaces`; `cargo test -p ink-compiler analysis`; `cargo test -p ink-test --test diagnostics interface`
- Full validation: `make gate`
- Review/fix commits: `56501e11 Cover overlapping interface members`; post-review validation: `cargo fmt --all --check`; `cargo test -p ink-compiler analysis::interfaces`; `cargo test -p ink-compiler analysis`; `cargo test -p ink-test --test diagnostics interface`; `make gate`
- Completion record commit: this task-list commit

### [x] Task 08: Type-Check Interface Values And Assignments

Goal: Allow variables, temps, constants, arrays, and struct fields with `interface<IItem>` types while rejecting incompatible values.

Implementation method:

- Treat module literals in `interface<IItem>` expected-type contexts as interface values when the module explicitly implements `IItem`.
- Require cross-module module literals to be authorized by `FROM name`; allow the current module to refer to itself without an import.
- Extend initializer and assignment checks so `interface<IItem>` values can be copied, assigned, stored in arrays, and stored in struct fields.
- Reject assigning strings, structs, divert targets, unknown modules, unimported cross-module literals, or modules that do not implement the required interface.
- Do not add a default initializer for interface types; require explicit values unless an enclosing array defaults to `[]`.

Acceptance criteria:

- `VAR route: interface<IItem> = left` is valid when `left implements IItem`.
- Cross-module `left` requires `FROM left`; without it, the compiler reports the missing module import.
- `interface<IItem>[]` and struct fields of type `interface<IItem>` are type-checked.
- `VAR route: interface<IItem>` without initializer reports no default exists.

Forbidden shortcuts:

- Do not make interface values interchangeable with plain strings at compile time.
- Do not accept structurally matching modules that lack `implements`.
- Do not bypass array/struct literal type checking for interface elements.
- Do not use `IMPORT left FROM left` as a substitute for module literal import authorization.

Modification boundaries:

- Allowed: expression type inference, initializer checks, assignment checks, array/struct literal checks, tests.
- Not allowed: new public runtime `ValueType` variants, dynamic member lowering, JSON instruction changes.

Validation commands:

- `cargo test -p ink-compiler analysis::initializers`
- `cargo test -p ink-compiler analysis::assignments`
- `cargo test -p ink-compiler analysis::array_literals`
- `cargo test -p ink-compiler analysis::struct_literals`
- `make gate`

Commit record:

- Implementation commit: `e80d56ce Type check interface module values`
- Focused validation: `cargo fmt --all --check`; `cargo test -p ink-compiler analysis::initializers`; `cargo test -p ink-compiler analysis::assignments`; `cargo test -p ink-compiler analysis::array_literals`; `cargo test -p ink-compiler analysis::struct_literals`; `cargo test -p ink-compiler analysis`; `cargo test -p ink-test --test diagnostics interface`; `cargo test -p ink-test --test diagnostics imports`
- Full validation: `make gate`
- Review/fix commits: `0c77459e Cover interface array default initializer`; post-review validation: `cargo fmt --all --check`; `cargo test -p ink-compiler analysis::initializers`; `cargo test -p ink-compiler analysis::assignments`; `cargo test -p ink-compiler analysis::array_literals`; `cargo test -p ink-compiler analysis::struct_literals`; `cargo test -p ink-compiler analysis`; `cargo test -p ink-test --test diagnostics interface`; `cargo test -p ink-test --test diagnostics imports`; `make gate`
- Completion record commit: this task-list commit

### [x] Task 09: Type-Check Dynamic Interface Knot Targets

Goal: Make `{route}::target` type-check as `->` when `route` has an interface type declaring the target knot.

Implementation method:

- Extend expression inference so dynamic interface target expressions require an interface-typed base and a declared knot member.
- Validate dynamic divert syntax `-> {{route}::target}` and `-> {{route}::target}(arg1, arg2)` through the existing dynamic divert and divert-argument checks.
- Check dynamic target arguments against the interface knot signature when the syntax supplies divert arguments.
- Reject dynamic target access through non-interface values and unknown interface members.

Acceptance criteria:

- `-> {{route}::target}` compiles when `route: interface<IItem>` and `IItem` declares `target`.
- `-> {{route}::target}(arg1, arg2)` compiles only when the interface knot signature accepts those arguments.
- Missing member and non-interface base diagnostics are clear.
- Existing `-> {next}` dynamic divert behavior remains unchanged.

Forbidden shortcuts:

- Do not allow arbitrary string variables as dynamic module selectors.
- Do not skip argument type checking for dynamic knot calls.
- Do not special-case only variable roots; field and index expressions of interface type must work.

Modification boundaries:

- Allowed: expression type inference, target/call diagnostics, interface member lookup tests.
- Not allowed: runtime execution and JSON serialization, except tests may pin that lowering is still absent until later tasks.

Validation commands:

- `cargo test -p ink-compiler analysis::expression_types`
- `cargo test -p ink-compiler analysis::targets`
- `cargo test -p ink-test --test diagnostics interface`
- `make gate`

- Implementation commit: `38cda122 Type check dynamic interface targets`
- Focused validation: `cargo fmt --all --check`; `cargo test -p ink-compiler analysis::expression_types`; `cargo test -p ink-compiler analysis::targets`; `cargo test -p ink-compiler analysis::modules`; `cargo test -p ink-test --test diagnostics interface`
- Full validation: `make gate`
- Review/fix commits: `6e143761 Preserve dynamic divert argument checks`; post-review validation: `cargo fmt --all --check`; `cargo test -p ink-compiler analysis::expression_types`; `cargo test -p ink-compiler analysis::targets`; `cargo test -p ink-compiler analysis::modules`; `cargo test -p ink-test --test diagnostics interface`; `make gate`
- Completion record commit: this task-list commit

### [x] Task 10: Type-Check Dynamic Interface Function Calls

Goal: Make `{route}::score(args)` type-check against interface function signatures.

Implementation method:

- Extend expression inference and call target diagnostics for dynamic interface function calls.
- Require the base expression to have `interface<IItem>` type and the member to be a function in `IItem`.
- Check argument count and argument types against the interface function signature.
- Infer the dynamic call return type from the interface function return type.

Acceptance criteria:

- Dynamic interface function calls compile with correct argument and return types.
- Calling a knot as a function, using a function as a target, wrong arguments, and non-interface bases report diagnostics.
- Calling through an interface member implemented by `EXTERNAL` is rejected because EXTERNAL implementations do not satisfy interface functions in this version.
- Existing static and qualified function call behavior remains unchanged.

Forbidden shortcuts:

- Do not lower dynamic calls as static calls to the first implementing module.
- Do not allow dynamic EXTERNAL calls in this task or treat EXTERNAL declarations as valid interface function implementations.
- Do not infer return type from an arbitrary implementation module instead of the interface signature.

Modification boundaries:

- Allowed: expression type inference, target/call diagnostics, flow checks, focused unit tests.
- Not allowed: runtime execution, JSON format, docs.

Validation commands:

- `cargo test -p ink-compiler analysis::expression_types`
- `cargo test -p ink-compiler analysis::targets`
- `cargo test -p ink-compiler analysis::flow`
- `make gate`

Commit record:

- Implementation commit: `522d5fa3 Type check dynamic interface function calls`
- Focused validation: `cargo fmt --all --check`; `cargo test -p ink-compiler analysis::expression_types`; `cargo test -p ink-compiler analysis::targets`; `cargo test -p ink-compiler analysis::flow`; `cargo test -p ink-compiler analysis::modules`; `cargo test -p ink-test --test diagnostics interface`
- Full validation: `make gate`
- Review/fix commits: reviewed; no fix commits required; post-review validation: `cargo fmt --all --check`; `cargo test -p ink-compiler analysis::expression_types`; `cargo test -p ink-compiler analysis::targets`; `cargo test -p ink-compiler analysis::flow`; `cargo test -p ink-compiler analysis::modules`; `cargo test -p ink-test --test diagnostics interface`; `make gate`
- Completion record commit: this task-list commit

## Milestone 3: Compiled Story JSON Format

### [ ] Task 11: Add Format Objects For Dynamic Interface Access

Goal: Add typed compiled-story JSON model support for dynamic interface targets, dynamic interface function calls, and runtime interface validation metadata in `ink-story-json-format`.

Implementation method:

- Add format crate object variants for dynamic interface target construction and dynamic interface function calls.
- Include interface name, member name, and argument count where needed.
- Add a format crate metadata model for interface implementations: interface name, implementing module names, and member names/kinds.
- Define stable token names in the format crate and document their ownership there.
- Add JSON serialization and deserialization tests in the format crate.

Acceptance criteria:

- The format crate can round-trip the new dynamic interface objects and interface metadata.
- Compiler and runtime crates depend on the typed format objects, not local duplicate schemas.
- Existing compiled JSON fixtures remain compatible.

Forbidden shortcuts:

- Do not handwrite JSON object fragments in compiler or runtime code.
- Do not add duplicate token constants outside `ink-story-json-format`.
- Do not change runtime save-state JSON ownership.

Modification boundaries:

- Allowed: `crates/ink-story-json-format`, format crate tests, minimal downstream compile fixes.
- Not allowed: compiler lowering behavior, runtime execution semantics, save-state schema changes.

Validation commands:

- `cargo test -p ink-story-json-format`
- `cargo check --workspace`
- `make gate`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending

### [ ] Task 12: Lower Interface Values To Runtime Strings

Goal: Lower interface-typed values to the implementing module name string while preserving compile-time interface type safety.

Implementation method:

- Extend value lowering so module literals in interface-typed contexts emit runtime strings using the module source name.
- Preserve arrays and structs containing interface values as arrays and objects containing strings.
- Ensure default global values for interface variables are represented as strings for runtime save comparison.
- Keep plain string assignment rejected by type analysis even though runtime representation is string.

Acceptance criteria:

- Compiled JSON initializes `interface<IItem>` globals, array elements, and struct fields as string values.
- Save comparison treats unchanged interface values like unchanged strings.
- Existing string, enum, struct, array, and divert target lowering remains unchanged.

Forbidden shortcuts:

- Do not introduce a runtime `module` or `interface` value variant.
- Do not treat all identifiers in string contexts as module literals.
- Do not bypass the format crate when emitting runtime values.

Modification boundaries:

- Allowed: compiler value lowering, lowering indexes, relevant tests/fixtures.
- Not allowed: dynamic member call instructions, runtime control logic, public runtime API changes.

Validation commands:

- `cargo test -p ink-compiler lower`
- `cargo test -p ink-test --test compiler_snapshots interface`
- `cargo test -p ink-test --test typed_values`
- `make gate`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending

### [ ] Task 13: Lower Dynamic Interface Knot Targets

Goal: Emit typed format objects for dynamic interface knot target construction.

Implementation method:

- Lower `{route}::target` by evaluating the base interface expression, pushing or encoding the target member name, and emitting the format crate dynamic target object.
- Lower `-> {{route}::target}(arg1, arg2)` so arguments are evaluated with the interface knot parameter signature before the dynamic target is used.
- Resolve dynamic target values to runtime dot paths such as `left.target` at runtime, not at compile time.
- Preserve existing static `module::target` and dynamic `-> {target_value}` lowering.
- Include interface name in the emitted instruction so runtime can validate injected strings against the expected interface.

Acceptance criteria:

- Compiler JSON fixtures show the new format token for `-> {{route}::target}` and the argument-bearing form `-> {{route}::target}(arg)`.
- Existing dynamic divert fixtures continue to emit their current JSON.
- The compiler does not emit raw JSON maps for the new instruction.

Forbidden shortcuts:

- Do not lower the target by branching over known implementation modules.
- Do not concatenate path strings in source text during parsing.
- Do not skip interface-name emission when it is needed for runtime validation.

Modification boundaries:

- Allowed: expression/divert lowering, format crate object usage, compiler JSON snapshots.
- Not allowed: runtime implementation of the new token, dynamic function call lowering.

Validation commands:

- `cargo test -p ink-compiler lower`
- `cargo test -p ink-test --test compiler_snapshots interface`
- `cargo test -p ink-story-json-format`
- `make gate`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending

### [ ] Task 14: Lower Dynamic Interface Function Calls

Goal: Emit typed format objects for dynamic interface function calls.

Implementation method:

- Lower dynamic interface call arguments using the interface function parameter signature.
- Emit an instruction that evaluates the base interface expression and calls the member function name dynamically with the argument count.
- Include the interface name for runtime validation.
- Preserve existing function tail-call optimization and static function call lowering where the call target is not dynamic.

Acceptance criteria:

- Compiler JSON fixtures show the new dynamic function call object.
- Argument lowering preserves by-value behavior and expected type handling.
- Existing static function, external, and built-in call fixtures remain unchanged.

Forbidden shortcuts:

- Do not emit a static call to an arbitrary implementation module.
- Do not treat dynamic interface calls as external host calls.
- Do not duplicate format token construction outside the format crate.

Modification boundaries:

- Allowed: expression lowering, call signature lookup, format crate object usage, compiler snapshots.
- Not allowed: runtime execution implementation, dynamic EXTERNAL dispatch.

Validation commands:

- `cargo test -p ink-compiler lower`
- `cargo test -p ink-test --test compiler_snapshots interface`
- `cargo test -p ink-test --test functions`
- `make gate`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending

### [ ] Task 15: Lower Interface Metadata And Document Compiled JSON Format

Goal: Emit interface implementation metadata into compiled story JSON and record the new JSON contract in maintained format notes.

Implementation method:

- Lower interface metadata into the format crate program model for reachable modules: interface name, implementing module names, and member names/kinds.
- Ensure imported implementation modules reached through `FROM module` are included in the metadata only when they are compiled into the story.
- Update `docs/ink_JSON_runtime_format.md` with the new format tokens, metadata shape, stack behavior, runtime validation behavior, and examples.
- State that interface values themselves are runtime strings in compiled JSON and save JSON.
- Keep runtime save-state ownership distinct from compiled-story JSON ownership.

Acceptance criteria:

- Compiler JSON snapshots include interface metadata for stories using interface dynamic access.
- JSON format docs describe dynamic interface target and function call tokens plus interface metadata.
- Docs distinguish compiled story JSON from save-state JSON.
- Existing format refactor terminology remains consistent.

Forbidden shortcuts:

- Do not synthesize runtime validation metadata in the runtime by scanning container names.
- Do not edit `docs/WritingWithInk.md`.
- Do not describe unsupported syntax as available.
- Do not move save-state JSON under `ink-story-json-format` in prose or code.

Modification boundaries:

- Allowed: compiler program lowering, format crate metadata usage, compiler JSON snapshots, `docs/ink_JSON_runtime_format.md`, and closely related docs references if needed.
- Not allowed: runtime execution implementation, source syntax reference updates, save-state schema changes.

Validation commands:

- `rg -n "dynamic interface|interface<|save-state|compiled story JSON" docs/ink_JSON_runtime_format.md`
- `cargo test -p ink-story-json-format`
- `cargo test -p ink-test --test compiler_snapshots interface`
- `make gate`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending

## Milestone 4: Runtime Execution And Save Behavior

### [ ] Task 16: Load Dynamic Interface Format Objects Into Runtime

Goal: Convert the new format crate dynamic interface objects and interface metadata into runtime execution objects.

Implementation method:

- Extend runtime compiled-story JSON loading through `ink-story-json-format` to recognize the new objects and metadata.
- Add runtime object types or control commands for dynamic interface target construction and dynamic interface function dispatch.
- Store interface metadata in a runtime-owned lookup that can validate interface name, module name, member name, and member kind.
- Preserve existing compiled-story loading for old JSON.
- Keep runtime save-state reader/writer support runtime-owned.

Acceptance criteria:

- Runtime can load compiled JSON containing dynamic interface access objects and interface metadata.
- Runtime exposes an internal metadata lookup for dynamic interface target and function execution.
- Old compiled story JSON still loads and runs unchanged.
- Loader code does not duplicate format crate token names or schemas.

Forbidden shortcuts:

- Do not bypass the format crate by inspecting raw `serde_json::Value`.
- Do not add compiler-only fallback JSON that runtime cannot load.
- Do not refactor unrelated runtime object graph code.

Modification boundaries:

- Allowed: runtime JSON reader, runtime object/control-command definitions, interface metadata lookup, focused loader tests.
- Not allowed: compiler lowering changes, save-state schema changes, dynamic execution semantics beyond object loading.

Validation commands:

- `cargo test -p ink-runtime json`
- `cargo test -p ink-story-json-format`
- `cargo test -p ink-test --test compiler_snapshots interface`
- `make gate`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending

### [ ] Task 17: Execute Dynamic Interface Knot Targets

Goal: Make runtime dynamic interface target instructions produce valid divert targets from saved module names.

Implementation method:

- At execution, require the base value to be a string module name and validate it against compiled interface implementation metadata for the expected knot member.
- Build the runtime path `<module>.<member>` and verify it resolves to a container before diverting.
- Preserve dynamic knot argument stack behavior for `-> {{route}::target}(arg1, arg2)`.
- Return `StoryError::InvalidStoryState` for non-string values, unknown modules, modules that do not implement the expected interface, or missing targets.
- Preserve existing `ValueType::DivertTarget` behavior.

Acceptance criteria:

- `-> {{route}::target}` runs to the current module's target knot.
- `-> {{route}::target}(arg)` passes arguments to the selected implementation knot.
- Changing `route` to another implementation module changes the destination.
- Invalid saved or host-injected module names report clear runtime errors.

Forbidden shortcuts:

- Do not trust arbitrary strings without interface validation.
- Do not use fixture-name checks or expected-output checks.
- Do not add a new save-state value type for interface values.

Modification boundaries:

- Allowed: runtime execution/control logic, interface implementation metadata loading, runtime tests/fixtures.
- Not allowed: dynamic function execution, compiler syntax changes, public API redesign.

Validation commands:

- `cargo test -p ink-runtime dynamic_interface`
- `cargo test -p ink-test --test modules interface`
- `make gate`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending

### [ ] Task 18: Execute Dynamic Interface Function Calls

Goal: Make runtime dynamic interface function calls dispatch to the implementation module selected by the interface value.

Implementation method:

- At execution, pop or read the evaluated interface module string and validate it against compiled interface implementation metadata for the expected function member.
- Build the runtime function target `<module>.<function>` and push a function call stack frame using existing function call mechanics.
- Preserve argument stack order and return value behavior.
- Reject dynamic EXTERNAL dispatch; interface function members must resolve to ink function containers.

Acceptance criteria:

- `{ {route}::score(3) }` returns the result from the selected implementation module.
- Assignment switching between two implementation modules changes the function result.
- Invalid module names or missing function targets return clear runtime errors.

Forbidden shortcuts:

- Do not route dynamic interface calls through host external binding names.
- Do not duplicate function call stack handling instead of using existing runtime mechanics.
- Do not statically branch over implementation modules in compiled JSON.
- Do not satisfy interface function calls with EXTERNAL declarations.

Modification boundaries:

- Allowed: runtime control logic, function dispatch helpers, runtime tests/fixtures.
- Not allowed: new source syntax, dynamic EXTERNAL support as required scope, save schema changes.

Validation commands:

- `cargo test -p ink-runtime dynamic_interface`
- `cargo test -p ink-test --test functions interface`
- `make gate`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending

### [ ] Task 19: Validate Save And Load For Interface Values

Goal: Pin save-state behavior for interface values, including arrays and structs.

Implementation method:

- Add runtime tests showing interface globals save as string tokens, unchanged defaults are omitted, and load restores defaults.
- Add tests for `interface<IItem>[]` and structs containing `interface<IItem>` fields.
- Verify dynamic access still works after save/load.
- Document runtime error behavior for tampered save values that reference non-implementing modules.

Acceptance criteria:

- Save JSON uses existing string, array, and object representations for interface values.
- Load validates dynamic access at use time and errors clearly for invalid module strings.
- Existing save/load behavior for strings, arrays, objects, and divert targets remains unchanged.

Forbidden shortcuts:

- Do not add save-state migrations unless the existing save version contract requires it.
- Do not serialize interface values as custom objects in save JSON.
- Do not hide invalid save values by falling back to defaults silently.

Modification boundaries:

- Allowed: runtime save/load tests, dynamic access runtime validation, documentation comments if needed.
- Not allowed: compiled-story JSON format changes beyond the earlier format tasks, broad save schema refactors.

Validation commands:

- `cargo test -p ink-runtime save`
- `cargo test -p ink-test --test runtime_api interface`
- `cargo test -p ink-test --test modules interface`
- `make gate`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending

## Milestone 5: Fixtures, Public Docs, And Closeout

### [ ] Task 20: Add End-To-End Interface Fixtures

Goal: Add integration fixtures that demonstrate the complete interface feature across compiler JSON and runtime execution.

Implementation method:

- Add fixtures covering interface declarations, modules implementing one interface, modules implementing multiple interfaces, dynamic knot targets, and dynamic function calls.
- Use the new import syntax in fixtures: `FROM module` for module literals and `FROM module IMPORT symbol` for static symbol references.
- Include runtime output tests where assignment switches the selected implementation.
- Include compiler JSON snapshots that show the new format crate tokens.
- Keep fixture names behavior-focused and avoid expected-output-only changes.

Acceptance criteria:

- End-to-end fixtures compile and run through `ink-test`.
- Fixtures demonstrate both bare module imports and symbol imports with the new `FROM` syntax.
- JSON snapshots cover both dynamic target and dynamic function call lowering.
- Existing module, import, function, and dynamic divert fixtures remain passing.

Forbidden shortcuts:

- Do not special-case fixture names in production code.
- Do not edit fixture expected output to hide a compiler/runtime defect.
- Do not add ignored tests or skip filters.

Modification boundaries:

- Allowed: `crates/ink-test` fixtures/tests and narrowly required production fixes found while wiring them.
- Not allowed: unrelated parser, language, runtime, or docs changes.

Validation commands:

- `cargo test -p ink-test --test compiler_snapshots interface`
- `cargo test -p ink-test --test modules interface`
- `cargo test -p ink-test --test functions interface`
- `make gate`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending

### [ ] Task 21: Add Interface Diagnostics Fixtures

Goal: Pin common failure modes for interface declarations, implementations, values, new import syntax, and dynamic access.

Implementation method:

- Add diagnostic fixtures for unknown interface, duplicate interface, invalid interface body content, unknown implemented interface, missing implementation member, signature mismatch, EXTERNAL attempting to satisfy an interface function, non-implementing module assignment, missing `FROM module` import for a module literal, obsolete `IMPORT symbol FROM module`, non-interface dynamic access, unknown interface member, and invalid dynamic call arguments.
- Ensure diagnostics mention the relevant interface/member/module names.
- Keep diagnostics stable without relying on source-order accidents.

Acceptance criteria:

- Diagnostics fixtures fail for the intended reason and pass with expected messages.
- Invalid syntax and invalid semantics are separated where useful.
- Import diagnostics clearly distinguish bare module imports from symbol imports.
- Existing diagnostics tests remain passing.

Forbidden shortcuts:

- Do not broaden accepted syntax just to avoid writing a diagnostic.
- Do not hide diagnostics behind generic parse failures when a clearer semantic diagnostic is possible.
- Do not add catch-all errors that obscure more specific interface errors.

Modification boundaries:

- Allowed: diagnostics fixtures/tests, import diagnostics fixtures, and narrowly scoped diagnostic wording/code fixes.
- Not allowed: runtime execution behavior, format schema changes, unrelated syntax features.

Validation commands:

- `cargo test -p ink-test --test diagnostics interface`
- `cargo test -p ink-test --test diagnostics imports`
- `cargo test -p ink-compiler analysis`
- `make gate`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending

### [ ] Task 22: Add Public Syntax Fixtures And Documentation

Goal: Add public-facing syntax fixtures and documentation for the latest supported interface and import syntax.

Implementation method:

- Add documentation-facing compiler/runtime fixtures that mirror the examples in `docs/SyntaxReference.md`.
- Update `docs/SyntaxUpdates.md` with the intentional language change, migration notes, and tests.
- Update `docs/SyntaxReference.md` with current syntax only: `FROM module`, `FROM module IMPORT symbol`, `=== interface IItem ===`, `=== module left implements IItem, IOther ===`, `interface<IItem>`, `-> {{route}::target}`, `-> {{route}::target}(arg)`, and dynamic interface function calls.
- Explain save behavior briefly as string-backed runtime values without describing unsupported `module` type syntax.
- Keep examples module-first and runnable.

Acceptance criteria:

- Maintained docs describe interface-typed dynamic module access and do not describe a supported plain `module` type.
- Maintained docs describe the new `FROM` import syntax and do not describe old `IMPORT symbol FROM module` as supported current syntax.
- Syntax reference excludes legacy alternatives and removed syntax.
- Syntax updates record the feature as an intentional ink-rs language addition.

Forbidden shortcuts:

- Do not edit `docs/WritingWithInk.md`.
- Do not document unimplemented dynamic EXTERNAL dispatch as supported.
- Do not include stale `VAR route: module` examples except as rejected/migration context in `SyntaxUpdates.md` if needed.
- Do not keep old import syntax examples in `SyntaxReference.md`.

Modification boundaries:

- Allowed: documentation-facing fixtures/tests, `docs/SyntaxUpdates.md`, `docs/SyntaxReference.md`, possibly `docs/Architecture.md` if a short architecture note is needed.
- Not allowed: production code changes or unrelated fixture updates in this task.

Validation commands:

- `rg -n "FROM .*IMPORT|interface<|implements|dynamic interface|\\{\\{.*\\}::" docs/SyntaxUpdates.md docs/SyntaxReference.md`
- `cargo test -p ink-test --test modules docs`
- `cargo test -p ink-test --test compiler_snapshots interface`
- `make gate`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending

### [ ] Task 23: Archive Completed Plan

Goal: Move this active plan to `docs/finished_plans/` after all implementation tasks are complete and validated.

Implementation method:

- Confirm Tasks 01-22 are marked `[x]` and `Progress: 22/23` before starting closeout.
- Move `docs/active_plan/interface-dynamic-module-access/` to `docs/finished_plans/interface-dynamic-module-access/`.
- Mark this task `[x]`, update the progress counter to `23/23`, and commit the completion record separately.

Acceptance criteria:

- No active files remain under `docs/active_plan/interface-dynamic-module-access/`.
- The completed task list exists under `docs/finished_plans/interface-dynamic-module-access/`.
- The final task list records validation and commit metadata for every task.

Forbidden shortcuts:

- Do not archive while implementation tasks are pending, in progress, blocked, or waiting review.
- Do not leave duplicate active and finished copies of this plan.
- Do not update `AGENTS.md` for plan-specific details.

Modification boundaries:

- Allowed: this plan directory only.
- Not allowed: production code, tests, or unrelated documentation.

Validation commands:

- `test ! -e docs/active_plan/interface-dynamic-module-access`
- `test -f docs/finished_plans/interface-dynamic-module-access/task_list.md`
- `git status --short`

Commit record:

- Implementation commit: pending
- Focused validation: pending
- Full validation: pending
- Review/fix commits: pending
- Completion record commit: pending
