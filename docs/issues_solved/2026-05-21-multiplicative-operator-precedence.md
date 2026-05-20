# Multiplicative Operators Use Nonstandard Precedence
Status: solved

Found while: reviewing arithmetic expression parsing after the compiler
analysis refactor.

Scope: `crates/ink-compiler/src/syntax/expression.rs`,
`crates/ink-test/fixtures/expressions/arithmetic.ink`, and maintained syntax
documentation.

Problem: The expression parser assigned different precedences to `*`, `/`,
`mod`, and `%`. This made `a * b / c` parse as `a * (b / c)` instead of the
conventional `(a * b) / c`.

Why it matters: Authors expect same-precedence multiplicative operators to
associate left-to-right. The old behavior silently produced wrong integer
arithmetic for common percentage formulas such as `8 * 100 / 56`.

Fix: `*`, `/`, `mod`, and `%` now use the same Pratt-parser precedence and keep
left associativity. Parser snapshots pin mixed chains such as `8 * 4 / 2`,
`8 / 4 * 2`, and `14 mod 5 % 3`; the arithmetic runtime fixture pins integer
percentage and mixed remainder outputs.

Validation: `cargo test -p ink-compiler syntax::expression` passed; `cargo test
-p ink-test --test integration arithmetic_fixture_runs -- --nocapture` passed;
`make gate` passed.

Evidence: Before the fix, `crates/ink-compiler/src/syntax/expression.rs` gave
`*` precedence `6`, `/` precedence `7`, `mod` precedence `8`, and `%`
precedence `9`. The old table parsed `{a * b / c}` as `a * (b / c)`.
