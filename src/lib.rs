//! Komodo: Cryptographically-proven Erasure Coding
//!
//! Komodo provides an easy-to-use Rust library and ecosystem that is composed of two main parts:
//! - support for FEC encoding and decoding with the [`fec`] submodule
//! - support for proving and verifying shards of encoded data with the [`semi_avid`], [`kzg`]* and
//! [`aplonk`]* submodules
//!
//! > **Note**
//! >
//! > modules marked with an `*`, e.g. [`kzg`]*, are hidden behind a _Cargo_ feature with the same
//! > name
//!
//! Other submodules define several fundamental building blocks to Komodo, but which are not
//! mandatory to explore to understand the protocols.
//!
//! # Example
//! Let's explain with a very simple example how things operate with Komodo.
//!
//! > **Note**
//! >
//! > the following example uses some syntax of Rust but is NOT valid Rust code and omits a lot of
//! > details for both Rust and Komodo
//!
//! 1. choose an _encoding matrix_ to encode the _input data_
//! ```ignore
//! let encoding_mat = Matrix::random(k, n, rng);
//! ```
//! 2. encode the data and build encoded _shards_
//! ```ignore
//! let shards = fec::encode(bytes, encoding_mat);
//! ```
//! 3. attach a _cryptographic proof_ to all the shards and get a proven _block_
//! ```ignore
//! let blocks = prove(bytes, k);
//! ```
//! 4. verify each _block_ individually
//! ```ignore
//! for block in blocks {
//!     assert!(verify(block));
//! }
//! ```
//! 5. decode the original data with any subset of _k_ blocks
//! ```ignore
//! assert_eq!(bytes, fec::decode(blocks[0..k]));
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
#[cfg(feature = "fs")]
pub mod fs;
#[cfg(feature = "kzg")]
pub mod kzg;
pub mod semi_avid;
pub mod zk;
