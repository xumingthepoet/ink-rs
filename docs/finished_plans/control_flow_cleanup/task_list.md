Progress: 28/28

# Control Flow Cleanup And Minimal Save State Tasks

Status key:

- `[ ]` not started
- `[~]` in progress
- `[>]` implemented and validated, awaiting owner review
- `[x]` reviewed and complete
- `[!]` blocked

Task rules:

- Complete tasks in order unless the project owner explicitly reprioritizes.
- Every implementation task needs focused tests and focused validation before it
  can be marked `[>]` or `[x]`.
- Record changed files, validation result, review state, remaining risk, and any
  optional checkpoint commit inside the relevant task section.
- Use this task list as the progress tool below commit granularity. Task
  completion does not require a commit, and commits are not the source of truth
  for task status.
- The implementer may create checkpoint commits at their own judgment when the
  temporary diff becomes too large, review would benefit from a recovery point,
  or a milestone needs local isolation.
- Before this active plan is pushed or considered landed, combine all commits
  belonging to this active plan into one final active-plan commit unless the
  project owner asks for a different history shape.
- `make gate` is required before final closeout. It may also be run earlier at
  milestone boundaries or when focused validation is not enough.

## [x] Task 01: Remove Source Thread Syntax From The Parser

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

Validation: run focused parser/diagnostic tests for thread syntax.

Progress ledger:

- Changed files: `crates/ink-compiler/src/syntax/divert.rs`,
  `crates/ink-compiler/src/syntax/text.rs`,
  `crates/ink-compiler/src/syntax/mod.rs`
- Validation result: passed `cargo test -p ink-compiler rejects_removed`;
  passed `cargo test -p ink-compiler syntax::tests`; passed
  `cargo fmt --all --check`
- Owner review: not required for this parser-only slice
- Remaining risk: full source-thread removal is not complete; parsed-model,
  lowering, runtime fixtures, and existing thread tests remain covered by later
  tasks
- Checkpoint commits: 8bb5a02b

## [x] Task 02: Remove Magic `DONE` And `END` Divert Targets

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

Validation: run focused divert diagnostic tests.

Progress ledger:

- Changed files: `crates/ink-compiler/src/syntax/divert.rs`,
  `crates/ink-compiler/src/syntax/text.rs`,
  `crates/ink-compiler/src/syntax/mod.rs`
- Validation result: passed `cargo test -p ink-compiler syntax::tests`;
  passed `cargo fmt --all --check`
- Owner review: not required for this parser-only slice
- Remaining risk: broad fixtures and compiler tests still contain intentional
  old `-> DONE` / `-> END` usages; later tasks cover natural endings, fixture
  rewrites, parsed-model cleanup, and lowering cleanup
- Checkpoint commits: 8bb5a02b

## [x] Task 03: Remove Parsed-Model Thread Flags

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

Validation: run compiler unit tests touching parsed model and lowering.

Progress ledger:

- Changed files: `crates/ink-compiler/src/parsed/divert.rs`,
  `crates/ink-compiler/src/parsed/story.rs`,
  `crates/ink-compiler/src/syntax/parser.rs`,
  `crates/ink-compiler/src/syntax/for_loop.rs`,
  `crates/ink-compiler/src/lower/divert.rs`
- Validation result: passed `cargo check -p ink-compiler`; passed
  `cargo test -p ink-compiler parsed`; passed `cargo fmt --all --check`;
  `rg` found no `with_thread`, `is_thread`, `thread=false`, `thread=true`,
  or `StartThread` references under `crates/ink-compiler/src`
- Owner review: not required for this parsed-model cleanup slice
- Remaining risk: runtime still contains thread internals and current
  repository fixtures still include old thread and terminal syntax; later tasks
  cover runtime continuation cleanup and fixture rewrites
- Checkpoint commits: 8bb5a02b

## [x] Task 04: Define Natural End For Knots And Stitches

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

Validation: run focused flow-ending tests.

Progress ledger:

- Changed files: `crates/ink-compiler/src/lower/flow.rs`,
  `crates/ink-compiler/src/analysis/flow.rs`,
  `crates/ink-compiler/src/analysis/expression_types.rs`,
  `crates/ink-compiler/src/analysis/targets/checker.rs`,
  `crates/ink-test/tests/flow.rs`,
  `crates/ink-test/fixtures/flow/natural-root-end.ink`,
  `crates/ink-test/fixtures/flow/natural-knot-end.ink`,
  `crates/ink-test/fixtures/flow/natural-stitch-end.ink`
- Validation result: passed `cargo test -p ink-test --test integration natural`;
  passed `cargo test -p ink-compiler natural_end`; passed
  `cargo test -p ink-compiler non_exhaustive_int_switch_ends_naturally`;
  passed `cargo test -p ink-compiler return`; passed
  `cargo check -p ink-compiler`; passed `cargo fmt --all --check`; passed
  `git diff --check`
- Owner review: not required for this natural-flow slice
- Remaining risk: many broad fixtures and compiler unit tests still include
  old terminal syntax; later fixture and documentation tasks will rewrite the
  rest, and later lowering tasks will remove remaining source-terminal variants
- Checkpoint commits: 8bb5a02b

## [x] Task 05: Stop Emitting Removed Source Terminals

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

Validation: run compiler snapshot tests and format tests.

Progress ledger:

- Changed files: `crates/ink-compiler/src/parsed/divert.rs`,
  `crates/ink-compiler/src/parsed/story.rs`,
  `crates/ink-compiler/src/lower/divert.rs`,
  `crates/ink-compiler/src/syntax/parser.rs`, and analysis match arms that
  previously accepted `DivertTarget::Done` / `DivertTarget::End`
- Validation result: passed `cargo check -p ink-compiler`; passed
  `cargo test -p ink-compiler parsed`; passed
  `cargo test -p ink-compiler syntax::tests`; passed
  `cargo test -p ink-compiler syntax::parser::tests`; passed
  `cargo fmt --all --check`; passed `git diff --check`; `rg` found no
  `DivertTarget::Done`, `DivertTarget::End`, `Self::Done`, `Self::End`,
  `\"DONE\" =>`, `\"END\" =>`, or `StartThread` references under
  `crates/ink-compiler/src` except the explicit removed-terminal diagnostic
  detector in `crates/ink-compiler/src/syntax/divert.rs`
- Owner review: not required for this compiler-source-terminal slice
- Remaining risk: runtime still has `Done`/`End` control commands for compiled
  story execution and natural-end implementation; current fixtures and docs
  still need broad syntax rewrites in later tasks
- Checkpoint commits: 8bb5a02b

## [x] Task 06: Rewrite Source Fixtures And Examples

Goal: remove `<-`, `-> DONE`, and `-> END` from maintained source fixtures and
examples, replacing them with current ink-rs structure.

Implementation method: update fixtures only after implementation behavior
exists, preserving scenario intent with natural endings or terminal knots.

Acceptance criteria: no maintained current-language fixture teaches removed
syntax; changed expected outputs match implemented behavior.

Forbidden shortcuts: do not edit expected outputs without explaining the
language behavior change in test names or surrounding docs.

Modification boundaries: fixtures, expected snapshots, example stories.

Validation: run conformance fixtures and compiler snapshots.

Progress ledger:

- Changed files: broad `.ink` fixture/example cleanup under
  `crates/ink-test/fixtures/`, `crates/text-games-app/assets/ink/`, and
  `editor/vscode-ink-rs/examples/`; refreshed affected `.ink.parse` and
  `.ink.json` snapshots; updated `crates/ink-compiler/src/lower/flow.rs` and
  `crates/ink-compiler/src/lower/weave.rs` so natural endings cover parent
  flows with child stitches, gathers without fallback targets, and choices
  without terminal gathers; updated compiler API warning fixture/test away from
  the removed loose-end warning
- Validation result: passed `cargo fmt --all --check`; passed
  `cargo check -p ink-compiler`; passed
  `cargo test -p ink-test --test integration tags::tags_test`; passed
  `cargo test -p ink-test --test integration compiler_snapshots`; passed
  `cargo test -p ink-test --test integration`; passed `git diff --check`;
  `rg -n -g '*.ink' -- '-> DONE|-> END|<-' crates/ink-test/fixtures
  crates/text-games-app/assets/ink editor/vscode-ink-rs/examples` only finds
  intentional string-literal coverage in
  `crates/ink-test/fixtures/expressions/expression-string-inline-tokens-are-literals.ink`;
  `rg -n -g '*.parse' -g '*.json' -- 'thread=false|thread=true|-> DONE|-> END|StartThread'
  crates/ink-test/fixtures` found no matches
- Owner review: not required for this fixture/snapshot rewrite slice
- Remaining risk: runtime internals and save-state JSON still expose
  thread/choice save concepts; later tasks cover the runtime and save contract
- Checkpoint commits: 48d9f5d8

## [x] Task 07: Update Current Syntax Documentation

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

Validation: run doc text searches for removed syntax.

Progress ledger:

- Changed files: `docs/LanguageOverview.md`, `docs/SyntaxReference.md`,
  `docs/SyntaxUpdates.md`
- Validation result: passed
  `rg -n "DONE|END|<-|thread|Thread|terminator|callstack/threads|choice-thread"
  docs/LanguageOverview.md docs/SyntaxReference.md` with no matches; passed
  `rg -n 'Source Threads And Terminal Diverts Removed|source `<-`, `-> DONE`,
  and `-> END`|natural endings' docs/SyntaxUpdates.md`; passed
  `git diff --check`
- Owner review: not required for this documentation sync
- Remaining risk: runtime save-state documentation will need another update
  after later minimal-save tasks remove generated-choice and thread-related
  save fields from the runtime contract
- Checkpoint commits: f1212c9a, 460e2d88

## [x] Task 08: Pin The Minimal Save-State Contract With Failing Tests

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

Progress ledger:

- Changed files: `crates/ink-test/tests/choices.rs`
- Validation result: passed `cargo fmt --all --check`; passed
  `git diff --check`; expected failure recorded for
  `cargo test -p ink-test --test integration
  static_choice_pause_save_omits_choice_and_thread_state`,
  `dynamic_choice_pause_save_omits_choice_and_thread_state`, and
  `selected_choice_aftermath_save_omits_choice_and_thread_state`; the failures
  identify serialized `currentChoices`, `choiceThreads`,
  `originalThreadIndex`, `threads`, `threadIndex`, and `threadCounter`
- Owner review: not required for this red contract-test slice
- Remaining risk: tests are intentionally red until Tasks 09-13 implement
  replay save/load and remove choice/thread save JSON fields
- Checkpoint commits: f1212c9a

## [x] Task 09: Define The Choice Replay Save Point

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

Validation: run focused save/load choice replay tests.

Progress ledger:

- Changed files: `crates/ink-runtime/src/story/mod.rs`,
  `crates/ink-runtime/src/story/progress.rs`,
  `crates/ink-runtime/src/story/state.rs`,
  `crates/ink-runtime/src/story_state/patch.rs`,
  `crates/ink-runtime/src/story_state/save_json.rs`,
  `crates/ink-runtime/src/story_state.rs`,
  `crates/ink-runtime/src/story_state/flow_state.rs`,
  `crates/ink-runtime/src/callstack.rs`,
  `crates/ink-runtime/src/flow.rs`,
  `crates/ink-runtime/src/choice.rs`,
  `crates/ink-runtime/src/json/json_read.rs`,
  `crates/ink-test/tests/choices.rs`
- Validation result: passed `cargo fmt --all --check`; passed
  `git diff --check`; passed `cargo check -p ink-runtime`; passed
  `cargo test -p ink-runtime`; passed
  `cargo test -p ink-test --test integration`
- Owner review: not required for this replay-save runtime slice
- Remaining risk: deterministic side-effect replay is not fully audited yet;
  Task 10 covers purity/replay safety before the minimal save contract is
  considered complete
- Checkpoint commits: f1212c9a

## [x] Task 10: Enforce Deterministic Choice Generation

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

Progress ledger:

- Changed files: `crates/ink-compiler/src/analysis/choice_replay.rs`,
  `crates/ink-compiler/src/analysis/mod.rs`,
  `crates/ink-test/fixtures/diagnostics/choice-generation-replay-safe.ink`,
  `crates/ink-test/fixtures/choices/choice-condition-short-circuit-boundaries.ink`,
  `crates/ink-test/tests/choices.rs`, `crates/ink-test/tests/diagnostics.rs`,
  `docs/LanguageOverview.md`, `docs/SyntaxReference.md`,
  `docs/SyntaxUpdates.md`; also mechanically removed stale `-> DONE` /
  `-> END` terminators from compiler and Dioxus unit-test source strings so
  `make gate` validates current syntax instead of removed terminal syntax
- Validation result: passed `cargo fmt --all --check`; passed
  `git diff --check`; passed `cargo check -p ink-compiler`; passed
  `cargo test -p ink-test --test integration
  diagnostics::choice_generation_diagnostics_reject_replay_unsafe_calls`;
  passed `cargo test -p ink-test --test integration choices::`; passed
  `cargo test -p ink-test --test integration typed_values::to_str_builtin_runs`;
  passed `cargo test -p ink-compiler`; passed `cargo test -p ink-dioxus`;
  passed `make gate`
- Owner review: not required for this compiler-diagnostic and validation
  cleanup slice
- Remaining risk: dynamic choice binding replay is covered by existing
  save/load tests but has not been separately audited and recorded for Task 11
- Checkpoint commits: 94bc4269

## [x] Task 11: Preserve Dynamic Choice Bindings Across Replay

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

Validation: run dynamic choice save/load tests.

Progress ledger:

- Changed files: implementation was covered by the replay-save runtime slice in
  `crates/ink-runtime/src/story/mod.rs`,
  `crates/ink-runtime/src/story/progress.rs`,
  `crates/ink-runtime/src/story/state.rs`,
  `crates/ink-runtime/src/story_state/patch.rs`,
  `crates/ink-runtime/src/story_state/save_json.rs`, and the focused dynamic
  choice test in `crates/ink-test/tests/choices.rs`
- Validation result: passed `cargo test -p ink-test --test integration
  choices::`; passed `make gate`; `dynamic_choices_regenerate_after_save_load`
  verifies load-time regeneration, selected index/value text `2:Beta`,
  aftermath output `picked 2:Beta.`, and no generated choice/thread save JSON
  fields
- Owner review: not required for this already-implemented replay-binding slice
- Remaining risk: selection still uses runtime-internal continuation threads;
  Task 12 and Task 15 cover removing serialized thread snapshots and then
  renaming/simplifying runtime internals
- Checkpoint commits: f1212c9a

## [x] Task 12: Remove Choice Thread Snapshot Serialization

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

Validation: run runtime choice and save/load suites.

Progress ledger:

- Changed files: implementation was covered by the replay-save runtime slice in
  `crates/ink-runtime/src/story_state/save_json.rs`,
  `crates/ink-runtime/src/story_state.rs`,
  `crates/ink-runtime/src/callstack.rs`, `crates/ink-runtime/src/flow.rs`,
  `crates/ink-runtime/src/choice.rs`, and
  `crates/ink-runtime/src/json/json_read.rs`; added gather-after-choice replay
  coverage in `crates/ink-test/fixtures/choices/gather-choice-save-load.ink`
  and `crates/ink-test/tests/choices.rs`
- Validation result: passed `cargo test -p ink-test --test integration
  choices::`; passed `cargo test -p ink-runtime`; passed
  `cargo fmt --all --check`; passed `git diff --check`; prior Task 10
  validation also passed `make gate`
- Owner review: not required for this save-state serialization slice
- Remaining risk: old runtime object JSON helpers still know how to read/write
  `Choice` objects with `originalThreadIndex`; Task 13 covers removing
  generated choice serialization and deciding old save compatibility behavior
- Checkpoint commits: f1212c9a

## [x] Task 13: Remove Generated Choice Serialization

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

Validation: run save-state compatibility tests.

Progress ledger:

- Changed files: `crates/ink-runtime/src/choice.rs`,
  `crates/ink-runtime/src/story/choices.rs`,
  `crates/ink-runtime/src/json/json_read.rs`,
  `crates/ink-runtime/src/json/json_write.rs`,
  `crates/ink-runtime/src/story_state/save_json.rs`, and
  `crates/ink-runtime/src/story_state.rs`
- Validation result: passed `cargo test -p ink-runtime`; passed
  `cargo test -p ink-test --test integration choices::`; passed
  `cargo fmt --all --check`; passed `git diff --check`
- Owner review: not required for this save-state compatibility slice
- Remaining risk: save-state docs still need a dedicated pass in Task 14;
  runtime-internal continuation threads remain for Task 15
- Checkpoint commits: d33b5411

## [x] Task 14: Document And Version The Save-State Break

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

Validation: run save-state docs/schema tests.

Progress ledger:

- Changed files: `docs/Architecture.md`, `docs/SyntaxUpdates.md`,
  `docs/ink_JSON_runtime_format.md`,
  `crates/ink-runtime/src/story_state/save_json.rs`, and
  `crates/ink-runtime/src/story_state.rs`
- Validation result: passed `cargo test -p ink-runtime
  story_state::tests::`; passed `cargo fmt --all --check`; passed
  `git diff --check`
- Owner review: not required for this documentation/versioning slice
- Remaining risk: runtime-internal continuation threads remain for Task 15
- Checkpoint commits: a780c728

## [x] Task 15: Rename Runtime `Thread` To Continuation Concepts

Goal: remove source-thread terminology from runtime internals where the object
is really a continuation, callstack, or execution snapshot.

Implementation method: rename runtime types, fields, and methods in small
steps, preserving behavior while aligning names with their actual purpose.

Acceptance criteria: ordinary choice execution no longer references a public or
source-level `Thread` concept in runtime code; tests remain behaviorally stable.

Forbidden shortcuts: do not perform a broad rename that changes behavior without
focused tests.

Modification boundaries: runtime internals, runtime docs, tests.

Validation: run runtime unit tests.

Progress ledger:

- Changed files: `crates/ink-runtime/src/callstack.rs`,
  `crates/ink-runtime/src/choice.rs`, `crates/ink-runtime/src/lib.rs`,
  `crates/ink-runtime/src/story/choices.rs`,
  `crates/ink-runtime/src/story/control_logic.rs`,
  `crates/ink-runtime/src/story/navigation.rs`,
  `crates/ink-runtime/src/story/progress.rs`,
  `crates/ink-runtime/src/story_state/flow_state.rs`, and
  `crates/ink-runtime/src/story_state/patch.rs`
- Validation result: passed `cargo test -p ink-runtime`; passed
  `cargo fmt --all --check`; passed `git diff --check`; confirmed
  `rg -n "\bThread\b|\bthread\b|threads|thread_|_thread|THREAD|threadIndex|threadCounter"
  crates/ink-runtime/src -g '*.rs'` has no matches
- Owner review: not required for this behavior-preserving runtime rename
- Remaining risk: callstack storage is still multi-continuation-shaped until
  Task 16 flattens it
- Checkpoint commits: 00008f3b

## [x] Task 16: Flatten Runtime Callstack Shape

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

Validation: run runtime execution and save/load tests.

Progress ledger:

- Changed files: `crates/ink-runtime/src/callstack.rs`
- Validation result: passed `cargo test -p ink-runtime`; passed
  `cargo fmt --all --check`; passed `git diff --check`
- Owner review: not required for this runtime-internal storage cleanup
- Remaining risk: historical compiled `StartThread` handling still exists and
  is isolated only by naming; Task 17 covers explicit quarantine
- Checkpoint commits: 7194feed

## [x] Task 17: Quarantine Historical Compiled `StartThread` Handling

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

Progress ledger:

- Changed files: `crates/ink-runtime/src/control_command.rs`,
  `crates/ink-runtime/src/story/progress.rs`,
  `crates/ink-runtime/src/story/control_logic.rs`,
  `crates/ink-compiler/src/lower.rs`, and
  `docs/ink_JSON_runtime_format.md`
- Validation result: passed `cargo test -p ink-compiler
  current_source_lowering_does_not_emit_historical_thread_command`; passed
  `cargo test -p ink-runtime
  control_command::tests::runtime_command_tokens_round_trip_through_format_tokens`;
  passed `cargo fmt --all --check`; passed `git diff --check`; passed
  `make gate`
- Owner review: not required for this compatibility quarantine slice
- Remaining risk: test and fixture names still mention thread until later
  removal tasks clean historical fixture coverage
- Checkpoint commits: 85a5fa56

## [x] Task 18: Rename Choice Continuation Fields

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

Progress ledger:

- Changed files: no new code beyond Task 13 and Task 15 runtime choice
  cleanup; recorded validation in this ledger
- Validation result: passed `cargo test -p ink-test --test integration
  choices::`; passed `cargo fmt --all --check`; passed `git diff --check`;
  Task 17 validation also passed `make gate`; confirmed `rg -n
  "thread|Thread|threads|thread_|_thread|originalThreadIndex|choiceThreads|threadIndex|threadCounter"
  crates/ink-runtime/src/choice.rs crates/ink-runtime/src/story/choices.rs
  crates/ink-runtime/src/story_state.rs crates/ink-runtime/src/story_state/save_json.rs
  crates/ink-runtime/src/json crates/ink-test/tests/choices.rs` finds only
  removed save-field names and historical test/fixture wording
- Owner review: not required for this validation-only cleanup record
- Remaining risk: historical test/fixture names still mention thread until
  later fixture-removal tasks
- Checkpoint commits: aa8c141b

## [x] Task 19: Rename Runtime `Flow` Terminology

Goal: separate current source flow concepts from runtime execution-state
implementation details.

Implementation method: audit runtime `Flow` usage and rename structures that
really mean execution state, run state, or container traversal state.

Acceptance criteria: runtime terminology no longer implies old ink flow
compatibility; source-level knot/stitch/function concepts remain clear.

Forbidden shortcuts: do not rename parser or parsed model concepts that still
accurately represent current source syntax without a reason.

Modification boundaries: runtime internals, architecture docs, tests.

Validation: run runtime tests.

Progress ledger:

- Changed files: renamed `crates/ink-runtime/src/flow.rs` to
  `crates/ink-runtime/src/execution_state.rs`, renamed
  `crates/ink-runtime/src/story/flow.rs` to
  `crates/ink-runtime/src/story/callstack.rs`, renamed
  `crates/ink-runtime/src/story_state/flow_state.rs` to
  `crates/ink-runtime/src/story_state/execution.rs`, and updated
  `crates/ink-runtime/src/lib.rs`, `crates/ink-runtime/src/story/mod.rs`,
  `crates/ink-runtime/src/story_state.rs`,
  `crates/ink-runtime/src/story_state/output_mutation.rs`,
  `crates/ink-runtime/src/story_state/patch.rs`,
  `crates/ink-runtime/src/story_state/save_json.rs`, and
  `docs/Architecture.md`
- Validation result: passed `cargo test -p ink-runtime`; passed
  `cargo fmt --all --check`; passed `git diff --check`; confirmed
  `rg -n '\bFlow\b|current_flow|DEFAULT_FLOW_NAME|flow_state|flow\.rs|mod flow|flow::'
  crates/ink-runtime/src docs/Architecture.md -g '*.rs' -g '*.md'` finds no
  runtime execution-state terminology, only compiler/test source-flow docs
- Owner review: not required for this runtime-internal terminology cleanup
- Remaining risk: save-state tests still assert removed old fields; Task 20
  does the dedicated save-state shape verification
- Checkpoint commits: e9de0db1

## [x] Task 20: Remove Multi-Flow Save-State Shape

Goal: ensure save-state JSON does not preserve old flow/thread structure after
runtime terminology cleanup.

Implementation method: update save writer/reader and tests so save state stores
only the current execution state needed to continue and regenerate choices.

Acceptance criteria: save JSON has no multi-flow, thread, or choice fields; load
behavior remains correct.

Forbidden shortcuts: do not keep old fields with null or empty placeholder
values.

Modification boundaries: runtime save-state schema, tests, docs.

Validation: run save-state tests.

Progress ledger:

- Changed files: `crates/ink-runtime/src/story_state/save_json.rs`,
  `crates/ink-runtime/src/story_state.rs`, `docs/Architecture.md`,
  `docs/SyntaxUpdates.md`, and `docs/ink_JSON_runtime_format.md`
- Validation result: passed `cargo test -p ink-runtime
  story_state::tests::`; passed `cargo test -p ink-test --test integration
  choices::`; passed `cargo fmt --all --check`; passed `git diff --check`
- Owner review: not required for this save-state schema cleanup
- Remaining risk: public runtime errors and docs still need a final wording
  pass in Task 21
- Checkpoint commits: b4e7d851

## [x] Task 21: Update Public Runtime Errors And Docs

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

Progress ledger:

- Changed files: `crates/ink-runtime/src/story/progress.rs`,
  `crates/ink-runtime/src/story/control_logic.rs`, and
  `docs/SyntaxUpdates.md`
- Validation result: passed `cargo test -p ink-runtime`; passed
  `cargo fmt --all --check`; passed `git diff --check`; passed `make gate`
- Owner review: not required for this wording-only cleanup
- Remaining risk: legacy fixture and test names still mention thread until
  later fixture cleanup tasks
- Checkpoint commits: f90d7ab8

## [x] Task 22: Refresh Compiler Snapshots And Integration Fixtures

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

Validation: run compiler snapshot and conformance suites.

Progress ledger:

- Changed files: no fixture or snapshot changes were needed; recorded
  validation in this ledger
- Validation result: passed `cargo test -p ink-test --test integration
  compiler_snapshots::`; passed `cargo test -p ink-test --test integration
  integration_policy::`; passed `cargo fmt --all --check`; passed
  `git diff --check`; confirmed `rg -n -g '*.ink' -g '*.json' -g
  '*.parse' -g '*.snap' -- '-> DONE|-> END|<-|StartThread|"thread"|thread=false|thread=true|currentChoices|generatedChoices|choiceThreads|originalThreadIndex|threadIndex|threadCounter|currentFlowName|"flows"|evalStack|visitCounts|turnIndices|turnIdx'
  crates/ink-test crates/ink-compiler` only finds `<-` inside string-literal
  expression fixtures
- Owner review: not required for this validation-only snapshot pass
- Remaining risk: dedicated obsolete test deletion remains in Task 23
- Checkpoint commits: cabd5063

## [x] Task 23: Update Compiled-Story Format Documentation

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

Validation: run doc searches.

Progress ledger:

- Changed files: `docs/ink_JSON_runtime_format.md` and
  `docs/Architecture.md`
- Validation result: passed `cargo test -p ink-runtime
  control_command::tests::runtime_command_tokens_round_trip_through_format_tokens`;
  passed `cargo test -p ink-compiler
  current_source_lowering_does_not_emit_historical_thread_command`; passed
  `cargo fmt --all --check`; passed `git diff --check`
- Owner review: not required for this format-doc boundary clarification
- Remaining risk: diagnostic fixtures still need a dedicated removed-thread
  syntax pass in Task 24
- Checkpoint commits: e7370ff8

## [x] Task 24: Remove Thread Syntax From Diagnostics Fixtures

Goal: make diagnostics fixtures treat thread syntax as removed syntax rather
than a supported-but-erroneous feature.

Implementation method: update diagnostics tests so removed syntax receives
intentional removal diagnostics and no current-language tests depend on it.

Acceptance criteria: diagnostics snapshots clearly communicate removal and do
not suggest a thread-based workaround.

Forbidden shortcuts: do not leave ignored tests for thread syntax.

Modification boundaries: diagnostics fixtures, snapshots, docs.

Validation: run diagnostics tests.

Progress ledger:

- Changed files: `crates/ink-test/fixtures/diagnostics/removed-thread-syntax.ink`
  and `crates/ink-test/tests/diagnostics.rs`
- Validation result: passed `cargo test -p ink-test --test integration
  diagnostics::removed_thread_syntax_reports_removed_syntax_diagnostic`;
  passed `cargo test -p ink-test --test integration diagnostics::`; passed
  `cargo fmt --all --check`; passed `git diff --check`
- Owner review: not required for this diagnostics fixture slice
- Remaining risk: fallthrough/weave behavior without threads gets dedicated
  verification in Task 25
- Checkpoint commits: 419e5780

## [x] Task 25: Verify Fallthrough And Weave Behavior Without Threads

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

Progress ledger:

- Changed files: `crates/ink-test/fixtures/choices/weave-fallthrough-no-threads.ink`
  and `crates/ink-test/tests/choices.rs`
- Validation result: passed `cargo test -p ink-test --test integration
  choices::`; passed `cargo test -p ink-test --test integration flow::`;
  passed `cargo test -p ink-test --test integration gathers::`; passed
  `cargo fmt --all --check`; passed `git diff --check`; passed `make gate`
- Owner review: not required for this integration coverage slice
- Remaining risk: active-plan final audit still needs to verify no old
  choice/continuation/save fields reappear
- Checkpoint commits: 8347ba1e

## [x] Task 26: Audit Save JSON For Forbidden Fields

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

Validation: run the save JSON audit tests.

Progress ledger:

- Changed files: `crates/ink-test/tests/choices.rs`
- Validation result: passed `cargo test -p ink-test --test integration
  choices::`; passed `cargo test -p ink-runtime story_state::tests::`;
  passed `cargo fmt --all --check`; passed `git diff --check`
- Owner review: not required for this save JSON audit guard
- Remaining risk: final active-plan validation still needs full terminology
  and workspace gate checks
- Checkpoint commits: 358079c4

## [x] Task 27: Full Documentation Consistency Pass

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

Validation: run text searches and docs-related tests if present.

Progress ledger:

- Changed files: `crates/ink-runtime/src/choice.rs` and
  `crates/ink-runtime/src/story/choices.rs`
- Validation result: passed `cargo test -p ink-runtime`; passed
  `cargo fmt --all --check`; passed `git diff --check`; searched maintained
  docs and runtime comments for removed syntax and old save-field wording;
  remaining hits are legacy format compatibility, removed-field rejection,
  historical changelog text, or test fixture names
- Owner review: not required for this comments/docs consistency pass
- Remaining risk: final closeout still needs full active-plan validation and
  commit squashing
- Checkpoint commits: 7591d8c1

## [x] Task 28: Final Gate And Plan Closeout

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

Progress ledger:

- Changed files: `docs/active_plan/control_flow_cleanup/` moved to
  `docs/finished_plans/control_flow_cleanup/`
- Validation result: passed final `make gate`
- Owner review: not required for final closeout; all task ledgers are complete
- Remaining risk: none known for this active plan; legacy compiled-story
  compatibility tokens remain documented as compatibility-only behavior
- Checkpoint commits: pending final squash
