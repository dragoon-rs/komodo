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

RUN rustup toolchain install 1.78
RUN rustup default 1.78
RUN rustup component add clippy rust-analyzer rustfmt

RUN cargo install cargo-script
