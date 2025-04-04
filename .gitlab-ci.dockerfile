FROM alpine:latest

RUN apk add --no-cache \
    curl \
    gcc \
    g++ \
    musl-dev \
    make \
    protobuf \
    protobuf-dev

# install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

COPY rust-toolchain.toml /
RUN rustup show && cargo --version

RUN cargo install cargo-script
