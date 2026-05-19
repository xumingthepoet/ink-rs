# Interface-Typed Dynamic Module Access Requirements

## Goal

Add interface-typed dynamic module access to ink-rs. Authors should be able to
store a module implementation in an interface-typed variable, then dynamically
divert to interface knots or call interface functions through that variable.

This feature replaces the earlier plain `module` variable idea. There is no
general `module` type in this plan; dynamic access is only allowed through
`interface<Name>` values.

## Source Syntax

Interface declarations are top-level declarations:

```ink
=== interface IItem ===
== target ==
== function score(amount: int) => int ==
```

Modules explicitly implement one or more interfaces:

```ink
=== module left implements IItem, IOther ===
== target ==
Left.
-> END

== function score(amount: int) => int ==
~ return amount + 1
```

Interface values use `interface<Name>`:

```ink
=== module game ===
FROM left
VAR route: interface<IItem> = left
```

Cross-module module literals require explicit static module imports. `FROM name`
imports the module itself and creates a compile-time dependency:

```ink
FROM left
FROM right
```

Symbol imports use the same leading `FROM` form with an `IMPORT` clause:

```ink
FROM left IMPORT target, score
```

`FROM left` authorizes `left` as a module literal in interface value positions.
`FROM left IMPORT target` authorizes static `left::target` access. The old
`IMPORT target FROM left` form is replaced by `FROM left IMPORT target`. The
redundant form `IMPORT left FROM left` is not used for module literals.

Dynamic interface access uses a braced interface expression before `::`:

```ink
-> {{route}::target}
-> {{route}::target}(arg1, arg2)
{ {route}::score(3) }
```

For `-> {{route}::target}`, the outer braces are the existing dynamic divert
expression syntax. The inner `{route}::target` is the dynamic interface member
expression.

## Semantics

- Interface declarations contain signatures only. They do not contain story
  content, choices, gathers, logic lines, declarations, or nested stitches.
- Interface members may be knots or functions.
- A module implements an interface only when its module header explicitly lists
  the interface in `implements`.
- Structural matching without `implements` is not enough.
- A module implementation must provide every interface member with matching
  member kind, parameter count, parameter types, and return type.
- `EXTERNAL` declarations do not satisfy interface function members in the first
  version. Interface functions must be implemented by ink `== function` flows.
- `interface<IItem>` values may be stored in globals, temps, constants, arrays,
  and struct fields.
- A cross-module module literal is valid only when the current module has
  imported that module with `FROM name`. The current module may refer to itself
  as a module literal without an import.
- Interface values do not have default values. `VAR route: interface<IItem>`
  without an initializer is invalid.
- Empty arrays such as `VAR routes: interface<IItem>[] = []` remain valid
  because array defaults are already explicit empty arrays.
- Dynamic member access is valid only on expressions with interface type.
- `{route}::target` has type `->` when `target` is an interface knot.
- Dynamic interface knot arguments use the existing divert argument form after
  the dynamic target expression: `-> {{route}::target}(arg1, arg2)`.
- `{route}::score(args)` has the return type declared by the interface
  function.
- Static `module::symbol` behavior remains unchanged and separate from dynamic
  `{expr}::symbol` behavior.
- `FROM name` contributes to module dependency/reachability analysis, so
  imported implementation modules are included in compiled story JSON when
  reachable from the entry module or host-callable root.

## JSON And Runtime Representation

Interface values are represented at runtime as strings containing the
implementing module source name.

Compiled story JSON initialization uses the existing string value encoding. For
example, `VAR route: interface<IItem> = left` initializes `route` with the
string value `left`.

Save-state JSON also uses existing string, array, and object representations:

```json
{
  "variablesState": {
    "game::route": "^left",
    "game::routes": ["^left", "^right"],
    "game::config": {
      "route": "^left"
    }
  }
}
```

Unchanged globals may be omitted from save-state JSON under the existing save
rules and restored from default globals on load.

Dynamic access needs new compiled-story JSON instructions owned by
`crates/ink-story-json-format`:

- Dynamic interface target construction: interface name, evaluated interface
  value, and member name produce a runtime divert target such as `left.target`.
- Dynamic interface function call: interface name, evaluated interface value,
  member name, and argument count dispatch to a runtime function target such as
  `left.score`.

Compiled story JSON also needs interface metadata owned by
`crates/ink-story-json-format`. The runtime metadata must be sufficient to
validate dynamic access against tampered save/API values:

- interface name
- implementing module names
- member names and member kind (`knot` or `function`)

Function signatures remain primarily compiler-owned in the first version; the
runtime metadata does not need full parameter and return type information unless
the runtime implementation needs it for a concrete validation path.

The runtime validates injected save/API strings at dynamic access time. If the
value is not a string, names an unknown module, names a module that does not
implement the expected interface, or names a module missing the expected
member, runtime returns `StoryError::InvalidStoryState`.

## Public Documentation Requirements

- `docs/SyntaxUpdates.md` records this as an intentional ink-rs language
  addition, records the import syntax replacement, and explains the absence of a
  plain `module` type.
- `docs/SyntaxReference.md` documents only the latest supported syntax.
- `docs/ink_JSON_runtime_format.md` documents the new compiled-story JSON
  tokens and clearly distinguishes compiled-story JSON from save-state JSON.
- `docs/WritingWithInk.md` is not edited.

## Non-Goals

- No plain `module` type.
- No `VAR route: module = left` syntax.
- No structural interface implementation without explicit `implements`.
- No dynamic EXTERNAL dispatch in the first version.
- No `EXTERNAL` implementation satisfying an interface function in the first
  version.
- No interface default values.
- No custom save-state JSON object for interface values.
- No runtime `ValueType::Interface` or `ValueType::Module` variant.
- No source syntax using `== module ... ==` or `== interface ... ==`.
- No `IMPORT name FROM name` workaround for importing module literals.
- No continued support for old `IMPORT symbol FROM module` syntax after this
  language change, except as a migration diagnostic if needed.
