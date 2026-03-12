//! Komodo: Cryptographically-proven Erasure Coding.
//!
//! Komodo provides a Rust library and ecosystem that is composed of two main parts:
//! - support for FEC encoding and decoding with the [`fec`] submodule
//! - support for proving and verifying shards of encoded data with the [`semi_avid`], [`kzg`]\*,
//! [`aplonk`]\* and [`fri`]\* submodules
//!
//! > **Note**
//! >
//! > modules marked with an `*`, e.g. [`kzg`]*, are hidden behind a _Cargo_ feature with the same
//! > name
//!
//! Other submodules define several fundamental building blocks to Komodo, but are not
//! mandatory to explore and understand the protocols.
//!
//! # What does Komodo prove ?
//! When some data $\Delta$ is $(k,n)$-encoded, shards $(s_i)_{1 \leq i \leq n}$ are generated
//!                         $$(s_i) = \texttt{encode}(\Delta, k, n)$$
//!
//! > see [`fec`] for details about the encoding process
//!
//! A [Merkle tree](https://en.wikipedia.org/wiki/Merkle_tree) could be used to prove that a given
//! shard $s$ has been drawn from the pool of encoded shards
//! (see [`github.com:antouhou/rs-merkle`](https://github.com/antouhou/rs-merkle)). However such
//! tree is incapable of proving that $s$ has been generated through a correct encoding process and
//! is not just a random shard from a set of custom shards.
//!
//! Komodo allows any verifier to assert that any shard $s \in (s_i)$ has been constructed from a
//! correct encoding of the data $\Delta$ without requiring a full $k$-decoding, which is an
//! expensive operation that does not isolate tampered shards trivially.
//!
//! > see protocol submodules for a more precise definition of "_correct encoding_".
//!
//! # Example
//! Let's explain with a very simple example how things operate with Komodo. The setup is that a
//! _prover_ wants to show a _verifier_ that a shard of encoded data $s$ has indeed been generated
//! with a linear combination of the $k$ source shards from data $\Delta$ (see [`fec`]).
//!
#![doc = simple_mermaid::mermaid!("lib.mmd")]
//!
//! > **Note**
//! >
//! > the following example uses some syntax of Rust but is NOT valid Rust code and omits a lot of
//! > details for both Rust and Komodo.
//! >
//! > Real complete examples can be found in the
//! > [`examples/`](https://gitlab.isae-supaero.fr/dragoon/komodo/-/tree/main/examples)
//! > directory in the repository.
//!
//! 1. build encoded _shards_ via a given encoding
//! ```ignore
//! let encoding_mat = Matrix::random(k, n, rng);
//! let shards = fec::encode(bytes, encoding_mat);
//! ```
//! 2. compute a _commitment_ that is shared across shards
//! ```ignore
//! let commitment = commit(bytes, k);
//! ```
//! 3. compute one _cryptographic proof_ per shard
//! ```ignore
//! let proofs = prove(bytes, k);
//! ```
//! 4. verify each _shard-proof_ pair individually
//! ```ignore
//! for (shard, proof) in shards.zip(proofs) {
//!     assert!(verify(shard, commitment, proof));
//! }
//! ```
//! 5. decode the original data with any subset of _k_ shards
//! ```ignore
//! assert_eq!(bytes, fec::decode(shards[0..k]));
//! ```
pub mod algebra;
#[cfg(feature = "aplonk")]
pub mod aplonk;
#[cfg(test)]
#[cfg(any(feature = "kzg", feature = "aplonk"))]
mod conversions;
pub mod error;
pub mod fec;
#[cfg(feature = "fri")]
pub mod fri;
#[cfg(feature = "kzg")]
pub mod kzg;
pub mod semi_avid;
pub mod zk;
