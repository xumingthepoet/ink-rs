# Control Flow Cleanup And Minimal Save State

## Purpose

This active plan removes the remaining source-language shadows of old ink
control-flow syntax and makes the runtime save model match the current ink-rs
language direction.

ink-rs is inspired by ink, but compatibility with ink syntax is not a project
goal. Source behavior should be defined by this repository, by the project
owner's requested language model, and by current tests and documentation.

## Final Language Direction

The source language should not expose thread syntax.

- `<-` is removed from accepted source syntax.
- `-> DONE` and `-> END` are removed from accepted source syntax.
- `DONE` and `END` are not language keywords or magic divert targets.
- Identifiers that merely contain the text `done` or `end` remain ordinary
  identifiers when they are otherwise valid names.
- A knot or stitch that reaches the end of its content ends naturally.
- The root entry flow reaches story end naturally when no more content is
  available.
- Early termination is expressed with ordinary structure, such as diverting to
  a terminal knot or stitch.
- Function return analysis remains explicit and separate from story-flow
  natural ending.

This means `done`, `end`, `thread`, and `flow` should not be source-language
concepts unless they describe current ink-rs concepts in current docs. Historical
compatibility notes belong in change logs or architecture notes, not in the
latest syntax reference.

## Runtime Direction

Runtime internals may temporarily keep names such as `Thread` while the source
feature is removed, but the final architecture should not model ordinary choices
as source threads. Any remaining continuation or callstack mechanism should be
renamed and represented as an implementation detail.

The target terminology is:

- a single active execution continuation or callstack, not source threads;
- runtime execution state, not a public or source-level flow abstraction;
- pending choices regenerated from restored execution state, not serialized as
  save-state data.

Historical compiled-story JSON tokens may continue to be decoded where needed
for compatibility, but current source compilation should stop emitting removed
syntax forms unless a later task explicitly documents a format-version break.

## Minimal Save-State Target

The final save-state JSON must not store choice-related information or
thread-related information.

Save files should not contain generated choices, choice thread snapshots,
original thread indexes, thread indexes, thread stacks, thread counters, or
choice continuation records. After load, the runtime must restore the execution
state to a stable replay point and regenerate the next visible choice list on the
next continue step.

This requires a stronger runtime contract than simply deleting JSON fields:

- choice generation must be deterministic at save-eligible pause points;
- replaying choice generation after load must not duplicate side effects;
- dynamic choice bindings must be recoverable from restored execution state;
- aftermath execution must still receive the selected dynamic item value and
  index;
- fallthrough and weave behavior must remain defined without serialized choices;
- load must reject or migrate old save-state versions deliberately, never by
  silently ignoring unknown choice/thread fields.

## Architectural Boundaries

Compiler changes should stay focused on parser behavior, parsed model shape,
lowering, diagnostics, snapshots, and language docs. Runtime changes should stay
focused on execution continuation, choice generation, load/save, and public
runtime behavior. The compiled-story JSON format crate remains the typed owner
of compiled story JSON data structures and codecs.

Do not add compatibility shims for removed source syntax unless a task explicitly
requires a diagnostic path. Do not hide behavior changes by editing fixtures
without corresponding code and documentation changes.

## Validation Strategy

Each implementation task must add or update focused tests for its behavior and
then pass the focused command listed in the task. A task may not be marked ready
for review until `make gate` passes, unless the task record documents a concrete
external blocker.

The final closeout must show that:

- source `<-`, `-> DONE`, and `-> END` are rejected with clear diagnostics;
- current source examples and docs no longer teach removed syntax;
- ordinary knots and stitches naturally end without old ink terminal syntax;
- dynamic and static choices can be saved, loaded, regenerated, selected, and
  followed through aftermath without serialized choice data;
- save-state JSON contains no choice-related or thread-related fields;
- runtime internals no longer expose source thread concepts for ordinary choice
  execution.
