---
title: 'Komodo: Cryptographically-proven Erasure Coding'
tags:
  - Rust
  - cryptography
  - erasure codes
  - distributed systems
  - data availability sampling
authors:
  - name: Antoine Stevan
    orcid: 0009-0003-5684-5862
    equal-contrib: true
    affiliation: 1
  - name: Jonathan Detchart
    orcid: 0000-0002-4237-5981
    equal-contrib: true
    affiliation: 1
  - name: Tanguy Pérennou
    orcid: 0009-0002-2542-0004
    equal-contrib: false
    affiliation: 1
  - name: Jérôme Lacan
    orcid: 0000-0002-3121-4824
    equal-contrib: false
    affiliation: 1
affiliations:
 - name: ISAE-SUPAERO, France
   index: 1
date: 01 January 1970
bibliography: paper.bib
---

# Abstract

We present **Komodo**, a library that allows to encode data using with
erasure-code techniques such as Reed-Solomon encoding, prove the resulting
shards with cryptographic protocols, verify their integrity on the other end
and decode the original data from a subset of shards.
The library is implemented in the _Rust_ programming language and
available on the ISAE-SUPAERO GitLab instance [^1] with a mirror on GitHub [^2].
**Komodo** should be of interest for people willing to explore the field of
cryptographically-proven shards of data in distributed systems or data
availability sampling.

[^1]: GitLab source code: [https://gitlab.isae-supaero.fr/dragoon/komodo](https://gitlab.isae-supaero.fr/dragoon/komodo)
[^2]: GitHub mirror for issues and pull requests: [https://github.com/dragoon-rs/komodo](https://github.com/dragoon-rs/komodo)

# Keywords

Cryptography; Erasure codes; Distributed systems; Data availability sampling;

# Summary

- encode data into shards with $(k, n)$ code
- prove all $n$ encoded shards with one of three cryptographic protocols
- verify any shard individually for its validity
- decode the original data using any subset of $k$ valid shards

the three methods:

- **KZG+**: Groth16 [@groth2016size],
  **KZG** [@kate2010constant; @boneh2020efficient]
- **aPlonK**: PlonK [@gabizon2019plonk] and **aPlonK** [@ambrona2022aplonk]
- **Semi-AVID**: **Semi-AVID** [@nazirkhanova2022information]

beta version used in the first performance evaluation paper
[@stevan2024performance] and available at
[https://gitlab.isae-supaero.fr/dragoon/pcs-fec-id](https://gitlab.isae-supaero.fr/dragoon/pcs-fec-id).

compare to Arkworks [@arkworks] and
[https://github.com/arkworks-rs](https://github.com/arkworks-rs) and Tezos
[@tezos2024aplonk].

mention Merkle trees [@merkle1987digital] and Fiat-Shamir [@fiat1986prove]?

## Implementation and architecture

show the code tree?

no variants or associated implementation differences.

## Quality control

all matrix operations are tested.

the encoding and decoding is tested.

all steps of the three protocols are tested.

run `make check clippy test`.

examples for the three protocols are provided in `examples/`.

```rust
use ark_bls12_381::{Fr as F, G1Projective as G};
use ark_poly::univariate::DensePolynomial as DP;

let mut rng = ark_std::test_rng();

let (k, n) = (3, 6_usize);
let bts = ...;

let ps = komodo::zk::setup::<F, G>(bts.len(), &mut rng).unwrap();

let m = &komodo::algebra::linalg::Matrix::random(k, n, &mut rng);
let ss = komodo::fec::encode(&bts, m).unwrap();
let p = prove::<F, G, DP<F>>(&bts, &ps, m.height).unwrap();
let bs = build::<F, G, DP<F>>(&ss, &p);

for b in &bs {
    assert!(verify::<F, G, DP<F>>(b, &ps).unwrap());
}

let s = bs[0..k].iter().cloned().map(|b| b.shard).collect();
assert_eq!(bts, komodo::fec::decode(ss).unwrap());
```

a more complete CLI application of **Semi-AVID** is available in `bins/saclin/`.

# Statement of need

the use case is any system that meet the following criteria

- distributed, e.g. drones
- need for data robustness, e.g. by introducing redundancy
- no trust in others nor the environment, need to prove the integrity of the data

Scroll [@scroll2024], Avail [@avail2024] and Danksharding [@danksharding2024].

**Komodo** can be extended with either

new encoding methods in the `fec` module
proof protocols, just as with the `kzg`, `aplonk` and `semi_avid` modules

contact us at `firstname.lastname@isae-supaero.fr` or at one of the
_support pages_ below

**support**: we provide support on GitHub

- bug reports and feature requests [https://gitlab.isae-supaero.fr/dragoon/komodo/-/issues](https://gitlab.isae-supaero.fr/dragoon/komodo/-/issues)
- contributions [https://gitlab.isae-supaero.fr/dragoon/komodo/-/merge_requests](https://gitlab.isae-supaero.fr/dragoon/komodo/-/merge_requests)

# Availability
## Operating system

Linux, untested on other OSs

## Programming language

install _Cargo_ [^3], e.g. with _rustup_ [^4]

[^3]: _Cargo_: [https://doc.rust-lang.org/cargo/](https://doc.rust-lang.org/cargo/)
[^4]: `rustup`: [https://rustup.rs/](https://rustup.rs/)

exact version taken care of by `rust-toolchain.toml`

## Additional system requirements

depends on the data for the memory usage

## Dependencies

all taken care of by Cargo and `Cargo.toml`

## Software location:

**Code repository**: GitLab

- Name: **Komodo**
- Persistent identifier: [https://gitlab.isae-supaero.fr/dragoon/komodo](https://gitlab.isae-supaero.fr/dragoon/komodo)
- Licence: MIT
- Date published: 05/11/2024

**Mirror**: GitHub

- Name: **Komodo**
- Persistent identifier: [https://github.com/dragoon-rs/komodo](https://github.com/dragoon-rs/komodo)
- Licence: MIT
- Date published: 05/11/2024

## Language

all written in english

# Acknowledgements

This work was supported by the Defense Innovation Agency (AID) of the French
Ministry of Defense through the Research Project DRAGOON: Dependable distRibuted
storAGe fOr mObile Nodes (2022 65 0082).

# References
