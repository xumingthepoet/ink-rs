.PHONY: fmt check test gate

TIMEOUT ?= timeout

fmt:
	cargo fmt --all --check

check:
	cargo check --workspace

test:
	$(TIMEOUT) 30s cargo test --workspace

gate: fmt check test
