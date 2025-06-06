# Komodo: Cryptographically-proven Erasure Coding

[![release](https://gitlab.isae-supaero.fr/dragoon/komodo/-/badges/release.svg)](https://gitlab.isae-supaero.fr/dragoon/komodo/-/releases)
[![crate](https://img.shields.io/crates/v/komodo)](https://crates.io/crates/komodo)
[![docs](https://img.shields.io/docsrs/komodo)](https://docs.rs/komodo/latest/komodo/)
[![source](https://gitlab.isae-supaero.fr/dragoon/komodo/badges/main/pipeline.svg?key_text=GitLab%20CI)](https://gitlab.isae-supaero.fr/dragoon/komodo/-/pipelines)
[![mirror](https://github.com/dragoon-rs/komodo/actions/workflows/ci.yml/badge.svg)](https://github.com/dragoon-rs/komodo/actions)

Komodo uses a build system entirely writen in Rust.
- [`cargo-script`](https://crates.io/crates/cargo-script) to build the script
- [`nob.rs`](https://gitlab.isae-supaero.fr/a.stevan/nob.rs) to run commands
- [`clap`](https://crates.io/crates/clap) to provide a nice and complete build API

First, [install `cargo-script`](https://github.com/DanielKeep/cargo-script#installation).

Then, run the script with `./make.rs --help`

## the library
```shell
./make.rs doc
```

## the tests
```shell
./make.rs check
./make.rs clippy
./make.rs test
```

Other examples that showcase the Komodo API are available in [`examples/`](examples/).

## the benchmarks
see [`benchmarks/`](benchmarks/README.md)

the results can be found in [`dragoon/komodo-benchmark-results`](https://gitlab.isae-supaero.fr/dragoon/komodo-benchmark-results).

## development

Komodo uses a Docker image as the base of the GitLab pipelines.

That means that there is nothing to build apart from the source code of Komodo itself when running jobs.

When the development environment needs to change, e.g. when the version of Rust is bumped in
[`rust-toolchain.toml`](./rust-toolchain.toml), one shall run the following commands to push the new
Docker image to the [_container registry_][gitlab.isae-supaero.fr:dragoon/komodo@containers].

```shell
./make.rs container --login
```
```shell
./make.rs container
```
```shell
./make.rs container --push
```

## contributors

Because the code for this project has been originally extracted from
[`dragoon/pcs-fec-id`][pcs-fec-id] by @a.stevan, some contributors might be
missing from the Git tree.

Please consult the original Git tree from [`dragoon/pcs-fec-id`][pcs-fec-id] or
note that the following people have contributed to this code base:
- @a.stevan
- @j.detchart

[pcs-fec-id]: https://gitlab.isae-supaero.fr/dragoon/pcs-fec-id
[gitlab.isae-supaero.fr:dragoon/komodo@containers]: https://gitlab.isae-supaero.fr/dragoon/komodo/container_registry/42
