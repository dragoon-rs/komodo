DEFAULT_GOAL: fmt-check check clippy test

.PHONY: fmt-check
fmt-check:
	cargo fmt --all -- --check

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: check
check:
	cargo check --workspace --all-targets
	cargo check --workspace --all-targets --features kzg
	cargo check --workspace --all-targets --features aplonk
	cargo check --workspace --all-targets --all-features

.PHONY: clippy
clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

.PHONY: test
test:
	cargo test --workspace --verbose --all-features
	cargo test --examples --verbose

.PHONY: show
show:
	@rustup --version 2> /dev/null
	@rustup show active-toolchain
	@rustc --version
	@cargo --version
	@cargo clippy --version

.PHONY: doc
doc:
	RUSTDOCFLAGS="--html-in-header katex.html" cargo doc --no-deps --open

.PHONY: build-examples
build-examples:
	cargo build --examples --release

print-%:
	@echo $($*)
