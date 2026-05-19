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

## Bug Rule

Do not work around a compiler or runtime bug in experiment ink. Stop at the
first real bug, record it under `docs/issues_found/`, and continue only after
the bug is scheduled or fixed.
