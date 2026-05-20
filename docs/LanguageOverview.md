# Language Overview

This is the short entry point for current ink-rs syntax. Use
`SyntaxReference.md` for the full reference and `SyntaxUpdates.md` for the
historical change log.

## Source Shape

Every runnable story is built from explicit modules. A module starts with
`=== module name ===`; knots and functions inside modules use `==`; stitches
use `=`. Exactly one compiled story must provide `== main ==`.

Story content belongs inside knots or stitches, not at module level. The first
non-blank line of each source file must be a module or interface declaration.
Modules are unique per compilation; declaring the same module name twice is an
error.

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
-> END

=== module shop ===
VAR price: int = 3

== function describe() => string ==
~ return "Shop open."
```

## Modules And Interfaces

A module can use another module's knots, functions, constants, globals, structs,
or externals only after an explicit `FROM module IMPORT symbol`. Cross-module
references use `module::symbol`.

Interfaces declare signature-only knots and functions. Modules opt in with
`implements`, and interface-typed values use `interface<Name>`.

```ink
=== interface IRoute ===
== target() ==
== function score(value: int) => int ==

=== module game ===
FROM left

VAR route: interface<IRoute> = left

== main ==
~ temp current_score: int = {route}::score(10)
Score: {current_score}
-> {{route}::target}

=== module left implements IRoute ===
== target() ==
Arrived.
-> END

== function score(value: int) => int ==
~ return value + 1
```

## Values

Supported source value types include:

- primitives: `int`, `float`, `bool`, `string`
- divert targets: `->`
- enums
- structs
- arrays: `T[]`
- dictionaries: `Dict<string, V>` and `Dict<int, V>`
- interface module values: `interface<Name>`

Global `VAR`, `CONST`, function parameters and returns, `EXTERNAL`
declarations, `INTERNAL` functions, temp variables, arrays, structs, and Dicts
use explicit types.

Omitted global initializers use type-specific defaults when the type supports a
default. Dict declarations without an initializer default to an empty Dict with
the declared key type.

```ink
VAR name: string = "Ada"
VAR scores: Dict<string, int> = {"ada": 10}
VAR slots: Dict<int, string> = {1: "ready"}
VAR empty_scores: Dict<string, int>

CONST BASE: Dict<string, int> = {"ada": 10}
```

Dict key type is part of the value. String-key Dicts use quoted string literal
keys; int-key Dicts use integer literal keys. Reads and writes use index syntax:

```ink
~ scores["ada"] = scores["ada"] + 1
~ slots[2] = "open"
```

Assigning through a Dict index inserts or replaces an entry. Reading a missing
key is a runtime error.

## Flow And Logic

Knot and stitch content can use text, choices, gathers, diverts, tunnels,
threads, conditionals, expressions, glue, tags, and temp variables. `VAR`
declarations are module-top-level globals; local calculation state uses typed
`temp` declarations inside logic lines.

```ink
== main ==
~ temp visits: int = 0
* [Look around]
  ~ visits += 1
  The room is quiet.
  -> END
```

## Host Integration

`EXTERNAL` declarations describe host-provided functions. `INTERNAL` functions
are implemented in ink and can be called through the runtime API.

```ink
=== module audio ===
EXTERNAL play(name: string) => int

=== module config ===
VAR reads: int = 0

== INTERNAL read_config(key: string) => string ==
~ reads += 1
~ return key
```

Use behavior-grouped fixtures under `crates/ink-test/fixtures/` as executable
examples for the maintained language surface.
