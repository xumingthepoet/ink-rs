# Notes.md

This file owns durable working notes for `ink-rs`. Keep volatile note churn here
instead of in `AGENTS.md`.

## Note Purpose

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
- `last_helped` and `last_hurt` must be updated when the matching count changes.
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
- Archive deleted notes only when their history remains useful; otherwise remove
  them.

## Active Notes

- [N001] [created:2026-04-25] [helps:27] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:validation/compatibility]
  Legacy C# tests run through `make gate`. Keep them passing for unchanged
  legacy behavior, but when the language intentionally changes, update or
  replace obsolete compatibility tests instead of preserving C# parity.

- [N002] [created:2026-04-24] [helps:14] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:parsed-model/lowering]
  When a fixture exposes structural data, add it to the parsed model first, then
  lower JSON from that model. `Divert`, `Flow`, and `TunnelOnwards` carry their
  own arguments and targets instead of leaking flow semantics into export code.

- [N003] [created:2026-04-24] [helps:9] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:expressions/functions]
  Function calls and string expressions are expressions: lower call arguments
  before runtime command tokens. External declarations only register
  signatures. String expressions lower as `str ... /str` and can contain nested
  mixed text or logic.

- [N004] [created:2026-04-24] [helps:13] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:weave/lowering]
  Weave handling needs a current runtime-container model. Grouping derives base
  indentation from the first weave point. Gathers are structural weave points.
  Content after gathers stays in that gather, and named metadata remains the
  container tail.

- [N005] [created:2026-04-24] [helps:7] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:braced-content/parser]
  Braced inline or multiline content is not always a sequence. Try sequence
  annotations, then `condition: content` conditionals, then expression/default
  sequences. Multiline suffix content after `}` must still be parsed.

- [N006] [created:2026-04-25] [helps:15] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:diagnostics/constants/diverts]
  Diagnostics and compiler-pipeline behavior belong over parsed-model
  semantics. `CONST` declarations require explicit types; constant refs expand
  during expression lowering. Variable divert targets need `->`; expression
  divert targets resolving to variables omit `->`.

- [N007] [created:2026-04-24] [helps:14] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:parser/trial-order]
  Preserve C# trial order and type shape in syntax. Route parser alternatives
  through `RuleParser` or multiline checkpoints so failed trials rewind cursor,
  index, and diagnostics. Comment elimination is a source prepass.

- [N008] [created:2026-04-24] [helps:5] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:read-counts/paths]
  Read/turn-count resolution needs a story-level target index for flow and weave
  labels. Metadata pre-scans must preserve path-mode context. Count metadata
  belongs on the target container-for-counting, with local labels winning.

- [N009] [created:2026-04-24] [helps:5] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:expression-operators]
  Expression operators should parse into `Expression::Binary` and lower by
  emitting operands followed by the runtime native-function token. Split only at
  top-level operators and preserve left associativity for equal precedence.

- [N010] [created:2026-04-26] [helps:8] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:variables/scoping]
  `VAR` declarations are story-top-level only. Nested `VAR` is removed and must
  move to root or become typed `temp` state. Variable refs resolve like C#:
  closest flow args/temps, then globals.

## Candidate Notes

No current candidates.

## Archived Notes

No archived notes.
