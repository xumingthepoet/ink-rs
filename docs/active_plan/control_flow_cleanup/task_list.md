Progress: 0/28

# Control Flow Cleanup And Minimal Save State Tasks

Status key:

- `[ ]` not started
- `[~]` in progress
- `[>]` implemented and committed, awaiting review
- `[x]` reviewed and complete
- `[!]` blocked

Task rules:

- Complete tasks in order unless the project owner explicitly reprioritizes.
- Every implementation task needs focused tests, focused validation, and `make
  gate` before it can be marked `[>]`.
- Commit implementation code separately from task-list progress records.
- A task becomes `[x]` only after review is complete and this task list is
  updated in a separate completion commit.

## [ ] Task 01: Remove Source Thread Syntax From The Parser

Goal: make `<-` invalid source syntax with a clear diagnostic that explains
thread syntax has been removed from ink-rs.

Implementation method: inspect parser handling for thread diverts, remove the
accepted parse path, and add an explicit diagnostic branch for the exact removed
surface form. Keep ordinary `<` and `-` expression/operator behavior unchanged
where currently supported.

Acceptance criteria: source using `<- target` fails with a stable diagnostic;
nearby ordinary syntax continues to parse; tests cover both the rejected syntax
and at least one non-thread case that should remain valid.

Forbidden shortcuts: do not add fixture-name checks, snapshot-only hacks, or a
generic parse failure with no removal message.

Modification boundaries: parser, diagnostics, parser tests, and syntax update
notes only.

Validation: run focused parser/diagnostic tests for thread syntax, then `make
gate`.

Commit record: pending.

## [ ] Task 02: Remove Magic `DONE` And `END` Divert Targets

Goal: make `-> DONE` and `-> END` invalid source syntax while allowing ordinary
identifiers that contain `done` or `end`.

Implementation method: remove parser or semantic special-casing for these magic
targets, add diagnostics for exact removed forms, and confirm normal identifier
resolution still handles non-magic names.

Acceptance criteria: `-> DONE` and `-> END` fail with clear diagnostics; names
such as `done_count`, `ending`, and user-defined non-magic targets behave
according to normal name rules.

Forbidden shortcuts: do not reserve broad substrings such as `done` or `end`;
do not treat all uppercase names as magic.

Modification boundaries: parser, semantic analysis, diagnostics, tests.

Validation: run focused divert diagnostic tests, then `make gate`.

Commit record: pending.

## [ ] Task 03: Remove Parsed-Model Thread Flags

Goal: delete parsed-model fields that represent source thread diverts or mark
ordinary choices as thread-bearing source constructs.

Implementation method: trace parsed structures from parser output through
semantic analysis and lowering, then remove or replace thread-specific fields
with current source-language concepts.

Acceptance criteria: parsed model no longer exposes source thread metadata;
compiler tests and snapshots reflect the simplified structures.

Forbidden shortcuts: do not leave unused `dead_code` fields or no-op accessors
to preserve old names.

Modification boundaries: compiler parsed model, semantic analysis, lowering
inputs, tests.

Validation: run compiler unit tests touching parsed model and lowering, then
`make gate`.

Commit record: pending.

## [ ] Task 04: Define Natural End For Knots And Stitches

Goal: make non-function knots and stitches naturally finish when their content
is exhausted, without requiring `DONE` or `END`.

Implementation method: inspect current loose-end, fallthrough, and return
analysis behavior; update semantic analysis and lowering so story flows have
well-defined natural endings distinct from function returns.

Acceptance criteria: fixtures without explicit terminal diverts pass; function
return diagnostics remain strict; tests cover root end, knot end, stitch end,
and function missing-return behavior.

Forbidden shortcuts: do not weaken function return checking to make story-flow
tests pass.

Modification boundaries: semantic analysis, lowering, runtime execution for
natural end, tests.

Validation: run focused flow-ending tests, then `make gate`.

Commit record: pending.

## [ ] Task 05: Stop Emitting Removed Source Terminals

Goal: ensure current source compilation no longer emits runtime objects that
come only from removed source `DONE`, `END`, or thread syntax.

Implementation method: audit lowering and JSON emission paths for terminal and
thread objects, then remove current-source emission paths while preserving
documented compiled-story compatibility where still needed.

Acceptance criteria: snapshots for current source contain no emitted objects
caused by removed syntax; compatibility handling remains explicit and tested if
kept.

Forbidden shortcuts: do not silently break compiled-story JSON compatibility
without a documented format decision.

Modification boundaries: compiler lowering, format emission tests, format docs
if behavior changes.

Validation: run compiler snapshot tests and format tests, then `make gate`.

Commit record: pending.

## [ ] Task 06: Rewrite Source Fixtures And Examples

Goal: remove `<-`, `-> DONE`, and `-> END` from maintained source fixtures and
examples, replacing them with current ink-rs structure.

Implementation method: update fixtures only after implementation behavior
exists, preserving scenario intent with natural endings or terminal knots.

Acceptance criteria: no maintained current-language fixture teaches removed
syntax; changed expected outputs match implemented behavior.

Forbidden shortcuts: do not edit expected outputs without explaining the
language behavior change in test names or surrounding docs.

Modification boundaries: fixtures, expected snapshots, example stories.

Validation: run conformance fixtures and compiler snapshots, then `make gate`.

Commit record: pending.

## [ ] Task 07: Update Current Syntax Documentation

Goal: make current-language docs describe ink-rs as its own narrative DSL and
stop presenting removed old ink syntax as supported.

Implementation method: update `docs/SyntaxReference.md`,
`docs/LanguageOverview.md`, and `docs/SyntaxUpdates.md`; keep historical notes
in the change log, not the latest syntax reference.

Acceptance criteria: docs state that ink-rs is inspired by ink but not
compatible with ink; thread syntax, `DONE`, and `END` are absent from current
syntax sections except as removed syntax in update notes.

Forbidden shortcuts: do not leave conflicting terminology between docs,
diagnostics, and examples.

Modification boundaries: maintained docs and diagnostics wording if needed.

Validation: run doc text searches for removed syntax and `make gate`.

Commit record: pending.

## [ ] Task 08: Pin The Minimal Save-State Contract With Failing Tests

Goal: add tests that define the final save JSON contract: no choice-related or
thread-related fields.

Implementation method: write runtime save/load tests around static choices,
dynamic choices, and aftermath; assert save JSON does not contain generated
choices, choice continuations, choice threads, original thread indexes, thread
indexes, thread stacks, or thread counters.

Acceptance criteria: tests initially fail against existing behavior for the
right reason, then pass once later tasks implement the contract.

Forbidden shortcuts: do not test by string-matching only one field while leaving
other choice/thread fields unguarded.

Modification boundaries: runtime tests and save-state test helpers.

Validation: run the focused new save-state tests and record the expected
failure before implementation continues.

Commit record: pending.

## [ ] Task 09: Define The Choice Replay Save Point

Goal: change save/load semantics so a save at a choice pause restores the story
to a stable pre-choice replay point and regenerates choices after load.

Implementation method: identify the execution state immediately before choice
generation, store only that state, and make load resume from that point without
serialized pending choices.

Acceptance criteria: saving at a choice pause, loading, continuing, and then
selecting a choice produces the same visible output and aftermath behavior as
the original run.

Forbidden shortcuts: do not keep hidden serialized choices under renamed fields.

Modification boundaries: runtime story state, save/load, choice generation
tests.

Validation: run focused save/load choice replay tests, then `make gate`.

Commit record: pending.

## [ ] Task 10: Enforce Deterministic Choice Generation

Goal: prevent load-time choice regeneration from duplicating or changing
side-effectful behavior.

Implementation method: audit expressions evaluated during choice text,
conditions, tags, and dynamic bindings; either reject side effects in those
contexts or add a replay guard that makes repeated generation observationally
pure.

Acceptance criteria: choice generation replay cannot duplicate external calls,
random values, variable mutations, output emission, or visit-count changes;
tests cover at least one previously risky side effect.

Forbidden shortcuts: do not assume purity without tests or diagnostics.

Modification boundaries: runtime evaluation, compiler diagnostics if purity is
checked statically, tests.

Validation: run focused side-effect and save/load replay tests, then `make
gate`.

Commit record: pending.

## [ ] Task 11: Preserve Dynamic Choice Bindings Across Replay

Goal: make dynamic choices regenerate from restored execution state while still
providing selected item value and index to aftermath.

Implementation method: ensure dynamic choice array evaluation, item binding,
index binding, selection, and aftermath execution are derived from restored
state and selected index, not from serialized generated choices.

Acceptance criteria: dynamic choices save, load, regenerate, select, and execute
aftermath with correct value and index; no generated dynamic choice entries are
stored in save JSON.

Forbidden shortcuts: do not serialize dynamic item values as choice data to make
aftermath work.

Modification boundaries: compiler dynamic choice lowering, runtime choice
generation and selection, tests.

Validation: run dynamic choice save/load tests, then `make gate`.

Commit record: pending.

## [ ] Task 12: Remove Choice Thread Snapshot Serialization

Goal: delete serialized choice thread snapshots and replace selection execution
with replay-derived continuation state.

Implementation method: trace fields such as choice thread snapshots and
original thread index through runtime selection, then remove their save-state
serialization and update selection to reconstruct necessary context from the
runtime execution state.

Acceptance criteria: save JSON has no choice thread data; selecting a loaded
choice works for nested containers, gathers, and dynamic choices.

Forbidden shortcuts: do not move the same serialized thread data into a less
obvious field.

Modification boundaries: runtime choice model, save-state writer/reader,
runtime tests.

Validation: run runtime choice and save/load suites, then `make gate`.

Commit record: pending.

## [ ] Task 13: Remove Generated Choice Serialization

Goal: remove pending generated choices from save-state JSON entirely.

Implementation method: delete current choice list fields from save writer and
reader, update save-state versioning, and make load regenerate choices from the
replay point.

Acceptance criteria: save JSON contains no current choice list; loading old
save files with choice fields is handled by a deliberate migration or explicit
version rejection.

Forbidden shortcuts: do not silently ignore incompatible old save fields without
version handling.

Modification boundaries: runtime save-state schema, reader/writer, tests, docs.

Validation: run save-state compatibility tests and `make gate`.

Commit record: pending.

## [ ] Task 14: Document And Version The Save-State Break

Goal: make the new minimal save-state contract explicit and durable.

Implementation method: update save-state docs and schema/version tests to state
that choice and thread information is not serialized and choices are regenerated
after load.

Acceptance criteria: docs and tests agree on the new version behavior; old save
fixtures are migrated or rejected deliberately.

Forbidden shortcuts: do not leave the version unchanged if the wire contract is
incompatible.

Modification boundaries: runtime docs, save-state fixtures, version constants,
tests.

Validation: run save-state docs/schema tests and `make gate`.

Commit record: pending.

## [ ] Task 15: Rename Runtime `Thread` To Continuation Concepts

Goal: remove source-thread terminology from runtime internals where the object
is really a continuation, callstack, or execution snapshot.

Implementation method: rename runtime types, fields, and methods in small
steps, preserving behavior while aligning names with their actual purpose.

Acceptance criteria: ordinary choice execution no longer references a public or
source-level `Thread` concept in runtime code; tests remain behaviorally stable.

Forbidden shortcuts: do not perform a broad rename that changes behavior without
focused tests.

Modification boundaries: runtime internals, runtime docs, tests.

Validation: run runtime unit tests and `make gate`.

Commit record: pending.

## [ ] Task 16: Flatten Runtime Callstack Shape

Goal: remove multi-thread-shaped callstack storage from current runtime
execution when current source no longer supports thread syntax.

Implementation method: replace `Vec<Thread>` or equivalent active-thread stacks
with a single active callstack or continuation, keeping compatibility isolated
where historical compiled JSON still needs it.

Acceptance criteria: runtime save/load and choice execution work with a single
active continuation model; no current-source path pushes or pops a source
thread.

Forbidden shortcuts: do not leave unused thread stacks in save state or public
runtime state.

Modification boundaries: runtime callstack, execution state, save/load, tests.

Validation: run runtime execution and save/load tests, then `make gate`.

Commit record: pending.

## [ ] Task 17: Quarantine Historical Compiled `StartThread` Handling

Goal: keep any needed compiled-story compatibility for historical thread
objects isolated from current source behavior.

Implementation method: move compatibility handling behind clearly named runtime
loading or legacy execution paths, and ensure current compiler output cannot
reach it.

Acceptance criteria: current source never emits historical thread objects;
legacy handling is documented, tested, or removed with an explicit format break.

Forbidden shortcuts: do not mix legacy compatibility branches into normal
current-source choice execution.

Modification boundaries: runtime loader/executor, compiler lowering tests,
format docs.

Validation: run format compatibility and compiler emission tests, then `make
gate`.

Commit record: pending.

## [ ] Task 18: Rename Choice Continuation Fields

Goal: remove remaining thread-named fields from choice execution internals.

Implementation method: after save-state choice data is removed, rename or delete
runtime fields that still describe ordinary choice continuation as thread data.

Acceptance criteria: `rg` for thread terminology in choice execution either
finds only historical compatibility comments/tests or no matches; behavior tests
pass.

Forbidden shortcuts: do not leave aliases whose only purpose is to preserve old
names.

Modification boundaries: runtime choice model, tests, docs.

Validation: run runtime choice tests, thread terminology searches, and `make
gate`.

Commit record: pending.

## [ ] Task 19: Rename Runtime `Flow` Terminology

Goal: separate current source flow concepts from runtime execution-state
implementation details.

Implementation method: audit runtime `Flow` usage and rename structures that
really mean execution state, run state, or container traversal state.

Acceptance criteria: runtime terminology no longer implies old ink flow
compatibility; source-level knot/stitch/function concepts remain clear.

Forbidden shortcuts: do not rename parser or parsed model concepts that still
accurately represent current source syntax without a reason.

Modification boundaries: runtime internals, architecture docs, tests.

Validation: run runtime tests and `make gate`.

Commit record: pending.

## [ ] Task 20: Remove Multi-Flow Save-State Shape

Goal: ensure save-state JSON does not preserve old flow/thread structure after
runtime terminology cleanup.

Implementation method: update save writer/reader and tests so save state stores
only the current execution state needed to continue and regenerate choices.

Acceptance criteria: save JSON has no multi-flow, thread, or choice fields; load
behavior remains correct.

Forbidden shortcuts: do not keep old fields with null or empty placeholder
values.

Modification boundaries: runtime save-state schema, tests, docs.

Validation: run save-state tests and `make gate`.

Commit record: pending.

## [ ] Task 21: Update Public Runtime Errors And Docs

Goal: remove obsolete thread, DONE, END, and old-flow wording from public
runtime errors and docs.

Implementation method: update runtime diagnostics, public API docs, and
architecture notes after implementation terminology changes.

Acceptance criteria: user-facing wording describes current ink-rs behavior and
does not recommend removed syntax.

Forbidden shortcuts: do not hide still-existing behavior behind vague wording.

Modification boundaries: runtime docs, error strings, docs tests if present.

Validation: run runtime tests, text searches for removed wording, and `make
gate`.

Commit record: pending.

## [ ] Task 22: Refresh Compiler Snapshots And Integration Fixtures

Goal: bring compiler and integration snapshots in line with the new control-flow
and save-state behavior.

Implementation method: update snapshots only after the corresponding
implementation is complete, and keep each snapshot change attributable to a
documented semantic change.

Acceptance criteria: snapshots no longer contain removed source syntax or
current-source terminal/thread artifacts.

Forbidden shortcuts: do not bless unrelated snapshot churn.

Modification boundaries: compiler snapshots, integration fixtures,
conformance outputs.

Validation: run compiler snapshot and conformance suites, then `make gate`.

Commit record: pending.

## [ ] Task 23: Update Compiled-Story Format Documentation

Goal: document the relationship between historical compiled-story tokens and
current source behavior.

Implementation method: update `docs/ink_JSON_runtime_format.md` and related
architecture notes to state which tokens are legacy-compatible, which are still
emitted, and which are no longer emitted by current source.

Acceptance criteria: format docs do not imply current source supports removed
syntax, and they clearly distinguish compiled-story compatibility from source
language support.

Forbidden shortcuts: do not delete compatibility notes that still describe
runtime behavior.

Modification boundaries: format docs and related tests if docs mention schema
versions.

Validation: run doc searches and `make gate`.

Commit record: pending.

## [ ] Task 24: Remove Thread Syntax From Diagnostics Fixtures

Goal: make diagnostics fixtures treat thread syntax as removed syntax rather
than a supported-but-erroneous feature.

Implementation method: update diagnostics tests so removed syntax receives
intentional removal diagnostics and no current-language tests depend on it.

Acceptance criteria: diagnostics snapshots clearly communicate removal and do
not suggest a thread-based workaround.

Forbidden shortcuts: do not leave ignored tests for thread syntax.

Modification boundaries: diagnostics fixtures, snapshots, docs.

Validation: run diagnostics tests and `make gate`.

Commit record: pending.

## [ ] Task 25: Verify Fallthrough And Weave Behavior Without Threads

Goal: ensure removing threads and terminal magic does not reintroduce ambiguous
fallthrough or weave behavior.

Implementation method: add integration tests for nested knots, stitches,
gathers, static choices, dynamic choices, and natural endings.

Acceptance criteria: fallthrough paths are deterministic; dynamic and static
choices can coexist; aftermath execution works after load.

Forbidden shortcuts: do not encode behavior only in one narrow fixture.

Modification boundaries: compiler/runtime integration tests and any required
fixes.

Validation: run flow, weave, dynamic choice, and save/load tests, then `make
gate`.

Commit record: pending.

## [ ] Task 26: Audit Save JSON For Forbidden Fields

Goal: add a durable regression guard that save JSON contains no choice-related
or thread-related information.

Implementation method: centralize test assertions over serialized save JSON and
include representative stories for static choices, dynamic choices, nested
containers, and post-load selection.

Acceptance criteria: tests fail if fields such as choices, choice
continuations, choice threads, original thread indexes, thread indexes, threads,
thread stacks, or thread counters reappear.

Forbidden shortcuts: do not rely on one golden string snapshot that can miss
new object fields.

Modification boundaries: runtime save-state test helpers and fixtures.

Validation: run the save JSON audit tests and `make gate`.

Commit record: pending.

## [ ] Task 27: Full Documentation Consistency Pass

Goal: make maintained docs, notes, diagnostics, tests, and code comments use the
same current-language terminology.

Implementation method: search for old ink compatibility phrasing, thread syntax,
`DONE`, `END`, and obsolete flow wording; update only maintained documentation
and comments that are now stale.

Acceptance criteria: `docs/SyntaxReference.md` contains only current syntax;
historical removal notes live in `docs/SyntaxUpdates.md`; architecture docs
describe runtime continuation and minimal save-state accurately.

Forbidden shortcuts: do not edit vendored or generated files just to satisfy a
text search.

Modification boundaries: docs, comments, diagnostics text, tests where wording
is asserted.

Validation: run text searches, docs-related tests if present, and `make gate`.

Commit record: pending.

## [ ] Task 28: Final Gate And Plan Closeout

Goal: close the active plan only after implementation, review, docs, and
validation are complete.

Implementation method: run the full validation required by repository policy,
move this active plan directory to `docs/finished_plans/`, and record the final
status.

Acceptance criteria: all prior tasks are `[x]`; `make gate` passes; the active
plan is moved to finished plans in a final documentation commit.

Forbidden shortcuts: do not close the plan with known failing validation or
unchecked save-state contract gaps.

Modification boundaries: active/finished plan docs and final status notes.

Validation: run `make gate`.

Commit record: pending.
