# Dynamic Choice Over Module Array Loses Captured Binding

Status: fixed.

## Scope

- `crates/ink-experiments/experiments/text-jrpg-vertical-slice/story.ink`
- Dynamic choices whose source collection is a module-qualified array constant,
  such as `save::slot_ids`.

## Problem

The compiler accepted the dynamic choice surface syntax but then reported the
generated choice binding as an unknown internal variable when the binding was
used in choice text, choice conditions, or selected-choice body.

Minimal shape:

```ink
=== module save ===
CONST slot_ids: int[] = [1, 2, 3]

== function slot_label(slot_id: int) => string ==
~ return "Slot " + to_str(slot_id)

=== module game ===
FROM save IMPORT slot_ids, slot_label

== main ==
* [slot_id in save::slot_ids] {save::slot_label(slot_id)}
    -> choose_slot(slot_id)

== choose_slot(slot_id: int) ==
Slot {to_str(slot_id)}.
-> DONE
```

Observed diagnostics in the experiment:

```text
Cannot type-check argument 'slot_id' for function 'save::slot_label':
Unknown variable '$choice80_1_v0'

Unresolved variable: $choice80_1_v0
Cannot type-check value for 'slot_id': Unknown variable '$choice80_1_v0'
```

## Fix

Variable-scope construction now indexes declarations in a first pass, then
indexes `for` loop and dynamic-choice generated variables in a second pass. This
lets dynamic choice iterable type inference see globals declared in later
modules while preserving nested loop and nested dynamic-choice capture behavior.

Coverage: `analysis::variables::tests::indexes_dynamic_choice_binding_from_later_module_array_constant`.

Validation:

```text
cargo test -p ink-compiler analysis::variables -- --nocapture
cargo test -p ink-experiments -- --nocapture
```
