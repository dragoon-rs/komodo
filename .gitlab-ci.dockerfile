FROM rust:1.78

# Suppress prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

RUN apt update --yes && apt upgrade --yes

RUN apt install --yes protobuf-compiler

RUN rustup toolchain install 1.78
RUN rustup component add clippy rust-analyzer rustfmt
RUN rustup show
RUN rustup show active-toolchain

RUN cargo install cargo-script

RUN apt clean && rm -rf /var/lib/apt/lists/*
