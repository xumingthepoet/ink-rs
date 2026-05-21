# Syntax Updates

This file records intentional language and semantic changes made by ink-rs. It
is a historical changelog, not the fastest way to learn current syntax.

`WritingWithInk.md` is the immutable upstream C# documentation snapshot.
Do not edit it for ink-rs language changes.

`LanguageOverview.md` is the short current-language entry point.
`SyntaxReference.md` is the full current ink-rs syntax reference. It should
teach only the current language syntax and should not include removed syntax,
migration notes, or compatibility explanations. Keep those historical details in
this file.

When syntax or semantics change, update this file first, then apply the same
change to `SyntaxReference.md` and, when relevant, `LanguageOverview.md`.

Each entry should include:

- date
- status: experimental, supported, deprecated, or removed
- upstream behavior
- ink-rs behavior
- documentation effect
- rationale
- migration guidance
- tests

## 2026-05-21: Array And Dict For Control Blocks

- status: supported
- upstream behavior: upstream Ink does not have typed array or `Dict<K, V>`
  source-level `for` control blocks.
- ink-rs behavior: multiline `{ for ... in ...: }` blocks now iterate typed
  arrays and Dicts. Arrays support `for item in array` and
  `for index, item in array`; Dicts support `for key, value in dict`.
  Nested `for` blocks are supported. Loop variables are body-scoped compiler
  temps and may shadow outer source names. Array loops fix `LEN(array)` at loop
  entry and read `array[index]` each iteration. Dict loops fix
  `DICT_KEYS(dict)` at loop entry and read `dict[key]` each iteration. Mutating
  the iterated collection can therefore still produce the normal later runtime
  index or missing-key error.
- documentation effect: `SyntaxReference.md` documents supported headers,
  body contents, nested `for`, unsupported flow-control forms, and fixed
  length/key-list behavior. `LanguageOverview.md` mentions `for` alongside
  current flow and logic syntax.
- rationale: typed arrays and Dicts need a concise source-level iteration form
  for text and logic generation without adding runtime object-model or compiled
  JSON format changes.
- migration guidance: replace hand-written counter loops and recursive helper
  functions with `{ for ... }` when the body only needs text, logic, nested
  `if`/`switch`/`for`, and no choices or diverts. Keep recursive helpers when
  generating choices or performing arbitrary flow control.
- tests: parser and analysis unit tests cover header parsing, nested
  conditionals, nested `for`, iterable typing, variable counts, and unsupported
  body objects. Typed integration fixtures cover runtime array output,
  index/item variables, nested arrays, Dict key order, and normal runtime
  errors after mutating an iterated collection.

## 2026-05-21: Explicit Logical Operators Short-Circuit

- status: supported
- upstream behavior: upstream Ink lowers `and` / `&&` and `or` / `||` as eager
  binary native-function calls, so both operands are evaluated before the
  operator runs.
- ink-rs behavior: explicit logical operators now short-circuit inside a single
  expression. `and` / `&&` skips the right-hand side when the left-hand side is
  `false`; `or` / `||` skips the right-hand side when the left-hand side is
  `true`. Adjacent choice condition blocks such as `{a}{b}` remain independent
  conditions and both blocks are evaluated.
- documentation effect: `SyntaxReference.md` documents short-circuit behavior
  for explicit logical operators and clarifies that multiple choice condition
  blocks are not themselves a short-circuit chain.
- rationale: guard expressions such as `index < LEN(items) && items[index] == x`
  should not evaluate an unsafe read after the guard fails.
- migration guidance: keep side effects that must always run in separate logic
  lines or separate choice condition blocks. Put side effects inside `&&` or
  `||` only when skipping them is intended.
- tests: expression and choice integration fixtures cover skipped out-of-bounds
  reads, word and symbol operators, RHS side effects when evaluation is still
  required, and eager evaluation between adjacent choice condition blocks.

## 2026-05-21: Array Append And Insert Helpers

- status: supported
- upstream behavior: upstream Ink does not have typed dynamic arrays or typed
  array mutation helpers.
- ink-rs behavior: arrays now support `ARRAY_PUSH(array, value) => void` and
  `ARRAY_INSERT(array, index, value) => void`. Both require a mutable lvalue and
  preserve the array element type. `ARRAY_INSERT` inserts before `index`, allows
  `index == LEN(array)`, and rejects indexes outside `0..=LEN(array)`.
- documentation effect: `SyntaxReference.md` documents `ARRAY_PUSH` and
  `ARRAY_INSERT` with the existing array builtins.
- rationale: growable logs, queues, plans, ledgers, and result lists should not
  require preallocated empty struct slots or separate depth counters.
- migration guidance: replace slot-fill patterns such as `items[count] = value`
  followed by `count = count + 1` with `ARRAY_PUSH(items, value)`. Use
  `ARRAY_INSERT` only when inserting before an existing position is required.
- tests: format, runtime native function, compiler analysis, lowering, typed
  integration, and experiment validation cover append, insert at head/middle/end,
  nested lvalue writeback, element type checking, and out-of-bounds runtime
  errors.

## 2026-05-21: Dict Collection Helpers

- status: supported
- upstream behavior: upstream Ink does not have explicit generic `Dict<K, V>`
  values or typed Dict collection helpers.
- ink-rs behavior: `Dict<string, V>` and `Dict<int, V>` now support
  `DICT_HAS(dict, key) => bool`, `DICT_SIZE(dict) => int`,
  `DICT_REMOVE(dict, key) => void`, and `DICT_KEYS(dict) => K[]`.
  `DICT_REMOVE` requires a mutable lvalue and ignores missing keys. `DICT_KEYS`
  returns string keys in lexical order and int keys in ascending order.
- documentation effect: `SyntaxReference.md` documents the new Dict helpers and
  removes the old Dict V1 note that collection helpers were intentionally
  absent.
- rationale: registry and table-shaped story state should not require parallel
  key arrays or active maps just to test, count, remove, or enumerate Dict
  entries.
- migration guidance: use `DICT_SIZE(dict)` instead of trying `LEN(dict)`, use
  `DICT_HAS(dict, key)` before optional reads, and replace active-map deletion
  patterns with `DICT_REMOVE(dict, key)` when the entry should no longer exist.
- tests: analysis, lowering, runtime native function, diagnostics, and typed
  integration fixtures cover valid and invalid helper calls, key typing,
  stable key ordering, missing-key removal, nested lvalue removal, and runtime
  output.

## 2026-05-21: Multiplicative Operators Share Precedence

- status: supported
- upstream behavior: conventional arithmetic treats multiplication, division,
  and remainder operators as one precedence group that associates
  left-to-right.
- ink-rs behavior: `*`, `/`, `mod`, and `%` now share one precedence level and
  associate left-to-right. For example, `8 * 100 / 56` parses as
  `(8 * 100) / 56`, and `14 mod 5 % 3` parses as `(14 mod 5) % 3`.
- documentation effect: `SyntaxReference.md` now states that multiplicative
  arithmetic operators share precedence and associate left-to-right.
- rationale: the previous parser table gave each multiplicative operator a
  different precedence, so mixed chains such as `a * b / c` could silently
  evaluate as `a * (b / c)`.
- migration guidance: remove defensive parentheses that only worked around the
  old parser bug. Keep parentheses when a non-left-associative grouping is
  intended.
- tests: parser snapshots cover mixed multiplicative chains, and the
  arithmetic runtime fixture covers integer percentage and mixed remainder
  formulas whose outputs differ under the old precedence table.

## 2026-05-20: Elixir-Style Struct And Dict Literals

- status: supported
- upstream behavior: upstream Ink does not have typed `STRUCT` or `Dict<K, V>`
  value literals. Earlier ink-rs builds used bare `{ field: value }`,
  `{"key": value}`, `{1: value}`, and `{}` composite expression literals.
- ink-rs behavior: struct literals now use an explicit typed form such as
  `%Player{hp: 10}` or `%module::Player{hp: 10}`. Dict literals keep the public
  `Dict<K, V>` type name but use map-style `%{"key": value}`, `%{1: value}`,
  and `%{}` syntax. Dict keys may be string or int keys, and a single Dict
  literal cannot mix key types. Empty `%{}` is valid only when an expected
  `Dict<K, V>` type is available. Bare composite expression literals are
  removed and report migration diagnostics.
- documentation effect: `SyntaxReference.md` and `LanguageOverview.md` use only
  `%Type{...}` struct literals and `%{...}` Dict literals. The removed bare
  composite spellings remain documented only here and in diagnostics tests.
- rationale: `%Type{...}` removes ambiguity between struct values and braced ink
  expression/control forms. `%{...}` gives Dicts a distinct map-style spelling
  while preserving the existing typed `Dict<K, V>` source type and runtime JSON
  format.
- migration guidance: replace struct literal expressions with `%Type{...}` and
  replace Dict literal expressions with `%{...}`. Use `%Type{}` for an empty
  struct value and `%{}` for an empty Dict in a typed context.
- tests: parser, diagnostics, analysis, lowering, typed fixtures, runtime API,
  experiments, and parse snapshots cover typed and qualified struct literals,
  string-key and int-key Dict literals, empty Dict literals with expected types,
  old syntax diagnostics, mixed Dict key diagnostics, modulo parsing, and
  composite literals in function, static divert, tunnel, tunnel-onwards, and
  dynamic interface argument positions.

## 2026-05-20: Typed Dict Values

- status: supported
- upstream behavior: upstream Ink does not have explicit generic `Dict<K, V>`
  value types with checked key and value types.
- ink-rs behavior: `Dict<string, V>` and `Dict<int, V>` are supported value
  types. Dict literals use string or integer keys, `%{}` is an empty Dict when
  the expected type is known, omitted `VAR` initializers default to an empty
  Dict with the declared key type, and index reads/writes use `dict[key]`.
  Assigning through an index inserts or replaces an entry, missing-key reads are
  runtime errors, and Dict values compare recursively with `==` and `!=`.
  Dict values can appear in globals, temps, constants, function parameters and
  returns, external signatures, struct fields, arrays, and host variable/API
  values.
- documentation effect: `SyntaxReference.md` documents the current
  `Dict<string, V>` and `Dict<int, V>` syntax, literals, defaulting, index
  read/write semantics, equality, function/external use, and V1 non-goals.
- rationale: story code needs typed key/value maps without losing integer key
  identity or weakening composite type checks.
- migration guidance: use `Dict<string, V>` or `Dict<int, V>` instead of
  encoding map-like data as parallel arrays or structs with open-ended fields.
  Use arrays or explicit structs when order or fixed field shape matters.
- tests: parser, analysis, lowering, format, runtime, save/load, runtime API,
  typed fixture, diagnostics, and parse snapshot coverage exercises string and
  integer keys, nested Dicts, defaults, literals, reads, writes, equality,
  functions, constants, externals, and host set/get.

## 2026-05-20: Interface-Typed Dynamic Module Access

- status: supported
- upstream behavior: upstream Ink does not have explicit module interfaces,
  interface-typed module values, or dynamic dispatch through module values.
- ink-rs behavior: `=== interface IItem ===` declares signature-only knot and
  function members. `=== module left implements IItem, IOther ===` explicitly
  declares implementations. `interface<IItem>` is the supported type for module
  values constrained by an interface; plain `module` is not a supported source
  type. Module literals require `FROM module`, while static qualified access
  still requires `FROM module IMPORT symbol`. Dynamic knot access uses
  `-> {{route}::target}` or `-> {{route}::target}(arg)`, and dynamic interface
  function calls use `{route}::score(arg)`.
- documentation effect: `SyntaxReference.md` documents only the current
  interface, implementation, import, interface value, dynamic target, and
  dynamic function syntax. Save behavior is described as string-backed runtime
  values using existing string, array, and object save JSON shapes.
- rationale: explicit interfaces keep dynamic module access type-checked while
  still letting story state choose an implementation at runtime. Bare module
  imports distinguish module literals from static symbol allow-lists.
- migration guidance: use `interface<IItem>` variables and explicit
  `implements` clauses for dynamic module dispatch. Replace any experimental
  module-value import workaround with `FROM module`, and keep
  `FROM module IMPORT symbol` only for static `module::symbol` references.
- tests: interface parser, analysis, lowering, runtime, save/load, diagnostics,
  compiler snapshot, and documentation-facing fixtures cover interface
  declarations, module implementations, interface values in arrays and structs,
  dynamic knot targets, dynamic function calls, save JSON, and import
  diagnostics.

## 2026-05-04: ENUM Declarations And Values

- status: supported
- upstream behavior: upstream Ink uses LIST declarations for symbolic value
  sets and list membership operations.
- ink-rs behavior: `ENUM Name { Member Member }` declares a nominal enum type
  at module top level. Enum members are referenced as `Name.Member` inside the
  same module or as `module::Name.Member` after importing the enum name. Enum
  types can be used in `VAR`, `CONST`, function parameters and returns, struct
  fields, and arrays. Omitted enum initializers default to the first declared
  member. Enum values support equality, inequality, assignment, function
  passing, switch cases, and text output; they do not interoperate with
  strings, ordering, arithmetic, or explicit numeric/string member values.
- documentation effect: `SyntaxReference.md` documents the current enum
  syntax, type positions, defaulting rule, member reference forms, and invalid
  operations. Removed list syntax remains documented only as historical
  migration material.
- rationale: story code needs named state sets without depending on removed
  upstream list behavior or integer/string constants that weaken type checks.
- migration guidance: replace string or integer state constants with an enum
  declaration and use `State.Member` or `module::State.Member` at value sites.
  Keep member order stable when relying on omitted initializer defaults.
- tests: parser and analysis tests cover declarations, imports, duplicate and
  empty enums, type resolution, member lookup, equality, invalid operators, and
  invalid string interop. Runtime and compiler snapshot fixtures cover module
  enum values, defaults, arrays, struct fields, functions, switch with `else`,
  text output, and compiled JSON string lowering.

## 2026-05-03: INTERNAL Host-Callable Functions

- status: supported
- upstream behavior: upstream Ink exposes `EvaluateFunction` as a host API but
  does not distinguish host-callable ink functions in source syntax.
- ink-rs behavior: `== INTERNAL name(args) => type ==` declares an ink function
  that host code may call through `Story::call_internal`. `EXTERNAL` remains
  ink-to-host; `INTERNAL` is host-to-ink. Host names use source-qualified
  module names such as `game::read_config`. Modules containing `INTERNAL`
  functions are compiled as host roots with their import dependencies, even when
  unreachable from `main`.
- documentation effect: `SyntaxReference.md` documents `INTERNAL` declarations
  next to `EXTERNAL`, and `ink_JSON_runtime_format.md` documents the
  `internalFunctions` compiled-story metadata.
- rationale: host games can put configuration and query logic in ink while
  keeping the callable host API explicit and type-checked.
- migration guidance: keep ordinary `function` declarations for ink-only
  helpers. Mark only functions intended for host calls as `INTERNAL`. Leave
  unused non-host modules unmarked so they stay out of compiled JSON.
- tests: parser tests cover `INTERNAL` signatures and missing parentheses,
  format tests cover `internalFunctions`, and runtime API tests cover host
  calls, type checks, metadata, save/load, and rejection of ordinary functions.

## 2026-04-30: Remove Plus Choice Marker

- status: removed
- upstream behavior: upstream Ink uses `+` as a sticky/repeatable choice
  marker, distinct from once-only `*` choices.
- ink-rs behavior: `*` is the only source-language choice marker. Repeating
  `*` still controls choice nesting depth. Lines that start with `+` in a
  statement position report a diagnostic telling authors to use `*`.
- documentation effect: `SyntaxReference.md` describes only `*` choice syntax
  and rewrites examples that previously used `+`.
- rationale: once-only choice behavior was already removed, so keeping a second
  choice marker with identical behavior added syntax surface without carrying
  distinct semantics.
- migration guidance: replace every leading choice marker `+` with `*`, keeping
  the same indentation, condition, text, and divert body.
- tests: compiler parser tests reject `+` choice markers, runtime choice
  fixtures use only `*`, and the removed-sequence diagnostic fixture still
  covers removed inline sequences from a `*` choice line.

## 2026-05-19: FROM Imports And Module Implementation Clauses

- status: supported/replaced
- upstream behavior: upstream Ink uses source includes rather than explicit
  module import allow-lists or interface implementation clauses.
- ink-rs behavior: module imports now use leading `FROM`. `FROM module IMPORT
  name, name` authorizes static qualified references such as `module::name`.
  `FROM module` imports the module itself as a distinct dependency and does not
  authorize static `module::symbol` access. The old `IMPORT name FROM module`
  form is no longer current syntax and reports a migration diagnostic.
  `=== module name implements IName, IOther ===` records explicit interface
  implementation declarations on the parsed module.
- documentation effect: `SyntaxReference.md` shows only `FROM` import syntax
  and module implementation clauses as current syntax.
- rationale: putting the source module first keeps static symbol imports and
  bare module imports in one grammar family, while avoiding the old
  `IMPORT module FROM module` workaround for module values.
- migration guidance: replace `IMPORT target FROM left` with
  `FROM left IMPORT target`. Replace block imports with a comma-separated
  `FROM module IMPORT ...` line.
- tests: parser tests cover bare imports, symbol imports, obsolete import
  diagnostics, duplicate implemented interfaces, and parse snapshots for module
  implementation clauses.

## 2026-04-30: Multiline Import Lists

- status: removed by the 2026-05-19 `FROM` import replacement
- upstream behavior: upstream Ink uses source includes rather than explicit
  module import allow-lists.
- ink-rs behavior: this older ink-rs-only form was replaced by
  `FROM module IMPORT name, name`; block imports are no longer current syntax.
- documentation effect: `SyntaxReference.md` no longer documents the block
  import form as current syntax.
- rationale: large modules can expose enough symbols that one-line import lists
  become hard to read.
- migration guidance: replace block imports with a comma-separated
  `FROM module IMPORT ...` line.
- tests: obsolete import syntax is covered by parser and integration
  diagnostics.

## 2026-04-29: Explicit Multiline If And Switch Blocks

- status: removed
- upstream behavior: upstream Ink and earlier ink-rs accepted multiline control
  blocks whose meaning was inferred from brace shape, such as
  `{ condition: ... }`, `{ - condition: ... }`, and `{ selector: - value: ... }`.
- ink-rs behavior: multiline control blocks now require an explicit control
  keyword. Use `{ if condition: ... }` for simple if/else, `{ if: - condition:
  ... }` for extended if/else-if, and `{ switch selector: - value: ... }` for
  switch comparisons. The old keywordless multiline forms are removed rather
  than kept as compatibility aliases.
- documentation effect: `SyntaxReference.md` rewrites multiline control
  examples to use explicit `if` and `switch`, and keeps inline conditional text,
  choice conditions, and dynamic divert targets as separate braced forms.
- rationale: keywordless multiline control coupled if and switch parsing to the
  same branch shape, making bool switch and else-if forms visually ambiguous.
  Explicit keywords make the grammar stable before further control-syntax work.
- migration guidance: add `if` after the opening brace for boolean multiline
  conditionals, add `if:` for extended else-if blocks, and add `switch` before
  selector expressions for switch blocks.
- tests: parser tests cover explicit if and switch parsing plus keywordless
  multiline rejection. Flow tests cover if branch-shape diagnostics and switch
  case checking. Runtime fixtures use the explicit syntax.

## 2026-04-29: Conditional Switch Type Checking And Exhaustive Flow

- status: supported
- upstream behavior: upstream Ink supports switch-style conditionals such as
  `{ x: - 0: ... }`, where branch values are compared against the selector.
- ink-rs behavior: explicit `{ switch x: - 0: ... }` conditionals are
  supported. The selector does not need to be `bool`; each case value must be
  comparable with it using the same type rules as `==`. Explicit `{ if ... }`
  conditionals still require boolean conditions. A conditional whose branches
  all terminate is treated as flow-ending when it has an `else` branch, or when
  a bool switch covers both `true` and `false`.
- documentation effect: `SyntaxReference.md` clarifies the difference
  between ordinary conditionals and switch conditionals, and documents when a
  conditional block closes flow without an extra `-> DONE`.
- rationale: the parser and lowering already modeled switch conditionals, but
  analysis incorrectly rejected typed non-bool selectors and emitted loose-end
  warnings for exhaustive bool switch diverts.
- migration guidance: use switch syntax for enumerated selector values. Add
  `- else:` or an explicit post-block terminator for non-exhaustive switches.
- tests: compiler flow-analysis tests cover switch selector typing, case type
  mismatch diagnostics, content-before-case diagnostics, bool switch
  exhaustiveness, and non-exhaustive int switch warnings. Conditionals fixtures
  cover runtime switch selection and bool switch divert flow.

## 2026-04-29: Module-First Writing Guide Rewrite

- status: supported
- upstream behavior: upstream Ink examples use root-level knots declared with
  `=== knot ===`, optional top-level story content before knots, and root-level
  function declarations.
- ink-rs behavior: runnable sources are module-first. Source files start with
  `=== module name ===`; knots and functions inside modules use `==`; exactly
  one reachable module defines `== main ==`; and source files are supplied
  explicitly instead of through `INCLUDE`.
- documentation effect: `SyntaxReference.md` rewrites the basics,
  diverts, functions, tunnels, threads, and advanced examples to use
  module-first syntax or clearly act as fragments inside a module. It also
  removes stale sequence/shuffle tutorial examples from the maintained guide
  and clarifies that module names are unique per compilation rather than merged
  across source files.
- rationale: the maintained writing guide should be directly usable by authors
  and library consumers without teaching obsolete root-knot syntax.
- migration guidance: wrap runnable examples in an explicit module, rename
  root `=== knot ===` headers to module-owned `== knot ==`, and move story
  entry flow into `== main ==`.
- tests: documentation-only rewrite validated by stale-syntax searches and the
  full project gate.

## 2026-04-29: Source Sequences Removed

- status: removed
- upstream behavior: upstream Ink supports source-level alternatives inside
  braces, including stopping sequences, cycles, once-only alternatives, and
  shuffles, with inline forms such as `{one|two}`, `{&one|two}`,
  `{!one|two}`, `{~one|two}`, and multiline forms such as `{ cycle: ... }`.
- ink-rs behavior: source sequences, cycles, shuffles, and once-only
  alternatives are no longer part of the source language. The compiler reports
  a removed-feature diagnostic instead of lowering them to visit-count-based
  JSON. Conditional text such as `{condition: true text | false text}` remains
  supported.
- documentation effect: `SyntaxReference.md` removes the alternatives
  tutorial and multiline sequence examples, and documents explicit variables
  plus conditional text as the replacement.
- rationale: sequence lowering depended on implicit visit counts, but the
  current runtime intentionally no longer stores visit-count state in saves.
  Keeping the syntax would compile broken stories or require new save-state
  fields for a feature the project owner chose to delete.
- migration guidance: model progression explicitly with typed variables and
  conditionals. For random variation, use `RANDOM`, `SEED_RANDOM`, host state,
  or explicit story variables rather than source shuffles.
- tests: compiler parser/lowering unit tests, `diagnostics/removed-sequence.ink`,
  and compiler snapshot fixtures cover the removed syntax and retained dynamic
  tag interpolation behavior.

## 2026-04-29: Choice Repeatability Documentation

- status: removed/supported
- upstream behavior: upstream Ink treats `*` choices as once-only by default
  and uses `+` for sticky/repeatable choices.
- ink-rs behavior: both `*` and `+` choices are repeatable. A choice only
  disappears when the author hides it with an explicit condition or changes
  control flow. The compiler and runtime no longer carry once-only choice flags
  for source-authored choices.
- documentation effect: `SyntaxReference.md` rewrites the old
  once-only-choice section to describe repeatable choices, explicit
  variable-based hiding, and fallback choices for the case where all visible
  choices are conditionally hidden.
- rationale: implicit once-only choices depended on removed visit-count state.
  Keeping the old tutorial and unused flags made the current repeatable choice
  model look inconsistent.
- migration guidance: replace reliance on implicit `*` disappearance with a
  typed variable and a choice condition such as `* { not asked } Ask`.
- tests: `star_and_plus_choices_are_repeatable`,
  `choice_conditions_still_control_visibility`, compiler choice parser tests,
  and full compiler/runtime gates.

## 2026-04-28: Choice Condition Boundary And Multiline Declarations

- status: supported
- upstream behavior: leading braced expressions on choice lines are parsed as
  choice conditions, and declaration initializers in this fork were previously
  parsed from one physical line.
- ink-rs behavior: a colon after one or more leading choice conditions ends the
  condition prefix, so `* {enabled}: {label}` means a conditional choice whose
  visible text is the dynamic expression `{label}`. Module-level `VAR` and
  `CONST` declarations may now write array and struct literal initializers
  across multiple lines. Temporary declarations and other `~` logic lines
  remain single-line syntax.
- documentation effect: `SyntaxReference.md` documents the explicit
  choice boundary, shows multiline `VAR`/`CONST` composite declarations, and
  updates the data-driven choice example to use
  `* {option.enabled}: {option.text}` instead of invisible glue.
- rationale: dynamic data-driven choices should not require incidental glue to
  separate conditions from visible text, and long data declarations should be
  readable without changing runtime data formats.
- migration guidance: replace `* {condition}<>{dynamic_text}` or
  `* <>{dynamic_text}` patterns with `* {condition}: {dynamic_text}` when a
  condition is present. Keep `~ temp` initializers on one line, or initialize a
  module-level `VAR`/`CONST` with the multiline value and copy from it.
- tests: `choice_condition_colon_boundary_allows_dynamic_choice_text`,
  `multiline_var_and_const_composite_literals_run_at_runtime`,
  `multiline_temp_initializer_remains_single_line_syntax`, and choice parser
  unit tests for colon boundaries, multiple conditions, and adjacent braces.

## 2026-04-27: Data-Driven Choice Generation With Threads

- status: supported
- upstream behavior: upstream Ink uses authored choice points and threads to
  collect choices from multiple flows. It does not provide a general source
  loop that expands an arbitrary data collection into choice syntax.
- ink-rs behavior: existing ink-rs arrays, structs, `LEN`, dynamic divert
  target values, and thread forking can be composed to generate a runtime number
  of choices from data. A recursive thread can walk an array and offer one
  choice per enabled element. The selected branch can then use the element's
  stored `->` target.
- documentation effect: `SyntaxReference.md` now documents the recursive
  thread pattern for dynamic data-driven choices, including why the recursion
  starts from the last index, why each element should be copied to a local temp
  before offering the choice, and why `<>` is needed when the displayed choice
  text starts with a dynamic expression.
- rationale: this is a useful authoring pattern made possible by already
  supported language features. Documenting it avoids mistaking the lack of a
  source-level `for` loop for a hard limit on runtime choice counts.
- migration guidance: replace fixed preallocated choice slots with a recursive
  thread helper when the number of generated options should follow an array's
  current length. Keep branch behavior in authored knots or stitches and store
  their divert targets in the data.
- tests: documentation-only change; the example pattern was manually compiled
  and run with the existing local `ink_compile` and runtime artifacts, producing
  choices `A`, `B`, and `D` from a four-item array where `C` was disabled.

## 2026-04-27: Explicit Modules And Imports Replace Include

- status: supported/removed
- upstream behavior: upstream Ink permits root-level story content, top-level
  knots in an implicit root namespace, story-global tags before the first knot,
  file concatenation through `INCLUDE`, and cross-file/global lookup without an
  explicit dependency declaration.
- ink-rs behavior: runnable stories require explicit modules. A module starts
  with `=== module name ===`; knots and functions inside modules use `==`;
  stitches still use `=`. Exactly one module must define `== main ==`, which is
  the story entry point. Module-level direct content and module-level tags are
  rejected; content and tags belong inside knots or stitches. Source files are
  passed explicitly to the compiler, and `INCLUDE` is removed. Cross-module
  access requires a `FROM module IMPORT name` declaration and qualified source
  references such as `shop::price`. Module `VAR` and `EXTERNAL` runtime names
  are module-qualified, for example `shop::price` and `audio::play`.
- documentation effect: `SyntaxReference.md` documents explicit modules
  as the current source shape, replaces the include section with module/import
  guidance, and describes module-scoped globals and host bindings.
- rationale: modules make source ownership, dependency checks, namespace
  boundaries, and multi-source compilation explicit instead of depending on
  text concatenation order or global visibility leaks.
- migration guidance: wrap runnable content in a module with `== main ==`,
  move top-level declarations to module top level, move root content and root
  tags into knots, replace `INCLUDE` with explicit compiler source inputs plus
  `IMPORT`, and rewrite cross-module references as `module::symbol`.
- tests: `docs_module_import_example_runs`,
  `module_imported_global_variable_reads_and_writes_run`, and
  `explicit_module_removed_root_behaviors_emit_diagnostics` in
  the behavior-focused files under `crates/ink-test/tests/`, plus module
  parser, analysis, lowering, compiler API, fixture, upstream-divergence
  divergence, and runtime-loading tests.

## 2026-04-28: Compiler Entry Requires Explicit Module Headers

- status: supported/removed
- upstream behavior: upstream Ink accepts root-level story content and top-level
  flows without an explicit module wrapper.
- ink-rs behavior: public compiler entry points reject source files whose first
  non-blank line is not `=== module name ===`. The parser still has internal
  root-weave structures for syntax tests and parsed-model bookkeeping, but
  user-authored compiled stories must enter through explicit modules.
- documentation effect: `SyntaxReference.md` now states that each source
  file must begin with a module declaration.
- rationale: accepting root stories in production kept a legacy path alive after
  maintained fixtures had moved to module syntax, making tests less faithful to
  the current language.
- migration guidance: move old root-level story content into a knot or stitch
  inside an explicit module, and ensure exactly one compiled module defines
  `== main ==`.
- tests: compiler entry tests cover the explicit-module diagnostic; integration
  policy rejects direct source-string construction and JSON-only runtime
  fixtures.

## 2026-04-26: Divert Target Values And Return Type Marker

- status: supported
- upstream behavior: upstream Ink uses divert target values in flow APIs and
  parameter shorthand such as `-> target`. It also uses `->` in typed function
  and external signature examples in this fork's earlier documentation.
- ink-rs behavior: divert targets are first-class typed values written as `->`.
  They may appear in `VAR`, `temp`, `CONST`, function parameters and returns,
  `EXTERNAL` parameters and returns, struct fields, and arrays such as `->[]`.
  A bare `->` declaration has no default initializer; `->[]` defaults to `[]`.
  Function and `EXTERNAL` return declarations now use `=>`, as in
  `== function pick() => -> ==` and `EXTERNAL pick() => ->`. The old typed
  return marker `->` is rejected with a diagnostic. The upstream parameter
  shorthand `-> target` remains accepted and means `target: ->`.
- documentation effect: `SyntaxReference.md` documents `->` as a value
  type, updates function and external signatures to `=>`, and removes language
  that said divert target variables were unsupported.
- rationale: `->` previously had two meanings: a flow operator and a typed
  return marker. Reserving `->` for divert target values and flow syntax makes
  signatures unambiguous while restoring dynamic divert targets as typed data.
- migration guidance: rewrite typed function and external returns from
  `-> Type` to `=> Type`. Declare stored targets as `name: ->`, construct them
  with `-> knot`, and use `=> ->` for functions or externals that return target
  values.
- tests: `typed_divert_target_fixture_runs`,
  `old_function_return_marker_reports_new_marker`, and
  `old_external_return_marker_reports_new_marker` in
  the behavior-focused files under `crates/ink-test/tests/`, plus compiler
  parser, analysis, and lowering unit tests.

## 2026-04-26: Explicit Dynamic Diverts And Minimal Stateful Runtime

- status: removed/supported
- upstream behavior: upstream Ink allows implicit variable divert targets such
  as `-> next`, uses visit counts and turn counts for read-count shorthand,
  `READ_COUNT`, `TURNS`, `TURNS_SINCE`, once-only `*` choices, and multi-flow
  runtime APIs. Choice square brackets split displayed choice text from selected
  output.
- ink-rs behavior: static diverts remain `-> knot.path`. Dynamic diverts must
  be braced, such as `-> {next}`, `-> {route.next}`, `-> {targets[0]}`, or
  `-> {pick(flag)}(arg)`, and the expression must type-check as `->`. Old
  variable-rooted direct diverts produce a migration diagnostic. Choice line
  text is display-only by default, `*` and `+` are both repeatable, and
  choice-only square bracket syntax is removed. `READ_COUNT`, `TURNS`,
  `TURNS_SINCE`, `CHOICE_COUNT`, and `{knot}` read-count shorthand are removed;
  authors should model state explicitly with variables. Runtime multi-flow APIs
  and multi-flow save state are removed; threads remain supported.
- ink-rs save behavior: save JSON is version 2 and stores only the active
  callstack, generated choices, optional choice thread snapshots, global
  variables, `storySeed`, `previousRandom`, `inkSaveVersion`, and
  `inkFormatVersion`. It no longer stores `flows`, `currentFlowName`,
  `evalStack`, `currentDivertTarget`, `visitCounts`, `turnIndices`, or
  `turnIdx`. Version 1 saves are rejected rather than migrated.
- documentation effect: `SyntaxReference.md` describes explicit dynamic
  divert syntax, repeatable display-only choices, removed count/turn features,
  and minimal save-state semantics. Runtime and JSON format docs mark remaining
  count-related runtime tokens as legacy compiled-story compatibility rather
  than current source-language behavior.
- rationale: `.` now names both static flow paths and struct field access.
  Requiring braces for dynamic target expressions removes ambiguous lowering.
  Removing implicit counts and multi-flow state keeps save files small and makes
  authored state explicit.
- migration guidance: rewrite `-> next` to `-> {next}` when `next` is a
  variable, parameter, constant, field, index expression, or target-returning
  function call. Move selected response text into the indented choice body.
  Replace visit/turn/count queries with typed variables maintained by the story
  or host code. Replace multi-flow runtime usage with separate story instances
  or explicit story variables.
- tests: dynamic divert parser, analysis, lowering, choice, save/load, removed
  count diagnostics, upstream-divergence coverage, compiler conformance,
  runtime unit, and language integration tests updated with this change.

## 2026-04-26: CONST Declarations Require Explicit Types

- status: supported
- upstream behavior: upstream Ink constants are dynamically typed and declared
  with `CONST name = value`.
- ink-rs behavior: constants use `CONST name: Type = value`. Supported constant
  types are the same maintained value types as globals and temps: `int`,
  `float`, `bool`, `string`, user `STRUCT` types, arrays written as `T[]`, and
  nested arrays. Constant initializers are checked against the declared type,
  and struct defaults are applied when typed struct constants omit fields.
- documentation effect: `SyntaxReference.md` updates the changed-from-
  upstream section and all constants examples to use explicit types.
- rationale: constants participate in expression type checking and lowering.
  Requiring a declared type keeps composite constants unambiguous, especially
  for empty arrays and partial struct literals.
- migration guidance: rewrite `CONST NAME = value` as `CONST NAME: Type =
  value`.
- tests: `typed_constants_support_struct_and_array_values` and
  `untyped_constant_declaration_reports_missing_type` in
  the behavior-focused files under `crates/ink-test/tests/`, plus compiler
  parser, initializer, struct literal, and array literal unit tests.

## 2026-04-26: Global VAR Declarations Restricted To Module Top Level

- status: removed
- upstream behavior: upstream Ink allows global variables to be introduced with
  `VAR` anywhere in the parsed story, including inside knots, stitches,
  functions, choices, and conditionals.
- ink-rs behavior: `VAR` declarations are only accepted at module top level,
  outside knots, stitches, functions, choices, and conditionals.
  Local executable state should use typed `temp` declarations instead.
- documentation effect: `SyntaxReference.md` records the scope
  restriction in "Changed from upstream Ink" and in the global variable
  declaration section.
- rationale: hidden global declarations inside executable flow content make
  source organization rules ambiguous. Keeping globals in module-level
  declaration areas makes global state explicit before flow execution.
- migration guidance: move nested `VAR` declarations to module top level. If
  the value is only needed inside a knot, stitch, function, choice, or
  conditional, replace it with a typed `temp` declaration.
- tests: `nested_global_var_declarations_report_current_syntax_error` in
  the behavior-focused files under `crates/ink-test/tests/`,
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
  type such as `== function add(a: int, b: int) => int ==`; use `=> void` for
  functions that only perform effects. `EXTERNAL` declarations also require
  typed argument and return signatures. `LEN(array)` returns an `int`, and
  `ARRAY_REMOVE(array, index)` mutates the array and returns `void`.
- ink-rs behavior: direct self tail recursion in the shape
  `return current_function(...)` is lowered to parameter reassignment plus a
  jump back to the function body, avoiding growth of the Ink function callstack
  for that recursive step. Mutual recursion and non-tail recursion keep normal
  call behavior.
- documentation effect: `SyntaxReference.md` must replace the upstream
  dynamic declaration examples with typed declarations, add struct and array
  syntax, document typed functions and externals, and document the array
  builtins and tail-call behavior as ink-rs language extensions.
- rationale: typed source declarations let the compiler reject mismatched
  assignments, bad function or external calls, non-bool conditions, invalid
  field/index access, and unsupported implicit conversions before runtime while
  preserving the existing dynamic story JSON execution model.
- migration guidance: add explicit type annotations to all `VAR` and `temp`
  declarations, all function parameters and returns, and all `EXTERNAL`
  signatures. Use `[]` only where the expected array type is known.
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
  the behavior-focused files under `crates/ink-test/tests/`, plus the related
  compiler analysis, parser, runtime, and JSON format unit tests.

## 2026-04-25: LIST Declarations Removed

- status: removed
- upstream behavior: upstream Ink supports `LIST` declarations for named list
  origins and list items, documented in the upstream "Advanced State Tracking"
  list sections. It also exposes `LIST_*` builtin functions and serializes list
  definitions and list values in compiled story JSON.
- ink-rs behavior: `LIST` declarations produce a removed-feature diagnostic.
  Compiled story JSON no longer serializes `listDefs`, list value objects, or
  list-specific runtime tokens.
- documentation effect: `SyntaxReference.md` removes the upstream list
  documentation from the main table of contents and body, and records the
  divergence in "Changed from upstream Ink".
- rationale: the Rust language surface is being reduced to features that are
  actively maintained and useful for the current project direction.
- migration guidance: use variables, functions, or host-side data for inventory
  and set-like game state until a replacement list design is added.
- tests: `removed_list_declaration_reports_removed_feature_diagnostic` in
  the behavior-focused files under `crates/ink-test/tests/`.
