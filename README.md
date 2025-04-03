# Komodo: Cryptographically-proven Erasure Coding

[![release](https://gitlab.isae-supaero.fr/dragoon/komodo/-/badges/release.svg)](https://gitlab.isae-supaero.fr/dragoon/komodo/-/releases)
[![crate](https://img.shields.io/crates/v/komodo)](https://crates.io/crates/komodo)
[![docs](https://img.shields.io/docsrs/komodo)](https://docs.rs/komodo/latest/komodo/)
[![source](https://gitlab.isae-supaero.fr/dragoon/komodo/badges/main/pipeline.svg?key_text=GitLab%20CI)](https://gitlab.isae-supaero.fr/dragoon/komodo/-/pipelines)
[![mirror](https://github.com/dragoon-rs/komodo/actions/workflows/ci.yml/badge.svg)](https://github.com/dragoon-rs/komodo/actions)

## the library
see `cargo doc` or [the library itself](src/)

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

## contributors

Because the code for this project has been originally extracted from
[`dragoon/pcs-fec-id`][pcs-fec-id] by @a.stevan, some contributors might be
missing from the Git tree.

Please consult the original Git tree from [`dragoon/pcs-fec-id`][pcs-fec-id] or
note that the following people have contributed to this code base:
- @a.stevan
- @j.detchart

[pcs-fec-id]: https://gitlab.isae-supaero.fr/dragoon/pcs-fec-id
