Progress: 29/29

# Compiled JSON Current-Only Format Break Tasks

Status key:

- `[ ]` pending
- `[~]` in progress
- `[>]` implemented and validated, awaiting owner review
- `[x]` complete
- `[!]` blocked

Task rules:

- Complete tasks in order unless the project owner explicitly reprioritizes.
- Do not treat task completion as a commit boundary.
- Optional checkpoint commits may be created when the temporary diff becomes too
  large, risky, or hard to review. They are development aids only.
- Before this active plan is pushed or considered landed, combine all commits
  belonging to it into one final active-plan commit unless the project owner
  asks for a different history shape.
- Update each task ledger immediately after focused validation passes or a
  blocker is found.
- Do not start the next implementation task until the current task is `[x]`,
  unless that task ledger records why limited overlap is safe.
- `make gate` is required before final closeout.

Common forbidden shortcuts:

- Do not keep compatibility shims for removed compiled JSON commands.
- Do not hardcode fixture names, fixture paths, JSON fragments, or expected
  output strings to satisfy tests.
- Do not delete a format feature before replacing current runtime/compiler
  behavior that still depends on it.
- Do not hide behavior changes by snapshot-only edits.
- Do not remove current choice-point behavior while deleting legacy container
  count metadata.
- Do not leave ignored tests, empty macros, dead variants, or no-op branches
  that only served the removed format.

## Milestone A: Format Contract

### [x] Task 01: Bump The Compiled Story JSON Version

Goal: make the format break explicit by moving the current compiled story JSON
version forward and ensuring old compiled stories are rejected by version before
compatibility decoding is considered.

Implementation method: update the shared format version constant, runtime
version checks, compiler emission expectations, and tests that assert current
and non-current version behavior. Use the task's implementation pass to locate
version literals in tests and snapshots and replace them with the new current
contract where appropriate.

Acceptance criteria: compiler output uses the new version, runtime accepts only
the new version, old `inkVersion: 1` stories fail with the existing version
mismatch path, and tests no longer imply that v1 is current.

Forbidden shortcuts: do not weaken version checking; do not special-case old
versions after the bump.

Modification boundaries: `ink-story-json-format`, runtime JSON loading tests,
compiler emission tests and snapshots that only need the version update.

Validation: `cargo test -p ink-story-json-format`; `cargo test -p ink-runtime
json_read`; `cargo test -p ink-compiler emit`.

Progress ledger:

- Changed files: `crates/ink-story-json-format/src/lib.rs`,
  `crates/ink-story-json-format/src/json/tests.rs`,
  `crates/ink-runtime/src/json/json_read.rs`,
  `crates/ink-runtime/src/story_state.rs`,
  `crates/ink-compiler/src/emit.rs`, regenerated snapshots under
  `crates/ink-test/fixtures/**/*.ink.json`.
- Validation result: passed `cargo test -p ink-story-json-format`; passed
  `cargo test -p ink-runtime json_read`; passed `cargo test -p ink-compiler
  emit`; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Old `inkVersion: 1` compiled stories are
  intentionally rejected.
- Checkpoint commits: none

### [x] Task 02: Delete Removed Control Commands From The Format Model

Goal: remove typed compiled JSON representation for `done`, `end`, `thread`,
`choiceCnt`, `turn`, `turns`, `readc`, `visit`, and `seq`.

Implementation method: delete the corresponding `ControlCommand` variants,
token mappings, serialization paths, and format tests that round-trip them.
Add or update tests proving those tokens are not current format commands.

Acceptance criteria: removed command tokens cannot parse as
`ControlCommand`; current command tokens still round-trip; no format test treats
removed commands as supported data.

Forbidden shortcuts: do not map removed commands to `NoOp`, `Pop`, or another
surviving command.

Modification boundaries: `crates/ink-story-json-format/src/model.rs`, format
JSON codec tests, and directly affected compile errors.

Validation: `cargo test -p ink-story-json-format control`; `cargo test -p
ink-story-json-format`.

Progress ledger:

- Changed files: `crates/ink-story-json-format/src/model.rs`,
  `crates/ink-story-json-format/src/json/tests.rs`,
  `crates/ink-runtime/src/control_command.rs`,
  `crates/ink-runtime/src/story/control_logic.rs`,
  `crates/ink-runtime/src/story/progress.rs`,
  `crates/ink-runtime/src/callstack.rs`.
- Validation result: passed `cargo test -p ink-story-json-format`; passed
  `cargo check -p ink-runtime`; passed `cargo test -p ink-runtime`; final
  `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Removed command tokens survive only in rejection
  tests and changelog text.
- Checkpoint commits: none

### [x] Task 03: Delete The Compiled `CNT?` Read-Count Object

Goal: remove compiled JSON `{"CNT?": ...}` from the current object model and
codec.

Implementation method: delete `Object::ReadCount`, JSON reader/writer handling,
runtime reader/writer conversion, and tests that encode it as supported current
JSON. Replace them with rejection tests where useful.

Acceptance criteria: `CNT?` is rejected as invalid current compiled JSON; there
is no typed `ReadCount` object in the format or runtime conversion layers.

Forbidden shortcuts: do not leave `ReadCount` as a dead enum variant or
deserialize it into a dummy value.

Modification boundaries: format model/codec, runtime JSON conversion,
format/runtime tests.

Validation: `cargo test -p ink-story-json-format read`; `cargo test -p
ink-runtime json_read`.

Progress ledger:

- Changed files: `crates/ink-story-json-format/src/model.rs`,
  `crates/ink-story-json-format/src/json/object.rs`,
  `crates/ink-story-json-format/src/json/tests.rs`,
  `crates/ink-runtime/src/json/json_read.rs`,
  `crates/ink-runtime/src/json/json_write.rs`,
  `crates/ink-runtime/src/variable_reference.rs`,
  `crates/ink-runtime/src/story/control_logic.rs`,
  `crates/ink-compiler/src/lower/path.rs`.
- Validation result: passed `cargo test -p ink-story-json-format`; passed
  `cargo test -p ink-runtime json_read`; passed `cargo test -p ink-runtime
  json_write`; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. `CNT?` is rejected by format and runtime loader
  tests.
- Checkpoint commits: none

### [x] Task 04: Remove Container Count Flags From The Format Model

Goal: delete compiled JSON container `#f` visit/turn/count-start metadata from
the current format.

Implementation method: remove `Container.flags`, constructors that only exist
to set flags, `#f` parsing/writing, and tests that expect count flags. Replace
any current name-preservation use of `#f` with current naming/path metadata.

Acceptance criteria: current compiled JSON never emits `#f`; current format
loading rejects a terminator containing `#f`; named containers and label/gather
paths still resolve correctly.

Forbidden shortcuts: do not keep `#f` parsing while ignoring it; do not use
another legacy count flag field as a replacement.

Modification boundaries: format container model/codec, compiler lowering that
sets container flags, runtime `Container` construction and JSON writer
conversion.

Validation: `cargo test -p ink-story-json-format container`; `cargo test -p
ink-compiler lower`; `cargo test -p ink-runtime json_read`.

Progress ledger:

- Changed files: `crates/ink-story-json-format/src/model.rs`,
  `crates/ink-story-json-format/src/json/container.rs`,
  `crates/ink-story-json-format/src/json/tests.rs`,
  `crates/ink-runtime/src/container.rs`,
  `crates/ink-runtime/src/json/json_read.rs`,
  `crates/ink-runtime/src/json/json_write.rs`,
  `crates/ink-runtime/src/story/navigation.rs`, compiler lowering under
  `crates/ink-compiler/src/lower*.rs`.
- Validation result: passed `cargo test -p ink-story-json-format`; passed
  `cargo test -p ink-runtime container`; passed `cargo test -p ink-runtime
  json_read`; passed `cargo test -p ink-runtime json_write`; final
  `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. `#f` is rejected and no current JSON output
  emits container count metadata.
- Checkpoint commits: none

### [x] Task 05: Add Current-Only Format Rejection Coverage

Goal: make removed compiled JSON forms fail deliberately and visibly.

Implementation method: add focused tests for removed command tokens, `CNT?`,
`#f`, and old `inkVersion` values. Keep successful round-trip tests for the
surviving current command/object set.

Acceptance criteria: each removed compiled JSON form has a test that would fail
if compatibility decoding returned; current minimal stories and current command
round-trips still pass.

Forbidden shortcuts: do not rely on broad snapshot failures as the only
coverage for the format break.

Modification boundaries: format and runtime JSON tests.

Validation: `cargo test -p ink-story-json-format`; `cargo test -p ink-runtime
json_read`.

Progress ledger:

- Changed files: `crates/ink-story-json-format/src/json/tests.rs`,
  `crates/ink-runtime/src/json/json_read.rs`,
  `crates/ink-test/tests/integration_policy.rs`.
- Validation result: passed `cargo test -p ink-story-json-format`; passed
  `cargo test -p ink-runtime json_read`; passed `cargo test -p ink-test
  --test integration current_compiled_json_surfaces_reject_removed_legacy_format`;
  final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known.
- Checkpoint commits: none

## Milestone B: Compiler Lowering

### [x] Task 06: Stop Emitting Root `done` And Global `end`

Goal: remove compiler output of `ControlCommand::Done` from root content and
`ControlCommand::End` from global declarations.

Implementation method: change module-story lowering so the root contains only
current entry diverts and named content, and global declaration containers end
after current evaluation work. Update focused lowering and emission tests.

Acceptance criteria: a minimal compiled story has no `done` or `end` tokens;
global declarations still initialize before entry execution; no root/global
test depends on terminal command placeholders.

Forbidden shortcuts: do not replace removed commands with another no-op token
solely to preserve old snapshots.

Modification boundaries: compiler lowering root/global paths and affected
tests.

Validation: `cargo test -p ink-compiler lower_module`; `cargo test -p ink-test
--test integration compiler_snapshots`.

Progress ledger:

- Changed files: `crates/ink-compiler/src/lower.rs`,
  `crates/ink-compiler/src/lower/flow.rs`, `crates/ink-compiler/src/emit.rs`,
  regenerated compiler snapshots.
- Validation result: passed `cargo test -p ink-compiler emit`; passed
  `cargo test -p ink-compiler lower`; passed `cargo test -p ink-test --test
  integration compiler_snapshots`; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Root and global declaration output now end by
  current structure instead of `done` or `end`.
- Checkpoint commits: none

### [x] Task 07: Replace Lowering Terminator Helpers With Current Semantics

Goal: remove compiler helper logic that treats `Done` and `End` as current flow
terminators.

Implementation method: rewrite helpers such as `ends_with_end_or_done`,
`ends_with_flow_terminator`, and `done_container` into current terms: explicit
diverts, function returns, tunnel onwards, choice/gather structural boundaries,
and natural container exhaustion.

Acceptance criteria: compiler lowering no longer imports or matches removed
terminal commands; existing current control-flow tests still pass.

Forbidden shortcuts: do not keep a renamed helper whose only behavior is
checking for deleted commands.

Modification boundaries: compiler lowering helpers and tests that cover
fallthrough/termination.

Validation: `cargo test -p ink-compiler lower`; `cargo test -p ink-test --test
integration flow`.

Progress ledger:

- Changed files: `crates/ink-compiler/src/lower.rs`,
  `crates/ink-compiler/src/lower/flow.rs`,
  `crates/ink-compiler/src/lower/weave.rs`,
  `crates/ink-compiler/src/lower/conditional.rs`.
- Validation result: passed `cargo test -p ink-compiler lower`; passed
  `cargo test -p ink-test --test integration flow`; final `make gate`
  passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Lowering helpers now use current explicit-flow
  terminology and `terminal_container` with `nop` where an addressable target is
  needed.
- Checkpoint commits: none

### [x] Task 08: Rework Gather And Choice Fallback Lowering Without `done`

Goal: preserve current weave fallthrough behavior after deleting `done` as a
branch stop marker.

Implementation method: update gather, terminal gather, static choice, and
dynamic choice lowering so paths that should stop do so through current
container structure or explicit current diverts. Add focused regression
fixtures for nested gather, no-fallback gather, static choice aftermath,
dynamic choice aftermath, and mixed static/dynamic choice lists.

Acceptance criteria: choices and gathers neither fall through too far nor stop
too early; dynamic choice aftermath receives the selected value and index; no
lowered weave JSON contains removed commands.

Forbidden shortcuts: do not solve fallthrough by reintroducing a hidden terminal
command under a new token.

Modification boundaries: compiler weave lowering, choice/dynamic choice tests,
affected snapshots.

Validation: `cargo test -p ink-compiler lower::weave`; `cargo test -p ink-test
--test integration choices`; run the new focused regression tests by name.

Progress ledger:

- Changed files: `crates/ink-compiler/src/lower/weave.rs`,
  `crates/ink-test/tests/choices.rs`,
  `crates/ink-test/fixtures/choices/weave-fallthrough-current.ink`,
  dynamic/static choice fixtures and snapshots.
- Validation result: passed `cargo test -p ink-test --test integration
  choices`; passed `cargo test -p ink-test --test integration flow`; passed
  `cargo test -p ink-test --test integration`; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Static, dynamic, nested, and regenerated choices
  keep current aftermath behavior without terminal commands.
- Checkpoint commits: none

### [x] Task 09: Remove Compiler Count-Flag Emission

Goal: stop compiler output from using container count flags for labels or
gathers.

Implementation method: remove `named_container_flags` and every `flags:` field
assignment in compiler-created format containers. If any path depended on
`#f: 4` to keep a name reachable, replace it with current named-content or
`#n` handling through the format model.

Acceptance criteria: compiler-created containers no longer set count flags;
labels, gathers, and dynamic choice target paths still compile and run.

Forbidden shortcuts: do not leave all flags set to `None` if the field still
exists only for old metadata; complete the format-model removal from Task 04.

Modification boundaries: compiler lowering and path tests.

Validation: `cargo test -p ink-compiler lower`; `cargo test -p ink-test --test
integration labels`; `cargo test -p ink-test --test integration choices`.

Progress ledger:

- Changed files: compiler lowering under `crates/ink-compiler/src/lower*.rs`,
  format container model, runtime container conversion, regenerated fixtures.
- Validation result: passed `cargo test -p ink-compiler lower`; passed
  `cargo test -p ink-test --test integration choices`; passed
  `cargo test -p ink-test --test integration`; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Compiler-created containers no longer set count
  flags.
- Checkpoint commits: none

### [x] Task 10: Refresh Compiler Snapshots For The New Format

Goal: update maintained compiler snapshots to the current-only compiled JSON
shape.

Implementation method: regenerate or manually update `.ink.json` snapshots
after code changes, keeping source fixtures unchanged unless they are still
teaching removed source syntax. Review diffs for real semantic changes rather
than accepting bulk output blindly.

Acceptance criteria: snapshots use the new `inkVersion`, contain no removed
compiled JSON tokens or `#f`, and still represent the intended current story
behavior.

Forbidden shortcuts: do not edit snapshots before the compiler emits the new
shape; do not mask failing runtime behavior by snapshot churn.

Modification boundaries: compiler snapshot fixtures and snapshot test support.

Validation: `cargo test -p ink-test --test integration compiler_snapshots`;
`rg -n '"done"|"end"|"thread"|"choiceCnt"|"turns"|"readc"|"visit"|"seq"|CNT\\?|#f'
crates/ink-test -g '*.ink.json'`.

Progress ledger:

- Changed files: regenerated `.ink.json` snapshots under
  `crates/ink-test/fixtures/**/*.ink.json`; renamed misleading choice-loop and
  save/load fixtures.
- Validation result: passed `cargo test -p ink-test --test integration
  compiler_snapshots`; passed the current compiled JSON policy test; final
  `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Snapshots use `inkVersion` 2 and contain no
  removed compiled command tokens, `CNT?`, or `#f`.
- Checkpoint commits: none

### [x] Task 11: Add Compiler Policy Tests For Current-Only JSON

Goal: prevent removed compiled JSON forms from returning through future
compiler changes.

Implementation method: add or update integration policy tests that compile
representative current sources and assert absence of removed tokens, `CNT?`,
and `#f` in emitted JSON. Include static choice, dynamic choice, gather, global
declaration, function, tunnel, tag, and random examples.

Acceptance criteria: the policy tests fail if compiler output reintroduces any
removed compiled JSON form; current supported commands still appear where
expected.

Forbidden shortcuts: do not implement policy tests as filename-only checks;
inspect emitted JSON values.

Modification boundaries: compiler/integration test helpers and fixtures.

Validation: run the new policy tests by name; `cargo test -p ink-test --test
integration compiler_snapshots`.

Progress ledger:

- Changed files: `crates/ink-test/tests/integration_policy.rs`.
- Validation result: passed `cargo test -p ink-test --test integration
  current_compiled_json_surfaces_reject_removed_legacy_format`; final
  `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Policy scans maintained compiled JSON fixtures
  and current format docs, not fixture filenames.
- Checkpoint commits: none

## Milestone C: Runtime Loading And Execution

### [x] Task 12: Remove Runtime Command Variants For Deleted Format Commands

Goal: delete runtime command enum variants and token mappings for removed
compiled JSON commands.

Implementation method: remove `Done`, `End`, `LegacyStartThread`,
`ChoiceCount`, `Turns`, `TurnsSince`, `ReadCount`, `VisitIndex`, and
`SequenceShuffleIndex` from runtime command handling. Keep tag commands and
surviving current format commands intact.

Acceptance criteria: runtime command round-trip tests cover only current
commands; no runtime command variant can execute removed compiled JSON.

Forbidden shortcuts: do not keep removed variants behind `#[allow(dead_code)]`
or private constructors.

Modification boundaries: runtime control command mapping and direct compile
errors.

Validation: `cargo test -p ink-runtime control_command`; `cargo check -p
ink-runtime`.

Progress ledger:

- Changed files: `crates/ink-runtime/src/control_command.rs`,
  `crates/ink-runtime/src/story/control_logic.rs`, runtime JSON reader/writer
  tests.
- Validation result: passed `cargo test -p ink-runtime control_command`;
  passed `cargo check -p ink-runtime`; passed `cargo test -p ink-runtime`;
  final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Runtime command enum now models only current
  command tokens plus runtime tag markers.
- Checkpoint commits: none

### [x] Task 13: Implement Natural Story End Without `Done` Or `End`

Goal: make story execution finish safely when current story content is
naturally exhausted, without terminal control commands.

Implementation method: update progress/step/next-content behavior so root,
knot, stitch, gather, and choice aftermath container exhaustion is a valid
natural end, while function and tunnel frames still error when they miss
`~ return` or `->->`. Remove state fields that existed only for `done/end`
safe-exit tracking if they are no longer current.

Acceptance criteria: natural root/knot/stitch/gather/choice endings pass;
missing function return and missing tunnel onwards still produce clear runtime
errors; no safe-exit branch depends on removed commands.

Forbidden shortcuts: do not mark every null pointer as success if it would hide
function/tunnel errors.

Modification boundaries: runtime progress/navigation/story state and focused
runtime tests.

Validation: `cargo test -p ink-runtime natural`; `cargo test -p ink-test --test
integration flow`; run new missing-return/tunnel regression tests by name.

Progress ledger:

- Changed files: `crates/ink-runtime/src/story/progress.rs`,
  `crates/ink-runtime/src/story/control_logic.rs`, flow fixtures under
  `crates/ink-test/fixtures/flow/`, `crates/ink-test/tests/flow.rs`.
- Validation result: passed `cargo test -p ink-test --test integration flow`;
  passed `cargo test -p ink-runtime`; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Natural endings are accepted for story content,
  while missing function return and tunnel onwards errors remain explicit.
- Checkpoint commits: none

### [x] Task 14: Delete Legacy Thread Continuation Support

Goal: remove runtime continuation machinery that exists only to execute old
compiled `"thread"` commands.

Implementation method: delete `legacy_parent_continuation`,
`push_continuation`, `pop_continuation`, `can_pop_continuation`,
`handle_legacy_start_thread_command`, and progress checks that mention legacy
continuation popping. Keep the ordinary callstack needed for current functions,
tunnels, and host evaluation.

Acceptance criteria: ordinary static and dynamic choices do not use legacy
thread continuation state; source/runtime tests still cover choice selection
and aftermath; no runtime code refers to legacy compiled threads.

Forbidden shortcuts: do not rename legacy continuation fields while preserving
the same removed `"thread"` behavior.

Modification boundaries: runtime callstack, progress, story state save/load
where affected, and tests.

Validation: `cargo test -p ink-runtime callstack`; `cargo test -p ink-test
--test integration choices`.

Progress ledger:

- Changed files: `crates/ink-runtime/src/callstack.rs`,
  `crates/ink-runtime/src/story/progress.rs`,
  `crates/ink-runtime/src/story/choices.rs`, choice-loop test/fixture renames.
- Validation result: passed `cargo test -p ink-runtime callstack`; passed
  `cargo test -p ink-test --test integration choices`; passed
  `cargo test -p ink-test --test integration`; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Legacy compiled-thread continuation support is
  deleted; the remaining continuation type is the current callstack/replay
  implementation detail.
- Checkpoint commits: none

### [x] Task 15: Remove Visit, Turn, Read-Count, And Sequence Runtime Fallbacks

Goal: delete runtime implementation that only supports removed count and
sequence compiled JSON commands.

Implementation method: remove control-logic branches for choice count, turn
count, turns-since, read-count, visit-index, and sequence-shuffle index. Delete
helper methods such as deterministic sequence shuffle support if they become
unused. Keep random/seed-random behavior if current tests use it.

Acceptance criteria: no runtime path pushes fallback count values for removed
commands; sequence shuffle helper code is gone unless another current feature
uses it.

Forbidden shortcuts: do not leave count commands as no-ops or fallback values.

Modification boundaries: runtime control logic, story helpers, and tests.

Validation: `cargo test -p ink-runtime control_logic`; `cargo check -p
ink-runtime`.

Progress ledger:

- Changed files: `crates/ink-runtime/src/story/control_logic.rs`,
  `crates/ink-runtime/src/story/mod.rs`,
  `crates/ink-runtime/src/variable_reference.rs`.
- Validation result: passed `cargo test -p ink-runtime control_logic`; passed
  `cargo check -p ink-runtime`; passed `cargo test -p ink-runtime`; final
  `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Removed count and sequence command fallbacks no
  longer push compatibility values.
- Checkpoint commits: none

### [x] Task 16: Remove Runtime Container Count-Flag Fields

Goal: delete runtime container fields and JSON writer behavior for old
visit/turn/count-start flags.

Implementation method: remove count flag constants, constructor parameters,
stored booleans, `get_count_flags`, and JSON writer output. Update runtime JSON
reader conversion to construct containers without count metadata.

Acceptance criteria: runtime containers no longer store visit/turn/count-start
state; JSON write never emits `#f`; path traversal and named content still
work.

Forbidden shortcuts: do not keep count fields set to false if they only exist
for removed metadata.

Modification boundaries: runtime container, JSON reader/writer, and affected
tests.

Validation: `cargo test -p ink-runtime container`; `cargo test -p ink-runtime
json_write`; `cargo test -p ink-runtime json_read`.

Progress ledger:

- Changed files: `crates/ink-runtime/src/container.rs`,
  `crates/ink-runtime/src/json/json_read.rs`,
  `crates/ink-runtime/src/json/json_write.rs`,
  `crates/ink-runtime/src/story/navigation.rs`,
  `crates/ink-runtime/src/story/mod.rs`.
- Validation result: passed `cargo test -p ink-runtime container`; passed
  `cargo test -p ink-runtime json_write`; passed `cargo test -p ink-runtime
  json_read`; passed `cargo test -p ink-runtime`; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Runtime container count metadata and its dead
  divert hook are removed.
- Checkpoint commits: none

### [x] Task 17: Update Runtime JSON Reader And Writer For Current Format

Goal: align runtime JSON conversion with the current-only format crate after
deleted commands, objects, and count flags.

Implementation method: remove conversion arms for deleted format objects and
commands, update minimal story fixtures to current JSON, and ensure malformed
old JSON reports clear load errors through the format crate or version check.

Acceptance criteria: runtime can load current compiler output, rejects old
compiled JSON, and writes current runtime object graphs without removed fields
or commands.

Forbidden shortcuts: do not duplicate old schema parsing in the runtime after
removing it from the format crate.

Modification boundaries: runtime JSON reader/writer and tests.

Validation: `cargo test -p ink-runtime json_read`; `cargo test -p ink-runtime
json_write`.

Progress ledger:

- Changed files: `crates/ink-runtime/src/json/json_read.rs`,
  `crates/ink-runtime/src/json/json_write.rs`, runtime save/load tests.
- Validation result: passed `cargo test -p ink-runtime json_read`; passed
  `cargo test -p ink-runtime json_write`; passed `cargo test -p ink-runtime`;
  final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Runtime loads current compiler output through the
  format crate and rejects removed JSON through format/version errors.
- Checkpoint commits: none

### [x] Task 18: Preserve Current Save/Load Replay Behavior Under The New Format

Goal: verify that deleting compiled JSON compatibility code does not regress
minimal save-state behavior for current stories.

Implementation method: update current save/load tests to use new compiled JSON
fixtures, then add focused static and dynamic choice save/load tests if the
existing coverage does not exercise regenerated choices, selected dynamic item
value, selected dynamic index, and aftermath execution.

Acceptance criteria: save-state JSON still excludes generated choices and
thread data; loading at a choice pause regenerates equivalent choices; selecting
a regenerated dynamic choice executes the correct aftermath.

Forbidden shortcuts: do not reintroduce serialized generated choices or thread
snapshots to fix replay regressions.

Modification boundaries: runtime story state tests, ink-test choice fixtures,
and docs only where needed for the save/load contract.

Validation: `cargo test -p ink-runtime story_state`; `cargo test -p ink-test
--test integration choices`; run dynamic choice save/load tests by name.

Progress ledger:

- Changed files: `crates/ink-runtime/src/story_state.rs`,
  `crates/ink-test/tests/choices.rs`, choice save/load fixtures.
- Validation result: passed `cargo test -p ink-runtime story_state`; passed
  `cargo test -p ink-test --test integration choices`; passed final
  `make gate`.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Save JSON continues to omit generated choices and
  removed thread/flow/count fields, and choice pauses replay after load.
- Checkpoint commits: none

### [x] Task 19: Update Runtime Error Tests For Removed Compiled JSON

Goal: convert tests that loaded historical compiled JSON into current-format
success tests or removed-format rejection tests.

Implementation method: inspect runtime tests containing removed tokens and
rewrite them. Success tests should use current JSON or compiler output.
Rejection tests should assert version mismatch, unknown command, unknown object,
or invalid container metadata as appropriate.

Acceptance criteria: runtime tests no longer use removed tokens for successful
stories; every remaining removed-token fixture is explicitly a rejection case.

Forbidden shortcuts: do not delete failure coverage just because old JSON no
longer loads.

Modification boundaries: runtime tests and small helpers.

Validation: `cargo test -p ink-runtime`.

Progress ledger:

- Changed files: runtime JSON read/write tests and runtime story-state tests.
- Validation result: passed `cargo test -p ink-runtime`; final `make gate`
  passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Runtime success tests use current JSON; removed
  JSON appears only in explicit rejection tests.
- Checkpoint commits: none

## Milestone D: Fixtures, Conformance, And Regression Coverage

### [x] Task 20: Refresh Integration Fixtures For Current Compiled JSON

Goal: update maintained integration fixture outputs to the new current-only
compiled JSON format.

Implementation method: refresh fixture snapshots after compiler and runtime
changes are in place. Review version, terminal command removal, and container
metadata diffs manually.

Acceptance criteria: fixture outputs are current-only; successful fixture
runtime behavior remains intentional; no fixture uses removed compiled JSON as
a success case.

Forbidden shortcuts: do not update expected output before confirming runtime
behavior with focused tests.

Modification boundaries: `crates/ink-test` snapshots and fixture support.

Validation: `cargo test -p ink-test --test integration`.

Progress ledger:

- Changed files: regenerated integration fixture snapshots under
  `crates/ink-test/fixtures/**/*.ink.json`, plus related parse/source fixture
  renames.
- Validation result: passed `cargo test -p ink-test --test integration`; final
  `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Successful fixtures are current-only.
- Checkpoint commits: none

### [x] Task 21: Add Fallthrough Regression Fixtures Without Terminal Commands

Goal: lock down weave behavior that previously depended on `done/end`.

Implementation method: add fixtures for root natural end, knot natural end,
stitch natural end, nested gather fallthrough, terminal gather, static choice
aftermath, dynamic choice aftermath, and mixed choice lists. Include output
assertions that would catch accidental over-fallthrough.

Acceptance criteria: new fixtures pass without removed compiled commands and
fail if a branch continues into unrelated content.

Forbidden shortcuts: do not encode expected behavior only in compiled JSON
snapshots; include runtime output assertions.

Modification boundaries: `crates/ink-test` fixtures and integration tests.

Validation: run the new integration fixtures by name; `cargo test -p ink-test
--test integration flow`; `cargo test -p ink-test --test integration choices`.

Progress ledger:

- Changed files: `crates/ink-test/tests/flow.rs`,
  `crates/ink-test/tests/choices.rs`, natural-end flow fixtures,
  `crates/ink-test/fixtures/choices/weave-fallthrough-current.ink`.
- Validation result: passed focused flow and choice integration tests; passed
  `cargo test -p ink-test --test integration`; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Regression coverage checks natural ends and
  weave fallthrough without terminal commands.
- Checkpoint commits: none

### [x] Task 22: Add Current-Only Loader Regression Fixtures

Goal: prove that old compiled JSON is not silently accepted through any public
runtime load path.

Implementation method: add small JSON fixtures or test literals for old
version, removed command token, `CNT?`, `#f`, and removed thread command
scenarios. Assert the precise class of load failure without overfitting the
full error string.

Acceptance criteria: all public runtime constructors reject removed compiled
JSON; current compiler-produced JSON loads through the same path.

Forbidden shortcuts: do not bypass the public runtime loading API in tests.

Modification boundaries: runtime and integration loader tests.

Validation: `cargo test -p ink-runtime json_read`; run new loader regression
tests by name.

Progress ledger:

- Changed files: `crates/ink-runtime/src/json/json_read.rs`,
  `crates/ink-story-json-format/src/json/tests.rs`.
- Validation result: passed `cargo test -p ink-runtime json_read`; passed
  `cargo test -p ink-story-json-format`; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Loader rejection covers old version, removed
  commands, `CNT?`, `#f`, and removed choice flag bits.
- Checkpoint commits: none

### [x] Task 23: Verify Dynamic Choice Save/Load After Format Cleanup

Goal: ensure the dynamic choice implementation remains sound after removing
legacy thread and count-command support.

Implementation method: add or strengthen fixtures that mix fixed and dynamic
choices, choose dynamic items by value and index, save at the choice pause,
load, regenerate choices, select an item, and execute aftermath. Cover nested
dynamic choices if the current language allows them.

Acceptance criteria: regenerated choices match the pre-save list; aftermath has
the correct selected value and index; save JSON contains no choice/thread
fields.

Forbidden shortcuts: do not rely on host-side cached choices to pass the test.

Modification boundaries: runtime save/load tests and ink-test dynamic choice
fixtures.

Validation: run the new dynamic choice save/load tests by name; `cargo test -p
ink-test --test integration choices`; `cargo test -p ink-runtime story_state`.

Progress ledger:

- Changed files: `crates/ink-test/tests/choices.rs`, dynamic choice fixtures,
  runtime story-state tests.
- Validation result: passed `cargo test -p ink-test --test integration
  choices`; passed `cargo test -p ink-runtime story_state`; passed final
  `make gate`.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Dynamic choice save/load regenerates choices and
  preserves selected value/index aftermath without host-side cached choices.
- Checkpoint commits: none

### [x] Task 24: Add Repository Residue Policy Coverage

Goal: make legacy compiled JSON residue visible at test time, not only during
manual review.

Implementation method: add or update policy tests that scan maintained current
fixtures, generated snapshots, docs that describe current format, and runtime
success tests for removed compiled JSON tokens. Allow explicit rejection tests,
diagnostics for removed source syntax, and finished plan history.

Acceptance criteria: policy tests fail on accidental reintroduction of removed
compiled JSON compatibility in maintained current surfaces.

Forbidden shortcuts: do not make the policy a brittle grep over every file in
the repository without classifying legitimate historical mentions.

Modification boundaries: integration policy tests and allowlist comments.

Validation: run the new policy tests by name with `cargo test -p ink-test
--test integration current_compiled_json_surfaces_reject_removed_legacy_format`.

Progress ledger:

- Changed files: `crates/ink-test/tests/integration_policy.rs`.
- Validation result: passed `cargo test -p ink-test --test integration
  current_compiled_json_surfaces_reject_removed_legacy_format`; passed final
  `make gate`.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. The policy deliberately allows rejection tests,
  removed-source diagnostics, and changelog history.
- Checkpoint commits: none

## Milestone E: Documentation And Terminology

### [x] Task 25: Rewrite The Compiled JSON Format Documentation

Goal: make `docs/ink_JSON_runtime_format.md` describe only the new current
compiled story JSON format.

Implementation method: remove historical compatibility wording, removed command
descriptions, `CNT?`, `#f`, old version statements, and examples containing
removed tokens. Update examples to the new version and current natural-ending
shape. Pair the doc change with policy coverage from Task 24 or a focused doc
residue test.

Acceptance criteria: the maintained format doc does not teach removed compiled
JSON as supported; examples load under the new current format if copied into a
test; policy coverage protects the current-only wording.

Forbidden shortcuts: do not move removed command documentation into another
current-format doc as "compatibility" guidance.

Modification boundaries: compiled JSON docs and doc policy tests.

Validation: `cargo test -p ink-test --test integration
current_compiled_json_surfaces_reject_removed_legacy_format`; run the residue
search from the plan overview and classify remaining hits.

Progress ledger:

- Changed files: `docs/ink_JSON_runtime_format.md`,
  `crates/ink-test/tests/integration_policy.rs`.
- Validation result: passed current compiled JSON policy test; residue search
  found no removed tokens in the current format doc except explicit rejection
  field names in save-state removal lists; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. The maintained format doc describes version 2 and
  current command/choice/save contracts only.
- Checkpoint commits: none

### [x] Task 26: Update Architecture, Language, And Syntax Notes

Goal: keep repository-authored documentation synchronized with the breaking
format change and current language identity.

Implementation method: update `docs/Architecture.md`, `docs/SyntaxUpdates.md`,
`docs/LanguageOverview.md`, `docs/SyntaxReference.md`, and related notes only
where they mention compiled JSON compatibility, historical command support,
thread runtime compatibility, or old count metadata. Pair doc edits with the
policy coverage that verifies maintained current docs.

Acceptance criteria: current docs say ink-rs owns its current format and does
not support legacy compiled JSON; source diagnostics for removed syntax remain
documented only as removed syntax, not compatibility.

Forbidden shortcuts: do not rewrite unrelated language sections; do not add old
Ink compatibility caveats back into the latest syntax reference.

Modification boundaries: maintained docs and doc policy tests.

Validation: `cargo test -p ink-test --test integration
current_compiled_json_surfaces_reject_removed_legacy_format`; run targeted `rg`
searches for `legacy`, `compat`, `thread`, `done`, `end`, `visit`, and `turn`
in maintained docs and classify remaining hits.

Progress ledger:

- Changed files: `docs/Architecture.md`, `docs/SyntaxUpdates.md`,
  `docs/SyntaxReference.md`, `docs/LanguageOverview.md` reviewed for current
  terminology.
- Validation result: passed current compiled JSON policy test; targeted doc
  residue search classified remaining `compat` mentions as project identity or
  anti-shim policy, and `thread` mentions as removed-field or removed-source
  diagnostics; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Current syntax docs no longer teach magic
  terminal diverts or source threads.
- Checkpoint commits: none

### [x] Task 27: Clean Runtime And Compiler Terminology

Goal: remove terminology that presents legacy thread/count/flow compatibility
as current implementation architecture.

Implementation method: update code comments, test names, helper names, and docs
affected by deleted implementation. Keep compiler-internal `Flow` terminology
only where it remains a current abstraction for knots/functions/modules, and
rename or delete runtime "thread" terminology that only referred to old
compiled `"thread"` behavior.

Acceptance criteria: no runtime success path or public doc describes ordinary
choices as threads; "flow" references that remain are current compiler/runtime
concepts, not old Ink compatibility.

Forbidden shortcuts: do not perform broad cosmetic renames unrelated to the
format break; do not leave misleading comments around deleted branches.

Modification boundaries: comments, test names, helper names, and directly
affected docs/tests.

Validation: `cargo check --workspace`; targeted `rg` terminology searches with
remaining hits classified in this task ledger.

Progress ledger:

- Changed files: runtime/compiler comments, renamed thread-oriented tests and
  fixtures to choice-loop/authored-choice terminology, `docs/Architecture.md`.
- Validation result: passed `cargo check --workspace`; targeted terminology
  search classified remaining thread hits as removed-field rejection tests,
  removed-source diagnostics, or policy token lists; final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known. Compiler/runtime `Flow` remains current parser and
  analysis terminology, not an Ink compatibility concept.
- Checkpoint commits: none

## Milestone F: Cleanup, Validation, And Closeout

### [x] Task 28: Remove Dead Code And Classify Residue

Goal: finish deleting support implementation and prove remaining mentions are
intentional.

Implementation method: run repository-wide searches for removed command names,
tokens, count metadata, old version examples, thread compatibility, and count
fallback APIs. Delete dead code, update tests, and record any intentional
remaining hits in this task ledger.

Acceptance criteria: `cargo check --workspace` passes without dead compatibility
paths; remaining residue search hits are limited to removed-source diagnostics,
rejection tests, active/finished plan history, or current terminology that is
not compatibility.

Forbidden shortcuts: do not silence warnings with `allow(dead_code)` for code
that should be deleted.

Modification boundaries: any directly affected source, tests, fixtures, and
docs needed for cleanup.

Validation: `cargo check --workspace`; `cargo test --workspace --no-run`; run
the residue search from the overview and classify hits.

Progress ledger:

- Changed files: runtime dead-code cleanup in `crates/ink-runtime/src/story`
  modules: `navigation.rs`, `mod.rs`, `progress.rs`, `choices.rs`, and
  `state.rs`; source and snapshot fixture cleanup for misleading `end` names.
- Validation result: passed `cargo check --workspace`; passed `cargo test
  --workspace --no-run`; passed residue searches. Remaining hits are limited
  to rejection tests, removed-source diagnostics, changelog history, project
  identity/anti-compatibility wording, current `EndString` commands, and normal
  prose words such as `return` or `visit`. Final `make gate` passed.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known.
- Checkpoint commits: none

### [x] Task 29: Final Gate, History Shape, And Plan Closeout

Goal: validate the full active plan and prepare one final active-plan commit.

Implementation method: run final formatting, diff checks, focused residue
searches, and `make gate`. Update all task ledgers, move this plan directory to
`docs/finished_plans/`, and combine active-plan checkpoint commits into one
final active-plan commit unless the owner asks for a different history shape.

Acceptance criteria: `make gate` passes; task progress is `29/29`; active plan
files are moved to finished plans; final commit contains implementation, tests,
fixtures, docs, and plan closeout state.

Forbidden shortcuts: do not mark the plan complete because only near-term tests
pass; do not push or consider landed with checkpoint commits still split unless
the owner explicitly approves that history shape.

Modification boundaries: final formatting, validation fixes, task ledger,
finished plan move, and git history shape.

Validation: `cargo fmt --all --check`; `git diff --check`; `make gate`;
residue searches recorded in this task ledger.

Progress ledger:

- Changed files: final formatting, task ledger, plan closeout, and move from
  `docs/active_plan/` to `docs/finished_plans/`.
- Validation result: passed `cargo fmt --all --check`; passed `git diff --check`;
  passed `cargo check --workspace`; passed `cargo test --workspace --no-run`;
  passed `cargo test -p ink-test --test integration`; passed final `make gate`.
- Owner review: final commit is ready for owner review.
- Remaining risk: none known.
- Checkpoint commits: none
