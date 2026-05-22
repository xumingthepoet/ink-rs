# Notes.md

Durable working notes for `ink-rs`. Read these before relying on compiler,
runtime, format-refactor, or language-evolution memory. Maintenance rules live
in `docs/workflows/notes.md`.

## Active Notes

- [N001] [created:2026-04-25] [helps:27] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:validation/compatibility]
  Current-language scenarios are pinned by ink-rs fixtures covered by
  `make gate`. Keep those fixtures passing for unchanged behavior, but when the
  language intentionally changes, update or replace stale scenarios instead
  of preserving stale behavior.

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

- [N005] [created:2026-04-24] [helps:7] [hurts:1]
  [last_helped:unknown] [last_hurt:2026-04-29] [scope:braced-content/parser]
  Source sequences are not part of current ink-rs syntax. Braced inline or
  multiline content should parse conditionals and expressions, but top-level
  source alternatives such as `{one|two}`, `{~one|two}`, or `{ cycle: ... }`
  must emit an unsupported syntax diagnostic instead of creating parsed
  sequence nodes.

- [N006] [created:2026-04-25] [helps:15] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:diagnostics/constants/diverts]
  Diagnostics and compiler-pipeline behavior belong over parsed-model
  semantics. `CONST` declarations require explicit types; constant refs expand
  during expression lowering. Variable divert targets need `->`; expression
  divert targets resolving to variables omit `->`.

- [N007] [created:2026-04-24] [helps:14] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:parser/trial-order]
  Preserve proven parser trial order and syntax shape for unchanged behavior.
  Route parser alternatives through `RuleParser` or multiline checkpoints so
  failed trials rewind cursor, index, and diagnostics. Comment elimination is a
  source prepass.

- [N008] [created:2026-04-24] [helps:5] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:read-counts/paths]
  Read/turn-count resolution needs a story-level target index for flow and
  weave labels. Metadata pre-scans must preserve path-mode context. Count
  metadata belongs on the target container-for-counting, with local labels
  winning.

- [N009] [created:2026-04-24] [helps:5] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:expression-operators]
  Expression operators should parse into `Expression::Binary` and lower by
  emitting operands followed by the runtime native-function token. Split only at
  top-level operators and preserve left associativity for equal precedence.

- [N010] [created:2026-04-26] [helps:8] [hurts:0]
  [last_helped:unknown] [last_hurt:never] [scope:variables/scoping]
  `VAR` declarations are story-top-level only. Nested `VAR` is not supported
  and must move to root or become typed `temp` state. Variable refs resolve by
  current visibility order: closest flow args/temps, then globals.

## Candidate Notes

No current candidates.

## Archived Notes

No archived notes.
