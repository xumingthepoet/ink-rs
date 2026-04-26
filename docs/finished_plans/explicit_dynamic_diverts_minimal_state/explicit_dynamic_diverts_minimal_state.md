# Explicit Dynamic Diverts and Minimal Stateful Runtime

## Summary

- Make dynamic diverts explicit: static jumps use `-> knot.path`; dynamic jumps use `-> {expr}` and `expr` must type-check as `->`.
- Remove implicit stateful Ink features: visit counts, turn counts, `CHOICE_COUNT`, once-only choices, and multi-flow runtime API.
- Keep choice conditions, threads, and deterministic random.
- Redesign save JSON as a breaking v2 format that stores execution position stacks, active temps, globals, RNG state, and generated choices when waiting for input.

## Language Semantics

- Direct divert syntax:
  - `-> knot.stitch`, `-> DONE`, and `-> END` are static only.
  - `-> {next}`, `-> {route.next}`, `-> {targets[0]}`, `-> {pick(flag)}`, and `-> {next}(args...)` are dynamic.
  - Old `-> next` variable, parameter, and const diverts are errors with a migration diagnostic.
- Choice behavior:
  - `*` and `+` are accepted as equivalent repeatable choices; neither hides after selection.
  - Choice conditions remain valid: `* {condition} Text`.
  - Choice-line text is display-only by default and is not printed after selection.
  - Old choice-only brackets `[...]` are removed and should produce a clear diagnostic.
- Removed language and API features:
  - Remove `{knot}` read-count shorthand, `READ_COUNT`, `TURNS`, `TURNS_SINCE`, and `CHOICE_COUNT`.
  - Remove visit-count and turn-count public APIs.
  - Remove multi-flow APIs such as `switch_flow`, `remove_flow`, and current-flow inspection.
- Retained features:
  - Retain threads through `<- target`.
  - Retain `RANDOM` and `SEED_RANDOM` with deterministic save/load behavior.

## Save Format

- Introduce a breaking `inkSaveVersion` bump; old saves are rejected, not migrated.
- New save stores:
  - active execution thread stacks: frame path, index, type, and frame temp variables
  - all global variables
  - RNG state: `storySeed` and `previousRandom`
  - generated choices when the story is paused for input, including text, target path, tags, and the choice thread snapshot needed to continue correctly
- New save does not store:
  - visit counts, turn indices, or turn index
  - output stream, current text, or current tags
  - eval stack or current divert target
  - named flows or current flow map
- `save_state` is supported only at stable public pause points: after output has been consumed, while waiting for choices, or after story end.
- `save_state` should reject expression, output, or internal mid-step state instead of serializing runtime internals.

## Implementation Changes

- Parser and parsed model:
  - Add parsed support for braced dynamic divert targets.
  - Reject old variable divert syntax and choice bracket syntax with actionable diagnostics.
  - Keep `Expression::DivertTarget` for static target values like `VAR next: -> = -> knot`.
- Analysis and lowering:
  - Stop resolving bare dotted expressions as dynamic targets.
  - Lower braced dynamic diverts by evaluating the expression to a temporary divert target and emitting a variable divert.
  - Lower field and index expressions rooted in visible variables or constants as data access, never as read-count lookup.
  - Remove count, turn, and choice-count builtin type rules and lowering.
- Choice and runtime:
  - Ignore once-only choice flags in runtime and stop emitting visit-count metadata for choices.
  - Lower choice-line text as choice display text only.
  - Preserve generated choices in save, so load does not rerun choice generation or thread side effects.
- Runtime state:
  - Remove multi-flow state paths and APIs.
  - Remove visit and turn storage and mutation.
  - Keep thread callstack serialization because threads are retained.
  - Keep RNG fields because random is retained.

## Test Plan

- Parser and analysis:
  - Accept `-> {next}`, `-> {route.next}`, `-> {targets[0]}`, `-> {pick(flag)}`, and `-> {next}(value)`.
  - Reject `-> next` when `next` is a variable, temp, const, or parameter.
  - Reject `READ_COUNT`, `TURNS`, `TURNS_SINCE`, `CHOICE_COUNT`, `{knot}` count expressions, and choice `[...]`.
- Runtime and compiler fixtures:
  - Verify all choices reappear when revisiting the same knot, for both `*` and `+`.
  - Verify choice conditions still filter choices.
  - Verify selected choice text is not printed unless authored in branch content.
  - Verify save/load at a choice point preserves generated choices and thread snapshots without rerunning generation.
  - Verify deterministic `RANDOM` and `SEED_RANDOM` across save/load.
- Regression coverage:
  - Add collision tests for `player.hp` as struct field versus static `player.hp` path.
  - Add dynamic divert tests for globals, params, consts, struct fields, arrays, function returns, tunnels, and divert args.
- Full validation:
  - Run focused parser, language, and runtime tests.
  - Run `cargo fmt --all --check`.
  - Run `cargo test --workspace`.
  - Update or remove obsolete C# compatibility fixtures intentionally.
  - Run `make gate`.

## Assumptions

- This is an intentional language and save-format break from upstream C# Ink.
- Old save JSON compatibility is not required.
- Compiled story JSON remains separate from save JSON; only save/state behavior is redesigned here.
- Documentation updates must be written in English.
