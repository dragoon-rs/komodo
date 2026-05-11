# Komodo: Cryptographically-proven Erasure Coding

[![release](https://gitlab.isae-supaero.fr/dragoon/komodo/-/badges/release.svg)](https://gitlab.isae-supaero.fr/dragoon/komodo/-/releases)
[![crate](https://img.shields.io/crates/v/komodo)](https://crates.io/crates/komodo)
[![docs](https://img.shields.io/docsrs/komodo)](https://docs.rs/komodo/latest/komodo/)
[![source](https://gitlab.isae-supaero.fr/dragoon/komodo/badges/main/pipeline.svg?key_text=GitLab%20CI)](https://gitlab.isae-supaero.fr/dragoon/komodo/-/pipelines)
[![mirror](https://github.com/dragoon-rs/komodo/actions/workflows/ci.yml/badge.svg)](https://github.com/dragoon-rs/komodo/actions)
[![DOI](https://joss.theoj.org/papers/10.21105/joss.09791/status.svg)](https://doi.org/10.21105/joss.09791)

Komodo uses a build system entirely writen in Rust.
- [`rust-script`](https://github.com/fornwall/rust-script) to build the script
- [`nob.rs`](https://gitlab.isae-supaero.fr/a.stevan/nob.rs) to run commands
- [`clap`](https://crates.io/crates/clap) to provide a nice and complete build API

First, install `rust-script` with
```bash
cargo +1.74 install rust-script --locked --rev 99d2c790b303c1d75de5cd90499800283e4b9681 --git https://github.com/fornwall/rust-script
```

Then, the executable script `make.rs` can be used to run commands
```bash
./make.rs --help
```

## the library
> [!tip]
> entry point of the local documentation is `./target/doc/komodo/index.html`

```bash
cargo doc                # raw documentation
cargo doc --all-features # raw documentation with optional features
./make.rs doc            # with the KaTeX formulae
./make.rs doc --features # with the KaTeX formulae and all the optional features
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

## contribute to Komodo
- open bug reports in the [issue tracker](https://github.com/dragoon-rs/komodo/issues)
- discuss the library and feature requests in the [discussion feed](https://github.com/dragoon-rs/komodo/discussions)
- submit changes to the [pull requests](https://github.com/dragoon-rs/komodo/pulls)

Even though [this repo](https://github.com/dragoon-rs/komodo) is a mirror of the
[original project](https://gitlab.isae-supaero.fr/dragoon/komodo), bugs and new
features can still be discussed and pull requests can be merged upstream by the
DRAGOON team.

## contributors

Because the code for this project has been originally extracted from
[`dragoon/pcs-fec-id`][pcs-fec-id] by @a.stevan, some contributors might be
missing from the Git tree.

Please consult the original Git tree from [`dragoon/pcs-fec-id`][pcs-fec-id] or
note that the following people have contributed to this code base:
- @a.stevan
- @j.detchart

## citation
If you use the `komodo` library in your research projects, please cite it using the following template:

```bibtex
@software{komodo@main,
  author  = {Komodo contributors},
  title   = {\texttt{komodo}: Cryptographically-proven Erasure Coding},
  url     = {https://gitlab.isae-supaero.fr/dragoon/komodo/-/tree/main},
  crate   = {},
  mirror  = {https://github.com/dragoon-rs/komodo/tree/main},
  docs    = {},
  version = {main},
  year    = {2023},
}
```

> [!tip] TIP
> mentions to `main` can be removed

Komodo has been accepted in the **J**ournal of **O**pen **S**ource **S**oftware (JOSS): [`10.21105/joss.09791`](https://joss.theoj.org/papers/10.21105/joss.09791)  
The JOSS paper can also be cited using [`CITATION.cff`](CITATION.cff).

[pcs-fec-id]: https://gitlab.isae-supaero.fr/dragoon/pcs-fec-id
[gitlab.isae-supaero.fr:dragoon/komodo@containers]: https://gitlab.isae-supaero.fr/dragoon/komodo/container_registry/42
