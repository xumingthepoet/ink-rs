# Compiler Refactor Plan

This plan tracks the compiler refactor for the new language-evolution phase.
The goal is not C# parity by default. The goal is to make `crates/ink-compiler`
as easy to locate, change, test, and maintain as `crates/ink-runtime`.

The plan is written as a tasklist. Every task includes:

- Purpose: why the task exists.
- Approach: the implementation strategy.
- Acceptance: what must be true before the task is considered done.

## Runtime-Level Quality Target

The compiler should reach the same maintainability standard as the runtime
layer:

- A reader can locate the owning module for a behavior quickly.
- A common language change touches a small, predictable set of files.
- Module boundaries are explicit and do not leak implementation details.
- Core concepts such as paths, symbols, targets, parsed nodes, and runtime IR
  are represented by types or narrow APIs, not scattered string conventions.
- Focused tests can validate local changes before the full gate runs.
- Large files are orchestration files only when their job is truly orchestration.
  They must not hide several unrelated systems in one place.

This is not a file-splitting exercise. Moving code without improving
findability, locality, testability, or conceptual clarity does not satisfy this
plan.

## Architecture Fitness Rules

These rules apply to every phase.

- `syntax/mod.rs`, `analysis/mod.rs`, and `lower/mod.rs` should become thin
  orchestration modules. If they grow again, the plan has failed locally.
- Parser modules may depend on parsed model types, source types, diagnostics,
  and parser utilities. They must not depend on analysis, lowering, or emit.
- Analysis modules may depend on parsed model traversal and indexes. They must
  not encode runtime JSON shape.
- Lowering modules may depend on checked parsed model data, lowering indexes,
  lowering context, and runtime IR. They must not depend on parser trial order.
- Emit should depend on runtime IR only, not on parsed syntax or analysis.
- Path, symbol, and target manipulation should move toward typed helpers. New
  code should not introduce fresh ad hoc `format!("{parent}.{child}")` path
  logic unless it is immediately contained behind a path API.
- Shared tree traversal should be reusable, but not abstract for its own sake.
  A visitor is acceptable only if it makes passes easier to read and change.
- A module over roughly 800 to 1000 lines must have a clear reason to remain
  that size. Mixed-responsibility files should be split.
- Each behavior-preserving refactor must keep parse snapshots, diagnostics,
  JSON output, and runtime behavior unchanged unless an intentional language
  change is documented.
- Intentional language changes must update tests and `docs/WritingWithInk.md`
  in the same change.

## Per-Phase Definition Of Done

Every phase must meet these conditions before it is marked complete:

- The phase has focused validation commands listed or referenced.
- The relevant focused tests pass.
- Public or crate-visible APIs are smaller or clearer than before.
- At least one concrete future change is easier to make after the phase.
- No new god file, broad utility dumping ground, or duplicate parser/lowering
  mechanism has been introduced.
- If behavior changed intentionally, the new behavior is covered by new language
  tests and documentation.

## Validation Ladder

- Parser-only changes: focused parser tests, then
  `cargo test -p ink-test --test compiler_conformance`.
- Analysis-only changes: focused diagnostic tests, then
  `cargo test -p ink-compiler`.
- Lowering and JSON changes: focused JSON/runtime fixture tests, then
  `cargo test -p ink-test`.
- Stage completion: `cargo fmt --all --check`, `cargo check --workspace`,
  `cargo test --workspace`, and `make gate`.

## Changeability Drills

Run these drills during reviews to prove the refactor is improving real
maintenance quality:

- Add or rename one expression operator. Expected touch points: expression
  tokenizer/parser, parsed expression type if needed, expression lowering,
  tests, docs.
- Add one choice syntax variant. Expected touch points: choice parser, parsed
  choice model if needed, choice lowering if semantics differ, tests, docs.
- Remove one legacy syntax feature. Expected touch points: parser diagnostics,
  tests, docs, and possibly analysis if existing nodes disappear.
- Change one path resolution rule. Expected touch points: typed path/target
  module, relevant analysis/lowering tests, not arbitrary string call sites.
- Improve one diagnostic message. Expected touch points: owning parser or
  analysis pass and focused diagnostic tests.

If a drill requires unrelated edits across many files, the architecture still
needs work.

## Current Refactor Status

- Current phase: Phase 8 in progress. R015 through R074 are complete. Phase 1
  is complete except the first real intentional-divergence fixture, which
  should wait until an actual language change is chosen.
- Last full validation: `make gate` on 2026-04-25, passed.
- Last focused validation:
  `cargo test -p ink-compiler` on 2026-04-25, passed.
- Known blockers: none for behavior-preserving refactors.

## Focused Validation Commands

Parser-focused commands:

- `cargo test -p ink-compiler syntax::`
- `cargo test -p ink-test --test compiler_conformance`
- `cargo test -p ink-test --features csharp-tests --test csharp_tests -- TestStringParser`

Lowering/runtime-focused commands:

- `cargo test -p ink-test --test compiler_conformance`
- `cargo test -p ink-test --test conformance`
- `cargo test -p ink-test --test language`
- `cargo test -p ink-test --features csharp-tests --test csharp_tests`

Full validation:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `make gate`

## Current Compiler Pipeline Map

- Source input and preprocessing: `Compiler::parse` in
  `crates/ink-compiler/src/compiler.rs`, with include expansion currently also
  in `compiler.rs`.
- Comment elimination and source line creation:
  `crates/ink-compiler/src/source.rs`.
- Parsing entry point and parser driver: `syntax::parse`, re-exported from
  `crates/ink-compiler/src/syntax/parser.rs`.
- Syntax rules and statement parsers: `crates/ink-compiler/src/syntax/mod.rs`
  plus focused submodules under `crates/ink-compiler/src/syntax/`.
- Multiline conditional parsing:
  `crates/ink-compiler/src/syntax/conditional.rs`.
- Multiline sequence parsing:
  `crates/ink-compiler/src/syntax/sequence.rs`.
- Weave grouping:
  `crates/ink-compiler/src/syntax/weave.rs`.
- Gather syntax:
  `crates/ink-compiler/src/syntax/gather.rs`.
- Variable syntax:
  `crates/ink-compiler/src/syntax/variable.rs`.
- Declaration and logic syntax:
  `crates/ink-compiler/src/syntax/declaration.rs` and
  `crates/ink-compiler/src/syntax/logic.rs`.
- Parsed model: `crates/ink-compiler/src/parsed/`.
- Analysis entry point: `analysis::analyze` in
  `crates/ink-compiler/src/analysis.rs`.
- Lowering entry point: `lower::lower` in
  `crates/ink-compiler/src/lower.rs`.
- JSON emission entry point: `emit::emit_json` in
  `crates/ink-compiler/src/emit.rs`.

## Current Ownership Baseline

The main maintainability bottlenecks at the start of this plan are:

- `crates/ink-compiler/src/lower.rs`: about 4,100 lines. Owns runtime IR,
  indexes, path context, flow lowering, weave/choice lowering, expression
  lowering, and path compaction.
- `crates/ink-compiler/src/syntax/mod.rs`: about 2,200 lines. Owns parser
  driver logic, multiline conditionals, multiline sequences, expression
  parsing, gather parsing, weave grouping, identifiers, and statement routing.
- `crates/ink-compiler/src/analysis.rs`: about 2,100 lines. Owns several
  independent analysis passes and repeats parsed-tree traversal logic.
- `crates/ink-compiler/src/compiler.rs`: about 330 lines. Mostly pipeline
  orchestration, but also owns include expansion and root/flow include
  reordering.

## Runtime-Level Quality Metrics

Use these checks during phase reviews:

- File size: `find crates/ink-compiler/src -maxdepth 3 -type f | sort | xargs wc -l | sort -n`.
- Mixed ownership: manually review modules over roughly 800 to 1000 lines.
- Duplicate scanner logic: `rg -n "in_string|paren_depth|brace_depth|escaped" crates/ink-compiler/src/syntax`.
- Raw path strings: `rg -n "HashMap<String, String>|format!\\(\".*\\{.*\\}\\.\" crates/ink-compiler/src`.
- Focused tests: run the commands listed in "Focused Validation Commands".
- Changeability: run the drills listed above and record touched files.

## Test Surface Boundary

- Legacy compatibility tests live in `crates/ink-test/tests/csharp_tests.rs`
  and `crates/ink-test/fixtures/csharp_tests/`. They protect unchanged legacy
  behavior.
- Existing conformance tests live in `crates/ink-test/tests/conformance.rs`,
  `crates/ink-test/tests/compiler_conformance.rs`, and
  `crates/ink-test/fixtures/conformance/`.
- Rust-first language evolution tests live in
  `crates/ink-test/tests/language.rs` and
  `crates/ink-test/fixtures/language/`. They are the preferred home for new
  syntax, removed syntax, intentional divergence, and language diagnostics.

## Phase 0: Baseline And Quality Instruments

- [x] R001 Record the current full gate baseline
  - Purpose: Establish the real starting point so future regressions are not
    confused with pre-existing failures.
  - Approach: Run `make gate` and record the date, command, pass/fail state,
    and a short summary of failures if any.
  - Acceptance: The baseline is recorded in this file or a linked note, and
    any failures are classified as blocking or non-blocking for refactoring.

- [x] R002 Record focused parser validation commands
  - Purpose: Make parser refactors cheap to validate locally.
  - Approach: Identify commands that cover parse snapshots, inline syntax,
    flow syntax, choice syntax, and diagnostics.
  - Acceptance: The plan lists 2 to 4 parser-focused commands that have been
    run successfully or have known baseline failures documented.

- [x] R003 Record focused lowering/runtime validation commands
  - Purpose: Catch JSON shape and runtime behavior regressions without waiting
    for the full gate every time.
  - Approach: Identify commands covering compiler conformance JSON, runtime
    integration, and relevant legacy fixtures.
  - Acceptance: The plan lists 2 to 4 lowering/runtime commands that have been
    run successfully or have known baseline failures documented.

- [x] R004 Map the current compiler pipeline
  - Purpose: Give every refactor a shared pipeline boundary.
  - Approach: Document the current entry points for source preprocessing,
    parsing, analysis, lowering, and JSON emission.
  - Acceptance: A reader can follow `Compiler::compile` to each stage without
    reading the implementation first.

- [x] R005 Record current god files and ownership problems
  - Purpose: Make the structural bottlenecks measurable.
  - Approach: Record line counts and responsibilities for `syntax/mod.rs`,
    `analysis.rs`, `lower.rs`, and any other mixed-responsibility files.
  - Acceptance: The plan has a short inventory of current ownership problems
    and the phase that is expected to address each one.

- [x] R006 Define runtime-level quality metrics
  - Purpose: Prevent the refactor from being judged only by test pass/fail.
  - Approach: Track file size, module ownership, duplicate scanners, naked path
    string use, focused test coverage, and changeability drill results.
  - Acceptance: Each metric has a simple review method, such as `wc -l`, `rg`,
    focused tests, or a recorded drill.

- [x] R007 Mark legacy compatibility tests explicitly
  - Purpose: Keep C# tests useful without letting them define all future
    language behavior.
  - Approach: Document `crates/ink-test/tests/csharp_tests.rs` as the legacy
    compatibility surface.
  - Acceptance: The plan explains which tests preserve unchanged legacy
    behavior and which tests define new Rust-first language behavior.

- [x] R008 Add a refactor status section
  - Purpose: Keep progress visible across long-running continuation work.
  - Approach: Add a short section or linked note that records current phase,
    last validation, and known blockers.
  - Acceptance: Future agents can resume without rediscovering the same state.

## Phase 1: Rust-First Language Test Surface

- [x] R009 Create a new language test module
  - Purpose: Give intentional language changes a first-class test surface that
    is separate from legacy C# compatibility.
  - Approach: Add `crates/ink-test/tests/language.rs` or
    `crates/ink-test/tests/language/mod.rs` and wire it into cargo tests.
  - Acceptance: `cargo test -p ink-test --test language` runs successfully,
    even if it initially contains only a smoke test.

- [x] R010 Create a new language fixture directory
  - Purpose: Keep new language fixtures separate from conformance and C# legacy
    fixtures.
  - Approach: Add `crates/ink-test/fixtures/language/` with a small README or
    fixture convention.
  - Acceptance: The directory contains at least one minimal fixture and a clear
    convention for `.ink`, `.parse`, `.json`, and runtime expectation files.

- [x] R011 Add a language fixture helper
  - Purpose: Avoid repeating file loading, compiling, snapshot comparison, and
    runtime execution boilerplate in new tests.
  - Approach: Reuse patterns from existing compiler conformance helpers, but
    keep the helper specific to Rust-first language behavior.
  - Acceptance: One smoke fixture compiles through the helper and asserts a
    parse, JSON, or runtime result.

- [ ] R012 Add an intentional-divergence test convention
  - Purpose: Make language changes explicit when they diverge from upstream
    Ink.
  - Approach: Require divergence fixtures or tests to state the old behavior,
    new behavior, and reason.
  - Acceptance: The fixture README or helper docs include this convention, and
    the first example follows it.

- [x] R013 Add focused diagnostic test helpers
  - Purpose: Language removals and parser changes need precise diagnostics,
    not only compile failure.
  - Approach: Add helpers for asserting diagnostic severity, message fragment,
    and span where possible.
  - Acceptance: At least one language test asserts diagnostics through the new
    helper.

- [x] R014 Review test naming for future progress tracking
  - Purpose: Make task progress easy to map to tests.
  - Approach: Name language tests by feature area rather than by temporary bug
    or fixture order.
  - Acceptance: New test names make it clear which language behavior they own.

## Phase 2: Parser Module Boundaries

- [x] R015 Extract `syntax/parser.rs`
  - Purpose: Separate the parser driver from syntax-specific parsing logic.
  - Approach: Move `Parser`, story parsing, flow parsing, and stitch parsing
    out of `syntax/mod.rs`.
  - Acceptance: `syntax/mod.rs` becomes a thin module/export file, and parser
    focused tests pass with no parse snapshot changes.
  - Completed: Parser driver and multiline orchestration now live in
    `syntax/parser.rs`; `cargo test -p ink-compiler syntax::` and
    `cargo test -p ink-test --test compiler_conformance` pass.

- [x] R016 Extract `syntax/conditional.rs`
  - Purpose: Give multiline conditional parsing its own owner.
  - Approach: Move conditional prefix parsing, branch builders, branch
    classification, nested conditional parsing, and suffix parsing.
  - Acceptance: Conditional parse snapshots and JSON fixtures are unchanged,
    and future conditional syntax changes have one obvious module to edit.
  - Completed: Conditional prefix parsing, branch classification, nested
    conditional handling, suffix parsing, and branch construction now live in
    `syntax/conditional.rs`; `cargo test -p ink-compiler syntax::` and
    `cargo test -p ink-test --test compiler_conformance` pass.

- [x] R017 Extract `syntax/sequence.rs`
  - Purpose: Isolate sequence syntax, which is likely to change or shrink in a
    Rust-first language design.
  - Approach: Move multiline sequence parsing and share sequence type annotation
    parsing with inline text parsing through a narrow API.
  - Acceptance: Sequence fixtures pass, and there is only one implementation
    of sequence type annotation parsing.
  - Completed: Multiline sequence parsing now lives in `syntax/sequence.rs`
    while sequence type annotation parsing remains shared through
    `syntax/text.rs`; `cargo test -p ink-compiler syntax::` and
    `cargo test -p ink-test --test compiler_conformance` pass.

- [x] R018 Extract `syntax/weave.rs`
  - Purpose: Make weave grouping explicit instead of hidden in the parser
    driver.
  - Approach: Move weave grouping, nested weave grouping, depth detection, and
    weave construction helpers.
  - Acceptance: Choice/gather/nested weave snapshots are unchanged, and weave
    grouping can be tested without reading the parser driver.
  - Completed: Weave grouping, depth detection, and parsed weave construction
    helpers now live in `syntax/weave.rs` with a focused unit test;
    `cargo test -p ink-compiler syntax::` and
    `cargo test -p ink-test --test compiler_conformance` pass.

- [x] R019 Extract `syntax/gather.rs`
  - Purpose: Keep gather statement parsing separate from both generic parser
    control flow and weave grouping.
  - Approach: Move gather statement parsing, bracketed identifier parsing, and
    gather-specific inline prefix parsing.
  - Acceptance: Gather fixtures pass, and parser driver no longer contains
    gather-specific parsing details.
  - Completed: Gather statement parsing, bracketed identifier parsing, and
    conditional gather-prefix parsing now live in `syntax/gather.rs` with
    focused tests; `cargo test -p ink-compiler syntax::` and
    `cargo test -p ink-test --test compiler_conformance` pass.

- [x] R020 Extract `syntax/variable.rs`
  - Purpose: Put `VAR`, `temp`, assignment, and inc/dec syntax in one place
    because they are likely language-design pressure points.
  - Approach: Move variable declaration, temporary declaration, assignment, and
    inc/dec statement parsing.
  - Acceptance: Variable fixtures pass, and all variable statement parser
    entry points are owned by one module.
  - Completed: Global variable declarations, temp declarations, assignments,
    and inc/dec parsing now live in `syntax/variable.rs` with focused tests;
    `cargo test -p ink-compiler syntax::` and
    `cargo test -p ink-test --test compiler_conformance` pass.

- [x] R021 Extract declaration and logic modules
  - Purpose: Remove remaining unrelated statement parsers from `syntax/mod.rs`.
  - Approach: Add `syntax/declaration.rs` for `CONST` and `EXTERNAL`, and
    `syntax/logic.rs` for return and logic-line statements.
  - Acceptance: Statement parsing uses module-level entry points, and
    declaration/logic tests pass.
  - Completed: `CONST` and `EXTERNAL` parsing now live in
    `syntax/declaration.rs`; `return`, logic-line parsing, and function-call
    content wrapping now live in `syntax/logic.rs`;
    `cargo test -p ink-compiler syntax::` and
    `cargo test -p ink-test --test compiler_conformance` pass.

- [x] R022 Make statement trial order explicit
  - Purpose: Parser behavior depends on rule order, so the order must be easy
    to inspect and defend.
  - Approach: Replace the anonymous statement rule array with named rule groups
    or a documented `StatementRuleSet`.
  - Acceptance: The code documents why the order exists, and failed rules still
    rewind exactly as before.
  - Completed: Statement parsing now uses a named `STATEMENT_RULES` rule set
    with an order comment and a focused order test;
    `cargo test -p ink-compiler syntax::` and
    `cargo test -p ink-test --test compiler_conformance` pass.

- [x] R023 Add parser module ownership tests
  - Purpose: Make extracted modules directly testable.
  - Approach: Add focused tests near each module or through language/compiler
    conformance helpers.
  - Acceptance: Every extracted syntax module has at least one focused test or
    fixture that would fail if the module were broken.
  - Completed: Parser, conditional, sequence, weave, gather, variable,
    declaration, and logic modules now have focused tests or direct parser
    ownership coverage; `cargo test -p ink-compiler syntax::` and
    `cargo test -p ink-test --test compiler_conformance` pass.

- [x] R024 Run the parser boundary changeability drill
  - Purpose: Verify that parser modularization improves real edit locality.
  - Approach: Pick a harmless parser diagnostic or small syntax rule adjustment
    and record which files needed edits.
  - Acceptance: The drill touches a small, expected set of parser files and
    focused tests validate it.
  - Completed: Used the unsupported-syntax diagnostic path as the harmless
    drill. Only `syntax/parser.rs` and this plan needed edits; the parser-owned
    diagnostic is covered by a focused test, and `cargo test -p ink-compiler
    syntax::parser::tests::unsupported_syntax_diagnostics_stay_parser_owned`
    passes.

## Phase 3: Shared Scanning Utilities

- [x] R025 Add `syntax/scan.rs`
  - Purpose: Eliminate duplicated state machines for strings, escapes,
    parentheses, braces, and top-level separators.
  - Approach: Implement a shared scanner with configurable delimiter searches
    and top-level token finding.
  - Acceptance: Unit tests cover nested strings, escaped quotes, braces,
    parentheses, commas, and top-level separators.
  - Completed: Added `syntax/scan.rs` with configurable top-level scanning,
    top-level splitting, token lookup, and delimiter matching. Focused scanner
    tests cover escaped quotes, nested parentheses/braces, commas, inline text
    escapes, and top-level token detection; `cargo test -p ink-compiler
    syntax::scan::tests::` passes.

- [x] R026 Replace expression top-level scans
  - Purpose: Expression parsing currently duplicates operator scanning logic.
  - Approach: Keep the old parser shape initially, but route operator splitting
    through `scan.rs`.
  - Acceptance: Existing expression behavior is unchanged, and duplicate
    expression scanner helpers are removed or marked for immediate removal.
  - Completed: Expression argument splitting, operator matching, textual
    operator matching, and enclosing-parenthesis checks now route through
    `syntax/scan.rs`; `cargo test -p ink-compiler syntax::` passes.

- [x] R027 Replace inline text scans
  - Purpose: Inline text, tags, glue, braces, and sequences must share the same
    understanding of escaped and nested syntax.
  - Approach: Replace local brace matching and top-level split helpers in
    `syntax/text.rs`.
  - Acceptance: Inline conditional, inline sequence, string expression, glue,
    and tag fixtures are unchanged.
  - Completed: Inline token lookup, brace matching, conditional/sequence
    top-level splitting, and multiline conditional branch splitting now use
    `syntax/scan.rs`; `cargo test -p ink-compiler syntax::` passes.

- [x] R028 Replace choice top-level divert scans
  - Purpose: Choice parsing should not misread arrows inside strings, braces, or
    parentheses.
  - Approach: Rewrite choice divert detection using `scan.rs`.
  - Acceptance: Existing choice fixtures pass, and a new focused test covers an
    arrow that should not split choice content.
  - Completed: Choice top-level divert detection now uses `syntax/scan.rs`;
    a focused test covers arrows inside braced strings, and
    `cargo test -p ink-compiler syntax::` passes.

- [x] R029 Replace multidivert scans
  - Purpose: Divert/tunnel parsing should not maintain its own scanner.
  - Approach: Rewrite multidivert segment splitting in `syntax/divert.rs` using
    `scan.rs`.
  - Acceptance: Tunnel and divert fixtures pass, including arguments or strings
    containing arrow-like text.
  - Completed: Multidivert segment splitting now uses `syntax/scan.rs`, with
    focused coverage for arrows inside arguments and tunnel-onwards override
    targets; `cargo test -p ink-compiler syntax::` passes.

- [x] R030 Add scanner ownership rules
  - Purpose: Prevent new syntax modules from adding another local scanner.
  - Approach: Document that top-level scanning must go through `scan.rs` unless
    there is a measured reason not to.
  - Acceptance: The parser module comments or this plan state the rule, and
    `rg` shows no new duplicated scanner helpers.
  - Completed: `syntax/scan.rs` now documents scanner ownership: syntax
    modules should add a `ScanOptions` mode instead of local
    string/escape/nesting state machines; `rg` confirms scanner state lives in
    `scan.rs` except specialized string-literal parsing.

- [x] R031 Remove obsolete scanner helpers
  - Purpose: Avoid old and new scanning paths coexisting.
  - Approach: Delete replaced private helpers from expression, text, choice,
    and divert modules.
  - Acceptance: `cargo check --workspace` passes, and `rg` confirms duplicate
    helper names are gone.
  - Completed: Removed text/conditional wrapper helpers that only forwarded to
    `scan.rs`, and deleted the old choice/divert local scan state machines.
    Expression operator helpers remain only as semantic operator mappers over
    scanner token matches; `cargo test -p ink-compiler syntax::` passes.

- [x] R032 Run the scanner changeability drill
  - Purpose: Prove scanner centralization improves future syntax work.
  - Approach: Add one focused scanner test for a new nested delimiter case and
    verify only scanner tests and scanner code change.
  - Acceptance: The drill does not require edits in expression/text/choice
    parser internals.
  - Completed: Added a nested-parentheses delimiter drill entirely in
    `syntax/scan.rs`; no expression/text/choice/divert parser internals needed
    edits, and `cargo test -p ink-compiler syntax::scan::tests::` passes.

## Phase 4: Expression Parser

- [x] R033 Move expression parsing into `syntax/expression.rs`
  - Purpose: Give expression syntax a dedicated module before changing its
    internals.
  - Approach: Move `parse_initial_expression`, literal parsing, operator
    parsing, function-call parsing, and expression helper functions without
    behavior changes.
  - Acceptance: All expression call sites use the module API, and parse
    snapshots are unchanged.
  - Completed: Moved expression parsing, operator splitting, function-call
    parsing, string-expression parsing, and argument splitting into
    `syntax/expression.rs`; `syntax/mod.rs` now re-exports the expression entry
    points for existing call sites, and `cargo test -p ink-compiler syntax::`
    passes.

- [x] R034 Add expression behavior baseline tests
  - Purpose: Lock current behavior before replacing the parser.
  - Approach: Add focused tests for precedence, associativity, unary operators,
    negative numbers, function calls, strings, divert targets, and parentheses.
  - Acceptance: At least 20 expression cases pass on the current implementation.
  - Completed: Added a table-driven expression snapshot baseline with 24 cases
    covering precedence, left associativity, unary operators, folded negative
    numbers, booleans, floats, strings, function calls, divert targets, path
    references, contains operators, comparisons, and parenthesized expressions;
    `cargo test -p ink-compiler
    syntax::expression::tests::parses_current_expression_behavior_baseline`
    passes.

- [x] R035 Introduce expression token types
  - Purpose: Move away from string splitting toward a parser that can evolve.
  - Approach: Define token enum variants for identifiers, literals, operators,
    parentheses, commas, arrows, and string literals.
  - Acceptance: Tokenizer tests cover all token categories used by existing
    expression syntax.
  - Completed: Added `ExpressionToken` and a tokenizer covering identifiers,
    int/float/string literals, operators, parentheses, commas, arrows, word
    operators, symbol operators, and dotted paths. The expression entry point
    now tokenizes without changing parse behavior, and `cargo test -p
    ink-compiler syntax::expression::tests::` passes.

- [x] R036 Preserve expression source spans
  - Purpose: Improve diagnostics and make parser errors easier to maintain.
  - Approach: Store byte or column positions on tokens and map them to
    `SourceSpan`.
  - Acceptance: Tests can assert token columns for at least identifiers,
    operators, and string literals.
  - Completed: Tokens now carry `kind`, `byte_index`, and `SourceSpan`; tests
    assert columns for identifiers, operators, string literals, and a unicode
    prefix case where byte offsets and character columns differ. `cargo test -p
    ink-compiler syntax::expression::tests::` passes.

- [x] R037 Implement a Pratt or precedence-climbing parser
  - Purpose: Make operator precedence and associativity explicit and easy to
    extend.
  - Approach: Parse tokens into the existing `Expression` enum first, without
    changing the parsed model.
  - Acceptance: R034 baseline tests pass on the new parser.
  - Completed: Added a token-backed precedence-climbing parser behind the
    current expression adapter. It parses literals, identifiers, function
    calls, divert targets, unary operators, parenthesized expressions, and the
    existing binary operator precedence table into the current `Expression`
    enum. A focused baseline test proves the new parser reproduces the 24-case
    R034 expression behavior set while the compiler entry point still returns
    the old parser result pending R039; `cargo test -p ink-compiler
    syntax::expression::tests::` passes.

- [x] R038 Add structured expression parse errors
  - Purpose: Avoid silent `None` failures that collapse into vague unsupported
    syntax diagnostics.
  - Approach: Return parse error kinds and spans from expression parsing, then
    let statement parsers decide recovery.
  - Acceptance: Invalid expression tests assert a useful diagnostic message and
    location.
  - Completed: The token-backed expression parser now returns
    `ExpressionParseError` with a stable kind and `SourceSpan`, while the
    public compiler expression adapter still preserves old behavior until the
    R039 switch. Focused tests cover missing right operands, missing closing
    parentheses, missing divert targets, malformed function-call separators,
    and trailing unknown tokens with asserted messages and columns;
    `cargo test -p ink-compiler syntax::expression::tests::` passes.

- [x] R039 Switch compiler entry points to the new expression parser
  - Purpose: Use the new expression parser in real compiler flows.
  - Approach: Route all expression entry points through the tokenizer-based
    parser.
  - Acceptance: Parser conformance, expression focused tests, and language
    smoke tests pass.
  - Completed: `parse_initial_expression` now routes through the token-backed
    expression parser. The switch preserved expression snapshots, parser
    syntax tests, and compiler conformance JSON snapshots; `cargo check -p
    ink-compiler`, `cargo test -p ink-compiler syntax::`, and `cargo test -p
    ink-test --test compiler_conformance` pass.

- [x] R040 Remove old expression string-splitting parser
  - Purpose: Avoid maintaining two expression implementations.
  - Approach: Delete old split helpers and old parser branches after the new
    parser is active.
  - Acceptance: No dead parser path remains, and `cargo check --workspace`
    passes.
  - Completed: Deleted the old recursive string-splitting expression parser,
    including its operator split helpers, parenthesis stripping, function-call
    splitting, unary-prefix branch, and standalone quoted-string parser. The
    expression module now has one parser path plus the shared argument splitter
    used by divert syntax; `cargo check -p ink-compiler` passes.

- [x] R041 Run the expression operator drill
  - Purpose: Prove a future operator change is localized.
  - Approach: Add or temporarily prototype one operator or alias and record the
    touched files.
  - Acceptance: The drill touches expression tokenizer/parser, lowering if
    needed, tests, and docs only.
  - Completed: Consolidated expression binary operator metadata into
    `BINARY_OPERATOR_RULES`, then used the existing `!?` alias as the drill
    case to prove tokenizer lookup and parser precedence lookup share one
    owner. The retained code change touched only
    `crates/ink-compiler/src/syntax/expression.rs` plus this plan; no lowering
    or docs were needed because no new language behavior was kept. Focused
    expression tests pass.

- [x] R042 Review expression AST fit
  - Purpose: Decide whether the current `Expression` enum still supports future
    language design.
  - Approach: Review variants such as `MultipleCondition`, `StringContent`, and
    `DivertTarget` against planned language changes.
  - Acceptance: Either the enum is accepted as good enough, or follow-up tasks
    are recorded for typed expression model changes.
  - Completed: Accepted the current `Expression` enum as good enough for the
    next refactor phases because literals, variable references, divert targets,
    calls, unary expressions, binary expressions, and string expressions are now
    parsed through one token-backed parser and are exhaustively matched by
    analysis and lowering. Follow-up pressure points are recorded here rather
    than expanded immediately: `StringContent(ContentList)` mixes string
    interpolation with general content trees and should be revisited if the new
    language narrows interpolation syntax; `MultipleCondition(Vec<Expression>)`
    is choice-condition aggregation rather than a pure expression and should be
    revisited with choice model changes; `DivertTarget(String)` should move to
    a typed target/path value during Phase 7; and `BinaryOperator` still carries
    source spelling aliases such as `And` versus `AndSymbol`, which is
    acceptable for compatibility snapshots but should stay behind syntax
    operator metadata for future aliases.

## Phase 5: Parsed Model Traversal And Model Quality

- [x] R043 Add `parsed/visit.rs`
  - Purpose: Replace repeated hand-written tree walks with reusable traversal.
  - Approach: Implement immutable traversal over `Story`, `Flow`, `Weave`,
    `ContentList`, `Object`, and `Expression`.
  - Acceptance: Visitor tests prove traversal reaches choices, gathers,
    sequences, conditionals, string expressions, and nested weaves.
  - Completed: Added `parsed::visit` with a crate-visible immutable
    `ParsedVisitor` trait and `walk_story` entry point. The walker descends
    through stories, flows, weaves, content lists, objects, expression trees,
    choice content and conditions, conditional branches, sequence elements,
    string-expression content, divert/tunnel arguments, returns, assignments,
    constants, and nested weave objects. A focused visitor test constructs a
    mixed parsed tree and verifies traversal reaches choices, gathers,
    sequences, conditionals, string expressions, nested weaves, flows, and
    expression variants; `cargo test -p ink-compiler parsed::visit::tests::`
    passes.

- [x] R044 Add traversal context
  - Purpose: Analysis passes need current flow path, parent flow, and function
    context.
  - Approach: Pass a context struct through traversal and update it when
    entering flows and nested content.
  - Acceptance: Tests prove context for root, knot, and child stitch traversal.
  - Completed: Added `VisitContext` to `parsed::visit`, carrying current flow
    path, parent flow path, and function context. The visitor test now verifies
    root context has no flow path, knot context has `knot`, and child stitch
    context has `knot.stitch`, parent `knot`, and function context when the
    child flow is marked as a function.

- [x] R045 Refactor author warnings onto traversal
  - Purpose: Validate the visitor on the simplest analysis pass first.
  - Approach: Collect `AuthorWarning` diagnostics through visitor callbacks.
  - Acceptance: Existing author warning behavior is unchanged, and old warning
    recursion helpers are removed.
  - Completed: Replaced the dedicated author-warning recursion helpers in
    `analysis.rs` with an `AuthorWarningVisitor` that collects diagnostics from
    `visit_object`. The traversal itself now owns descent through choices,
    conditionals, sequences, content lists, and nested weaves. `cargo test -p
    ink-test --features csharp-tests --test csharp_tests --
    TestAuthorWarningsInsideContentListBug` and `cargo test -p ink-compiler`
    pass.

- [x] R046 Refactor constant redefinition onto traversal
  - Purpose: Reduce repeated recursion while preserving story-wide constant
    semantics.
  - Approach: Use traversal callbacks to collect `ConstantDeclaration` nodes
    and compare expressions.
  - Acceptance: Constant tests pass, including nested content positions.
  - Completed: Replaced the constant-redefinition recursion helpers in
    `analysis.rs` with a `ConstantRedefinitionVisitor` that tracks story-wide
    constant expressions from `visit_object`. Traversal now owns nested content
    descent for constants, matching the author-warning pass. `cargo test -p
    ink-compiler` and `cargo test -p ink-test --features csharp-tests --test
    csharp_tests -- TestConstRedefinition` pass.

- [x] R047 Refactor variable scope collection onto traversal
  - Purpose: Make variable visibility easier to change later.
  - Approach: Use traversal context to build globals and locals by flow path.
  - Acceptance: Temp/global/function-argument tests pass, and scope collection
    logic has one owner.
  - Completed: Replaced the story/flow variable-scope recursion helpers in
    `analysis.rs` with a `VariableScopeVisitor` over `parsed::visit`.
    Story-scope constants, globals, and root temps populate the global set.
    Flow arguments and flow-local temps populate `locals_by_flow_path` through
    `VisitContext.current_flow_path`, so child-flow locals stay isolated from
    parent flows. `cargo test -p ink-compiler`, `cargo test -p ink-test
    --features csharp-tests --test csharp_tests -- TestTemp`, and `cargo test
    -p ink-test --features csharp-tests --test csharp_tests -- TestVariable`
    pass.

- [x] R048 Refactor target symbol collection onto traversal
  - Purpose: Centralize labels, flow symbols, choice identifiers, and gather
    identifiers.
  - Approach: Use traversal context to build target indexes.
  - Acceptance: Divert, gather, read-count, and stitch resolution tests pass.
  - Completed: Replaced target-symbol recursion helpers in `analysis.rs` with
    a two-phase `TargetSymbolVisitor` over `parsed::visit`. The first traversal
    registers flow/function symbols from `visit_flow`; the second traversal
    registers choice and gather labels from `visit_object` using
    `VisitContext.current_flow_path`. The two-phase order preserves the old
    `or_insert` precedence where flow symbols win before labels are added.
    `cargo test -p ink-compiler`, `cargo test -p ink-test --features
    csharp-tests --test csharp_tests -- TestDivert`, `TestGather`,
    `TestReadCount`, `TestStitch`, and `TestPath` pass.

- [x] R049 Refactor call-target diagnostics onto traversal
  - Purpose: Reduce long recursive parameter lists and make target checking
    easier to modify.
  - Approach: Encapsulate shared indexes in a checker struct that runs through
    traversal.
  - Acceptance: Call/divert/variable diagnostics pass, and checker call sites
    no longer pass many unrelated parameters through every recursion layer.
  - Completed: Replaced the call-target object/content recursion helpers with
    `CallTargetChecker`, a parsed visitor that owns diagnostics plus the target
    symbol, variable-target, variable-scope, and flow-argument indexes. The
    checker uses `VisitContext.current_flow_path` for target and variable
    visibility, records current-flow arguments from `visit_flow`, and leaves
    expression recursion local so existing expression-owner spans are preserved.
    Focused function-call, divert-target, variable-target, unresolved-variable,
    and empty-divert C# tests pass.

- [x] R050 Refactor flow-control diagnostics onto traversal where useful
  - Purpose: Keep loose-end and function-return rules close to flow semantics.
  - Approach: Use shared traversal for discovery, but keep sequential flow
    checks explicit where order matters.
  - Acceptance: Flow-control warnings/errors are unchanged, and the code makes
    order-sensitive logic obvious.
  - Completed: Moved the function-only "no diverts/no choices" scan to a
    `FunctionFlowControlVisitor` over `parsed::visit`, while keeping loose-end,
    return, and nested sealed-choice checks in explicit sequential helpers.
    `VisitContext` now marks choice-content and expression-content traversal so
    the visitor preserves the old function-flow-control surface and does not
    report inside regions the previous pass intentionally skipped. Focused
    function, loose-end, nested-choice, return-warning, gather, and knot
    termination C# tests pass.

- [x] R051 Review parsed model ownership
  - Purpose: Ensure language concepts are represented in parsed types before
    lowering.
  - Approach: Review recent parser/lowering code for data that is inferred late
    from raw strings instead of stored in parsed nodes.
  - Acceptance: Any missing parsed-model fields are added or recorded as
    follow-up tasks.
  - Completed: Reviewed parsed, analysis, syntax, and lowering call sites for
    raw string path/target inference. The two durable ownership gaps are not
    small enough for this traversal phase: `DivertTarget::Path(String)` and
    `Expression::DivertTarget(String)` still leave dotted path semantics to
    analysis/lowering string helpers, and `Expression` nodes still lack source
    spans, forcing analysis to use owner-object spans. The path gap is recorded
    against the Phase 7 typed path work, especially R061 and R062. The
    expression-span gap is recorded against Phase 6 analysis pass extraction and
    focused diagnostic tests, especially R055 and R058. No immediate parsed
    field was added because both changes would touch parser snapshots,
    diagnostics, and lowering contracts.

## Phase 6: Analysis Pass Architecture

- [x] R052 Split `analysis.rs` into pass modules
  - Purpose: Remove the current analysis god file and give each semantic check
    an owner.
  - Approach: Create `analysis/mod.rs`, `analysis/constants.rs`,
    `analysis/names.rs`, `analysis/variables.rs`, `analysis/targets.rs`, and
    `analysis/flow.rs`.
  - Acceptance: `analysis/mod.rs` only orchestrates pass ordering, and focused
    analysis tests pass.
  - Completed: Moved `analysis.rs` to `analysis/mod.rs` and extracted pass
    owners into `constants`, `warnings`, `names`, `variables`, `targets`, and
    `flow`, with a small `span` helper module for shared source-span lookup.
    `analysis/mod.rs` now declares modules and orchestrates pass ordering only.
    Focused compiler, constant, naming, temp/scope, divert/target, and function
    flow-control tests pass.

- [x] R053 Add `analysis/context.rs`
  - Purpose: Share symbol, variable, and flow context types without duplicating
    structs across passes.
  - Approach: Move shared indexes and context into crate-private types.
  - Acceptance: Variables and targets passes reuse context types without
    circular dependencies.
  - Completed: Added `analysis/context.rs` for `VariableScopeIndex`,
    `FlowSymbol`, target/variable-target index aliases, and `FlowContext`.
    `variables` now builds the shared variable-scope index, while `targets`
    consumes the shared index and flow context instead of owning duplicate
    context structs and parallel argument/function maps. Focused compiler,
    temp, variable, divert, and variable-target typing tests pass.

- [x] R054 Make analysis pass ordering explicit
  - Purpose: Future language rules need a clear place in the analysis pipeline.
  - Approach: Use an ordered pass list or direct sequence with comments about
    dependencies.
  - Acceptance: A reader can see which passes depend on symbols, variables, or
    prior diagnostics.
  - Completed: Added `run_analysis_passes` in `analysis/mod.rs`, leaving
    `analyze` as pipeline wrapping only. The pass sequence is now grouped and
    commented by dependency: story-wide discovery, naming, order-sensitive
    flow checks, then target/variable resolution. Focused compiler, function,
    and loose-end tests pass.

- [x] R055 Add focused tests per analysis pass
  - Purpose: Make each analysis pass locally verifiable.
  - Approach: Add tests for constants, names, variables, targets, and flow
    rules.
  - Acceptance: Each pass has at least one direct test that would fail if the
    pass were removed.
  - Completed: Added shared analysis test support plus direct unit tests for
    constant redefinition, author warning conversion, naming collisions,
    variable scope indexing, flow loose-end warnings, and missing target
    diagnostics. Focused compiler tests pass.

- [x] R056 Decide whether checked story should carry indexes
  - Purpose: Avoid rebuilding the same indexes in analysis and lowering if they
    are semantically shared.
  - Approach: Review which indexes are pure analysis artifacts and which should
    become part of `CheckedStory`.
  - Acceptance: A decision is recorded, and any chosen index sharing is covered
    by tests.
  - Completed: Decision is to keep `CheckedStory` parsed-model-only for now.
    The current analysis target and variable indexes are diagnostic artifacts,
    while lowering builds runtime-label, constant, external-signature,
    local-variable, and counted-path indexes whose semantics depend on JSON
    container layout. No index sharing was chosen, so no new shared-index test
    surface is required. Added a boundary comment to `CheckedStory` and kept
    existing focused analysis/lowering tests as validation.

- [x] R057 Add analysis API boundary checks
  - Purpose: Keep analysis independent from lowering and JSON shape.
  - Approach: Review imports and move any runtime-specific logic out of
    analysis.
  - Acceptance: Analysis modules do not import lower/emit modules.
  - Completed: Added a focused analysis boundary test that scans analysis
    source files and fails if they import `lower` or `emit` module APIs.
    Existing analysis imports were reviewed and already had no runtime
    lowering or JSON emission dependency.

- [x] R058 Run the diagnostic drill
  - Purpose: Prove a diagnostic change is easy to locate.
  - Approach: Improve one small diagnostic message and record the touched files.
  - Acceptance: The change touches one owning analysis/parser module plus a
    focused diagnostic test.
  - Completed: Improved the non-function call target diagnostic in
    `analysis/targets.rs` by fixing `delcare` to `declare`, and added a direct
    target-pass test for the exact diagnostic. Touched files:
    `analysis/targets.rs` and `CompilerRefactorPlan.md`. Focused diagnostic
    test passes.

- [x] R059 Review analysis file sizes and responsibilities
  - Purpose: Ensure splitting analysis did not create new dumping grounds.
  - Approach: Check line counts and module responsibilities after extraction.
  - Acceptance: No analysis module is both large and mixed-responsibility.
  - Completed: Reviewed analysis line counts and responsibilities after Phase
    6 extraction. `targets.rs` was both the largest file and mixed diagnostic
    checking with target-symbol and variable-target index construction, so
    those indexes were moved to `target_symbols.rs` and
    `variable_targets.rs`. After the split, the larger files are cohesive:
    `names.rs` owns naming diagnostics, `flow.rs` owns flow diagnostics, and
    `targets.rs` owns call-target diagnostics. Focused compiler tests pass.

## Phase 7: Lowering Architecture And Typed Paths

- [x] R060 Extract `lower/ir.rs`
  - Purpose: Separate runtime IR data types from lowering algorithms.
  - Approach: Move `RuntimeProgram`, `Container`, `RuntimeObject`, and
    `ControlCommand`.
  - Acceptance: `emit.rs` depends on runtime IR only, and JSON output is
    unchanged.
  - Completed: Added `lower/ir.rs` for `RuntimeProgram`, `Container`,
    `RuntimeObject`, and `ControlCommand`. `lower.rs` now imports those IR
    types for lowering algorithms, while `emit.rs` and compiler API references
    use `lower::ir`. Focused compiler tests pass, including the existing JSON
    emission assertions.

- [x] R061 Add minimal path helper before deeper lowering extraction
  - Purpose: Prevent lower splitting from copying existing string path logic.
  - Approach: Add a small `lower/path.rs` helper for child, parent, relative,
    canonical, and component operations, even before full typed paths.
  - Acceptance: New lowering modules use the helper instead of fresh path
    string formatting.
  - Completed: Added `lower/path.rs` for child, parent, component, relative
    compaction, absolute-runtime-path, semantic-key, and canonical-runtime-path
    helpers. Existing lower path compaction now imports these helpers instead
    of owning the logic inline, with focused path helper tests and compiler
    tests passing.

- [x] R062 Extract `lower/indexes.rs`
  - Purpose: Give constants, labels, globals, external signatures, and counted
    paths a clear owner.
  - Approach: Move index builders into one module and aggregate them in a
    `LoweringIndexes` type.
  - Acceptance: `lower::lower` builds indexes once and passes a clear context
    object to lowering code.
  - Completed: Added `lower/indexes.rs` with `LoweringIndexes`,
    `RuntimeLenEstimator`, external signatures, counted-flow paths, global
    variable declarations, constants, labels, and counted-path builders. The
    top-level lowering entry point now builds indexes once and passes the
    index context into root, flow, and global-declaration lowering.

- [x] R063 Extract `lower/context.rs`
  - Purpose: Make path mode, flow context, local variables, and fallback gather
    behavior explicit.
  - Approach: Move `ChoicePathMode` and related context operations into a
    dedicated module.
  - Acceptance: Flow/weave/expression lowering access context through methods,
    not by manually inspecting many enum fields everywhere.
  - Completed: Added `lower/context.rs` as the owner for `ChoicePathMode`,
    flow-local checks, runtime index paths, scoped label lookup, sibling-stitch
    resolution, fallback gather behavior, and choice/gather target helpers.
    Lowering and index code now call context methods instead of free target
    resolution helpers.

- [x] R064 Extract `lower/expression.rs`
  - Purpose: Separate expression bytecode emission from flow/weave structure.
  - Approach: Move expression lowering, function call lowering, operator
    runtime names, and builtin function handling.
  - Acceptance: Expression, string, and function-call JSON fixtures are
    unchanged.
  - Completed: Added `lower/expression.rs` for output expression, logic-line
    expression, recursive expression emission, function-call lowering,
    by-reference argument lowering, operator runtime names, and builtin
    function dispatch. `lower.rs` now imports expression emission entry points
    and retains only structural lowering calls.

- [x] R065 Extract `lower/flow.rs`
  - Purpose: Give knot/stitch/function lowering an owner.
  - Approach: Move root flow lowering, child flow lowering, argument
    assignment, local collection needed by flow context, and auto-divert logic.
  - Acceptance: Knot, stitch, and function fixtures are unchanged, and flow
    lowering does not contain choice section internals.
  - Completed: Added `lower/flow.rs` for root weave entry lowering, flow and
    child-flow lowering, flow argument assignment, flow-local variable
    collection, flow container flags, and child-stitch auto-divert behavior.
    The flow module calls existing weave lowering entry points without owning
    choice/gather section internals.

- [x] R066 Extract `lower/weave.rs`
  - Purpose: Give choice/gather/weave lowering an owner.
  - Approach: Move linear weave lowering, choice weave lowering, weave section
    lowering, choice container creation, and gather container handling.
  - Acceptance: Choice, gather, and weave fixtures are unchanged, and weave
    lowering can be tested without reading flow lowering.
  - Completed: Added `lower/weave.rs` for linear weave lowering, choice weave
    lowering, weave section traversal, gather container placement, choice
    container creation, local weave labels, content-list lowering entry points,
    and weave helper predicates. Flow, expression, indexes, and remaining
    structural lowering now import weave entry points directly.

- [x] R067 Extract sequence and conditional lowering if still large
  - Purpose: Keep `lower/weave.rs` from becoming the new god file.
  - Approach: If sequence or conditional lowering remains substantial, move it
    to `lower/sequence.rs` or `lower/conditional.rs`.
  - Acceptance: `lower/weave.rs` stays focused on weave structure, not every
    nested content feature.
  - Completed: Added `lower/sequence.rs` for sequence branch runtime emission
    and return-divert handling, and `lower/conditional.rs` for conditional
    branch evaluation and branch content containers. `lower.rs` now delegates
    sequence and conditional objects through narrow module entry points.

- [x] R068 Move path compaction fully into `lower/path.rs`
  - Purpose: Treat path compaction and semantic path canonicalization as a path
    subsystem.
  - Approach: Move compaction, semantic path indexing, absolute/relative path
    checks, and user-named component logic.
  - Acceptance: Path unit tests cover relative and canonical behavior, and JSON
    paths are unchanged.
  - Completed: Moved runtime target compaction, semantic path indexing, named
    content traversal, and canonical target replacement into `lower/path.rs`.
    Added a path unit test covering indexed target canonicalization followed by
    relative path compaction. `lower.rs` now calls one path compaction entry
    point after building the root container.

- [x] R069 Introduce typed path wrappers
  - Purpose: Reduce confusion between flow paths, runtime paths, semantic label
    keys, and target paths.
  - Approach: Add `FlowPath`, `RuntimePath`, and `TargetPath` or equivalent
    wrappers where they reduce ambiguity.
  - Acceptance: Key lowering indexes no longer use `HashMap<String, String>`
    for conceptually different path kinds.
  - Completed: Added `RuntimePath`, `LabelAlias`, and `LabelIndex` in
    `lower/path.rs`. `LoweringIndexes.global_labels` now uses `LabelIndex`
    instead of exposing `HashMap<String, String>`, and label resolution APIs
    accept the typed index. Local choice-label maps remain raw strings and are
    scheduled for the follow-up typed-map pass.

- [x] R070 Replace label and target maps with typed maps
  - Purpose: Make path resolution safer and easier to change.
  - Approach: Replace raw label maps gradually, starting from label indexes and
    divert target resolution.
  - Acceptance: Path-related tests pass, and remaining raw string maps are
    documented as intentional or scheduled follow-ups.
  - Completed: Replaced local choice/gather label maps with `LabelIndex` as
    well as the global label index. Expression, divert, sequence, conditional,
    and weave lowering now resolve label aliases through the typed path index.
    Remaining raw maps in lowering are for non-path data such as constants and
    external signatures.

- [x] R071 Add runtime container builder helpers
  - Purpose: Reduce noisy direct vector manipulation in lowering without hiding
    JSON shape.
  - Approach: Add small helpers for container construction, named content tail
    metadata, and common control-command sequences.
  - Acceptance: At least one complex flow/weave construction becomes easier to
    read, and JSON output remains unchanged.
  - Completed: Added `Container::unnamed`, `Container::named`,
    `Container::named_with_flags`, and small `RuntimeObject` container helpers.
    Applied them to conditional branch containers and choice/weave return and
    named-content containers, removing repeated default field initialization
    while preserving the runtime IR shape.

- [x] R072 Run the path resolution drill
  - Purpose: Prove path handling is centralized.
  - Approach: Prototype or adjust one path resolution rule and record touched
    files.
  - Acceptance: The drill touches path/context/index modules and focused tests,
    not scattered string formatting call sites.
  - Completed: Moved scoped label-target lookup and scoped label-alias
    insertion into `LabelIndex` in `lower/path.rs`. `lower/context.rs` now
    passes only the current flow path into that path API, and
    `lower/indexes.rs` delegates flow/container alias insertion to it. Focused
    path tests cover scoped lookup precedence and scoped alias insertion.

- [x] R073 Review lower module sizes and ownership
  - Purpose: Ensure lower extraction does not recreate a god file under another
    name.
  - Approach: Check line counts, imports, and responsibilities of lower modules.
  - Acceptance: No lower module is both large and mixed-responsibility.
  - Completed: Reviewed lower module sizes and ownership. Extracted
    `lower/labels.rs` so label-index construction no longer lives inside the
    broader index builder. After extraction, `indexes.rs` is 680 lines and
    focused on non-label lowering indexes; `weave.rs` remains the only module
    above 800 lines at 881 lines, but its responsibility is concentrated on
    choice/gather/weave lowering. Other lower modules are below 600 lines.

## Phase 8: Source Preprocessing, Diagnostics, And Documentation

- [x] R074 Extract include/preprocess logic
  - Purpose: Keep `compiler.rs` focused on the pipeline rather than source file
    expansion details.
  - Approach: Move include expansion, recursive include detection, root/flow
    reordering, and preprocessing entry points into `source/preprocess.rs` or
    `source/include.rs`.
  - Acceptance: `Compiler::parse` calls a small preprocessing API, and include
    behavior is unchanged.
  - Completed: Added `source/preprocess.rs` with `preprocess_includes`,
    include expansion, recursive include detection, root/flow line ordering,
    and include parsing helpers. `Compiler::parse` now calls this small source
    preprocessing API before syntax parsing. Existing compiler tests and
    `TestInclude` pass unchanged.

- [ ] R075 Add include behavior tests
  - Purpose: Include behavior is a language file-organization surface and must
    be protected.
  - Approach: Test root includes, flow includes, recursive includes, missing
    handlers, and missing files.
  - Acceptance: Each include behavior has a focused test with diagnostic
    assertions where relevant.

- [ ] R076 Improve include source span strategy
  - Purpose: Diagnostics from included files should point to useful source
    names.
  - Approach: Evaluate whether `SourceInput` needs a richer source map; make
    the smallest practical improvement first.
  - Acceptance: Diagnostics from included content can identify the included
    source name in at least one focused test.

- [ ] R077 Add diagnostic categories or codes
  - Purpose: Make parser, analysis, removed-feature, and unsupported-feature
    diagnostics easier to test and maintain.
  - Approach: Extend `Diagnostic` with an optional category/code while
    preserving existing messages initially.
  - Acceptance: Existing tests still pass, and at least one new test asserts a
    category or code.

- [ ] R078 Improve expression error recovery
  - Purpose: Invalid expressions should not collapse into vague parser failure.
  - Approach: Use structured expression parse errors and recover at statement
    boundaries.
  - Acceptance: Invalid expression tests assert clear messages and do not hide
    diagnostics from later lines.

- [ ] R079 Improve choice and inline syntax recovery
  - Purpose: Choice bracket and inline brace mistakes are common and should be
    easy to diagnose.
  - Approach: Replace silent `None` paths with specific diagnostics where the
    parser knows the intended construct.
  - Acceptance: Invalid choice/inline tests assert specific messages and useful
    spans.

- [ ] R080 Update architecture documentation
  - Purpose: `docs/ArchitectureAndDevOverview.md` should describe the Rust
    compiler architecture, not primarily the old C# model.
  - Approach: Rewrite the compiler sections around source, syntax, parsed
    model, analysis, lowering, and emit modules.
  - Acceptance: The documentation matches the post-refactor module layout.

- [ ] R081 Add a language divergence documentation location
  - Purpose: Intentional language changes need a stable home.
  - Approach: Add a "Changed from upstream Ink" section to
    `docs/WritingWithInk.md` or create a linked language changes document.
  - Acceptance: The first intentional divergence can be documented without
    inventing a new structure.

- [ ] R082 Run the removed-feature drill
  - Purpose: Prove that removing a legacy syntax feature is localized and clear.
  - Approach: Prototype removal or diagnostic-only rejection of a small feature
    and record touch points.
  - Acceptance: The drill touches parser diagnostics, tests, and docs, with no
    lowering hack or fixture-specific branch.

- [ ] R083 Review public compiler API after refactor
  - Purpose: Keep external API stable and clear while internals change.
  - Approach: Review `lib.rs`, `Compiler`, `CompilerOptions`, `StageOutput`,
    and exported parsed/lower types.
  - Acceptance: Public API changes are intentional, documented, and tested.

## Phase 9: Runtime-Level Quality Review

- [ ] R084 Compare compiler maintainability against runtime
  - Purpose: Verify the compiler now feels closer to the runtime layer in
    findability and local change cost.
  - Approach: Compare module layout, file sizes, API surfaces, and focused test
    entry points between `ink-compiler` and `ink-runtime`.
  - Acceptance: A short review records what now matches runtime quality and
    what still falls short.

- [ ] R085 Check for remaining compiler god files
  - Purpose: Ensure the original structural problem did not move elsewhere.
  - Approach: Run line-count checks and manually review responsibilities.
  - Acceptance: No compiler file over roughly 800 to 1000 lines has mixed
    responsibilities without a documented split task.

- [ ] R086 Check for remaining raw path/symbol string abuse
  - Purpose: Confirm typed path and symbol work improved safety.
  - Approach: Search for raw path maps, repeated path formatting, and stringly
    target resolution.
  - Acceptance: Remaining raw strings are either low-risk, wrapped by helpers,
    or recorded as follow-up work.

- [ ] R087 Check new feature placement
  - Purpose: Ensure language features are not implemented as parser hacks plus
    lowering special cases.
  - Approach: Pick one new language behavior and trace parser, parsed model,
    analysis, lowering, tests, and docs.
  - Acceptance: The feature has explicit model/data ownership and no
    fixture-specific code path.

- [ ] R088 Run all changeability drills
  - Purpose: Validate maintainability through real edits, not just review.
  - Approach: Run or simulate the expression operator, choice syntax,
    removed-feature, path resolution, and diagnostic drills.
  - Acceptance: Each drill has recorded touch points, validation commands, and
    any follow-up tasks.

- [ ] R089 Remove obsolete C# parity comments
  - Purpose: Keep code comments aligned with the new language-evolution phase.
  - Approach: Search for C# parity comments and rewrite them as legacy
    reference notes only where still useful.
  - Acceptance: Remaining C# comments clearly describe compatibility reference
    or historical behavior, not default project direction.

- [ ] R090 Remove obsolete helpers and transitional code
  - Purpose: Avoid maintaining duplicate old and new mechanisms.
  - Approach: Delete unused scanner, parser, analysis, path, and lowering
    helpers after replacements are complete.
  - Acceptance: `cargo check --workspace` passes without new dead-code warnings,
    and `rg` confirms known old helpers are gone.

- [ ] R091 Run formatting and compile checks
  - Purpose: Confirm the refactor is mechanically clean.
  - Approach: Run `cargo fmt --all --check` and `cargo check --workspace`.
  - Acceptance: Both commands pass, or failures are fixed before the task is
    marked complete.

- [ ] R092 Run the full validation ladder
  - Purpose: Complete the refactor with project-wide confidence.
  - Approach: Run focused tests, `cargo test --workspace`, and `make gate`.
  - Acceptance: The full gate passes. If an intentional language divergence
    invalidates a legacy test, the test, language fixture, and documentation are
    updated in the same change.

- [ ] R093 Update the next refactor plan
  - Purpose: Preserve any remaining quality gaps as actionable work.
  - Approach: Convert findings from R084-R088 into new tasks or a follow-up
    plan.
  - Acceptance: Every unresolved runtime-level quality gap has an owner, a
    reason, and a concrete next step.
