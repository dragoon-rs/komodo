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
date: 24 November 2025
bibliography: paper.bib
---

# Summary

We present **Komodo**, a library that allows to encode data with erasure-code
techniques such as Reed-Solomon encoding, prove the resulting shards with
cryptographic protocols, verify their integrity on the other end of any
distributed network and decode the original data from a subset of said shards.
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

[^1]: GitLab source code: [https://gitlab.isae-supaero.fr/dragoon/komodo](https://gitlab.isae-supaero.fr/dragoon/komodo)
[^2]: GitHub mirror for issues and pull requests: [https://github.com/dragoon-rs/komodo](https://github.com/dragoon-rs/komodo)

# Keywords

Cryptography; Erasure codes; Distributed systems; Verifiable information
dispersal; Data availability;

# Statement of need

Komodo provides mechanisms that satisfy various distributed systems' needs such
as verifiable information dispersal or data availability. Such systems range
from private drone swarms to public blockchains.

For instance, in a distributed storage system, nodes can encode data into
shards, prove their integrity, and distribute them across the network. Other
nodes can then verify the shards' validity before storing or retrieving them,
ensuring data robustness and trustworthiness.

In blockchain systems, Komodo can be used as the key enabling mechanism for
checking data availability, similar to how 2D Reed-Solomon codes and Danksharding
[@ethereum2024danksharding] are used within Ethereum 2.0, or similar mechanisms in the
Celestia or Avail blockchains, among many others.

A few libraries provide similar functionalities, with a few gaps filled by
`Komodo`.

The `arkworks` ecosystem [@arkworks] is probably the closest library, providing
many of the necessary building blocks involved in Data Availability Sampling:
prime fields, possibly paired with elliptic curves like BLS12-381 or BN254 among
many others; linear algebra operations like polynomial operations and matrix
operations; and polynomial commitment. On top of those features, `Komodo` adds
Reed-Solomon encoding, tightly integrated with proof generation.

The Rust implementation of Reed-Solomon erasure coding [@rust-rse] provides
mechanisms to encode and decode data into raw shards, using elements of finite
fields $\mathbb{F}_{2^8}$ or $\mathbb{F}_{2^{16}}$, containing respectively
$2^8$ and $2^{16}$ elements. `Komodo` adds the proving mechanisms, and makes it
possible to use elements from `arkworks`' prime fields, possibly paired with
elliptic curves.

`Komodo` also adds a unified high-level API, allowing to benchmark and compare
different combinations of prime fields, elliptic curves and polynomial
commitment schemes, as we did in two publications [@stevan2024performance;
@stevan2023assessing]. Finally, a modular design allows to extend `Komodo`
with new polynomial commitment schemes or new encoding methods, which
performance can be evaluated in the same benchmarking conditions.

# Komodo
The following section introduces the **Komodo** library with more details about the implemented protocols, a graphical representation of the flow of data in an end-to-end communication, a simplified code example and some key performance results.

**Komodo**, whose key steps have been introduced in _Summary_, use some basic mathematical objects.
On one hand, `encode` and `decode` use elements of a finite field $\mathbb{F}$
with a large prime order $p$. $p$ is required to be large, usually $64$ bits or
more, for security reasons, to avoid collisions between shards. Elements in
$\mathbb{F}$ support the usual operations on numbers: _addition_, _substraction_,
_multiplication_ and _division_.
On the other hand, `commit`, `prove` and `verify` use elements of the additive
subgroup $\mathbb{G}$ of an elliptic curve $\mathbb{E}$. For consistency, there
has to be an isomorphism between $\mathbb{G}$ and $\mathbb{F}$. Elements in
$\mathbb{G}$ support the operations of any additive group: _addition_ and _subtraction_.
Multiplication by an integer scalar value can be constructed as a repeated
_addition_.

This version of **Komodo** ships three cryptographic methods to prove the
integrity of encoded data:

- **KZG+**: This method is based on the well-known _zero-knowledge_ protocol
  **KZG** [@kate2010constant] and its multi-polynomial extension
  [@boneh2020efficient]. In **KZG**, the data is interpreted as a polynomial.
  Then a commitment of this polynomial, common to all shards, is computed.
  Finally, a proof, unique per shard, is computed and attached to the associated
  shard. The multi-polynomial extension allows to scale to bigger data by still
  computing a single proof per shard regardless of the size of the input data.
- **aPlonK**: This method is based on the following works: **PlonK**
  [@gabizon2019plonk] and **aPlonK** [@ambrona2023aplonk]. Through recursion and
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

A first method that has been considered was _Merkle trees_ [@merkle1987digital].
They cut the data in leaves of a binary tree where the value inside a node is
computed as the hash of the concatenation of its two children. This process
produces a root, the _Merkle root_, and any leaf can be proven as being part of
the tree by giving a _Merkle path_ in the tree, which is simply a path of
intermediate hashes that allow to recompute the _Merkle root_ from the leaf.
This method, once applied to our use case and despite its simplicity, was only
proving that one shard was part of the _Merkle tree_ and not that it had been
generated with a $(k,n)$ code, thus allowing reconstruction from any subset of
$k$ shards.

As described in [@stevan2024performance], the protocols are usually introduced
interactively, i.e. the _prover_ and the _verifier_ need to be involved in an
interactive discussion where the _verifier_ imposes challenges to the _prover_
and the latter tries to convince the former. This is not very practical and the
implementation uses a technic known as the _Fiat-Shamir transform_ from
[@fiat1986prove].

## Some measurements

Building on the work from [@stevan2024performance], we have conducted some
measurements of the performance of the three methods. All experiments were run
on a laptop with x86‑64 Intel Core i7‑12800H (14 cores / 20 threads,
0.4–4.8 GHz) system with a 3-level cache hierarchy (L1d 544 KiB, L1i 704 KiB, L2
11.5 MiB, L3 24 MiB) and a single NUMA node. Only one thread was used for all
experiments.

The time to run `commit`, `prove` and `verify` has been measured for $k = 8$ and
a code rate $\rho = \frac{1}{2}$, i.e. $n = 16$, on the BN-254 elliptic curve, and
for small and large input data.

![Performance for small files. Average over $10$ runs.\label{fig:small}](figures/99613e59eb168636525c71d3f3d7a71fa773912ff80fbc70db035d076468633f.png)

![Performance for large files. Average over $10$ runs for \textbf{KZG+} and \textbf{Semi-AVID}. Only $1$ run for \textbf{aPlonK}. \label{fig:large}](figures/30f6bd95df8c5bd92d9d45585c5050a2e41be2814fd31b0f54f268d9bbbe3d3f.png)

\autoref{fig:small} shows that **Semi-AVID** is the best for committing, proving and verifying small files.

\autoref{fig:large} shows that **aPlonK** is slightly better for verifying large files but still suffers from performance orders of magnitude worst than **Semi-AVID** for committing and proving.

**KZG+** is neither good nor too bad.

# Additional information

**Komodo** is fully written in _Rust_ and thus all dependencies are taken care of by _Cargo_ and `Cargo.toml`.

## Contact

- by email: `firstname.lastname@isae-supaero.fr`
- ticket tracker: [https://github.com/dragoon-rs/komodo/issues](https://github.com/dragoon-rs/komodo/issues)
- contributions: [https://github.com/dragoon-rs/komodo/pulls](https://github.com/dragoon-rs/komodo/pulls)

# Acknowledgements

This work was supported by the Defense Innovation Agency (AID) of the French
Ministry of Defense through the Research Project DRAGOON: Dependable distRibuted
storAGe fOr mObile Nodes (2022 65 0082).

# References
