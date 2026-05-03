.PHONY: fmt check test gate

TIMEOUT ?= ./tools/run-with-timeout

fmt:
	cargo fmt --all --check

check:
	cargo check --workspace

test:
	$(TIMEOUT) 30s cargo test --workspace --exclude ink-test
	$(TIMEOUT) 30s cargo test -p ink-test --test integration_policy
	$(TIMEOUT) 30s cargo test -p ink-test

gate: fmt check test
