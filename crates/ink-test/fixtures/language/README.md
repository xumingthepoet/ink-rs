# Language Fixtures

This directory contains Rust-first language behavior fixtures.

Use these fixtures for current Rust-first language behavior: supported syntax,
semantics, and diagnostics that define the maintained language. Runnable
fixtures use explicit `module` declarations and a single `main` knot. Legacy C#
compatibility coverage remains in `fixtures/csharp_tests` and should not be
treated as the default source of truth for new language design.

The test helpers in `tests/language.rs` can compile fixtures, run compiled
stories, and assert diagnostics.

## Fixture Groups

### `typed/*.ink`

These fixtures cover explicit typed value declarations, structs, arrays, nested
arrays, arrays of structs, typed functions, typed externals, recursive equality,
`LEN`, `ARRAY_REMOVE`, and direct self tail recursion lowering.
