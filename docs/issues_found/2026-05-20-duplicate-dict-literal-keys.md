# Duplicate Dict Literal Keys Are Not Diagnosed

Status: found

Found while: Task 08 lower Dict defaults and literals

Scope: `crates/ink-compiler/src/analysis/dict_literals.rs`,
`crates/ink-compiler/src/lower/value.rs`

Problem: Dict literals can contain duplicate keys, but analysis does not report
them. Lowering stores entries in a `BTreeMap`, so duplicate keys collapse to one
entry in compiled story JSON.

Why it matters: Silent duplicate-key collapse can hide authored mistakes and
make source order decide which value survives without an explicit language
contract.

Suggested fix: Add duplicate-key tracking to the Dict literal analysis pass and
emit a focused diagnostic before lowering. Cover both string and int keys.

Evidence: `DictLiteralChecker::check_dict_literal` currently validates key type
and value type only; `lower_dict_literal` inserts entries into a `BTreeMap`.
