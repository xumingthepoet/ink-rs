# Expression String Inline Tokens Parse As Story Content
Status: solved
Found while: adding `text-snake-10x10` under `crates/ink-experiments`
Scope: `crates/ink-compiler/src/syntax/expression/string.rs`

Problem: Quoted expression strings reused the story inline text parser, so `"#"` inside an expression string was parsed as tag begin/end tokens instead of a string value. In `text + "#"`, the comparison `x >= width + 2` only exposed the bug by reaching the recursive branch that lowered `"#"` as `str`, `#`, `/#`, `/str`.

Why it matters: Expression strings must be logic/expression values. Story-only inline text syntax such as tags, glue, and inline diverts should not be active inside quoted expression strings, or ordinary string concatenation can corrupt the runtime evaluation stack.

Resolution: Expression string parsing now uses a string-specific parser that only recognizes `{...}` expression interpolation. `#`, `<>`, `->`, and `<-` remain literal string text. Added regression coverage for the `border_row` repro and JSON lowering sequence `str`, `^#`, `/str`.

Validation: `cargo test -p ink-compiler syntax::expression::string -- --nocapture`; `cargo test -p ink-test --test integration expressions -- --nocapture`; `cargo test -p ink-experiments playthrough_snapshots_match_when_present -- --nocapture`; `make gate`.
