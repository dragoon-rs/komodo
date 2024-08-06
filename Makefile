.PHONY: fmt fmt-check check clippy test-rs test-nu test example show build-examples

DEFAULT_GOAL: fmt-check check clippy test-rs

fmt-check:
	cargo fmt --all -- --check

fmt:
	cargo fmt --all

check:
	cargo check --workspace --all-targets
	cargo check --workspace --all-targets --features kzg
	cargo check --workspace --all-targets --features aplonk
	cargo check --workspace --all-targets --all-features

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test-rs:
	cargo test --workspace --verbose --all-features
	cargo test --examples --verbose

test-nu:
	nu bins/saclin/tests/cli.nu
	nu bins/saclin/tests/binary.nu

test: test-rs test-nu

example:
	nu bins/saclin/examples/cli.nu

show:
	rustup --version
	rustup show --verbose
	rustc --version
	cargo --version
	cargo clippy --version
	nu --version

doc:
	cargo doc --document-private-items --no-deps --open

build-examples:
	cargo build --examples --release
