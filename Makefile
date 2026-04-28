.PHONY: fmt check test gate

TIMEOUT ?= timeout

fmt:
	cargo fmt --all --check

check:
	cargo check --workspace

test:
	$(TIMEOUT) 30s cargo test --workspace --exclude ink-test
	$(TIMEOUT) 30s cargo test -p ink-test --test integration_policy
	$(TIMEOUT) 30s cargo test -p ink-test --test language

gate: fmt check test
