//! $\text{Semi-AVID}$: a proving scheme suited for an _information dispersal_ context
//!
//! In their paper, [Nazirkhanova et al., 2022](https://arxiv.org/abs/2111.12323)
//! ([PDF](https://eprint.iacr.org/2021/1544.pdf)) introduce a new proving scheme.
//!
//! In opposition to how it is commonly done in protocols such as [`crate::kzg`], the data is
//! interpreted as column-oriented polynomials.
//!
//! Using FEC notations, there are $k$ such column-oriented polynomials, i.e. the $k$ source shards.
//! They are all commited using a common trusted setup and these $k$ commitments are used to prove
//! the integrity of encoded shards.
//!
//! In order to verify this property, i.e. that a given shard has been computed as a linear
//! combination of the $k$ source shards, the _homomorphic_ property of the commit operation is
//! used: _the commitment of a linear combination of polynomials is equal to the same linear
//! combination of the commiments of the same polynomials_.
//!
//! > **Note**
//! >
//! > In the following, we denote by $\text{com}$ the commitment operation and by
//! > $\mathbb{F}_p\[X\]$ the ring of all polynomials of one variable over $\mathbb{F}_p$.
//!
//! $$\forall (\alpha_i) \in \mathbb{F}_p, (P_i) \in \mathbb{F}_p\[X\], \quad \text{com}\left(\sum\limits_i \alpha_i P_i\right) = \sum\limits_i \alpha_i \text{com}(P_i)$$
//!
//! This give us a simple, lightweight and fast commitment scheme.
//!
//! > **Note**
//! >
//! > In the following, the data $\Delta$ is arranged in an $m \times k$ matrix and $i$ will denote
//! the number of a row and $j$ the number of a column
//! > - $0 \leq i \leq m - 1$
//! > - $0 \leq j \leq k - 1$
//!
//! Let’s explain with a very simple example how things operate with $\text{Semi-AVID}$. The setup is that a
//! prover wants to show a verifier that a shard of encoded data $s_\alpha$ has indeed been
//! generated with a linear combination of the $k$ source shards from data $\Delta$. $\alpha$ is
//! the number that identifies shard $s_\alpha$ and $\text{lincomb}(s_\alpha)$ is the linear combination
//! used to compute $s_\alpha$ from the $k$ source shards.
#![doc = simple_mermaid::mermaid!("semi_avid.mmd")]
//!
//! # Example
//! > **Note**
//! >
//! > Below, `F`, `G` and `DP<F>` are explicitely specified everywhere but, in _real_ code, i.e.
//! > using generic types as it's commonly done in Arkworks, it should be possible to specify them
//! > once and Rust will take care of _carrying_ the types in the rest of the code. Also, `DP<F>`
//! > will likely be its own generic type, usually written `P` in this code base.
//! >
//! > See the $\text{Semi-AVID}$ example for a fully-typed code.
//!
//! - first, let's import some types...
//! ```
//! use ark_bls12_381::{Fr as F, G1Projective as G};
//! use ark_poly::univariate::DensePolynomial as DP;
//! ```
//! - and setup the input data
//! ```
//! # fn main() {
//! let mut rng = ark_std::test_rng();
//!
//! let (k, n) = (3, 6);
//! let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! # }
//! ```
//! - then, $\text{Semi-AVID}$ requires a trusted setup to commit and verify. This example shows a trusted
//! setup big enough to support data as big as $10 \times 1024$ elements of $\mathbb{F}_p$, to
//! allow users to reuse it with multiple files of varying lengths.
//! ```
//! # use ark_bls12_381::{Fr as F, G1Projective as G};
//! # fn main() {
//! # let mut rng = ark_std::test_rng();
//! #
//! # let (k, n) = (3, 6);
//! # let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! let powers = komodo::zk::setup::<F, G>(10 * 1_024, &mut rng).unwrap();
//! # }
//! ```
//! - we can now build an encoding matrix, encode the data and commit it, proving the shards implicitely
//! ```
//! # use ark_bls12_381::{Fr as F, G1Projective as G};
//! # use ark_poly::univariate::DensePolynomial as DP;
//! #
//! # use komodo::semi_avid::{commit, verify};
//! # use komodo::algebra::linalg::Matrix;
//! #
//! # fn main() {
//! # let mut rng = ark_std::test_rng();
//! #
//! # let (k, n) = (3, 6);
//! # let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! # let powers = komodo::zk::setup::<F, G>(10 * 1_024, &mut rng).unwrap();
//! #
//! let encoding_mat: Matrix<F> = komodo::algebra::linalg::Matrix::random(k, n, &mut rng);
//! let shards = komodo::fec::encode(&bytes, &encoding_mat).unwrap();
//! let commitment = commit::<F, G, DP<F>>(&bytes, &powers, encoding_mat.height).unwrap();
//! # }
//! ```
//! - finally, each [`Shard`] can be verified individually, using the same trusted setup
//! ```
//! # use ark_bls12_381::{Fr as F, G1Projective as G};
//! # use ark_poly::univariate::DensePolynomial as DP;
//! #
//! # use komodo::semi_avid::{commit, verify};
//! # use komodo::algebra::linalg::Matrix;
//! #
//! # fn main() {
//! # let mut rng = ark_std::test_rng();
//! #
//! # let (k, n) = (3, 6);
//! # let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! # let powers = komodo::zk::setup::<F, G>(10 * 1_024, &mut rng).unwrap();
//! #
//! # let encoding_mat: Matrix<F> = komodo::algebra::linalg::Matrix::random(k, n, &mut rng);
//! # let shards = komodo::fec::encode(&bytes, &encoding_mat).unwrap();
//! # let commitment = commit::<F, G, DP<F>>(&bytes, &powers, encoding_mat.height).unwrap();
//! #
//! for shard in &shards {
//!     assert!(verify::<F, G, DP<F>>(shard, &commitment, &powers).unwrap());
//! }
//! # }
//! ```
//! - and decoded using any $k$ of the shards, here the first $k$
//! ```
//! # use ark_bls12_381::{Fr as F, G1Projective as G};
//! # use ark_poly::univariate::DensePolynomial as DP;
//! #
//! # use komodo::semi_avid::commit;
//! # use komodo::algebra::linalg::Matrix;
//! #
//! # fn main() {
//! # let mut rng = ark_std::test_rng();
//! #
//! # let (k, n) = (3, 6);
//! # let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! # let powers = komodo::zk::setup::<F, G>(10 * 1_024, &mut rng).unwrap();
//! #
//! # let encoding_mat: Matrix<F> = komodo::algebra::linalg::Matrix::random(k, n, &mut rng);
//! # let shards = komodo::fec::encode(&bytes, &encoding_mat).unwrap();
//! # let commitment = commit::<F, G, DP<F>>(&bytes, &powers, encoding_mat.height).unwrap();
//! #
//! assert_eq!(bytes, komodo::fec::decode(&shards).unwrap());
//! # }
//! ```
//!
//! # Recoding
//! By constrution, $\text{Semi-AVID}$ supports an operation on shards known as _recoding_. This allows to
//! combine an arbitrary number of shards together on the fly, without decoding the data and then
//! re-encoding brand new shards.
//!
//! This is great because any node in the system can locally augment its local pool of shards.
//! However, this operation will introduce linear dependencies between recoded shards and their
//! _parents_, which might decrease the diversity of shards and harm the decoding process.

use std::ops::Index;

use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::ops::Div;
use tracing::{debug, info};

use crate::{
    algebra,
    error::KomodoError,
    fec::Shard,
    zk::{self, Powers},
};

#[derive(Debug, Default, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Commitment<F: PrimeField, G: CurveGroup<ScalarField = F>>(pub Vec<zk::Commitment<F, G>>);

impl<F: PrimeField, G: CurveGroup<ScalarField = F>> Index<usize> for Commitment<F, G> {
    type Output = G::Affine;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i].0
    }
}

/// Computes the Semi-AVID commitment for some data.
pub fn commit<F, G, P>(
    bytes: &[u8],
    powers: &Powers<F, G>,
    k: usize,
) -> Result<Commitment<F, G>, KomodoError>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    info!("encoding and committing {} bytes", bytes.len());

    debug!("splitting bytes into polynomials");
    let elements = algebra::split_data_into_field_elements(bytes, k);
    let polynomials = elements
        .chunks(k)
        .map(|c| P::from_coefficients_vec(c.to_vec()))
        .collect::<Vec<_>>();
    info!(
        "data is composed of {} polynomials and {} elements",
        polynomials.len(),
        elements.len()
    );

    debug!("transposing the polynomials to commit");
    let polynomials_to_commit = (0..k)
        .map(|i| {
            P::from_coefficients_vec(
                polynomials
                    .iter()
                    .map(|p| {
                        #[allow(clippy::clone_on_copy)]
                        p.coeffs().get(i).unwrap_or(&F::zero()).clone()
                    })
                    .collect(),
            )
        })
        .collect::<Vec<P>>();

    debug!("committing the polynomials");
    let commitments = zk::batch_commit(powers, &polynomials_to_commit)?;

    Ok(Commitment(commitments))
}

/// Verifies that a single shard is valid.
pub fn verify<F, G, P>(
    shard: &Shard<F>,
    commitment: &Commitment<F, G>,
    verifier_key: &Powers<F, G>,
) -> Result<bool, KomodoError>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let polynomial = P::from_coefficients_vec(shard.data.clone());
    let commit = zk::commit(verifier_key, &polynomial)?;

    let rhs = shard
        .linear_combination
        .iter()
        .enumerate()
        .map(|(i, w)| commitment[i].into() * w)
        .sum();
    Ok(commit.0.into() == rhs)
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::{Fr, G1Projective};
    const CURVE_NAME: &str = "bls12-381";
    use ark_ec::CurveGroup;
    use ark_ff::PrimeField;
    use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
    use ark_std::{ops::Div, test_rng};
    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        algebra::linalg::Matrix,
        error::KomodoError,
        fec::{encode, recode_random},
        zk::{self, setup, Powers},
    };

    use super::{commit, verify, Commitment};

    fn bytes() -> Vec<u8> {
        include_bytes!("../assets/dragoon_133x133.png").to_vec()
    }

    /// verify all `n` shards
    fn verify_template<F, G, P>(bytes: &[u8], encoding_mat: &Matrix<F>) -> Result<(), KomodoError>
    where
        F: PrimeField,
        G: CurveGroup<ScalarField = F>,
        P: DenseUVPolynomial<F>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let rng = &mut test_rng();

        let powers = setup::<F, G>(bytes.len(), rng)?;

        let shards = encode(bytes, encoding_mat)?;
        let commitment = commit(bytes, &powers, encoding_mat.height)?;

        for shard in &shards {
            assert!(verify(shard, &commitment, &powers)?);
        }

        Ok(())
    }

    /// attack a part of the commitment
    fn attack<F, G>(
        commitment: &Commitment<F, G>,
        c: usize,
        base: u128,
        pow: u64,
    ) -> Commitment<F, G>
    where
        F: PrimeField,
        G: CurveGroup<ScalarField = F>,
    {
        // modify a field in the struct b to corrupt the commitment without corrupting the data serialization
        let a = F::from_le_bytes_mod_order(&base.to_le_bytes());
        let mut commits: Vec<G> = commitment.0.iter().map(|c| c.0.into()).collect();
        commits[c] = commits[c].mul(a.pow([pow]));

        Commitment(commits.iter().map(|&c| zk::Commitment(c.into())).collect())
    }

    /// verify all `n` shards and then make sure an attacked shard does not verify
    fn verify_with_errors_template<F, G, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<F>,
        attacks: &[(usize, usize, u128, u64)],
    ) -> Result<(), KomodoError>
    where
        F: PrimeField,
        G: CurveGroup<ScalarField = F>,
        P: DenseUVPolynomial<F>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let rng = &mut test_rng();

        let powers = setup::<F, G>(bytes.len(), rng)?;

        let shards = encode(bytes, encoding_mat)?;
        let commitment = commit(bytes, &powers, encoding_mat.height)?;

        for shard in &shards {
            assert!(verify(shard, &commitment, &powers)?);
        }

        for &(b, c, base, pow) in attacks {
            assert!(!verify(
                &shards[b],
                &attack(&commitment, c, base, pow),
                &powers
            )?);
        }

        Ok(())
    }

    /// make sure recoded shards still verify correctly
    fn verify_recoding_template<F, G, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<F>,
        recodings: &[Vec<usize>],
    ) -> Result<(), KomodoError>
    where
        F: PrimeField,
        G: CurveGroup<ScalarField = F>,
        P: DenseUVPolynomial<F>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let rng = &mut test_rng();

        let powers = setup::<F, G>(bytes.len(), rng)?;

        let shards = encode(bytes, encoding_mat)?;
        let commitment = commit(bytes, &powers, encoding_mat.height)?;

        let min_nb_shards = recodings.iter().flatten().max().unwrap() + 1;
        assert!(
            shards.len() >= min_nb_shards,
            "not enough shards, expected {}, found {}",
            min_nb_shards,
            shards.len()
        );

        for shard_indices in recodings {
            assert!(verify(
                &recode_random(
                    &shard_indices
                        .iter()
                        .map(|&i| shards[i].clone())
                        .collect::<Vec<_>>(),
                    rng
                )
                .unwrap()
                .unwrap(),
                &commitment,
                &powers
            )?);
        }

        Ok(())
    }

    /// run the `test` with a _(k, n)_ encoding and on both a random and a Vandermonde encoding
    ///
    /// NOTE: this is part of an experiment, to be honest, to be able to see how
    /// much these tests could be refactored and simplified
    fn run_template<F, P, Fun>(k: usize, n: usize, test: Fun)
    where
        F: PrimeField,
        Fun: Fn(&[u8], &Matrix<F>) -> Result<(), KomodoError>,
        P: DenseUVPolynomial<F>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let mut rng = ark_std::test_rng();

        let bytes = bytes();

        let test_case = format!("TEST | data: {} bytes, k: {}, n: {}", bytes.len(), k, n);

        test(&bytes, &Matrix::random(k, n, &mut rng)).unwrap_or_else(|_| {
            panic!("verification failed for {CURVE_NAME} and random encoding matrix\n{test_case}")
        });
        test(
            &bytes,
            &Matrix::vandermonde_unchecked(
                &(0..n)
                    .map(|i| F::from_le_bytes_mod_order(&i.to_le_bytes()))
                    .collect::<Vec<_>>(),
                k,
            ),
        )
        .unwrap_or_else(|_| {
            panic!(
                "verification failed for {CURVE_NAME} and Vandermonde encoding matrix\n{test_case}"
            )
        });
    }

    #[test]
    fn verification() {
        run_template::<Fr, DensePolynomial<Fr>, _>(
            3,
            6,
            verify_template::<Fr, G1Projective, DensePolynomial<Fr>>,
        );
    }

    #[test]
    fn verify_with_errors() {
        run_template::<Fr, DensePolynomial<Fr>, _>(3, 6, |b, m| {
            verify_with_errors_template::<Fr, G1Projective, DensePolynomial<Fr>>(
                b,
                m,
                &[(0, 0, 123u128, 4321u64)],
            )
        });
    }

    #[test]
    fn verify_recoding() {
        run_template::<Fr, DensePolynomial<Fr>, _>(3, 6, |b, m| {
            verify_recoding_template::<Fr, G1Projective, DensePolynomial<Fr>>(
                b,
                m,
                &[vec![2, 3], vec![3, 5]],
            )
        });
    }

    #[test]
    fn prove_with_holes() {
        let mut rng = StdRng::seed_from_u64(42);
        let powers: Powers<Fr, G1Projective> = setup(300, &mut rng).unwrap();

        let data = std::fs::read("assets/bin_with_holes").unwrap();
        commit::<Fr, G1Projective, DensePolynomial<Fr>>(&data, &powers, 5).unwrap();
    }
}
