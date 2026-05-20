Progress: 12/24

# Compiler Analysis Refactor Active Plan

This plan refactors compiler analysis internals without changing ink-rs language
semantics, compiled story JSON, lowering output, or runtime behavior. The work
is split into reviewable implementation tasks. Each implementation task must be
validated with its focused commands and `make gate` before it can move to `[>]`.
Completion records are committed separately from implementation commits.

## Global Constraints

- Preserve all existing accepted and rejected language behavior unless a task
  explicitly identifies a diagnostic-span-only improvement.
- Do not edit `docs/WritingWithInk.md`.
- Do not hardcode fixture names, paths, JSON fragments, runtime paths, or
  snapshot strings to satisfy tests.
- Do not start the next implementation task until the previous task reaches
  `[x]` and its completion-record commit exists.
- If a task requires broad parser span rewrites, defer that part to
  `docs/issues_found/` instead of expanding this refactor.
- Keep unrelated working-tree edits out of this plan.

## Milestone 1: Shared Analysis Inputs

### [x] Task 01: Introduce shared analysis index container

Goal: Add a pass-local shared index container for compiler analysis so later
tasks can reuse the same analysis inputs instead of rebuilding them per pass.

Implementation method: Add an internal `AnalysisIndexes` type under
`crates/ink-compiler/src/analysis/` that owns or references the existing
variable scope, struct, enum, target symbol, interface member, module import,
and module implementation indexes. Build it once from `Story` and existing
module analysis data in `run_analysis_passes_with_modules`. Initially keep
existing diagnostics pass APIs working by adding overloads or internal helper
entry points instead of migrating every pass in this task.

Acceptance criteria: Existing analysis tests pass; `AnalysisIndexes` has no
dependency on lowering or emit modules; no diagnostic order or message changes
are introduced.

Forbidden shortcuts: Do not remove existing pass-local builders before all
callers are migrated. Do not store mutable diagnostic state in the shared index
container.

Modification boundaries: `crates/ink-compiler/src/analysis/mod.rs` and a new or
adjacent analysis support module only.

Validation commands: `cargo test -p ink-compiler analysis`; `make gate`.

Validation record: `cargo test -p ink-compiler analysis` passed;
`make gate` passed. Review validation repeated after implementation review:
`cargo test -p ink-compiler analysis` passed; `make gate` passed.

Review record: Reviewed implementation commit `d250386c` with `git show --check`;
no follow-up changes required.

Commit record: Implementation commit `d250386c`; waiting-review record
`598d1f60`; completion-record commit recorded by this task-list update.

### [x] Task 02: Migrate target diagnostics to shared indexes

Goal: Make call target diagnostics consume `AnalysisIndexes` instead of
rebuilding target, variable, struct, enum, interface, module import, module
implementation, and interface-module-literal-use inputs.

Implementation method: Add an indexed entry point for target diagnostics and
route `run_analysis_passes_with_modules` through it. Preserve the public or
test-only wrapper by having it build `AnalysisIndexes` locally for isolated
tests. Keep the `CallTargetChecker` logic behavior-equivalent.

Acceptance criteria: Target diagnostics still report the same errors and
warnings for missing targets, wrong function/knot usage, static and dynamic
arguments, interface module literals, and unresolved variables.

Forbidden shortcuts: Do not relax unresolved-variable checks. Do not bypass
interface module literal use collection.

Modification boundaries: Target diagnostics and shared analysis wiring only.

Validation commands: `cargo test -p ink-compiler targets`; `cargo test -p ink-compiler analysis`; `make gate`.

Validation record: `cargo test -p ink-compiler targets` passed;
`cargo test -p ink-compiler analysis` passed; `make gate` passed. Review
validation repeated after implementation review: `cargo test -p ink-compiler
targets` passed; `cargo test -p ink-compiler analysis` passed; `make gate`
passed.

Review record: Reviewed implementation commit `39d762cc` with `git show --check`;
no follow-up changes required.

Commit record: Implementation commit `39d762cc`; waiting-review record
`b35af4dc`; completion-record commit recorded by this task-list update.

### [x] Task 03: Migrate initializer and assignment diagnostics

Goal: Make initializer and assignment diagnostics consume `AnalysisIndexes`.

Implementation method: Change variable initializer and variable assignment
diagnostics to accept shared indexes from analysis orchestration while keeping
test wrappers available. Reuse the same references for interface module literal
typing and primitive/enum assignment typing.

Acceptance criteria: Initializer and reassignment diagnostics remain identical
except for any ordering already guaranteed by existing sort behavior. Interface
module literal initializers and assignment target type resolution still work.

Forbidden shortcuts: Do not merge initializer and assignment passes in this
task. Do not weaken constant, global variable, local assignment, field, or index
assignment diagnostics.

Modification boundaries: Initializer diagnostics, assignment diagnostics, and
analysis pass orchestration.

Validation commands: `cargo test -p ink-compiler initializers assignments`; `cargo test -p ink-compiler analysis`; `make gate`.

Validation record: `cargo test -p ink-compiler initializers` passed;
`cargo test -p ink-compiler assignments` passed; `cargo test -p ink-compiler
analysis` passed; `make gate` passed. Review validation repeated after
implementation review: `cargo test -p ink-compiler initializers` passed;
`cargo test -p ink-compiler assignments` passed; `cargo test -p ink-compiler
analysis` passed; `make gate` passed.

Review record: Reviewed implementation commit `60ace477` with `git show --check`;
no follow-up changes required.

Commit record: Implementation commit `60ace477`; waiting-review record
`0d326822`; completion-record commit recorded by this task-list update.

### [x] Task 04: Migrate field/index/flow diagnostics

Goal: Make field access, index access, and flow diagnostics consume shared
analysis indexes where they currently rebuild them.

Implementation method: Add indexed entry points for the three passes. Leave
their internal checker behavior intact while replacing local index construction
with references from `AnalysisIndexes`.

Acceptance criteria: Flow shape diagnostics, condition type diagnostics, field
access diagnostics, and index access diagnostics remain behavior-equivalent.

Forbidden shortcuts: Do not change flow ordering rules, condition type rules,
or index lvalue semantics.

Modification boundaries: Field access, index access, flow diagnostics, and
analysis pass orchestration.

Validation commands: `cargo test -p ink-compiler flow field_access index_access`; `cargo test -p ink-compiler analysis`; `make gate`.

Validation record: `cargo test -p ink-compiler flow` passed; `cargo test -p
ink-compiler field_access` passed; `cargo test -p ink-compiler index_access`
passed; `cargo test -p ink-compiler analysis` passed; `make gate` passed.
Review validation repeated after implementation review: `cargo test -p
ink-compiler flow` passed; `cargo test -p ink-compiler field_access` passed;
`cargo test -p ink-compiler index_access` passed; `cargo test -p ink-compiler
analysis` passed; `make gate` passed.

Review record: Reviewed implementation commit `bf229a49` with `git show --check`;
no follow-up changes required.

Commit record: Implementation commit `bf229a49`; waiting-review record
`cbc5a4fc`; completion-record commit recorded by this task-list update.

### [x] Task 05: Migrate literal diagnostics

Goal: Make array, dict, and struct literal diagnostics consume shared analysis
indexes.

Implementation method: Add indexed entry points for literal diagnostics and
wire analysis orchestration through those entry points. Keep existing isolated
test wrappers by building shared indexes inside the wrapper.

Acceptance criteria: Array, dict, and struct literal diagnostics remain
behavior-equivalent, including nested literals and interface module literals in
literal values.

Forbidden shortcuts: Do not combine literal checker logic in this task. Do not
drop expected-expression tracking yet.

Modification boundaries: Literal diagnostics and analysis pass orchestration.

Validation commands: `cargo test -p ink-compiler array_literals dict_literals struct_literals`; `cargo test -p ink-compiler analysis`; `make gate`.

Validation record: `cargo test -p ink-compiler array_literals` passed; `cargo
test -p ink-compiler dict_literals` passed; `cargo test -p ink-compiler
struct_literals` passed; `cargo test -p ink-compiler analysis` passed; `make
gate` passed. Review validation repeated after implementation review: `cargo
test -p ink-compiler array_literals` passed; `cargo test -p ink-compiler
dict_literals` passed; `cargo test -p ink-compiler struct_literals` passed;
`cargo test -p ink-compiler analysis` passed; `make gate` passed.

Review record: Reviewed implementation commit `53690a65` with `git show
--check`; no follow-up changes required.

Commit record: Implementation commit `53690a65`; waiting-review record
`66d152f8`; completion-record commit recorded by this task-list update.

### [x] Task 06: Remove obsolete per-pass index construction

Goal: Delete now-unused repeated index construction from migrated analysis
passes.

Implementation method: Remove stale `build_*_index(story)` calls and imports
from passes that now receive `AnalysisIndexes`. Keep builders that remain needed
for isolated test wrappers or unrelated modules.

Acceptance criteria: No migrated pass rebuilds indexes during the main analysis
pipeline. Isolated tests still compile and pass.

Forbidden shortcuts: Do not remove index builders that are still part of the
module public API or test support.

Modification boundaries: Analysis pass entry points and imports only.

Validation commands: `cargo test -p ink-compiler analysis`; `make gate`.

Validation record: `cargo test -p ink-compiler analysis` passed; `make gate`
passed. Review validation repeated after implementation review: `cargo test -p
ink-compiler analysis` passed; `make gate` passed.

Review record: Reviewed implementation commit `509a5586` with `git show
--check`; no follow-up changes required.

Commit record: Implementation commit `509a5586`; waiting-review record
`18ded162`; completion-record commit recorded by this task-list update.

## Milestone 2: Expected-Type Checking

### [x] Task 07: Extract expected interface expression inference wrapper

Goal: Centralize the repeated call pattern around
`infer_expected_interface_expression_type` and fallback `infer_expression_type`.

Implementation method: Add a helper that takes an expression, expected type,
shared indexes, current module, and current flow path, then returns the inferred
actual type or the existing interface-value style error. Keep bare
`VariableReference`, visible-variable precedence, import requirements, and
module implementation checks unchanged.

Acceptance criteria: All existing interface module literal tests pass with the
same messages. The helper is usable by initializers, assignments, targets, and
literal checkers.

Forbidden shortcuts: Do not treat qualified references as module literals. Do
not change dynamic interface argument behavior.

Modification boundaries: Interface value analysis helper and call sites touched
only to adopt the wrapper.

Validation commands: `cargo test -p ink-compiler interface_module`; `cargo test -p ink-compiler targets`; `make gate`.

Validation record: `cargo test -p ink-compiler interface_module` passed; `cargo
test -p ink-compiler targets` passed; `make gate` passed. Review validation
repeated after implementation review: `cargo test -p ink-compiler
interface_module` passed; `cargo test -p ink-compiler targets` passed; `make
gate` passed.

Review record: Reviewed implementation commit `4b6a2db5` with `git show
--check`; no follow-up changes required.

Commit record: Implementation commit `4b6a2db5`; waiting-review record
`88632b6a`; completion-record commit recorded by this task-list update.

### [x] Task 08: Centralize exact expected-type diagnostics

Goal: Reduce duplicated exact expected-type comparison and diagnostic formatting
without changing messages.

Implementation method: Add a small helper for "infer against expected type,
compare actual to expected, and report with caller-provided context formatter".
Callers keep ownership of their diagnostic message prefixes so public messages
remain stable.

Acceptance criteria: Existing initializer, assignment, target, array, dict, and
struct literal diagnostics remain text-compatible.

Forbidden shortcuts: Do not replace caller-specific diagnostic messages with a
generic message. Do not suppress inference errors for primitive, enum, dict, or
array contexts that currently report them.

Modification boundaries: Shared expected-type helper and minimal call-site
rewiring.

Validation commands: `cargo test -p ink-compiler analysis`; `make gate`.

Validation record: `cargo test -p ink-compiler analysis` passed; `make gate`
passed. Review validation repeated after implementation review: `cargo test -p
ink-compiler analysis` passed; `make gate` passed.

Review record: Reviewed implementation commit `5fde59e2` with `git show
--check`; no follow-up changes required.

Commit record: Implementation commit `5fde59e2`; waiting-review record
`79561b66`; completion-record commit recorded by this task-list update.

### [x] Task 09: Migrate initializers and assignments to shared checker

Goal: Make initializer and assignment type checks use the centralized
expected-type checker.

Implementation method: Replace duplicated interface-literal-first inference and
fallback expression inference in initializer and simple assignment paths.
Compound assignment stays on its numeric/string operator-specific path.

Acceptance criteria: Global, temp, constant, reassignment, field assignment,
index assignment, primitive, enum, and interface module literal tests pass.

Forbidden shortcuts: Do not change compound assignment behavior. Do not alter
default-initialization diagnostics.

Modification boundaries: Initializer and assignment type checking only.

Validation commands: `cargo test -p ink-compiler initializers assignments interface_module`; `make gate`.

Validation record: `cargo test -p ink-compiler initializers` passed; `cargo
test -p ink-compiler assignments` passed; `cargo test -p ink-compiler
interface_module` passed; `make gate` passed.

Review record: Initializer and simple assignment shared-checker migration was
covered by Task 07 implementation commit `4b6a2db5` and Task 08 implementation
commit `5fde59e2`; no additional code changes were required for this task.

Commit record: Implementation covered by commits `4b6a2db5` and `5fde59e2`;
completion-record commit recorded by this task-list update.

### [x] Task 10: Migrate static function and flow arguments

Goal: Make static function calls and static divert/tunnel arguments use the
centralized expected-type checker.

Implementation method: Replace duplicated per-argument inference in target
checking with the shared checker while preserving arity diagnostics, parameter
names, qualified-module type qualification, and composite literal handling.

Acceptance criteria: Static function and static flow argument diagnostics are
unchanged. Interface module literal arguments keep working in static argument
positions.

Forbidden shortcuts: Do not change builtin function behavior. Do not change
static divert target validation.

Modification boundaries: Target argument type checking only.

Validation commands: `cargo test -p ink-compiler targets interface_module`; `cargo test -p ink-test --test integration typed_values`; `make gate`.

Validation record: `cargo test -p ink-compiler targets` passed; `cargo test -p
ink-compiler interface_module` passed; `cargo test -p ink-test --test
integration typed_values` passed; `make gate` passed.

Review record: Static function and flow argument migration was covered by Task
07 implementation commit `4b6a2db5` and Task 08 implementation commit
`5fde59e2`; no additional code changes were required for this task.

Commit record: Implementation covered by commits `4b6a2db5` and `5fde59e2`;
completion-record commit recorded by this task-list update.

### [x] Task 11: Migrate dynamic interface member arguments

Goal: Make dynamic interface knot and function member argument checking use the
centralized expected-type checker.

Implementation method: Replace duplicated dynamic interface argument inference
with the shared helper while preserving member-kind diagnostics and the existing
dynamic interface argument behavior.

Acceptance criteria: Dynamic interface target/function tests pass. Static and
dynamic interface argument behavior remains equivalent where expected types are
the same.

Forbidden shortcuts: Do not widen accepted dynamic interface syntax. Do not
change member-kind mismatch messages.

Modification boundaries: Dynamic interface argument checking only.

Validation commands: `cargo test -p ink-compiler targets interface_module`; `cargo test -p ink-test --test integration typed_values`; `make gate`.

Validation record: `cargo test -p ink-compiler targets` passed; `cargo test -p
ink-compiler interface_module` passed; `cargo test -p ink-test --test
integration typed_values` passed; `make gate` passed.

Review record: Dynamic interface member argument migration was covered by Task
07 implementation commit `4b6a2db5` and Task 08 implementation commit
`5fde59e2`; no additional code changes were required for this task.

Commit record: Implementation covered by commits `4b6a2db5` and `5fde59e2`;
completion-record commit recorded by this task-list update.

### [x] Task 12: Migrate nested literal value checking

Goal: Make nested array, dict, and struct literal value checks use the
centralized expected-type checker for non-literal leaf expressions.

Implementation method: Replace duplicate exact-expression inference in literal
checkers with the shared helper. Keep literal-specific validation for array
shape, dict key consistency, duplicate keys, struct field existence, duplicate
fields, and missing required fields local to each checker.

Acceptance criteria: Nested literal diagnostics and interface module literals
inside literals remain correct.

Forbidden shortcuts: Do not collapse array, dict, and struct literal semantics
into a single generic validator if doing so changes error messages.

Modification boundaries: Literal non-literal leaf type checking only.

Validation commands: `cargo test -p ink-compiler array_literals dict_literals struct_literals interface_module`; `make gate`.

Validation record: `cargo test -p ink-compiler array_literals` passed; `cargo
test -p ink-compiler dict_literals` passed; `cargo test -p ink-compiler
struct_literals` passed; `cargo test -p ink-compiler interface_module` passed;
`make gate` passed.

Review record: Nested literal non-literal leaf checking migration was covered
by Task 07 implementation commit `4b6a2db5` and Task 08 implementation commit
`5fde59e2`; no additional code changes were required for this task.

Commit record: Implementation covered by commits `4b6a2db5` and `5fde59e2`;
completion-record commit recorded by this task-list update.

## Milestone 3: Argument Dispatch Consolidation

### [>] Task 13: Extract static target argument resolution helper

Goal: Remove repeated static divert/tunnel target argument resolution logic.

Implementation method: Add a shared helper that resolves a `DivertTarget::Path`
or `DivertTarget::QualifiedPath` to a flow symbol and qualified expected
parameter types. The helper should not emit diagnostics itself unless the
caller already did so at that layer.

Acceptance criteria: Array, dict, struct, interface literal use collection, and
target checkers can all reuse the same static target argument resolution.

Forbidden shortcuts: Do not change target-not-found diagnostics or
cross-module stitch diagnostics.

Modification boundaries: Shared argument resolution helper and one pilot caller.

Validation commands: `cargo test -p ink-compiler targets array_literals`; `make gate`.

Validation record: `cargo test -p ink-compiler targets` passed; `cargo test -p
ink-compiler array_literals` passed; `make gate` passed.

Commit record: Implementation commit `11d3f854`; completion-record commit TBD.

### [ ] Task 14: Extract function-call argument resolution helper

Goal: Remove repeated function call argument resolution logic.

Implementation method: Add a shared helper that resolves plain and qualified
Ink function calls to flow symbols and expected parameter types. Preserve
builtin and runtime builtin handling in target diagnostics.

Acceptance criteria: Function-call arguments in target and literal checkers use
the same parameter type qualification behavior.

Forbidden shortcuts: Do not route runtime builtins through target symbol
resolution. Do not change non-function target diagnostics.

Modification boundaries: Shared helper and function-call argument callers.

Validation commands: `cargo test -p ink-compiler targets array_literals dict_literals struct_literals`; `make gate`.

Commit record: Implementation commit TBD; completion-record commit TBD.

### [ ] Task 15: Extract dynamic interface signature helper

Goal: Replace duplicated dynamic interface signature lookup code.

Implementation method: Add a helper that infers the target interface type,
looks up the member signature, checks expected member kind, and returns the
signature. Provide a diagnostic-emitting mode for target diagnostics and a
silent mode for collector/literal passes that currently ignore unresolved
dynamic signatures.

Acceptance criteria: Existing dynamic interface diagnostics remain unchanged in
target checks. Literal and interface module literal collectors keep silently
skipping unresolved dynamic signatures.

Forbidden shortcuts: Do not make silent callers emit new diagnostics. Do not
make target callers silently ignore errors.

Modification boundaries: Dynamic interface signature lookup only.

Validation commands: `cargo test -p ink-compiler targets interface_module array_literals dict_literals struct_literals`; `make gate`.

Commit record: Implementation commit TBD; completion-record commit TBD.

### [ ] Task 16: Replace duplicated dispatch in array literal checker

Goal: Use shared argument dispatch helpers in the array literal checker.

Implementation method: Replace array checker local implementations of static
target argument lookup, function-call argument lookup, and dynamic interface
signature lookup with the helpers from Tasks 13-15. Keep array-specific
checking functions intact.

Acceptance criteria: Array literal tests pass unchanged.

Forbidden shortcuts: Do not remove array-specific diagnostics for untyped array
literals or element mismatches.

Modification boundaries: Array literal checker dispatch code only.

Validation commands: `cargo test -p ink-compiler array_literals`; `make gate`.

Commit record: Implementation commit TBD; completion-record commit TBD.

### [ ] Task 17: Replace duplicated dispatch in dict literal checker

Goal: Use shared argument dispatch helpers in the dict literal checker.

Implementation method: Replace dict checker local implementations of static
target argument lookup, function-call argument lookup, and dynamic interface
signature lookup with the shared helpers. Keep dict-specific filtering based on
expected dict types and dict literal presence.

Acceptance criteria: Dict literal tests pass unchanged.

Forbidden shortcuts: Do not change duplicate key, mixed key type, or expected
Dict diagnostics.

Modification boundaries: Dict literal checker dispatch code only.

Validation commands: `cargo test -p ink-compiler dict_literals`; `make gate`.

Commit record: Implementation commit TBD; completion-record commit TBD.

### [ ] Task 18: Replace duplicated dispatch in struct literal checker

Goal: Use shared argument dispatch helpers in the struct literal checker.

Implementation method: Replace struct checker local implementations of static
target argument lookup, function-call argument lookup, and dynamic interface
signature lookup with the shared helpers. Keep struct-specific literal
validation local.

Acceptance criteria: Struct literal tests pass unchanged.

Forbidden shortcuts: Do not change duplicate field, unknown field, or missing
required field diagnostics.

Modification boundaries: Struct literal checker dispatch code only.

Validation commands: `cargo test -p ink-compiler struct_literals`; `make gate`.

Commit record: Implementation commit TBD; completion-record commit TBD.

## Milestone 4: Literal Visitor State And Spans

### [ ] Task 19: Replace or contain expected-expression pointer tracking

Goal: Reduce reliance on raw pointer-derived expression ids for literal
expected-expression tracking.

Implementation method: First try to replace pointer-id tracking with explicit
visitor state that records when traversal is entering an expected expression. If
that would require broad visitor changes, contain the pointer-id logic in a
single small helper type with clear ownership and tests.

Acceptance criteria: Untyped standalone array, dict, and struct literal
diagnostics still fire exactly where they should; expected literals in
initializers, assignments, function arguments, dynamic interface arguments, and
nested literal values do not produce false positives.

Forbidden shortcuts: Do not disable standalone literal diagnostics. Do not rely
on fixture-specific suppression.

Modification boundaries: Literal checker expected-expression state only.

Validation commands: `cargo test -p ink-compiler array_literals dict_literals struct_literals`; `make gate`.

Commit record: Implementation commit TBD; completion-record commit TBD.

### [ ] Task 20: Pass real spans through function-call literal checks

Goal: Replace fallback `SourceSpan::new(None, 1, 1)` in literal checker
function-call paths where a real containing object span is available.

Implementation method: Thread object spans from visitor object handling into
expression traversal state for array, dict, and struct literal checkers. Use
the nearest containing object span when expression nodes do not carry their own
span.

Acceptance criteria: New or updated diagnostics tests show function-call
argument literal errors point at the containing object instead of line 1 column
1 where that object span is available.

Forbidden shortcuts: Do not add parser-wide expression spans in this task. Do
not change diagnostic messages beyond line/column/source filename.

Modification boundaries: Literal checker span plumbing and tests.

Validation commands: `cargo test -p ink-compiler array_literals dict_literals struct_literals`; `make gate`.

Commit record: Implementation commit TBD; completion-record commit TBD.

### [ ] Task 21: Pass real spans through qualified import-use collection where available

Goal: Improve qualified import-use diagnostics that currently use fallback
line 1 column 1 when a containing object span is available.

Implementation method: Thread fallback spans from object, assignment, divert,
tunnel, return, conditional, and choice traversal into qualified use collection.
Continue using `QualifiedName` module spans when those are already available.

Acceptance criteria: Import-use diagnostics for qualified references inside
objects use the nearest real source span when expression-level span is absent.

Forbidden shortcuts: Do not change module import resolution rules. Do not
invent spans for nodes that have no containing object span.

Modification boundaries: Module import qualified-use collection and focused
tests.

Validation commands: `cargo test -p ink-compiler modules`; `make gate`.

Commit record: Implementation commit TBD; completion-record commit TBD.

### [ ] Task 22: Add regression tests for diagnostic span accuracy

Goal: Pin the diagnostic span improvements from Tasks 20 and 21.

Implementation method: Add focused tests that assert line and column for
literal function-call argument diagnostics and qualified import-use diagnostics.
Prefer existing test support helpers; add small helpers only if needed.

Acceptance criteria: Tests fail on the old fallback `1:1` behavior and pass
with the new span plumbing.

Forbidden shortcuts: Do not assert only on messages. Do not alter production
code just to satisfy line/column in one fixture.

Modification boundaries: Compiler analysis tests and minimal test support.

Validation commands: `cargo test -p ink-compiler array_literals dict_literals struct_literals modules`; `make gate`.

Commit record: Implementation commit TBD; completion-record commit TBD.

### [ ] Task 23: Run focused validation and make gate

Goal: Validate the full refactor as a cohesive compiler-analysis change before
closing the active plan.

Implementation method: Run the focused validation set for all touched surfaces,
then run `make gate`. Fix any regressions in follow-up implementation commits
within this task before marking it `[>]`.

Acceptance criteria: All listed validation commands pass.

Forbidden shortcuts: Do not skip slow commands. Do not update snapshots unless
the changed snapshot describes an intentional diagnostic span-only improvement.

Modification boundaries: Validation fixes only; no new refactor scope.

Validation commands: `cargo test -p ink-compiler analysis`; `cargo test -p ink-compiler targets`; `cargo test -p ink-compiler modules`; `cargo test -p ink-compiler array_literals dict_literals struct_literals`; `cargo test -p ink-test --test integration typed_values`; `cargo test -p ink-experiments`; `make gate`.

Commit record: Implementation commit TBD; completion-record commit TBD.

### [ ] Task 24: Close active plan and move to finished plans

Goal: Finish active plan bookkeeping after all implementation tasks are
complete and reviewed.

Implementation method: Confirm Tasks 01-23 are `[x]`, update progress to
`24/24`, move `docs/active_plan/compiler-analysis-refactor/` to
`docs/finished_plans/compiler-analysis-refactor/`, and record final validation
and commit metadata in this task before the move.

Acceptance criteria: No active compiler-analysis-refactor plan remains under
`docs/active_plan/`; the completed plan exists under `docs/finished_plans/`;
final validation evidence is recorded.

Forbidden shortcuts: Do not close the plan while any implementation task is
pending, blocked, in progress, or waiting review.

Modification boundaries: Plan bookkeeping only.

Validation commands: `make gate`.

Commit record: Implementation commit not applicable; completion-record commit TBD.
