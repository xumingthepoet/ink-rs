# Syntax Reference

This is the maintained syntax reference for ink-rs. ink-rs is a new
domain-specific language for narrative games, implemented in Rust and inspired
by ink by inkle. Ink compatibility is not a language goal; this file describes
the current ink-rs language.

For a shorter introduction, start with `LanguageOverview.md`.

## Source Shape

Every runnable story is made from explicit modules. A module starts with
`=== module name ===`. Knots and functions inside modules use `==`; stitches
inside knots use `=`.

Exactly one compiled story must provide a `== main ==` knot. Story content,
choices, gathers, diverts, tags, and logic lines belong inside knots or
stitches, not directly at module level.

```ink
=== module game ===
FROM shop IMPORT price, describe

VAR gold: int = 5

== main ==
{shop::describe()}
Gold: {gold}
Price: {shop::price}
~ shop::price += 2
Updated price: {shop::price}

=== module shop ===
VAR price: int = 3

== function describe() => string ==
~ return "Shop open."
```

A source file starts with a module or interface declaration. Pass every source
file explicitly to the compiler; module names are unique in one compilation.

## Text, Comments, And Tags

Plain lines inside knots and stitches produce output text.

```ink
== main ==
Hello.
World.
```

Use comments for author-only text:

```ink
// line comment
/* block comment */
```

Tags use `#` and are exposed through runtime tag APIs. Tags can appear on normal
content or choices.

```ink
# scene:intro
Welcome.
* Continue # ui:primary
    -> next
```

Text interpolation uses `{expression}`. Use `to_str(value)` when building string
values explicitly.

```ink
VAR hp: int = 20

== main ==
HP: {hp}
~ temp label: string = "HP " + to_str(hp)
{label}
```

## Values And Types

Declarations use explicit types. Supported source value types are:

- `int`, `float`, `bool`, `string`
- divert target values: `->`
- enums
- structs
- arrays: `T[]`
- dictionaries: `Dict<string, V>` and `Dict<int, V>`
- interface module values: `interface<Name>`

Global variables use `VAR`. Constants use `CONST`.

```ink
VAR name: string = "Ada"
VAR score: int = 0
VAR ready: bool = false
CONST MAX_SCORE: int = 10
```

Omitted global initializers use type defaults when available. Dict declarations
without initializers default to an empty Dict with the declared key type.

```ink
VAR count: int
VAR flags: Dict<string, bool>
```

Temporary variables use `~ temp` inside flows and functions.

```ink
~ temp subtotal: int = score + 1
```

Assignment supports simple and compound forms on mutable lvalues:

```ink
~ score = score + 1
~ score += 1
~ player.hp -= 2
~ items[0] = "potion"
```

## Structs, Enums, Arrays, And Dicts

Enums define named state values.

```ink
ENUM QuestState {
    Hidden
    Active
    Complete
}
```

Structs define typed object values. Struct literals use `%Type{...}`.

```ink
STRUCT Player {
    name: string
    hp: int
    state: QuestState
}

VAR hero: Player = %Player{name: "Lio", hp: 20, state: QuestState.Active}
VAR default_hero: Player = %Player{}
```

Arrays use `[]` literals when an expected element type is available.

```ink
VAR numbers: int[] = [1, 2, 3]
~ ARRAY_PUSH(numbers, 4)
~ ARRAY_INSERT(numbers, 1, 9)
~ ARRAY_REMOVE(numbers, 0)
```

Array helpers:

- `LEN(array) => int`
- `ARRAY_PUSH(array, value) => void`
- `ARRAY_INSERT(array, index, value) => void`
- `ARRAY_REMOVE(array, index) => void`

Dict literals use `%{...}`. String-key Dicts use quoted keys; int-key Dicts use
integer keys.

```ink
VAR scores: Dict<string, int> = %{"ada": 10, "bea": 11}
VAR slots: Dict<int, string> = %{1: "ready", 2: "open"}
```

Dict helpers:

- `DICT_HAS(dict, key) => bool`
- `DICT_SIZE(dict) => int`
- `DICT_REMOVE(dict, key) => void`
- `DICT_KEYS(dict) => K[]`

Reads and writes use index syntax. Assigning through a Dict index inserts or
replaces an entry. Reading a missing key is a runtime error, so guard optional
reads with `DICT_HAS`.

```ink
{ if DICT_HAS(scores, "ada"):
    Ada: {scores["ada"]}
}
```

## Expressions

Supported expression features include:

- numeric arithmetic: `+`, `-`, `*`, `/`, `%`, `mod`
- comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
- boolean operators: `not`, `!`, `and`, `&&`, `or`, `||`
- field access: `player.hp`
- index access: `items[0]`, `scores["ada"]`
- function calls
- module-qualified names: `module::symbol`
- dynamic interface access: `{route}::target`, `{route}::score(10)`

`and` / `&&` and `or` / `||` short-circuit inside a single expression. Separate
choice condition blocks are evaluated independently.

## Flow

Diverts move story execution.

```ink
-> next
-> module::target
```

Non-function knots and stitches end naturally when execution reaches the end of
their content. Use an explicit divert when execution should continue at another
target.

Dynamic divert targets use braces and evaluate an expression of type `->`.

```ink
VAR next_target: -> = -> finish

== main ==
-> {next_target}
```

Tunnels call flow and return with `->->`.

```ink
== main ==
-> visit_room ->
Back in main.

== visit_room ==
Room text.
->->
```

Tunnel return can target another destination:

```ink
->-> fallback
```

## Choices And Weave

Choices use `*`. Repeating `*` controls nesting depth.

```ink
== main ==
You reach a door.
* Open it
    The door opens.
    -> done
* Leave
    You step away.
    -> done
- done
End.
```

Choices are repeatable. Hide or show choices with explicit state and conditions.

```ink
VAR visited: bool = false

== main ==
* {not visited}: Visit Paris
    ~ visited = true
    -> main
* {visited}: Return to Paris
    -> main
```

When a choice has leading conditions and then visible text, put `:` after the
condition prefix.

```ink
* {visited}: Return to Paris
* {visited} {not bored}: Return again
```

Without the colon, a leading braced expression is displayed choice text.

Gathers use `-` to join weave branches. Named gathers can be divert targets.

```ink
- regroup
Back together.
```

Choice labels are supported on static choices and gathers.

```ink
* (open_door) Open
    -> after_open
```

## Dynamic Choices

Dynamic choices generate a runtime number of choices from an array expression.
The binding goes immediately after the choice marker.

```ink
STRUCT Option {
    text: string
    enabled: bool
    target: ->
}

CONST options: Option[] = [
    %Option{text: "North", enabled: true, target: -> north},
    %Option{text: "South", enabled: false, target: -> south}
]

== main ==
* [option in options] {option.enabled}: {option.text}
    -> {option.target}
* Wait
```

Use one binding variable for the value, or two variables for `index, value`.

```ink
* [i, option in options] {option.enabled}: {to_str(i)}. {option.text}
    -> choose(i, option.target)
```

The array expression is evaluated once before expansion. Save/load stores the
stable replay point before choice generation and regenerates pending choices
after load instead of serializing generated choices.

Choice generation must be replay-safe. Dynamic iterable expressions, choice
conditions, and displayed choice text cannot call story functions, external
functions, dynamic interface functions, `RANDOM`, `SEED_RANDOM`, mutating
collection builtins, or other statement-level side effects. Move side effects
to ordinary story content before the choice pause or into the selected-choice
body.

Dynamic binding variables are visible in the choice condition, displayed choice
text, selected-choice body, tags, and nested choices. Dynamic choices can mix
with static choices and can use nested choice depth such as `**` or `***`.

Dynamic choices do not support labels. Use item data, explicit state, or gather
labels when a stable identity is needed.

## Conditionals And Switches

Inline conditionals use `{ if ... }` blocks.

```ink
{ if hp > 0:
    Still standing.
- else:
    Down.
}
```

Switch blocks compare a selector against case expressions.

```ink
{ switch state:
- QuestState.Hidden:
    Hidden.
- QuestState.Active:
    Active.
- else:
    Done.
}
```

Branches use `- condition:` and `- else:`. A switch with no exhaustive branch
may still need an explicit divert after the block when later execution should
continue somewhere else.

## Loops

`for` loops iterate arrays and Dicts inside logic-friendly content blocks.

```ink
{ for item in items:
    Item: {item}
}

{ for index, item in items:
    {to_str(index)}: {item}
}

{ for key, value in scores:
    {key}: {value}
}
```

Array loops fix `LEN(array)` once before the loop starts. Dict loops fix
`DICT_KEYS(dict)` once before the loop starts. Choices, gathers, diverts, and
tunnels are not valid inside `for` bodies; use dynamic choices for data-driven
choice lists.

## Modules And Imports

Modules isolate source ownership. A module can use another module only through
explicit imports.

```ink
=== module game ===
FROM inventory IMPORT add_item, item_count
FROM routes

== main ==
~ inventory::add_item("key")
Keys: {inventory::item_count("key")}
-> routes::start
```

`FROM module IMPORT symbol` imports named symbols. `FROM module` records a
module dependency for interface module values and fully qualified access where
allowed by the analyzer.

Cross-module references use `module::symbol`.

## Interfaces

Interfaces declare required knot/function signatures. Modules opt in with
`implements`. Interface-typed values store modules that implement the interface.

```ink
=== interface IRoute ===
== target() ==
== function score(value: int) => int ==

=== module game ===
FROM left

VAR route: interface<IRoute> = left

== main ==
Score: {{route}::score(10)}
-> {{route}::target}

=== module left implements IRoute ===
== target() ==
Arrived.

== function score(value: int) => int ==
~ return value + 1
```

Dynamic interface function calls are expressions. Dynamic interface targets can
be used in dynamic diverts.

## Functions, Externals, And Internals

Functions are declared with explicit argument and return types.

```ink
== function add(a: int, b: int) => int ==
~ return a + b

== function mark_seen() => void ==
~ seen = true
```

Functions can call other functions and return typed values. They cannot offer
choices or arbitrary flow control.

External declarations describe host functions available at runtime.

```ink
EXTERNAL play_sound(name: string) => void
EXTERNAL score_bonus(base: int) => int
```

`INTERNAL` functions are ink-defined functions that the host may evaluate by
name through runtime APIs.

```ink
INTERNAL summary() => string
== function summary() => string ==
~ return "ready"
```

## Runtime State And Save/Load

Language-level state that should survive save/load belongs in globals, arrays,
Dicts, structs, or explicit host state. Runtime `save_state()` stores stable
pause-point state: globals, callstack/temps, random state, and enough resume
mode information to continue from the saved point.

Saves do not serialize pending generated choices. When saved at a choice pause,
the runtime restores the pre-choice replay point and regenerates static and
dynamic choices after load.

## Identifiers

Identifiers may use ASCII letters, digits, underscores, and supported Unicode
identifier characters. They cannot start with a digit.

Use clear module names and explicit imports. Keep story-facing state in typed
variables and data structures so compiler diagnostics can check it early.
