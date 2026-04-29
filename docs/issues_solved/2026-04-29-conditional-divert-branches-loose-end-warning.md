# Conditional Divert Branches Still Report Loose End
Status: solved
Found while: tracking user bug reports from 2026-04-29
Scope: crates/ink-compiler/src/analysis/flow.rs, crates/ink-compiler/src/lower/conditional.rs
Problem: A conditional block whose branches all divert can still produce an apparent loose-end warning after the block. The compiler currently requires an explicit post-conditional terminator such as `-> DONE`, `-> END`, a choice, or another divert even when every authored branch already leaves the current flow.
Why it matters: The warning is noisy for authors and can obscure real loose ends. It also encourages redundant `-> DONE` statements after conditionals that are already exhaustive by authored intent.
Suggested fix: Teach flow analysis to recognize conditionals whose reachable branches all terminate via divert, `-> DONE`, `-> END`, or equivalent flow-ending constructs. Add focused tests for all-branch diverts, non-exhaustive conditionals, and conditionals with choices or mixed fallthrough behavior. If the current behavior is intentional, document the required explicit post-conditional terminator in the maintained guide.
Evidence: Repro source:

```ink
=== module game ===
VAR done: bool = true

== main ==
{done:
    - true:
        -> finish
    - false:
        -> finish
}

== finish ==
-> END
```

`cargo run -q -p ink-tools -- <repro>` exits successfully but reports `/dev/fd/11:7:9: Apparent loose end exists where the flow runs out. Do you need a '-> DONE' statement, choice or divert?`. The generated JSON includes branch diverts to `game.finish`, followed by rejoin/fallthrough structure.
Owner decision: Treat exhaustive terminating conditionals as flow-ending instead of requiring a redundant post-block terminator.
Resolution: Flow analysis now treats a conditional as terminating when every branch terminates and the conditional is exhaustive by `else`, or by a bool switch that explicitly covers both `true` and `false`. Non-exhaustive int/string/struct switches still require `else` or a following terminator.
Follow-up: The keywordless bool switch spelling was later removed as part of the explicit multiline control syntax change. The current migration target for the repro is `{ switch done: - true: ... - false: ... }`.
Validation: `cargo test -p ink-compiler analysis::flow -- --nocapture`; `cargo test -p ink-test --test conditionals -- --nocapture`; `cargo fmt --all --check`; `cargo test -p ink-test --test integration_policy`; `make gate TIMEOUT='f(){ shift; "$$@"; }; f'`.
