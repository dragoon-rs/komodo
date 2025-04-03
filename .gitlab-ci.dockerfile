FROM rust:1.78

# Suppress prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

RUN apt update --yes && apt upgrade --yes

RUN apt install --yes protobuf-compiler
RUN cargo install cargo-script

RUN apt clean && rm -rf /var/lib/apt/lists/*
