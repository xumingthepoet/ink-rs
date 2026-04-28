# Multiline Composite Literals Are Not Accepted In Declarations

Status: solved

Found while: discussing dynamic choice option arrays

Scope: `crates/ink-compiler/src/syntax/variable.rs`,
`crates/ink-compiler/src/syntax/declaration.rs`,
`crates/ink-compiler/src/syntax/mod.rs`,
`crates/ink-compiler/src/syntax/expression.rs`,
`crates/ink-test/tests/language.rs`

Problem: Array and struct literals were supported as expressions, but
declaration initializers were parsed from only the current line. As a result,
`VAR` and `CONST` declarations that initialized long array or struct values had
to keep the whole composite literal on one line. A readable multi-line value,
such as an array of option structs for dynamic choices, was rejected or split
into unrelated source lines.

Why it matters: Data-oriented story declarations became hard to read, review,
and maintain once composite values contained several fields or nested values.
This was especially awkward for dynamic choice tables, where each option may
need text, enabled state, and follow-up behavior metadata.

Suggested fix: Extend declaration initializer parsing to collect a balanced
multi-line expression body before invoking the expression parser. Cover
module-level `VAR`, `CONST`, array literals, struct literals, nested composite
literals, and clear diagnostics for unclosed delimiters. Update
parser/language tests and the maintained writing guide in the same change.

Evidence: `declaration_statement` in
`crates/ink-compiler/src/syntax/variable.rs` and `constant_statement` in
`crates/ink-compiler/src/syntax/declaration.rs` called
`parse_expression_remainder` after `=`. `parse_expression_remainder` in
`crates/ink-compiler/src/syntax/mod.rs` parses `parser.line_remainder()`, so the
expression source was limited to one physical line. The expression parser
already had inline `parse_array_literal` and `parse_struct_literal` support in
`crates/ink-compiler/src/syntax/expression.rs`, and existing language tests in
`crates/ink-test/tests/language.rs` covered one-line composite literals.

Resolution: Module-level `VAR` and `CONST` declarations now collect following
physical lines while the initializer has unclosed expression delimiters, then
reuse the existing declaration and expression parsers. `~ temp` initializers
were intentionally left as single-line logic syntax for this pass.

Validation:

- `cargo test -p ink-test --test language multiline_var_and_const_composite_literals_run_at_runtime -- --nocapture`
- `cargo test -p ink-test --test language multiline_temp_initializer_remains_single_line_syntax -- --nocapture`
- `make gate`
