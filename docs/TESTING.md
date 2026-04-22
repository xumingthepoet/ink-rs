# Testing Strategy

## Goals

- Verify parser behavior in isolation.
- Verify parsed hierarchy behavior before runtime export.
- Verify generated JSON loads in `ink_runtime`.
- Verify runtime behavior for representative stories.
- Preserve compatibility with the official C# compiler where practical.

## Test Layers

### Unit Tests

Use unit tests for low-level parser state, character helpers, parsed nodes, and
diagnostic formatting.

Examples:

- Parser rollback restores index and line state.
- `CharacterSet` includes and excludes expected ranges.
- Parsed object traversal finds descendants in C# order.

### Parser Golden Tests

Use small `.ink` snippets and expected parsed summaries. Keep summaries stable
and focused; do not require full debug dumps for every test.

Examples:

- Plain text.
- Comments.
- Knots and stitches.
- Diverts.
- Choices.

### JSON Export Tests

For compiler export, compare generated JSON against either:

- A small hand-approved golden fixture.
- Official compiler output for the same input.
- A normalized JSON shape when ordering is not meaningful.

### Runtime Smoke Tests

Load generated JSON with `ink_runtime::story::Story::new` and execute simple
stories to verify observable output.

Examples:

- Plain text continues to expected lines.
- Choice count and choice text match expected values.
- Selecting a choice diverts to expected content.

### Conformance Tests

Later milestones should reuse local fixtures from:

- `ink-runtime/conformance-tests/inkfiles`
- `ink-csharp/tests`

Start with a narrow subset and expand only after the compiler pipeline is
stable.

## Validation Commands

Default checks:

```sh
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
```

Narrow checks should be used while developing a feature:

```sh
cargo test -p ink-compiler parser
cargo test -p ink-compiler parsed
cargo test -p ink-compiler runtime_export
```

## Golden Fixture Policy

- Keep fixtures small and readable.
- Name fixtures after the ink feature being tested.
- Store expected output in the test tree, not in ignored reference directories.
- When a golden changes, explain why in `DOCUMENTATION.md`.

## Failure Policy

If a test fails after a change:

1. Determine whether the failure is caused by the current diff.
2. Fix current-diff failures immediately.
3. If unrelated, document the existing failure in `DOCUMENTATION.md`.
4. Do not mark the milestone task complete while relevant validation is failing.

## Bug Reproduction Policy

When a bug is found during porting:

- Prefer writing a failing test before changing implementation.
- Keep the test focused on the smallest ink snippet or parsed structure that
  reproduces the bug.
- After fixing, keep the test as a regression.
- Record the issue and fix summary in `DOCUMENTATION.md`.

## Determinism Policy

Determinism matters for compiler outputs and diagnostics:

- Sort diagnostics by source position when aggregating.
- Normalize JSON before comparing golden output when object key order is not
  meaningful.
- Avoid user-visible output based on hash-map iteration order.
- Prefer stable fixture names and explicit expected outputs.
