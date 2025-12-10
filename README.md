# Komodo: Cryptographically-proven Erasure Coding

[![release](https://gitlab.isae-supaero.fr/dragoon/komodo/-/badges/release.svg)](https://gitlab.isae-supaero.fr/dragoon/komodo/-/releases)
[![crate](https://img.shields.io/crates/v/komodo)](https://crates.io/crates/komodo)
[![docs](https://img.shields.io/docsrs/komodo)](https://docs.rs/komodo/latest/komodo/)
[![source](https://gitlab.isae-supaero.fr/dragoon/komodo/badges/main/pipeline.svg?key_text=GitLab%20CI)](https://gitlab.isae-supaero.fr/dragoon/komodo/-/pipelines)
[![mirror](https://github.com/dragoon-rs/komodo/actions/workflows/ci.yml/badge.svg)](https://github.com/dragoon-rs/komodo/actions)

**Komodo** is a library that allows to encode data with erasure-code
techniques such as Reed-Solomon encoding, prove the resulting shards with
cryptographic protocols, verify their integrity on the other end of any
distributed network and decode the original data from a subset of said shards [@stevan2024performance] and [@stevan2023assessing].
The library is implemented in the _Rust_ programming language and
available on the ISAE-SUPAERO GitLab instance [^1] with a mirror on GitHub [^2],
both released under the MIT license.
**Komodo** should be of interest for people willing to explore the field of
cryptographically-proven shards of data in distributed systems or data
availability sampling settings.

**Komodo** provides a _Rust_ API to achieve the
following on any input data in a distributed network or setup:

- `encode`: encodes data into _shards_ with a $(k, n)$ code. This adds
  redundancy to the data, making the network more resilient to failure,
  fragmentation, partitioning, loss or corruption.
- `commit` and `prove`: generate cryptographic commitments and proofs for all
  $n$ encoded shards with one of three available cryptographic protocols (see
  below for more information). This step consists in attaching extra information
  to them and sharing augmented _blocks_ of data onto the network. This extra
  information should guarantee with a very high probability that a given shard
  has been generated indeed through an expected encoding process, namely a
  polynomial evaluation or vector inner-product encoding such as Reed-Solomon.
- `verify`: verifies any shard individually for its validity. This allows to
  discriminate invalid or corrupted shards without any decoding attempt. Without
  this shard-level verification step, it is impossible to know if a shard is
  valid until the decoding step. Then, when decoding fails, it is not
  possible to know which shards were invalid, leading to a _try-and-error_
  process that is not scalable.
- `decode`: decodes the original data using any subset of $k$ valid shards.

**Komodo** provides the three following cryptographic protocols:

- **KZG+**: based on [@kate2010constant] and its multi-polynomial extension [@boneh2020efficient]
- **aPlonK**: based on **PlonK** [@gabizon2019plonk] and **aPlonK** [@ambrona2023aplonk]
- **Semi-AVID**: based on **Semi-AVID-PR** [@nazirkhanova2022information]

**Komodo** is based on the Arkworks library [@arkworks] which provides
implementations of elliptic curves, fields and polynomial algebra.

[^1]: GitLab source code: [https://gitlab.isae-supaero.fr/dragoon/komodo](https://gitlab.isae-supaero.fr/dragoon/komodo)
[^2]: GitHub mirror for issues and pull requests: [https://github.com/dragoon-rs/komodo](https://github.com/dragoon-rs/komodo)

[@stevan2024performance]: https://doi.org/10.5220/0012377900003648
[@stevan2023assessing]: https://link.springer.com/chapter/10.1007/978-3-031-89518-0_13
[@kate2010constant]: https://doi.org/10.1007/978-3-642-17373-8_11
[@boneh2020efficient]: https://eprint.iacr.org/2020/081
[@gabizon2019plonk]: https://eprint.iacr.org/2019/953
[@ambrona2023aplonk]: https://doi.org/10.1007/978-3-031-41326-1_11
[@nazirkhanova2022information]: https://doi.org/10.1145/3558535.3559778

> [!tip]
> Komodo uses a build system entirely writen in Rust.
> - [`cargo-script`](https://crates.io/crates/cargo-script) to build the script
> - [`nob.rs`](https://gitlab.isae-supaero.fr/a.stevan/nob.rs) to run commands
> - [`clap`](https://crates.io/crates/clap) to provide a nice and complete build API
>
> First, [install `cargo-script`](https://github.com/DanielKeep/cargo-script#installation).
>
> Then, run the script with `./make.rs --help`

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

[pcs-fec-id]: https://gitlab.isae-supaero.fr/dragoon/pcs-fec-id
[gitlab.isae-supaero.fr:dragoon/komodo@containers]: https://gitlab.isae-supaero.fr/dragoon/komodo/container_registry/42
