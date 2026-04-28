# Multiline Composite Literals Are Not Accepted In Declarations

Status: found

Found while: discussing dynamic choice option arrays

Scope: `crates/ink-compiler/src/syntax/variable.rs`,
`crates/ink-compiler/src/syntax/declaration.rs`,
`crates/ink-compiler/src/syntax/mod.rs`,
`crates/ink-compiler/src/syntax/expression.rs`,
`crates/ink-test/tests/language.rs`

Problem: Array and struct literals are supported as expressions, but declaration
initializers are parsed from only the current line. As a result, `VAR`, `CONST`,
and `~ temp` declarations that initialize long array or struct values must keep
the whole composite literal on one line. A readable multi-line value, such as an
array of option structs for dynamic choices, is rejected or split into unrelated
source lines.

Why it matters: Data-oriented story declarations become hard to read, review,
and maintain once composite values contain several fields or nested values. This
is especially awkward for dynamic choice tables, where each option may need text,
enabled state, and follow-up behavior metadata.

Suggested fix: Extend declaration initializer parsing to collect a balanced
multi-line expression body before invoking the expression parser. Cover `VAR`,
`CONST`, `~ temp`, array literals, struct literals, nested composite literals,
trailing commas if the language wants them, and clear diagnostics for unclosed
delimiters. Update parser/language tests and the maintained writing guide in the
same change.

Evidence: `declaration_statement` in
`crates/ink-compiler/src/syntax/variable.rs`, `temp_declaration_statement` in
the same file, and `constant_statement` in
`crates/ink-compiler/src/syntax/declaration.rs` all call
`parse_expression_remainder` after `=`. `parse_expression_remainder` in
`crates/ink-compiler/src/syntax/mod.rs` parses `parser.line_remainder()`, so the
expression source is limited to one physical line. The expression parser already
has inline `parse_array_literal` and `parse_struct_literal` support in
`crates/ink-compiler/src/syntax/expression.rs`, and existing language tests in
`crates/ink-test/tests/language.rs` cover one-line composite literals such as
`VAR party: Player[] = [{ hp: 10, name: "Ada" }]` and
`CONST party: Stats[] = [{ hp: 1 }, {}]`.
