.PHONY: fmt check test gate compiler-gate csharp-gate

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
	$(COMPILER_TIMEOUT) 30s cargo test -p ink-test --features compiler-conformance --test compiler_conformance $(COMPILER_TEST)

csharp-gate:
	$(COMPILER_TIMEOUT) 30s cargo test -p ink-test --features csharp-tests --test csharp_tests $(COMPILER_TEST)

gate: fmt check test
