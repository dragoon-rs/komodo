//! Komodo: Cryptographically-proven Erasure Coding.
//!
//! Komodo provides an easy-to-use Rust library and ecosystem that is composed of two main parts:
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
//! mandatory to explore to understand the protocols.
//!
//! # Example
//! Let's explain with a very simple example how things operate with Komodo. The setup is that a
//! _prover_ wants to show a _verifier_ that a shard of encoded data $s$ has indeed been generated
//! with a linear combination of the $k$ source shards from data $\Delta$.
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
//! 1. choose an _encoding matrix_ to encode the _input data_
//! ```ignore
//! let encoding_mat = Matrix::random(k, n, rng);
//! ```
//! 2. encode the data and build encoded _shards_
//! ```ignore
//! let shards = fec::encode(bytes, encoding_mat);
//! ```
//! 3. generate a _cryptographic proof_ for all the shards and commit the data
//! ```ignore
//! let commitment = commit(bytes);
//! let proofs = prove(bytes, k);
//! ```
//! 4. verify each "_block_" individually
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
