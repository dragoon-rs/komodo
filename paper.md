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

\usetikzlibrary{shapes,arrows,positioning,calc}

# Abstract

We present **Komodo**, a library that allows to encode data with erasure-code
techniques such as Reed-Solomon encoding, prove the resulting shards with
cryptographic protocols, verify their integrity on the other end of any
distributed network and decode the original data from a subset of said shards.
The library is implemented in the _Rust_ programming language and
available on the ISAE-SUPAERO GitLab instance [^1] with a mirror on GitHub [^2].
**Komodo** should be of interest for people willing to explore the field of
cryptographically-proven shards of data in distributed systems or data
availability sampling settings.

[^1]: GitLab source code: [https://gitlab.isae-supaero.fr/dragoon/komodo](https://gitlab.isae-supaero.fr/dragoon/komodo)
[^2]: GitHub mirror for issues and pull requests: [https://github.com/dragoon-rs/komodo](https://github.com/dragoon-rs/komodo)

# Keywords

Cryptography; Erasure codes; Distributed systems; Data availability sampling;

# Summary

**Komodo** is a software library that provides a _Rust_ API to achieve the
following on any input data in a distributed network or setup:

- `encode`: data is encoded into _shards_ with a $(k, n)$ code. This adds
  redundancy to the data, making the network more resilient to failure,
  fragmentation, partitioning, loss or corruption.
- `commit` and `prove`: all $n$ encoded shards are proven with one of three
  available cryptographic protocols (see below for more information. This step
  consists of attaching extra information to them and sharing augmented _blocks_
  of data onto the network. This extra information should guarantee, maybe only
  with a very high probability, that a given shard has been generated indeed
  though an expected encoding process, namely a polynomial evaluation or vector
  innner-product encoding such as Reed-Solomon.
- `verify`: any shard is verified individually for its validity. This allows to
  discriminate invalid or corrupted shards without requiring a full decoding of
  the original data.
- `decode`: the original data is decoded using any subset of $k$ valid shards.

This version of **Komodo** ships three cryptographic methods to prove the
integrity of encoded data:

- **KZG+**: This method is based on the well-known _zero-knowledge_ protocol
  **KZG** [@kate2010constant] and its multi-polynomial extension
  [@boneh2020efficient]. In **KZG**, the data is interpreted as a polynomial.
  Then a commitment of this polynomial, common to all shards, is computed.
  Finally, a proof, unique per shard, is computed and attached to the associated
  shard. The multi-polynomial extension allows to scale to bigger data by still
  computing a single proof per shard regardless of the size of the input data.
- **aPlonK**: This method is based on the following work: PlonK
  [@gabizon2019plonk] and **aPlonK** [@ambrona2022aplonk]. Through recursion and
  tree _folding_, it achieves smaller commitment sizes as compared to **KZG+**
  at the cost of very expensive proving times.
- **Semi-AVID**: This last method is the simplest and the fastest. It is based
  on the work of **Semi-AVID-PR** [@nazirkhanova2022information]. Instead of
  computing proofs as extra cryptographic elements, **Semi-AVID** leverages the
  _homomorphic_ property of the `commit` operation which makes sure _the linear
  combination of commitments is equal to the commitment of the same linear
  combination_.

A beta version of **Komodo** has been used in a previous evaluation paper
[@stevan2024performance] and is still available for reference at
[https://gitlab.isae-supaero.fr/dragoon/pcs-fec-id](https://gitlab.isae-supaero.fr/dragoon/pcs-fec-id).

**Komodo** is based on the Arkworks library [@arkworks] which provides
implementations of elliptic curves, fields and polynomial algebra used in all
the proving protocols.

> mention Merkle trees [@merkle1987digital] and Fiat-Shamir [@fiat1986prove]?

## General data flow in **Komodo**

\tikzset{
    block/.style = {draw, fill=white, rectangle, minimum height=3em, minimum width=3em},
    tmp/.style  = {coordinate},
    sum/.style= {draw, fill=white, circle, node distance=1cm},
    input/.style = {coordinate},
    output/.style= {coordinate},
    pinstyle/.style = {pin edge={to-,thin,black}}
}

\begin{tikzpicture}[auto, node distance=2cm,>=latex']
    \node [block, fill=red!50] (source) {$(s_i)$};
    \node [block, right of=source, node distance=4cm] (encoded) {$(e_j)$};
    \node [block, right of=encoded, node distance=3cm, fill=yellow!20] (commitment) {$c$};
    \node [block, below of=commitment, node distance=1.3cm, fill=yellow!30] (proof) {$(\pi_j)$};
    \node [block, right of=commitment, node distance=1.5cm, fill=blue!50] (blocks) {$(b_j)$};
    \node [block, right of=blocks, node distance=3cm, fill=blue!20] (verified) {$(b^*_j)$};
    \node [block, below of=verified, node distance=2cm, fill=red!20] (decoded) {$(\tilde{s}_i)$};
    \draw [->] (source) -- node{\texttt{encode(k, n)}} (encoded);
    \draw [->] (encoded) -- node[name=a,anchor=south]{\texttt{commit}} (commitment);
    \draw [->] (a) |- node[anchor=north]{\texttt{prove}} (proof);
    \draw [->] (commitment) -- (blocks);
    \draw [->] (proof) -| (blocks);
    \draw [->] (blocks) -- node{\texttt{verify}} (verified);
    \draw [->] (verified) -- node{\texttt{decode}} (decoded);
\end{tikzpicture}

where

- $S = (s_i)_{1 \leq i \leq k} \in \mathcal{M}_{m \times k}(\mathbb{F})$ is the
  matrix of source shards
- $M \in \mathcal{M}_{k \times n}(\mathbb{F})$ is the encoding matrix
- $E = (e_j)_{1 \leq j \leq n} = S \times M \in \mathcal{M}_{m \times n}(\mathbb{F})$
  is the matrix of encoded shards
- $c \in \mathbb{F}$ is the common _commitment_
- $(\pi_j)_{1 \leq j \leq n} \in \mathbb{F}^{n}$ are the proofs for each _shard_
- $(b_j)_{1 \leq j \leq n}$ are the final proven blocks, i.e. $(e_j, c, \pi_j)$

A valid and robust system should satisfy and guarantee the two following
properties:

- all blocks $(b^*_j)$ are valid and all other blocks are invalid
- $(\tilde{s}_i) \stackrel{?}{=} (s_i)$

> **Note**
>
> In the case of **Semi-AVID**, there could be more steps before the
> \texttt{verify} stage. Indeed, because it is the only method that does not
> require the full original data to produce proofs, it does support a technic
> that we call _recoding_, i.e. generating new shards on the fly with any amount
> of other shards, including strictly less than $k$ shards.

## Examples

We provide full examples for the three protocols in `examples/`. Below is a
simplified version of these examples that follows the diagram from the previous
section.

> **Note**
>
> The following snippets of code are not fully-valid _Rust_ code. They have been
> slightly simplified for the sake of readability in this document. An example
> of such simplification is that we have ommitted the use of a `main` function,
> which is mandatory in a _Rust_ program.
>
> All dependencies used below are defined unambiguously in `Cargo.toml`.

First, some definitions need to be imported.

```rust
// definitions used to specify generic types
use ark_bls12_381::{Fr as F, G1Projective as G};
use ark_poly::univariate::DensePolynomial as DP;

// the code from the Komodo library
use komodo::{algebra::linalg::Matrix, fec::{decode, encode}, zk::setup}
```

Then we can define a pseudo-random number generator, the parameters of our code
$(k, n)$, the input bytes and a _trusted setup_, which is a sequence of powers
of a secret element of $\mathbb{F}$.

```rust
let mut rng = ark_std::test_rng();

let (k, n) = (3, 6);
let bytes: Vec<u8> = vec![
  // fill with real data
];

let powers = setup::<F, G>(bytes.len(), &mut rng)?;
```

The next step, following the diagram above is to encode and prove the data to
generate $n$ encoded and proven blocks.

```rust
let encoding_matrix = Matrix::random(k, n, &mut rng);
let shards = encode(&bytes, &encoding_matrix)?;
let proofs = prove::<F, G, DP<F>>(&bytes, &powers, encoding_matrix.height)?;
let blocks = build::<F, G, DP<F>>(&shards, &proofs);
```

Finally, these blocks can be verified with `verify`.

```rust
// we assume here that all blocks are still valid
for b in &blocks {
    assert!(verify::<F, G, DP<F>>(b, &powers)?);
}
```

And the original data can be decoded using any subset of $k$ valid blocks
```rust
assert_eq!(
    bytes,
    decode(blocks[0..k].iter().cloned().map(|b| b.shard).collect())?;
);
```

> **Note**
>
> A more complete CLI application of **Semi-AVID** is available in
> `bins/saclin/`.

## Quality control

**Komodo** provides a test suite to give the highest confidence possible in the
validity of the source code.

To achieve this goal, all matrix operations are tested as well as the encoding
and decoding process and the three cryptographic protocols.

To run the test suite, please run

```bash
make check clippy test
```

## Some measurements

Building on the work from [@stevan2024performance], we have conducted some
measurements of the performance of the three methods.

The time to run `commit`, `prove` and `verify` has been measured for $k = 8$ and
a code rate $\rho = \frac{1}{2}$, i.e. $n = 16$ on the BN-254 elliptic curve and
for small and large input data.

![Performance for small files.\label{fig:small}](figures/small.png)

**Semi-AVID** is the best for small files as can be seen in \autoref{fig:small}.

![Performance for large files.\label{fig:large}](figures/large.png)

**aPlonK** is slightly better for verifying large files, see
\autoref{fig:large}, but still suffers from performance orders of magnitude
worst than **Semi-AVID** for committing and proving.

**KZG+** is neither good nor too bad.

# Statement of need

the use case is any system that meet the following criteria

- distributed, e.g. drones
- need for data robustness, e.g. by introducing redundancy
- no trust in others nor the environment, need to prove the integrity of the data

Scroll [@scroll2024], Avail [@avail2024] and Danksharding [@danksharding2024].

**Komodo** can be extended with either

new encoding methods in the `fec` module
proof protocols, just as with the `kzg`, `aplonk` and `semi_avid` modules

and Tezos [@tezos2024aplonk].

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
