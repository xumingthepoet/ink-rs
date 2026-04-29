# Switch-Like Condition Selector Is Rejected As Non-Bool
Status: solved
Found while: tracking user bug reports from 2026-04-29
Scope: crates/ink-compiler/src/analysis/flow.rs, crates/ink-compiler/src/lower/conditional.rs, docs/WritingWithInk-latest.md
Problem: Authors migrating from official Ink may write switch-like conditional syntax such as `{quest_stage: - 0: ...}`. The current parser/lowering has switch-like conditional handling, but flow analysis still checks the initial conditional expression as a boolean condition and reports `Conditional condition has type int but expected bool`.
Why it matters: The diagnostic makes the construct look like an invalid boolean conditional rather than a possibly unsupported or partially supported switch form. Migration is also easy to misunderstand because the maintained writing guide still contains upstream switch wording.
Suggested fix: Decide whether ink-rs supports official Ink-style switch conditionals. If supported, update flow analysis and tests so selector expressions are type-checked against branch values instead of requiring `bool`. If unsupported, emit a clear unsupported/removed-feature diagnostic for switch-like conditional syntax and remove or rewrite maintained documentation that implies support.
Evidence: Repro source:

```ink
=== module game ===
VAR quest_stage: int = 0

== main ==
{quest_stage:
    - 0:
        stage zero
}
-> END
```

`cargo run -q -p ink-tools -- <repro>` exits with compilation failure and reports `/dev/fd/11:7:1: Conditional condition has type int but expected bool`. Existing analysis tests in `crates/ink-compiler/src/analysis/flow.rs` cover non-bool conditional rejection, while `crates/ink-compiler/src/lower/conditional.rs` contains `switch_like` lowering logic.
Owner decision: Support official Ink-style switch conditionals in ink-rs.
Resolution: Flow analysis now distinguishes ordinary boolean conditionals from switch conditionals. Switch selectors may be non-bool, case values are checked for `==` comparability with the selector, and content before the first case is diagnosed as invalid switch fallback syntax.
Validation: `cargo test -p ink-compiler analysis::flow -- --nocapture`; `cargo test -p ink-test --test conditionals -- --nocapture`; `cargo fmt --all --check`; `cargo test -p ink-test --test integration_policy`; `make gate TIMEOUT='f(){ shift; "$$@"; }; f'`.
