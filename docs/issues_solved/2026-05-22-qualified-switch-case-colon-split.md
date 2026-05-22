# Qualified Switch Case Label Splits On Module Separator

Status: fixed.

## Scope

- Multiline `switch` branch headers.
- Case labels that use module-qualified constants, such as
  `save::SAVE_POINT_VILLAGE`.

## Problem

The parser treated the first `:` in `save::SAVE_POINT_VILLAGE:` as the branch
header/content separator. The case expression was reduced to `save`, which then
produced unresolved-variable diagnostics.

Minimal shape:

```ink
=== module save ===
CONST SAVE_POINT_VILLAGE: int = 1

=== module game ===
FROM save IMPORT SAVE_POINT_VILLAGE

VAR point: int = save::SAVE_POINT_VILLAGE

== main ==
{ switch point:
- save::SAVE_POINT_VILLAGE:
    village
- else:
    other
}
-> DONE
```

Observed diagnostic in the RPG experiment:

```text
story.ink:109:1: error: Unresolved variable: save
```

## Fix

Conditional branch header splitting now ignores `::` module separators when
looking for the branch label terminator `:`.

Coverage: `syntax::conditional::tests::switch_case_header_keeps_qualified_constant_separator`.

Validation:

```text
cargo test -p ink-compiler syntax::conditional -- --nocapture
cargo test -p ink-experiments -- --nocapture
```
