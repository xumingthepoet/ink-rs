# Language Fixtures

This directory contains Rust-first language behavior fixtures.

Use these fixtures for intentional language evolution: new syntax, changed
semantics, removed legacy syntax, and diagnostics that define the future
language. Legacy C# compatibility coverage remains in `fixtures/csharp_tests`
and should not be treated as the default source of truth for new language
design.

When a fixture intentionally diverges from upstream Ink, record:

- Intent: what language design goal this fixture protects.
- Old behavior: what upstream or previous Rust behavior did.
- New behavior: what this project now expects.

The test helpers in `tests/language.rs` can compile fixtures, run compiled
stories, and assert diagnostics.

