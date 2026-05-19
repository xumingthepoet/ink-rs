# Make Gate Ink Test Timeout Is Too Tight
Status: solved
Found while: diagnostics policy API validation
Scope: Makefile, tools/run-with-timeout, crates/ink-test test target
Problem: The default `make gate` uses `./tools/run-with-timeout 30s cargo test -p ink-test`, which can terminate with exit 124 even when the test suite is passing. In this run, the command reached later `ink-test` targets before timing out, while the same test command passed with a 120s timeout.
Why it matters: A passing change can look like a gate failure on normal local hardware, which wastes debugging time and weakens confidence in the required validation workflow.
Suggested fix: Increase the default timeout for the `ink-test` segment or split the `ink-test` suite into smaller timed commands that fit the current timeout reliably.
Evidence: `make gate` failed with `make: *** [test] Error 124`; `./tools/run-with-timeout 120s cargo test -p ink-test` passed afterward.

Resolution: The `Makefile` now gives the `ink-test` segment its own `INK_TEST_TIMEOUT ?= 120s` while keeping the other test segments on `UNIT_TEST_TIMEOUT ?= 30s`.

Validation: `make gate` completed successfully on 2026-05-20 after the timeout split.
