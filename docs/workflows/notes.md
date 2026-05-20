# Durable Notes Workflow

`Notes.md` owns durable working notes for `ink-rs`. Keep volatile note churn
there instead of in `AGENTS.md`.

## Purpose

Notes are a ranked working-memory cache, not a popularity list. A note earns its
place only when it gives concrete help during compiler, runtime, format-refactor,
or language-evolution work.

## Entry Format

Use this format for every active or candidate note:

```md
- [N001] [created:YYYY-MM-DD] [helps:0] [hurts:0]
  [last_helped:never] [last_hurt:never] [scope:parser/lowering/runtime]
  Actionable note text.
```

- `helps` means the note materially helped a real task.
- `hurts` means the note caused confusion, wasted work, or a wrong direction.
- `last_helped` and `last_hurt` must be updated when the matching count
  changes.
- Use `unknown` only for migrated legacy counts whose event dates are not known.

## Active Note Ranking

Keep at most 10 active notes. Sort active notes by current engineering value,
using these factors in order:

1. Recent concrete help beats old total count.
2. Specific, actionable notes beat broad generic advice.
3. Hurt events carry more weight than help events.
4. Redundant notes should be merged instead of competing for slots.
5. Notes with `last_helped:unknown` must not outrank recently helpful notes
   based only on historical count.

Do not keep generic low-value guidance in the top 10 just because it collected
many helps. If a note is broadly true for any Rust project, move it into stable
instructions, merge it into another note, or delete it.

## Help Events

- A note may receive at most one help event per user task.
- A user task may give at most two total help events across all notes.
- Add a help only when the note changed an implementation choice, prevented a
  wrong change, explained a failing test, or directly selected useful
  validation.
- Do not add helps because a note merely looked related.
- Do not bulk-increment helps after a long task. Pick only the one or two notes
  with the clearest evidence.

## Hurt Events

- Add a hurt when a note misled the work, encouraged overfitting, hid a better
  design, wasted meaningful time, or contradicted the intended language model.
- A hurt should trigger action, not just reduce a counter:
  - `hurts:1`: rewrite the note or narrow its scope.
  - `hurts:2`: demote it to Candidate Notes unless it was already fixed.
  - `hurts:3+`: delete or archive it unless the user explicitly keeps it.
- When a hurt is added, update `last_hurt` and briefly rewrite the note so the
  same failure is less likely to recur.

## Lifecycle

- New notes start in Candidate Notes unless they immediately replaced or merged
  an active note.
- Promote a candidate only after it helps a real task or clearly supersedes an
  active note.
- Demote active notes that are stale, generic, redundant, or hurtful.
- Archive deleted notes only when their history remains useful; otherwise
  remove them.
