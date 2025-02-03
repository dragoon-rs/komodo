NU="nu"
NU_FLAGS="--no-config-file"

NU_ARCH="x86_64-unknown-linux-musl"
NU_VERSION="0.101.0"
NU_BUILD="nu-${NU_VERSION}-${NU_ARCH}"
NU_DEST="/tmp/"

DEFAULT_GOAL: fmt-check check clippy test-rs

.PHONY: fmt-check
fmt-check:
	cargo fmt --all -- --check

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: install-nu
install-nu:
	mkdir -p "${NU_DEST}"
	curl -fLo "${NU_DEST}/nu.tar.gz" "https://github.com/nushell/nushell/releases/download/${NU_VERSION}/${NU_BUILD}.tar.gz"
	tar xvf "${NU_DEST}/nu.tar.gz" --directory "${NU_DEST}"
	cp "${NU_DEST}/${NU_BUILD}/nu" "${NU_DEST}/nu"

.PHONY: check
check:
	${NU} ${NU_FLAGS} scripts/check-nushell-files.nu
	cargo check --workspace --all-targets
	cargo check --workspace --all-targets --features kzg
	cargo check --workspace --all-targets --features aplonk
	cargo check --workspace --all-targets --all-features

.PHONY: clippy
clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

.PHONY: test-rs
test-rs:
	cargo test --workspace --verbose --all-features
	cargo test --examples --verbose

.PHONY: test-nu
test-nu:
	${NU} ${NU_FLAGS} bins/saclin/tests/cli.nu
	${NU} ${NU_FLAGS} bins/saclin/tests/binary.nu

.PHONY: test
test: test-rs test-nu

.PHONY: example
example:
	${NU} ${NU_FLAGS} bins/saclin/examples/cli.nu

.PHONY: show
show:
	@rustup --version 2> /dev/null
	@rustup show active-toolchain
	@rustc --version
	@cargo --version
	@cargo clippy --version
	@${NU} ${NU_FLAGS} --commands "version"

.PHONY: doc
doc:
	cargo doc --document-private-items --no-deps --open

.PHONY: build-examples
build-examples:
	cargo build --examples --release

print-%:
	@echo $($*)
