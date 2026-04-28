Progress: 26/26

# Integration Test Overhaul

This finished plan makes integration tests exercise the current module-based
language directly. Test inputs must be repository `.ink` fixtures written in the
current syntax. Integration helpers must not wrap legacy Ink snippets in modules
or synthesize story source strings. The closeout audit removed the remaining
legacy root-story production path.

Status key: `[ ]` pending, `[~]` in progress, `[>]` waiting review, `[x]`
complete, `[!]` blocked.

## Milestone 1: Test Policy And Harness

### [x] Task 01: Add integration fixture policy checks

Goal: make unsupported test shapes visible before large fixture migration starts.

Implementation method: add a focused test or helper under `crates/ink-test` that
walks integration `.ink` fixtures used by `ink-test`, verifies every runnable
fixture starts with `=== module `, and reports files that are still legacy root
stories. Include the inventory inside the failure output instead of keeping a
separate research log.

Acceptance criteria: the new check can be run directly, fails with actionable
paths while legacy fixtures remain, and has an explicit allowlist only for
fixtures whose task has not migrated yet.

Forbidden shortcuts: do not weaken the check by scanning only one directory; do
not skip a fixture just because it is hard to migrate; do not add path-specific
compiler behavior.

Modification boundaries: `crates/ink-test/tests`, `crates/ink-test/src`, and
`crates/ink-test/fixtures` policy metadata if needed.

Validation commands: `cargo test -p ink-test --test language fixture_policy`.

Validation:

- `cargo test -p ink-test --test integration_policy` passed.
- `make gate` passed.

Commit record: implementation commit `2c1625af` (`Add integration test policy checks`).

Review record: reviewed implementation commit `2c1625af` with `git show
--check`; no follow-up fixes were needed. Re-ran `cargo test -p ink-test --test
integration_policy` and `make gate`, both passed.

### [x] Task 02: Remove module-wrapping from language helpers

Goal: ensure language integration tests compile exactly the source stored in
fixtures.

Implementation method: replace `explicit_game_module`, `compile_language_source`,
and `diagnostics_for_language_source` with fixture-loading helpers that require a
`.ink` fixture path. Convert the small inline cases they currently cover into
new fixture files before removing the wrapper.

Acceptance criteria: no helper in `tests/language.rs` constructs module headers,
`main` knots, or `-> END`; all remaining language tests compile fixture text
unchanged.

Forbidden shortcuts: do not keep a hidden compatibility wrapper; do not use
inline string Ink snippets in replacement tests.

Modification boundaries: `crates/ink-test/tests/language.rs` and
`crates/ink-test/fixtures/language`.

Validation commands: `cargo test -p ink-test --test language`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 03: Convert compiler API tests to fixture-backed sources

Goal: remove direct Ink source strings from public compiler API integration
tests.

Implementation method: add fixture files for single-source, multi-source,
diagnostic, empty-input, and source-order API cases. Keep Rust test code focused
on API calls and assertions while loading all Ink text from files.

Acceptance criteria: `tests/compiler_api.rs` contains no inline Ink programs;
source filename assertions still use realistic fixture names.

Forbidden shortcuts: do not preserve old root-story snippets as strings; do not
weaken diagnostic assertions.

Modification boundaries: `crates/ink-test/tests/compiler_api.rs` and
`crates/ink-test/fixtures/language/compiler_api`.

Validation commands: `cargo test -p ink-test --test compiler_api`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 04: Put compiler API tests into the gate

Goal: make the project gate cover all maintained `ink-test` integration tests.

Implementation method: update `Makefile` so `compiler_api` runs alongside the
other `ink-test` targets. Keep C# compatibility targeted until the feature layout
is removed later in the plan.

Acceptance criteria: `make test` runs `cargo test -p ink-test --test
compiler_api`.

Forbidden shortcuts: do not replace focused `ink-test` commands with a command
that currently fails under all features; do not drop existing targets.

Modification boundaries: `Makefile`.

Validation commands: `make test`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

## Milestone 2: Rust-First Language Fixtures

### [x] Task 05: Move module and import language cases to fixtures

Goal: make module/import behavior tests read real `.ink` fixtures.

Implementation method: move the module import, qualified call, qualified path,
same-module stitch, and module global/external scenarios from inline strings
into `fixtures/language/modules/*.ink`.

Acceptance criteria: the moved cases still assert the same runtime output,
diagnostics, and save-state shape without inline Ink strings.

Forbidden shortcuts: do not generate the fixture source at runtime; do not keep a
duplicate inline source next to the fixture.

Modification boundaries: `tests/language.rs` and `fixtures/language/modules`.

Validation commands: `cargo test -p ink-test --test language module`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 06: Move choice and save-load language cases to fixtures

Goal: make choice generation and save-load behavior tests fixture-backed.

Implementation method: create `fixtures/language/choices/*.ink` for selected
choice text, repeatable choices, generated choice save/load, thread choice
save/load, and deterministic random save/load cases.

Acceptance criteria: runtime behavior assertions remain intact and no choice
scenario relies on an inline Ink source string.

Forbidden shortcuts: do not change expected behavior to match an easier fixture;
do not keep JSON-only expected output files.

Modification boundaries: `tests/language.rs` and `fixtures/language/choices`.

Validation commands: `cargo test -p ink-test --test language choice save_load`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 07: Move dynamic divert and tunnel language cases to fixtures

Goal: cover dynamic divert and tunnel behavior through current module syntax
fixtures.

Implementation method: create `fixtures/language/diverts/*.ink` for explicit
dynamic diverts, dynamic tunnels, arguments, type checks, and variable-target
diagnostics.

Acceptance criteria: tests load fixture files, diagnostics still check current
messages, and runtime behavior is unchanged.

Forbidden shortcuts: do not use legacy `-> target` root snippets; do not hide
type-check diagnostics behind broad substring checks.

Modification boundaries: `tests/language.rs` and `fixtures/language/diverts`.

Validation commands: `cargo test -p ink-test --test language divert tunnel`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 08: Move typed value mutation cases to fixtures

Goal: make struct, array, assignment, indexing, and mutation tests real
integration fixtures.

Implementation method: move inline typed cases into `fixtures/language/typed`
or narrower subdirectories, preserving module declarations in each file.

Acceptance criteria: runtime outputs for field access, index access, compound
assignment, copies, `LEN`, and `ARRAY_REMOVE` still pass.

Forbidden shortcuts: do not combine unrelated assertions into one fixture only
to reduce file count; do not use helper-generated declarations.

Modification boundaries: `tests/language.rs` and `fixtures/language/typed`.

Validation commands: `cargo test -p ink-test --test language typed`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 09: Move diagnostics-only language cases to fixtures

Goal: make syntax and type diagnostics test real invalid `.ink` files.

Implementation method: add invalid fixtures under
`fixtures/language/diagnostics/*.ink` and load them from diagnostic tests.

Acceptance criteria: every diagnostics test compiles a fixture file, preserves
line/source filename assertions where relevant, and contains no inline Ink
program strings.

Forbidden shortcuts: do not collapse diagnostics into one fixture if it hides
line ownership; do not accept vague parse failures when a specific diagnostic is
expected.

Modification boundaries: `tests/language.rs` and
`fixtures/language/diagnostics`.

Validation commands: `cargo test -p ink-test --test language diagnostic`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

## Milestone 3: Ported Inkling Examples

### [x] Task 10: Convert Inkling happy-path examples to module fixtures

Goal: stop compiling ported Inkling examples from inline legacy snippets.

Implementation method: create `.ink` fixtures under `fixtures/language/inkling`
for the runnable examples and update `tests/inkling_examples.rs` to load them
without source rewriting.

Acceptance criteria: happy-path Inkling tests contain no inline Ink source and
still assert the same runtime output.

Forbidden shortcuts: do not keep `explicit_game_module`; do not write a generic
fixture generator.

Modification boundaries: `tests/inkling_examples.rs` and
`fixtures/language/inkling`.

Validation commands: `cargo test -p ink-test --test inkling_examples`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 11: Convert Inkling diagnostic examples to module fixtures

Goal: keep Inkling-derived error coverage while using real invalid module
fixtures.

Implementation method: move address validation, condition address, and
alternative sequence diagnostic snippets into fixture files with clear names.

Acceptance criteria: diagnostics still assert actionable messages and no
diagnostic test constructs source strings.

Forbidden shortcuts: do not remove diagnostics simply because they need fixture
files; do not replace specific assertions with only `has_errors`.

Modification boundaries: `tests/inkling_examples.rs` and
`fixtures/language/inkling/diagnostics`.

Validation commands: `cargo test -p ink-test --test inkling_examples`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

## Milestone 4: Compiler Conformance Fixtures

### [x] Task 12: Migrate basic text, knot, and divert compiler fixtures

Goal: convert the first compiler conformance fixtures from root story syntax to
explicit module syntax.

Implementation method: update the `.ink` files in basic text, knot, and divert
categories to include `=== module game ===` and `== main ==` as appropriate;
regenerate and review parse and JSON snapshots.

Acceptance criteria: fixture source, parse snapshot, and JSON snapshot all match
current module output.

Forbidden shortcuts: do not change compiler output to preserve old snapshots; do
not keep duplicate legacy fixtures.

Modification boundaries: `fixtures/conformance/inkfiles/{basictext,knot,divert}`
and matching `.parse`/`.json` files.

Validation commands: `cargo test -p ink-test --test compiler_conformance
basictext`, `cargo test -p ink-test --test compiler_conformance divert`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 13: Migrate glue and conditional compiler fixtures

Goal: convert glue and conditional compiler fixtures to current module syntax.

Implementation method: update source fixtures, regenerate snapshots, and compare
runtime text behavior where the fixture also has runtime coverage.

Acceptance criteria: module fixtures compile cleanly and snapshots represent the
new module-scoped JSON structure.

Forbidden shortcuts: do not special-case glue or conditional lowering for the
old root path.

Modification boundaries:
`fixtures/conformance/inkfiles/{glue,conditional}` and matching snapshots.

Validation commands: `cargo test -p ink-test --test compiler_conformance glue`,
`cargo test -p ink-test --test compiler_conformance conditional`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 14: Migrate function and variable compiler fixtures

Goal: convert function and variable compiler conformance fixtures to current
module syntax.

Implementation method: place declarations inside `=== module game ===`, keep
functions and entry flow under module scope, regenerate parse and JSON snapshots.

Acceptance criteria: snapshots pass and variable/function behavior remains
covered by runtime integration tests.

Forbidden shortcuts: do not leave global declaration fixtures at root scope; do
not change expected semantics for convenience.

Modification boundaries:
`fixtures/conformance/inkfiles/{function,variable}` and matching snapshots.

Validation commands: `cargo test -p ink-test --test compiler_conformance
function`, `cargo test -p ink-test --test compiler_conformance variable`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 15: Migrate runtime, tags, misc, tunnels, and typed compiler fixtures

Goal: finish converting compiler conformance `.ink` fixtures to explicit module
syntax.

Implementation method: update the remaining compiler conformance fixture
categories and regenerate snapshots, preserving `count_all_visits` options where
used.

Acceptance criteria: no compiler conformance `.ink` fixture starts as a legacy
root story.

Forbidden shortcuts: do not remove tests to avoid snapshot updates; do not
disable `count_all_visits` cases.

Modification boundaries:
`fixtures/conformance/inkfiles/{runtime,tags,misc,tunnels,typed}` and matching
snapshots.

Validation commands: `cargo test -p ink-test --test compiler_conformance`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

## Milestone 5: Runtime Conformance From Source

### [x] Task 16: Add compile-then-run runtime conformance helper

Goal: make runtime conformance tests load compiled JSON produced from module
fixtures.

Implementation method: replace `get_json_string` usage with a helper that loads
the matching `.ink` file, compiles it, and constructs the runtime story from the
generated JSON. Keep a separate raw-JSON helper only for tests whose purpose is
compiled JSON reader compatibility, and name that purpose explicitly.

Acceptance criteria: runtime conformance tests default to `.ink` source input.

Forbidden shortcuts: do not silently fall back to old `.ink.json` fixtures; do
not generate module wrappers around source files.

Modification boundaries: `tests/conformance/common.rs`,
`tests/conformance/api.rs`, and runtime conformance test modules.

Validation commands: `cargo test -p ink-test --test conformance`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 17: Add source fixtures for JSON-only choice and conditional cases

Goal: replace choice and conditional JSON-only runtime coverage with source
fixtures.

Implementation method: write module `.ink` fixtures for each JSON-only case in
choices and conditional categories, then point tests at compile-then-run helper.

Acceptance criteria: those runtime tests no longer read `.ink.json` fixtures
directly.

Forbidden shortcuts: do not delete coverage for choices that are not yet
implemented by the compiler; file a blocked task only if compiler behavior is
genuinely missing.

Modification boundaries: `fixtures/conformance/inkfiles/{choices,conditional}`
and related conformance tests.

Validation commands: `cargo test -p ink-test --test conformance choice
conditional`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 18: Add source fixtures for JSON-only gather, stitch, thread, and tags cases

Goal: replace structural runtime JSON-only cases with current source fixtures.

Implementation method: write module fixtures and update tests for gather,
stitch, thread, and tag cases, preserving choice paths and save behavior.

Acceptance criteria: runtime tests compile current source for these categories.

Forbidden shortcuts: do not hide unsupported compiler behavior with checked-in
JSON; do not narrow fixtures to only the text output if the structural feature is
the point of the test.

Modification boundaries:
`fixtures/conformance/inkfiles/{gather,stitch,threads,tags}` and related tests.

Validation commands: `cargo test -p ink-test --test conformance gather stitch
thread tag`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 19: Add source fixtures for JSON-only runtime and variable API cases

Goal: replace runtime API JSON-only fixtures with current source fixtures.

Implementation method: write module fixtures for load-save, set/get variables,
variable observers, variable diverts, and string increment behavior; update
tests to compile them.

Acceptance criteria: runtime API tests exercise compiler output and current
module syntax.

Forbidden shortcuts: do not keep old JSON as the primary input; do not change
public runtime API assertions.

Modification boundaries:
`fixtures/conformance/inkfiles/{runtime,variable}` and related tests.

Validation commands: `cargo test -p ink-test --test conformance runtime
variable`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 20: Remove obsolete raw compiled JSON fixtures from integration coverage

Goal: ensure integration tests no longer rely on checked-in compiled-story JSON
as the primary story source.

Implementation method: delete `.ink.json` files that are superseded by source
fixtures, or move remaining raw JSON reader coverage to a clearly named format
compatibility test location.

Acceptance criteria: `tests/conformance` does not use old `.ink.json` as a
normal integration input.

Forbidden shortcuts: do not delete JSON reader compatibility coverage if it is
the only test for deserialization; move it to a purpose-specific test instead.

Modification boundaries: `fixtures/conformance`, `tests/conformance`, and
format/runtime compatibility tests if needed.

Validation commands: `cargo test -p ink-test --test conformance`, focused
runtime JSON reader tests.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

## Milestone 6: C# Compatibility Suite Retirement

### [x] Task 21: Classify or migrate C# compatibility coverage

Goal: make the old C# compatibility test suite stop being an opaque legacy
source dependency.

Implementation method: for each retained scenario in `tests/csharp_tests`, move
the source into module `.ink` fixtures or map it to an existing migrated
integration fixture. Record intentionally removed upstream behavior in test
names and docs instead of in a large exclusion list.

Acceptance criteria: no retained C# scenario compiles inline legacy source.

Forbidden shortcuts: do not preserve upstream behavior that conflicts with
current language direction; do not hide missing scenarios behind an exclusion
list without an explicit reason.

Modification boundaries: `tests/csharp_tests`, `fixtures/csharp_tests`, and
`fixtures/language`.

Validation commands: `cargo test -p ink-test --features csharp-tests --test
csharp_tests`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 22: Remove the `csharp-tests` feature and API cfg split

Goal: stop feature flags from changing the public test API shape.

Implementation method: after migrated C# coverage no longer needs the special
adapter, delete the `csharp-tests` feature, remove cfg-gated methods from
`tests/conformance/api.rs`, and update gate commands.

Acceptance criteria: `cargo test -p ink-test --all-features` is no longer a
failing hidden path, or there are no `ink-test` features left to activate.

Forbidden shortcuts: do not leave cfg-gated API holes; do not rely on
`--test csharp_tests` to hide feature conflicts.

Modification boundaries: `crates/ink-test/Cargo.toml`, `tests/conformance/api.rs`,
`tests/csharp_tests`, and `Makefile`.

Validation commands: `cargo test -p ink-test`, `cargo test -p ink-test
--all-features`, `make gate`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

## Milestone 7: Delete Legacy Compiler Source Support

### [x] Task 23: Reject non-module source at parser entry

Goal: make current syntax require explicit modules for compiler input.

Implementation method: change parser/compiler entry behavior so a source file
without `=== module name ===` emits a clear diagnostic instead of building root
weave content and root flows.

Acceptance criteria: legacy root story snippets fail with a precise diagnostic,
and all integration fixtures still pass because they use module syntax.

Forbidden shortcuts: do not keep a compiler option that re-enables legacy root
parsing for tests; do not auto-wrap input.

Modification boundaries: `crates/ink-compiler/src/syntax`,
`crates/ink-compiler/src/compiler.rs`, and related tests.

Validation commands: `cargo test -p ink-compiler`, `cargo test -p ink-test`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 24: Remove root-story lowering and analysis branches

Goal: delete dead compiler code that only exists for legacy root stories.

Implementation method: remove branches that analyze, index, resolve, or lower
root weave/flows as user-authored source, while preserving the runtime JSON root
container generated for module programs.

Acceptance criteria: compiler tests pass without root-story analysis/lowering
support and no production path accepts legacy root story input.

Forbidden shortcuts: do not delete runtime root container generation; do not
conflate compiled JSON root with source root-story syntax.

Modification boundaries: `crates/ink-compiler/src/analysis`,
`crates/ink-compiler/src/lower`, and `crates/ink-compiler/src/parsed` only as
needed.

Validation commands: `cargo test -p ink-compiler`, `cargo test -p ink-test
--test compiler_conformance`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 25: Remove legacy documentation and test terminology

Goal: make docs and tests describe module syntax as the maintained language.

Implementation method: update architecture and writing guide references that
still describe legacy root stories as supported, and remove obsolete compatibility
language from tests.

Acceptance criteria: docs clearly state explicit modules are required for source
compilation; upstream legacy behavior is framed only as historical context.

Forbidden shortcuts: do not edit `docs/WritingWithInk-origin.md`; do not leave
contradictory guidance in `Notes.md` or test README files.

Modification boundaries: `docs/Architecture.md`,
`docs/WritingWithInk-updates.md`, `docs/WritingWithInk-latest.md`,
`crates/ink-test/fixtures/language/README.md`, and `Notes.md` if needed.

Validation commands: `cargo test -p ink-test --test language`, `make gate`.

Commit record: completed during integration-test-overhaul implementation; audited and validated by implementation commit `6dd83974e4c7172f59fd1d695a318bbb33056b56`.

### [x] Task 26: Close out the integration-test overhaul plan

Goal: finish the active plan after implementation and validation are complete.

Implementation method: verify no integration test constructs Ink source strings
or wraps legacy snippets, run focused validations and `make gate`, then move the
plan directory to `docs/finished_plans`.

Acceptance criteria: `Progress: 26/26`, all tasks are complete, and the plan is
archived under `docs/finished_plans`.

Forbidden shortcuts: do not close the plan while any legacy source path remains
accepted or any test still requires wrapper-generated source.

Modification boundaries: `docs/active_plan/integration-test-overhaul` and
`docs/finished_plans/integration-test-overhaul`.

Validation commands: `cargo fmt --all --check`, `cargo check --workspace`,
`cargo test --workspace`, `make gate`.

Validation:

- `cargo test -p ink-compiler` passed.
- `cargo test -p ink-test` passed.
- `cargo fmt --all --check` passed.
- `cargo check --workspace` passed.
- `cargo test --workspace` passed.
- `make gate` passed.

Commit record: implementation audit and remaining fixes landed in
`6dd83974e4c7172f59fd1d695a318bbb33056b56`; the plan archive is recorded by the
closeout commit.
