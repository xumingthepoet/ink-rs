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
- Reason: why the new behavior exists.

The test helpers in `tests/language.rs` can compile fixtures, run compiled
stories, and assert diagnostics.

## Intentional Divergence Examples

### `removed-list.ink`

- Intent: keep removed legacy collection syntax out of the new language surface.
- Old behavior: upstream Ink accepts `LIST` declarations for named list values.
- New behavior: ink-rs reports `DiagnosticCode::RemovedFeature`.
- Reason: the current language direction favors variables, functions, or host
  data over carrying the legacy list declaration feature.
