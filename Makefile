.PHONY: fmt check test gate csharp-gate

TIMEOUT ?= timeout
CSHARP_TEST ?=

fmt:
	cargo fmt --all --check

check:
	cargo check --workspace

test:
	$(TIMEOUT) 30s cargo test --workspace

csharp-gate:
	$(TIMEOUT) 30s cargo test -p ink-test --features csharp-tests --test csharp_tests $(CSHARP_TEST)

gate: fmt check test
	$(TIMEOUT) 30s cargo test -p ink-test --features compiler-conformance --test compiler_conformance
