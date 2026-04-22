.PHONY: fmt check test gate compiler-gate

TIMEOUT ?= timeout
COMPILER_TIMEOUT ?= $(TIMEOUT)
COMPILER_TEST ?=

fmt:
	cargo fmt --all --check

check:
	cargo check --workspace

test:
	$(TIMEOUT) 30s cargo test --workspace

compiler-gate:
	$(COMPILER_TIMEOUT) 30s cargo test -p ink-test --features compiler-conformance --test compiler_conformance_legacy $(COMPILER_TEST)

gate: fmt check test
