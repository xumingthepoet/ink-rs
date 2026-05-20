.PHONY: fmt check test gate

TIMEOUT ?= ./tools/run-with-timeout
UNIT_TEST_TIMEOUT ?= 600s

fmt:
	cargo fmt --all --check

check:
	cargo check --workspace

test:
	$(TIMEOUT) $(UNIT_TEST_TIMEOUT) cargo test --workspace --quiet

gate: fmt check test
