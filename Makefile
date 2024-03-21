.PHONY: fmt check clippy test example show

DEFAULT_GOAL: fmt check clippy test

fmt:
	cargo fmt --all -- --check

check:
	cargo check --workspace --all-targets

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace --verbose
	nu tests/cli.nu
	nu tests/binary.nu

example:
	nu examples/cli.nu

show:
	rustup --version
	rustup show --verbose
	rustc --version
	cargo --version
	cargo clippy --version
