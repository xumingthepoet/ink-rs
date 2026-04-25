.PHONY: fmt check test gate

TIMEOUT ?= timeout

fmt:
	cargo fmt --all --check

check:
	cargo check --workspace

test:
	$(TIMEOUT) 30s cargo test --workspace --exclude ink-test
	$(TIMEOUT) 30s cargo test -p ink-test --test conformance
	$(TIMEOUT) 30s cargo test -p ink-test --test compiler_conformance
	$(TIMEOUT) 30s cargo test -p ink-test --test language
	$(TIMEOUT) 30s cargo test -p ink-test --test inkling_examples
	$(TIMEOUT) 30s cargo test -p ink-test --features csharp-tests --test csharp_tests

gate: fmt check test
