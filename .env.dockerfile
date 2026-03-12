FROM rust:latest

# Suppress prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

RUN apt update --yes && apt upgrade --yes
RUN apt install --yes protobuf-compiler

COPY rust-toolchain.toml /
RUN rustup show && cargo --version

RUN cargo +1.74 install rust-script --locked --rev 99d2c790b303c1d75de5cd90499800283e4b9681 --git https://github.com/fornwall/rust-script

RUN apt clean && rm -rf /var/lib/apt/lists/*
