# Implementation Runbook

Use this file whenever the user asks to continue, especially when the prompt is
just `继续`.

## Required Loop

1. Read `AGENTS.md`, `PLAN.md`, `DOCUMENTATION.md`, and this file.
2. Inspect the current Git status.
3. Select the first unchecked, unblocked task in `PLAN.md`.
4. Read only the C# and Rust reference files needed for that task.
5. State the selected task briefly to the user.
6. Implement the task with a scoped diff.
7. Add or update tests for the task.
8. Run the task-specific validation from `PLAN.md`.
9. If validation fails, repair failures before moving on.
10. Update `PLAN.md` checkboxes when acceptance criteria are met.
11. Update `DOCUMENTATION.md` with status, decisions, validation, and next task.
12. Commit a completed green milestone with a clear message unless the user has
    told you not to commit or unrelated user changes would be swept into the
    commit.
13. End with a concise summary and mention any validation not run.

For a full local verification pass, `make gate` is the preferred unified entry
point. It runs format, check, and test with workspace warnings denied.

The legacy compiler-to-runtime conformance suite is available behind the
`compiler-conformance` Cargo feature:

```sh
cargo test -p ink-test --features compiler-conformance --test compiler_conformance_legacy
```

## Non-Negotiable Continuation Rule

When the user says `继续`, do not stop after planning or after one read-only
analysis step. Continue through the selected task end-to-end:

- make the code or documentation change,
- validate it,
- repair failures,
- update durable project memory,
- and leave a clear next task.

Ask a question only when a reasonable technical decision cannot be made from
`AGENTS.md`, `PLAN.md`, existing code, or the local reference implementations.

## Stop-and-Fix Rule

Validation failure is not a reason to move to the next milestone. Fix it in the
same turn unless:

- The failure is unrelated to the current diff.
- The failure existed before the current task.
- The failure requires a product decision not captured in `AGENTS.md`.

If any exception applies, document it in `DOCUMENTATION.md` before stopping.

## Scope Control

- Do not port multiple compiler subsystems in one patch.
- Do not rewrite `blade-ink-rs` runtime code unless the selected task explicitly
  requires a small integration fix.
- Do not silently change public API names once tests or docs depend on them.
- Do not add large dependencies without recording the reason in
  `DOCUMENTATION.md`.
- Keep commits reviewable and milestone-scoped.
- Before committing, inspect `git status --short` and only stage files that
  belong to the current milestone.

## Bug Rule

If a bug is discovered while implementing:

- Add or update a test that reproduces the bug whenever feasible.
- Confirm the test fails for the expected reason before the fix when practical.
- Fix the bug.
- Confirm the test passes.
- Record a short note in `DOCUMENTATION.md`.

## Reference Reading Order

For each feature:

1. Read the relevant C# parser file under `ink-csharp/compiler/InkParser`.
2. Read the relevant parsed hierarchy file under
   `ink-csharp/compiler/ParsedHierarchy`.
3. Read any runtime type used by the generated object in
   `ink-csharp/ink-engine-runtime`.
4. Read the corresponding Rust runtime type under `blade-ink-rs/lib/src`.
5. Implement only the smallest Rust slice needed for the selected task.

## Documentation Update Format

After each task, update `DOCUMENTATION.md`:

- Current milestone and task.
- What changed.
- Commands run and results.
- Decisions made.
- Known issues.
- Next task.

Keep the audit log concise. It should help the next Codex run resume work
without reading the entire chat history.

## Completion Criteria for Each Turn

A turn is complete only when:

- The selected task is implemented or a concrete blocker is documented.
- Relevant tests exist or the reason for deferring tests is documented.
- Formatting/check/test validation has been run as far as practical.
- `PLAN.md` and `DOCUMENTATION.md` reflect the latest state.
- The next task is obvious from `PLAN.md` and `DOCUMENTATION.md`.
