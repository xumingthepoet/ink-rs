# Writing with Ink Updates

This file records intentional language documentation changes made by ink-rs.

`WritingWithInk-origin.md` is the immutable upstream C# documentation snapshot.
Do not edit it for ink-rs language changes.

`WritingWithInk-latest.md` is the current ink-rs documentation:

```text
WritingWithInk-latest.md = WritingWithInk-origin.md + WritingWithInk-updates.md
```

When syntax or semantics change, update this file first, then apply the same
change to `WritingWithInk-latest.md`.

Each entry should include:

- date
- status: experimental, supported, deprecated, or removed
- upstream behavior
- ink-rs behavior
- documentation effect
- rationale
- migration guidance
- tests

## 2026-04-25: LIST Declarations Removed

- status: removed
- upstream behavior: upstream Ink supports `LIST` declarations for named list
  origins and list items, documented in the upstream "Advanced State Tracking"
  list sections.
- ink-rs behavior: `LIST` declarations produce a removed-feature diagnostic.
- documentation effect: `WritingWithInk-latest.md` removes the upstream list
  documentation from the main table of contents and body, and records the
  divergence in "Changed from upstream Ink".
- rationale: the Rust language surface is being reduced to features that are
  actively maintained and useful for the current project direction.
- migration guidance: use variables, functions, or host-side data for inventory
  and set-like game state until a replacement list design is added.
- tests: `removed_list_declaration_reports_removed_feature_diagnostic` in
  `crates/ink-test/tests/language.rs`.
