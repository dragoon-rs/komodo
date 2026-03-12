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
//! # Common trait
//! **Komodo** provides a common trait [`Protocol`] that allows a unified API to control any of the
//! cryptographic protocols implemented in the library.
//!
//! > Some notes about the following code snippets
//! >  - `E` is the base generic trait and is [`ark_ec::pairing::Pairing`]
//! >  - $\text{KZG}^+$ and $\text{aPlonK}$ require the use of a [_Vandermonde_ encoding][`crate::algebra::linalg::Matrix::vandermonde`]
//! >
//! > The full example is available at `./examples/trait.rs` and can be run with `cargo run --example trait --all-features`
//!
//! First we define the input parameters
//! ```rust
//! # use komodo::{fec, algebra, error::KomodoError};
//! # use ark_ff::PrimeField;
//! # use ark_ec::pairing::Pairing;
//! # use ark_poly::{DenseUVPolynomial, univariate::DensePolynomial};
//! # use ark_std::{rand::{Rng, prelude::SliceRandom, rngs::StdRng, SeedableRng}, ops::Div};
//! #
//! # fn example<E, P>(bytes: &[u8], (k, n): (usize, usize), rng: &mut impl Rng) -> Result<(), KomodoError>
//! # where
//! #     E: Pairing,
//! #     P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
//! #     for<'a, 'b> &'a P: Div<&'b P, Output = P>,
//! # {
//! #     let encoding_mat = algebra::linalg::Matrix::vandermonde(
//! #         &(0..n)
//! #             .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
//! #             .collect::<Vec<_>>(),
//! #         k,
//! #     )?;
//! #     let mut shards = fec::encode(bytes, &encoding_mat)?;
//! #
//! #     let ff_byte_size = E::ScalarField::MODULUS_BIT_SIZE as usize / 8;
//! #     let max_degree = bytes.len() / ff_byte_size;
//! #
//! #     shards.shuffle(rng);
//! #     shards = shards.iter().take(k).cloned().collect();
//! #     assert_eq!(bytes, fec::decode(&shards)?);
//! #
//! #     Ok(())
//! # }
//! #
//! # fn main () {
//! let (k, n): (usize, usize) = /* ... */
//! #       (3, 6);
//! let bytes: Vec<u8> = /* ... */
//! #       include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! #     // NOTE: aPlonK requires the size of the data to be a "power of 2" multiple of the field element size
//! #     let ff_bit_size = <ark_bls12_381::Bls12_381 as Pairing>::ScalarField::MODULUS_BIT_SIZE;
//! #     let ff_byte_size = ff_bit_size as usize / 8;
//! #     let nb_bytes = k * 8 * ff_byte_size;
//! #
//! #     example::<
//! #         ark_bls12_381::Bls12_381,
//! #         DensePolynomial<<ark_bls12_381::Bls12_381 as Pairing>::ScalarField>,
//! #     >(&bytes[0..nb_bytes], (k, n), &mut StdRng::seed_from_u64(0));
//! # }
//! ```
//!
//! Then, redundancy can be added to the data with FEC codes
//!
//! - $(k,n)$-encode with a _Vandermonde_ matrix
//! ```rust
//! # use komodo::{fec, algebra, error::KomodoError};
//! # use ark_ff::PrimeField;
//! # use ark_ec::pairing::Pairing;
//! # use ark_poly::{DenseUVPolynomial, univariate::DensePolynomial};
//! # use ark_std::{rand::{Rng, prelude::SliceRandom, rngs::StdRng, SeedableRng}, ops::Div};
//! #
//! # fn example<E, P>(bytes: &[u8], (k, n): (usize, usize), rng: &mut impl Rng) -> Result<(), KomodoError>
//! # where
//! #     E: Pairing,
//! #     P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
//! #     for<'a, 'b> &'a P: Div<&'b P, Output = P>,
//! # {
//! let encoding_mat = algebra::linalg::Matrix::vandermonde_unchecked(
//!     &(0..n)
//!         .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
//!         .collect::<Vec<_>>(),
//!     k,
//! );
//! let mut shards = fec::encode(bytes, &encoding_mat)?;
//! #
//! #     let ff_byte_size = E::ScalarField::MODULUS_BIT_SIZE as usize / 8;
//! #     let max_degree = bytes.len() / ff_byte_size;
//! #
//! #     shards.shuffle(rng);
//! #     shards = shards.iter().take(k).cloned().collect();
//! #     assert_eq!(bytes, fec::decode(&shards)?);
//! #
//! #     Ok(())
//! # }
//! #
//! # fn main () {
//! #     let (k, n) = (3, 6);
//! #     let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! #     // NOTE: aPlonK requires the size of the data to be a "power of 2" multiple of the field element size
//! #     let ff_bit_size = <ark_bls12_381::Bls12_381 as Pairing>::ScalarField::MODULUS_BIT_SIZE;
//! #     let ff_byte_size = ff_bit_size as usize / 8;
//! #     let nb_bytes = k * 8 * ff_byte_size;
//! #
//! #     example::<
//! #         ark_bls12_381::Bls12_381,
//! #         DensePolynomial<<ark_bls12_381::Bls12_381 as Pairing>::ScalarField>,
//! #     >(&bytes[0..nb_bytes], (k, n), &mut StdRng::seed_from_u64(0));
//! # }
//! ```
//!
//! - $k$-decode random independent shards
//! ```rust
//! # use komodo::{fec, algebra, error::KomodoError};
//! # use ark_ff::PrimeField;
//! # use ark_ec::pairing::Pairing;
//! # use ark_poly::{DenseUVPolynomial, univariate::DensePolynomial};
//! # use ark_std::{rand::{Rng, prelude::SliceRandom, rngs::StdRng, SeedableRng}, ops::Div};
//! #
//! # fn example<E, P>(bytes: &[u8], (k, n): (usize, usize), rng: &mut impl Rng) -> Result<(), KomodoError>
//! # where
//! #     E: Pairing,
//! #     P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
//! #     for<'a, 'b> &'a P: Div<&'b P, Output = P>,
//! # {
//! #     let encoding_mat = algebra::linalg::Matrix::vandermonde_unchecked(
//! #         &(0..n)
//! #             .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
//! #             .collect::<Vec<_>>(),
//! #         k,
//! #     );
//! #     let mut shards = fec::encode(bytes, &encoding_mat)?;
//! #
//! {
//!     shards.shuffle(rng);
//!     shards = shards[..k].to_vec();
//! }
//! assert_eq!(bytes, fec::decode(&shards)?);
//! #
//! #     Ok(())
//! # }
//! #
//! # fn main () {
//! #     let (k, n) = (3, 6);
//! #     let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! #     // NOTE: aPlonK requires the size of the data to be a "power of 2" multiple of the field element size
//! #     let ff_bit_size = <ark_bls12_381::Bls12_381 as Pairing>::ScalarField::MODULUS_BIT_SIZE;
//! #     let ff_byte_size = ff_bit_size as usize / 8;
//! #     let nb_bytes = k * 8 * ff_byte_size;
//! #
//! #     example::<
//! #         ark_bls12_381::Bls12_381,
//! #         DensePolynomial<<ark_bls12_381::Bls12_381 as Pairing>::ScalarField>,
//! #     >(&bytes[0..nb_bytes], (k, n), &mut StdRng::seed_from_u64(0));
//! # }
//! ```
//!
//! > We define the size, in bytes, of a field element as
//! > ```ignore
//! > let ff_byte_size = E::ScalarField::MODULUS_BIT_SIZE as usize / 8;
//! > ```
//! > and the number $m$ as
//! > ```ignore
//! > let m = bytes.len() / ff_byte_size / k;
//! > ```
//!
//! Thanks to the [`Protocol`] trait, the following procedure applies to any of the protocols
//! ```rust
//! # use komodo::{fec, algebra, semi_avid, Protocol, error::KomodoError};
//! # use ark_ff::PrimeField;
//! # use ark_ec::pairing::Pairing;
//! # use ark_poly::{DenseUVPolynomial, univariate::DensePolynomial};
//! # use ark_std::{rand::{Rng, prelude::SliceRandom, rngs::StdRng, SeedableRng}, ops::Div};
//! #
//! # fn example<E, P>(bytes: &[u8], (k, n): (usize, usize), rng: &mut impl Rng) -> Result<(), KomodoError>
//! # where
//! #     E: Pairing,
//! #     P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
//! #     for<'a, 'b> &'a P: Div<&'b P, Output = P>,
//! # {
//! #     let encoding_mat = algebra::linalg::Matrix::vandermonde_unchecked(
//! #         &(0..n)
//! #             .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
//! #             .collect::<Vec<_>>(),
//! #         k,
//! #     );
//! #     let mut shards = fec::encode(bytes, &encoding_mat)?;
//! #
//! #     let ff_byte_size = E::ScalarField::MODULUS_BIT_SIZE as usize / 8;
//! #
//! #     let max_degree = bytes.len() / ff_byte_size;
//! #     let protocol = semi_avid::SemiAVID::<E::ScalarField, E::G1, P>::new(k);
//! #
//! let (setup, vk) = protocol.setup(max_degree, rng)?;
//!
//! let commitment = protocol.commit(bytes, &setup)?;
//!
//! let proofs = protocol.prove(bytes, &commitment, &shards, &setup)?;
//!
//! for (shard, proof) in shards.iter().zip(proofs.iter()) {
//!     assert!(protocol.verify(&commitment, shard, proof, &vk)?);
//! }
//! #
//! #     shards.shuffle(rng);
//! #     shards = shards.iter().take(k).cloned().collect();
//! #     assert_eq!(bytes, fec::decode(&shards)?);
//! #
//! #     Ok(())
//! # }
//! #
//! # fn main () {
//! #     let (k, n) = (3, 6);
//! #     let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! #     // NOTE: aPlonK requires the size of the data to be a "power of 2" multiple of the field element size
//! #     let ff_bit_size = <ark_bls12_381::Bls12_381 as Pairing>::ScalarField::MODULUS_BIT_SIZE;
//! #     let ff_byte_size = ff_bit_size as usize / 8;
//! #     let nb_bytes = k * 8 * ff_byte_size;
//! #
//! #     example::<
//! #         ark_bls12_381::Bls12_381,
//! #         DensePolynomial<<ark_bls12_381::Bls12_381 as Pairing>::ScalarField>,
//! #     >(&bytes[0..nb_bytes], (k, n), &mut StdRng::seed_from_u64(0));
//! # }
//! ```
//!
//! The `protocol` and the `max_degree` used to _setup_ are defined as follows
//!
//! ## $\text{Semi-AVID}$
//! ```rust
//! # use komodo::{fec, algebra, semi_avid, Protocol, error::KomodoError};
//! # use ark_ff::PrimeField;
//! # use ark_ec::pairing::Pairing;
//! # use ark_poly::{DenseUVPolynomial, univariate::DensePolynomial};
//! # use ark_std::{rand::{Rng, prelude::SliceRandom, rngs::StdRng, SeedableRng}, ops::Div};
//! #
//! # fn example<E, P>(bytes: &[u8], (k, n): (usize, usize), rng: &mut impl Rng) -> Result<(), KomodoError>
//! # where
//! #     E: Pairing,
//! #     P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
//! #     for<'a, 'b> &'a P: Div<&'b P, Output = P>,
//! # {
//! #     let encoding_mat = algebra::linalg::Matrix::vandermonde_unchecked(
//! #         &(0..n)
//! #             .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
//! #             .collect::<Vec<_>>(),
//! #         k,
//! #     );
//! #     let mut shards = fec::encode(bytes, &encoding_mat)?;
//! #
//! #     let ff_byte_size = E::ScalarField::MODULUS_BIT_SIZE as usize / 8;
//! #     let m = bytes.len() / ff_byte_size / k;
//! #
//! let max_degree = m - 1;
//! let protocol = semi_avid::SemiAVID::<E::ScalarField, E::G1, P>::new(k);
//! #
//! #     let (setup, vk) = protocol.setup(max_degree, rng)?;
//! #
//! #     let commitment = protocol.commit(bytes, &setup)?;
//! #
//! #     let proofs = protocol.prove(bytes, &commitment, &shards, &setup)?;
//! #
//! #     for (shard, proof) in shards.iter().zip(proofs.iter()) {
//! #         assert!(protocol.verify(&commitment, shard, proof, &vk)?);
//! #     }
//! #
//! #     shards.shuffle(rng);
//! #     shards = shards.iter().take(k).cloned().collect();
//! #     assert_eq!(bytes, fec::decode(&shards)?);
//! #
//! #     Ok(())
//! # }
//! #
//! # fn main () {
//! #     let (k, n) = (3, 6);
//! #     let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! #     // NOTE: aPlonK requires the size of the data to be a "power of 2" multiple of the field element size
//! #     let ff_bit_size = <ark_bls12_381::Bls12_381 as Pairing>::ScalarField::MODULUS_BIT_SIZE;
//! #     let ff_byte_size = ff_bit_size as usize / 8;
//! #     let nb_bytes = k * 8 * ff_byte_size;
//! #
//! #     example::<
//! #         ark_bls12_381::Bls12_381,
//! #         DensePolynomial<<ark_bls12_381::Bls12_381 as Pairing>::ScalarField>,
//! #     >(&bytes[0..nb_bytes], (k, n), &mut StdRng::seed_from_u64(0));
//! # }
//! ```
//!
//! ## $\text{KZG+}$
//! ```rust
//! # use komodo::{fec, algebra, kzg, Protocol, error::KomodoError};
//! # use ark_ff::PrimeField;
//! # use ark_ec::pairing::Pairing;
//! # use ark_poly::{DenseUVPolynomial, univariate::DensePolynomial};
//! # use ark_std::{rand::{Rng, prelude::SliceRandom, rngs::StdRng, SeedableRng}, ops::Div};
//! #
//! # fn example<E, P>(bytes: &[u8], (k, n): (usize, usize), rng: &mut impl Rng) -> Result<(), KomodoError>
//! # where
//! #     E: Pairing,
//! #     P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
//! #     for<'a, 'b> &'a P: Div<&'b P, Output = P>,
//! # {
//! #     let encoding_mat = algebra::linalg::Matrix::vandermonde_unchecked(
//! #         &(0..n)
//! #             .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
//! #             .collect::<Vec<_>>(),
//! #         k,
//! #     );
//! #     let mut shards = fec::encode(bytes, &encoding_mat)?;
//! #
//! #     let ff_byte_size = E::ScalarField::MODULUS_BIT_SIZE as usize / 8;
//! #     let m = bytes.len() / ff_byte_size / k;
//! #
//! let max_degree = k - 1;
//! let protocol = kzg::Kzg::<E, P>::new(k);
//! #
//! #     let (setup, vk) = protocol.setup(max_degree, rng)?;
//! #
//! #     let commitment = protocol.commit(bytes, &setup)?;
//! #
//! #     let proofs = protocol.prove(bytes, &commitment, &shards, &setup)?;
//! #
//! #     for (shard, proof) in shards.iter().zip(proofs.iter()) {
//! #         assert!(protocol.verify(&commitment, shard, proof, &vk)?);
//! #     }
//! #
//! #     shards.shuffle(rng);
//! #     shards = shards.iter().take(k).cloned().collect();
//! #     assert_eq!(bytes, fec::decode(&shards)?);
//! #
//! #     Ok(())
//! # }
//! #
//! # fn main () {
//! #     let (k, n) = (3, 6);
//! #     let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! #     // NOTE: aPlonK requires the size of the data to be a "power of 2" multiple of the field element size
//! #     let ff_bit_size = <ark_bls12_381::Bls12_381 as Pairing>::ScalarField::MODULUS_BIT_SIZE;
//! #     let ff_byte_size = ff_bit_size as usize / 8;
//! #     let nb_bytes = k * 8 * ff_byte_size;
//! #
//! #     example::<
//! #         ark_bls12_381::Bls12_381,
//! #         DensePolynomial<<ark_bls12_381::Bls12_381 as Pairing>::ScalarField>,
//! #     >(&bytes[0..nb_bytes], (k, n), &mut StdRng::seed_from_u64(0));
//! # }
//! ```
//!
//! ## $\text{aPlonK}$
//! ```rust
//! # use komodo::{fec, algebra, aplonk, Protocol, error::KomodoError};
//! # use ark_ff::PrimeField;
//! # use ark_ec::pairing::Pairing;
//! # use ark_poly::{DenseUVPolynomial, univariate::DensePolynomial};
//! # use ark_std::{rand::{Rng, prelude::SliceRandom, rngs::StdRng, SeedableRng}, ops::Div};
//! #
//! # fn example<E, P>(bytes: &[u8], (k, n): (usize, usize), rng: &mut impl Rng) -> Result<(), KomodoError>
//! # where
//! #     E: Pairing,
//! #     P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
//! #     for<'a, 'b> &'a P: Div<&'b P, Output = P>,
//! # {
//! #     let encoding_mat = algebra::linalg::Matrix::vandermonde_unchecked(
//! #         &(0..n)
//! #             .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
//! #             .collect::<Vec<_>>(),
//! #         k,
//! #     );
//! #     let mut shards = fec::encode(bytes, &encoding_mat)?;
//! #
//! #     let ff_byte_size = E::ScalarField::MODULUS_BIT_SIZE as usize / 8;
//! #     let m = bytes.len() / ff_byte_size / k;
//! #
//! let max_degree = k - 1;
//! let protocol = aplonk::Aplonk::<E, P>::new(k, m);
//! #
//! #     let (setup, vk) = protocol.setup(max_degree, rng)?;
//! #
//! #     let commitment = protocol.commit(bytes, &setup)?;
//! #
//! #     let proofs = protocol.prove(bytes, &commitment, &shards, &setup)?;
//! #
//! #     for (shard, proof) in shards.iter().zip(proofs.iter()) {
//! #         assert!(protocol.verify(&commitment, shard, proof, &vk)?);
//! #     }
//! #
//! #     shards.shuffle(rng);
//! #     shards = shards.iter().take(k).cloned().collect();
//! #     assert_eq!(bytes, fec::decode(&shards)?);
//! #
//! #     Ok(())
//! # }
//! #
//! # fn main () {
//! #     let (k, n) = (3, 6);
//! #     let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! #     // NOTE: aPlonK requires the size of the data to be a "power of 2" multiple of the field element size
//! #     let ff_bit_size = <ark_bls12_381::Bls12_381 as Pairing>::ScalarField::MODULUS_BIT_SIZE;
//! #     let ff_byte_size = ff_bit_size as usize / 8;
//! #     let nb_bytes = k * 8 * ff_byte_size;
//! #
//! #     example::<
//! #         ark_bls12_381::Bls12_381,
//! #         DensePolynomial<<ark_bls12_381::Bls12_381 as Pairing>::ScalarField>,
//! #     >(&bytes[0..nb_bytes], (k, n), &mut StdRng::seed_from_u64(0));
//! # }
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

use ark_std::rand::Rng;

use crate::error::KomodoError;

pub trait Protocol {
    type Setup;
    type Commitment;
    type Shard;
    type Proof;
    type VerifierKey;

    fn setup(
        &self,
        degree: usize,
        rng: &mut impl Rng,
    ) -> Result<(Self::Setup, Self::VerifierKey), KomodoError>;
    fn commit(&self, bytes: &[u8], setup: &Self::Setup) -> Result<Self::Commitment, KomodoError>;
    fn prove(
        &self,
        bytes: &[u8],
        commitment: &Self::Commitment,
        shards: &[Self::Shard],
        setup: &Self::Setup,
    ) -> Result<Vec<Self::Proof>, KomodoError>;
    fn verify(
        &self,
        commitment: &Self::Commitment,
        shard: &Self::Shard,
        proof: &Self::Proof,
        vk: &Self::VerifierKey,
    ) -> Result<bool, KomodoError>;
}
