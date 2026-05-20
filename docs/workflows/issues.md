# Deferred Issue Workflow

Use deferred issue records for real problems discovered during implementation,
debugging, review, or validation that are useful but outside the current task
scope.

## When To File

File an issue under `docs/issues_found/` when you discover a design bug, real
defect, awkward API, unreasonable workflow, missing test coverage, obsolete
compatibility path, or clearly useful improvement that is outside the current
task scope.

Do not derail the current task just to fix a deferred issue. If the issue blocks
the current task or invalidates the current approach, handle it as part of the
current work instead of filing it as deferred.

Do not file speculative cleanups, personal style preferences, or issues that
are already fixed in the same change.

## File Rules

- Keep one issue per Markdown file.
- Use a stable, descriptive file name such as
  `YYYY-MM-DD-short-kebab-title.md`.
- Avoid generic names like `issue.md`.
- Write issue records in English.
- Keep each record concise but actionable enough that a later prompt can fix
  the issue without rediscovering all context.

## Template

```md
# Short Title

Status: found

Found while: <task or command>

Scope: <crate/module/files>

Problem: <what is wrong or inconvenient>

Why it matters: <risk, maintenance cost, or user impact>

Suggested fix: <first plausible repair path>

Evidence: <file paths, failing command, or observed behavior>
```

## Solving Issues

When an issue is fixed, move its Markdown file from `docs/issues_found/` to
`docs/issues_solved/`, change `Status: found` to `Status: solved`, and add the
fixing commit or validation evidence when available.

Mention any new files added under `docs/issues_found/` in the final response
for the task so the project owner can decide when to schedule a separate fix
prompt.
