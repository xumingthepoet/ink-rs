# ink-experiments

This crate owns executable experiments for the current ink-rs language surface.
It is for practical, uncommon ink programs that demonstrate current syntax and
help reveal missing syntax, awkward authoring patterns, compiler bugs, or
runtime bugs.

## Experiment Files

Put each experiment in its own directory under `experiments/`. The Rust test
harness discovers `.ink` files recursively and requires every source file to
compile without diagnostics.

If an experiment should also pin runtime text output, add a sibling file named
after the source plus `.stdout`:

```text
experiments/my-case/story.ink
experiments/my-case/story.ink.stdout
```

The `.stdout` file is compared against `Story::continue_maximally()`.

Interactive experiments can add a sibling `.playthrough.json` file. Each step
continues the story, compares text output and current choice text, then
optionally selects a choice:

```json
[
  {
    "output": "Battle starts.\n",
    "choices": ["Strike", "Potion"],
    "choose": 0
  },
  {
    "output": "Choose target.\n",
    "choices": ["Slime A", "Slime B"],
    "choose": 1
  },
  {
    "output": "Rin uses Strike on Slime B.\n"
  }
]
```

## Source Style

Keep experiment ink readable enough to serve as syntax examples. Indent fields
inside `STRUCT` declarations with four spaces, matching `docs/SyntaxReference.md`.
The parser accepts flush-left fields, but experiments should prefer the
documented style.

## Bug Rule

Do not work around a compiler or runtime bug in experiment ink. Stop at the
first real bug, record it under `docs/issues_found/`, and continue only after
the bug is scheduled or fixed.

## Experiment Recap

After adding or updating an experiment, include a short recap in the final
response:

- what current ink-rs behavior the experiment proves
- what felt awkward, repetitive, or insufficiently general
- whether each limitation is syntax, runtime behavior, or local experiment
  shape
- possible future syntax or helper improvements
- whether a real bug was found and recorded under `docs/issues_found/`

Do not record design discomfort as a bug unless compiler or runtime behavior is
actually incorrect.
