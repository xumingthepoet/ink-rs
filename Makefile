.PHONY: fmt check test gate

fmt:
	cargo fmt --all --check

check:
	cargo check --workspace

test:
	cargo test --workspace

gate: fmt check test
