# Compiled JSON Current-Only Format Break

Status: completed. Final validation passed with `make gate`.

## Purpose

This active plan turns compiled story JSON into a current ink-rs-only contract.
The project owner has explicitly allowed a compiled JSON format break. The goal
is to delete legacy and compatibility meaning from the format and remove the
compiler/runtime implementation that only exists to support those historical
forms.

ink-rs remains inspired by ink, but compatibility with Ink compiled JSON is not
a goal. Current language behavior and current compiled JSON are defined by this
repository.

## Owner Decisions

- A compiled story JSON format break is allowed.
- The current format should reject older format versions instead of migrating or
  decoding them.
- The format crate should stop representing historical control commands and
  count metadata.
- The compiler should stop emitting legacy terminal commands as structural
  placeholders.
- The runtime should stop executing compatibility fallbacks for removed compiled
  JSON commands.
- Diagnostic support for removed source syntax can remain where it gives
  authors a clear error, but compiled JSON compatibility paths should be
  deleted.

## Removed Residue

The initial audit found these current compatibility residues. The task list is
the closeout ledger for what was removed and how the remaining references were
classified.

- Compiled JSON control command tokens: `"done"`, `"end"`, `"thread"`,
  `"choiceCnt"`, `"turn"`, `"turns"`, `"readc"`, `"visit"`, and `"seq"`.
- Compiled JSON read-count object: `{"CNT?": ...}`.
- Compiled JSON container count flags: `"#f"` visit/turn/count-start metadata.
- Compiler lowering that still emits `ControlCommand::Done` or
  `ControlCommand::End` for root, global declarations, gathers, choice fallback
  paths, and helper containers.
- Runtime command variants and execution branches for legacy thread,
  choice-count, turn-count, read-count, visit-index, sequence-shuffle, done, and
  end behavior.
- Runtime callstack continuation support that exists only for legacy compiled
  `"thread"` command execution.
- Runtime sequence shuffle helper code and visit/turn/read-count fallback code.
- Tests, snapshots, docs, comments, and examples that describe historical
  compatibility as part of the current compiled story JSON contract.

## Target Contract

- `ink_story_json_format::INK_VERSION_CURRENT` is bumped for this breaking
  compiled story JSON change.
- The runtime loads only the new current compiled story JSON version.
- Removed tokens and objects are not represented in typed format data
  structures.
- Removed tokens and objects are rejected as unknown or invalid current JSON,
  not decoded into no-op or fallback runtime behavior.
- Containers no longer store count flags. Named content uses current container
  naming/path metadata only.
- Story root, knots, stitches, gathers, and ordinary choice aftermath end
  naturally by reaching the end of their lowered current container structure.
- Functions and tunnels remain explicit callstack constructs and still require
  the current return/onwards commands that belong to ink-rs.
- Dynamic and static choices keep their current behavior without relying on
  serialized pending choices or legacy thread continuations.
- Random and seed-random commands remain because they are current runtime
  functionality, not compatibility residue.
- Choice-point flags remain only where they represent current choice behavior.
  Do not remove them merely because container count flags are removed.

## Risk Areas

- Removing `"done"` and `"end"` changes the runtime's safe-exit path. The new
  natural-end behavior must distinguish normal story completion from missing
  `~ return` or `->->`.
- Removing count flags can affect named gather path serialization if any code
  was using `"#f": 4` as a name-preservation side effect.
- Choice and gather lowering previously used `done` as a stop marker. Those
  paths need explicit current structure or verified natural container endings so
  fallthrough stays correct.
- Deleting runtime compatibility branches may expose tests that were loading
  hand-written old JSON. Tests must be rewritten to current JSON or changed into
  rejection tests.
- Format snapshots are broad. Use focused validation while changing each layer,
  then run the full gate before closeout.

## Validation Strategy

Focused validation is required for every task before it can move to `[>]` or
`[x]`. `make gate` is required before the active plan is completed.

Use residue searches throughout the plan, including:

```sh
rg -n 'ControlCommand::(Done|End|StartThread|ChoiceCount|Turns|TurnsSince|ReadCount|VisitIndex|SequenceShuffleIndex)|LegacyStartThread|choiceCnt|readc|"\bturn\b"|turns|"\bvisit\b"|"\bseq\b"|"\bdone\b"|"\bend\b"|CNT\?|#f'
```

Search results are not automatically failures. They must be classified as one
of:

- current functionality that should remain;
- source diagnostic text for removed syntax;
- tests asserting rejection of removed compiled JSON;
- active or finished plan history;
- residue that must be deleted before closeout.

## Completion Conditions

- No current compiled story JSON emission contains removed commands, `CNT?`, or
  container `#f` metadata.
- The format crate has no typed representation for removed compiled JSON
  commands, `CNT?`, or container count flags.
- The runtime has no execution branch that implements removed compiled JSON
  compatibility.
- Runtime natural-end behavior passes focused tests for root, knot, stitch,
  gather, static choice, dynamic choice, function, and tunnel scenarios.
- Maintained docs describe only the current compiled story JSON format.
- `make gate` passes.
