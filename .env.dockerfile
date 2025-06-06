FROM rust:latest

# Suppress prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

RUN apt update --yes && apt upgrade --yes
RUN apt install --yes protobuf-compiler

COPY rust-toolchain.toml /
RUN rustup show && cargo --version

RUN cargo install cargo-script

RUN apt clean && rm -rf /var/lib/apt/lists/*
