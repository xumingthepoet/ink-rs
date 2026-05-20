# Validation Workflow

Use the smallest relevant validation first, then widen coverage. `make gate` is
the single project-level gate.

## Focused Validation

Choose focused commands that match the changed surface:

```text
cargo fmt --all --check
cargo check --workspace
cargo test -p ink-compiler <focused_filter>
cargo test -p ink-runtime <focused_filter>
cargo test -p ink-test --test integration <focused_filter>
cargo test -p ink-test --test integration integration_policy
cargo test -p ink-test
```

When touching compiler logic, favor focused runs in `crates/ink-test` before
running the full workspace. `ink-test` compiles its behavior tests as the
single `integration` Cargo test target; pass a module or test-name substring
after the target to run a focused behavior group. A package-level filter such
as `cargo test -p ink-test diagnostics` selects tests by test name substring and
can legitimately filter out most integration tests.

## Project Gate

`make gate` runs:

```text
cargo fmt --all --check
cargo check --workspace
cargo test --workspace --quiet
```

The Makefile wraps workspace tests with `tools/run-with-timeout` and the
`UNIT_TEST_TIMEOUT` setting. `--quiet` reduces log noise; it does not filter
tests.

Crates without runnable Rust doc examples disable Cargo's doctest harness in
their manifests so workspace tests keep runnable doctest coverage without paying
rustdoc startup cost for zero-doctest crates. Library and binary targets with no
unit tests set `test = false`; crates with integration tests still compile their
libraries as dependencies, but Cargo does not build empty lib/bin test harnesses.
`ink-test` policy coverage fails the gate if a disabled harness target gains
unit-test markers.
`ink-runtime` keeps its runnable doctest coverage enabled.

## Required Coverage

For the current compiled-story format refactor, the minimum required validation
before marking a change done is:

- focused tests that cover the changed format data, JSON serialization, JSON
  deserialization, compiler output, or runtime loading behavior
- `make gate`

If intentional language changes make upstream-derived scenarios obsolete,
update or replace the current ink-rs fixtures as part of the same
language-change work rather than hiding failures.
