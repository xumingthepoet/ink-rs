.PHONY: fmt check test gate

TIMEOUT ?= ./tools/run-with-timeout
UNIT_TEST_TIMEOUT ?= 30s
INK_TEST_TIMEOUT ?= 120s

fmt:
	cargo fmt --all --check

check:
	cargo check --workspace

test:
	$(TIMEOUT) $(UNIT_TEST_TIMEOUT) cargo test --workspace --exclude ink-test
	$(TIMEOUT) $(UNIT_TEST_TIMEOUT) cargo test -p ink-test --test integration_policy
	$(TIMEOUT) $(INK_TEST_TIMEOUT) cargo test -p ink-test

gate: fmt check test
