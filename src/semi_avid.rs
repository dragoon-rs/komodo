//! Semi-AVID: a proving scheme suited for an _information dispersal_ context
//!
//! In their paper, [Nazirkhanova et al.](https://arxiv.org/abs/2111.12323) introduce a new proving
//! scheme.
//!
//! In opposition to how it is commonly done in protocols such as
//! [KZG](https://link.springer.com/chapter/10.1007/978-3-642-17373-8_11), the data is interpreted
//! as column-oriented polynomials.
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
//! This give us a simple, lightweight and fast commitment scheme.
//!
//! # Example
//! > **Note**
//! >
//! > below, `F`, `G` and `DP<F>` are explicitely specified everywhere but, in _real_ code, i.e.
//! > using generic types as it's commonly done in Arkworks, it should be possible to specify them
//! > once and Rust will take care of _carrying_ the types in the rest of the code. Also, `DP<F>`
//! > will likely be its own generic type, usually written `P` in this code base.
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
//! let (k, n) = (3, 6_usize);
//! let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! # }
//! ```
//! - then, Semi-AVID requires a trusted setup to prove and verify
//! ```
//! # use ark_bls12_381::{Fr as F, G1Projective as G};
//! # fn main() {
//! # let mut rng = ark_std::test_rng();
//! #
//! # let (k, n) = (3, 6_usize);
//! # let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! let powers = komodo::zk::setup::<F, G>(bytes.len(), &mut rng).unwrap();
//! # }
//! ```
//! - we can now build an encoding matrix, encode the data, prove the shards and build [`Block`]s
//! ```
//! # use ark_bls12_381::{Fr as F, G1Projective as G};
//! # use ark_poly::univariate::DensePolynomial as DP;
//! #
//! # use komodo::semi_avid::{build, prove, verify};
//! #
//! # fn main() {
//! # let mut rng = ark_std::test_rng();
//! #
//! # let (k, n) = (3, 6_usize);
//! # let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! # let powers = komodo::zk::setup::<F, G>(bytes.len(), &mut rng).unwrap();
//! #
//! let encoding_mat = &komodo::algebra::linalg::Matrix::random(k, n, &mut rng);
//! let shards = komodo::fec::encode(&bytes, encoding_mat).unwrap();
//! let proof = prove::<F, G, DP<F>>(&bytes, &powers, encoding_mat.height).unwrap();
//! let blocks = build::<F, G, DP<F>>(&shards, &proof);
//! # }
//! ```
//! - finally, each [`Block`] can be verified individually
//! ```
//! # use ark_bls12_381::{Fr as F, G1Projective as G};
//! # use ark_poly::univariate::DensePolynomial as DP;
//! #
//! # use komodo::semi_avid::{build, prove, verify};
//! #
//! # fn main() {
//! # let mut rng = ark_std::test_rng();
//! #
//! # let (k, n) = (3, 6_usize);
//! # let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! # let powers = komodo::zk::setup::<F, G>(bytes.len(), &mut rng).unwrap();
//! #
//! # let encoding_mat = &komodo::algebra::linalg::Matrix::random(k, n, &mut rng);
//! # let shards = komodo::fec::encode(&bytes, encoding_mat).unwrap();
//! # let proof = prove::<F, G, DP<F>>(&bytes, &powers, encoding_mat.height).unwrap();
//! # let blocks = build::<F, G, DP<F>>(&shards, &proof);
//! #
//! for block in &blocks {
//!     assert!(verify::<F, G, DP<F>>(block, &powers).unwrap());
//! }
//! # }
//! ```
//! - and decoded using any $k$ of the shards
//! ```
//! # use ark_bls12_381::{Fr as F, G1Projective as G};
//! # use ark_poly::univariate::DensePolynomial as DP;
//! #
//! # use komodo::semi_avid::{build, prove};
//! #
//! # fn main() {
//! # let mut rng = ark_std::test_rng();
//! #
//! # let (k, n) = (3, 6_usize);
//! # let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
//! #
//! # let powers = komodo::zk::setup::<F, G>(bytes.len(), &mut rng).unwrap();
//! #
//! # let encoding_mat = &komodo::algebra::linalg::Matrix::random(k, n, &mut rng);
//! # let shards = komodo::fec::encode(&bytes, encoding_mat).unwrap();
//! # let proof = prove::<F, G, DP<F>>(&bytes, &powers, encoding_mat.height).unwrap();
//! # let blocks = build::<F, G, DP<F>>(&shards, &proof);
//! #
//! let shards = blocks[0..k].iter().cloned().map(|b| b.shard).collect();
//! assert_eq!(bytes, komodo::fec::decode(shards).unwrap());
//! # }
//! ```
//!
//! # Recoding
//! By constrution, Semi-AVID supports an operation on shards known as _recoding_. This allows to
//! combine an arbitrary number of shards together on the fly, without decoding the data and then
//! re-encoding brand new shards.
//!
//! This is great because any node in the system can locally augment its local pool of shards.
//! However, this operation will introduce linear dependencies between recoded shards and their
//! _parents_, which might decrease the diversity of shards and harm the decoding process.
use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::ops::Div;
use ark_std::rand::RngCore;

use tracing::{debug, info};

use crate::{
    algebra,
    error::KomodoError,
    fec::{self, Shard},
    zk::{self, Commitment, Powers},
};

/// representation of a block of proven data.
///
/// this is a wrapper around a [`fec::Shard`] with some additional cryptographic
/// information that allows to prove the integrity of said shard.
#[derive(Debug, Default, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Block<F: PrimeField, G: CurveGroup<ScalarField = F>> {
    pub shard: fec::Shard<F>,
    proof: Vec<Commitment<F, G>>,
}

impl<F: PrimeField, G: CurveGroup<ScalarField = F>> std::fmt::Display for Block<F, G> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{")?;
        write!(f, "shard: {{")?;
        write!(f, "k: {},", self.shard.k)?;
        write!(f, "comb: [")?;
        for x in &self.shard.linear_combination {
            if x.is_zero() {
                write!(f, "0,")?;
            } else {
                write!(f, r#""{}","#, x)?;
            }
        }
        write!(f, "]")?;
        write!(f, ",")?;
        write!(f, "bytes: [")?;
        for x in &self.shard.data {
            if x.is_zero() {
                write!(f, "0,")?;
            } else {
                write!(f, r#""{}","#, x)?;
            }
        }
        write!(f, "]")?;
        write!(f, ",")?;
        write!(
            f,
            "hash: {:?},",
            self.shard
                .hash
                .iter()
                .map(|x| format!("{:x}", x))
                .collect::<Vec<_>>()
                .join("")
        )?;
        write!(f, "size: {},", self.shard.size)?;
        write!(f, "}}")?;
        write!(f, ",")?;
        write!(f, "commits: [")?;
        for commit in &self.proof {
            write!(f, r#""{}","#, commit.0)?;
        }
        write!(f, "]")?;
        write!(f, "}}")?;

        Ok(())
    }
}

/// compute a recoded block from an arbitrary set of blocks
///
/// coefficients will be drawn at random, one for each block.
///
/// if the blocks appear to come from different data, e.g. if the commits are
/// different, an error will be returned.
///
/// > **Note**
/// >
/// > this is a wrapper around [`fec::recode_random`].
pub fn recode<F: PrimeField, G: CurveGroup<ScalarField = F>>(
    blocks: &[Block<F, G>],
    rng: &mut impl RngCore,
) -> Result<Option<Block<F, G>>, KomodoError> {
    for (i, (b1, b2)) in blocks.iter().zip(blocks.iter().skip(1)).enumerate() {
        if b1.proof != b2.proof {
            return Err(KomodoError::IncompatibleBlocks(format!(
                "proofs are not the same at {}: {:?} vs {:?}",
                i, b1.proof, b2.proof
            )));
        }
    }
    let shard = match fec::recode_random(
        &blocks.iter().map(|b| b.shard.clone()).collect::<Vec<_>>(),
        rng,
    )? {
        Some(s) => s,
        None => return Ok(None),
    };

    Ok(Some(Block {
        shard,
        proof: blocks[0].proof.clone(),
    }))
}

/// compute the Semi-AVID proof for some data
pub fn prove<F, G, P>(
    bytes: &[u8],
    powers: &Powers<F, G>,
    k: usize,
) -> Result<Vec<Commitment<F, G>>, KomodoError>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    info!("encoding and proving {} bytes", bytes.len());

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
    let polynomials_to_commit = (0..polynomials[0].coeffs().len())
        .map(|i| P::from_coefficients_vec(polynomials.iter().map(|p| p.coeffs()[i]).collect()))
        .collect::<Vec<P>>();

    debug!("committing the polynomials");
    let commits = zk::batch_commit(powers, &polynomials_to_commit)?;

    Ok(commits)
}

/// attach a Semi-AVID proof to a collection of encoded shards
#[inline(always)]
pub fn build<F, G, P>(shards: &[Shard<F>], proof: &[Commitment<F, G>]) -> Vec<Block<F, G>>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    shards
        .iter()
        .map(|s| Block {
            shard: s.clone(),
            proof: proof.to_vec(),
        })
        .collect::<Vec<_>>()
}

/// verify that a single block of encoded and proven data is valid
pub fn verify<F, G, P>(
    block: &Block<F, G>,
    verifier_key: &Powers<F, G>,
) -> Result<bool, KomodoError>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let elements = block.shard.data.clone();
    let polynomial = P::from_coefficients_vec(elements);
    let commit = zk::commit(verifier_key, &polynomial)?;

    let rhs = block
        .shard
        .linear_combination
        .iter()
        .enumerate()
        .map(|(i, w)| block.proof[i].0.into() * w)
        .sum();
    Ok(commit.0.into() == rhs)
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::CurveGroup;
    use ark_ff::PrimeField;
    use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
    use ark_std::{ops::Div, test_rng};

    use crate::{
        algebra::linalg::Matrix,
        error::KomodoError,
        fec::{decode, encode, Shard},
        zk::{setup, Commitment},
    };

    use super::{build, prove, recode, verify};

    fn bytes() -> Vec<u8> {
        include_bytes!("../assets/dragoon_133x133.png").to_vec()
    }

    macro_rules! full {
        ($b:ident, $p:ident, $m:ident) => {
            build::<F, G, P>(&encode($b, $m)?, &prove($b, &$p, $m.height)?)
        };
    }

    fn verify_template<F, G, P>(bytes: &[u8], encoding_mat: &Matrix<F>) -> Result<(), KomodoError>
    where
        F: PrimeField,
        G: CurveGroup<ScalarField = F>,
        P: DenseUVPolynomial<F>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let rng = &mut test_rng();

        let powers = setup::<F, G>(bytes.len(), rng)?;

        let blocks = full!(bytes, powers, encoding_mat);

        for block in &blocks {
            assert!(verify(block, &powers)?);
        }

        Ok(())
    }

    fn verify_with_errors_template<F, G, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<F>,
    ) -> Result<(), KomodoError>
    where
        F: PrimeField,
        G: CurveGroup<ScalarField = F>,
        P: DenseUVPolynomial<F>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let rng = &mut test_rng();

        let powers = setup(bytes.len(), rng)?;

        let blocks = full!(bytes, powers, encoding_mat);

        for block in &blocks {
            assert!(verify(block, &powers)?);
        }

        let mut corrupted_block = blocks[0].clone();
        // modify a field in the struct b to corrupt the block proof without corrupting the data serialization
        let a = F::from_le_bytes_mod_order(&123u128.to_le_bytes());
        let mut commits: Vec<G> = corrupted_block.proof.iter().map(|c| c.0.into()).collect();
        commits[0] = commits[0].mul(a.pow([4321_u64]));
        corrupted_block.proof = commits.iter().map(|&c| Commitment(c.into())).collect();

        assert!(!verify(&corrupted_block, &powers)?);

        Ok(())
    }

    fn verify_recoding_template<F, G, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<F>,
    ) -> Result<(), KomodoError>
    where
        F: PrimeField,
        G: CurveGroup<ScalarField = F>,
        P: DenseUVPolynomial<F>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let rng = &mut test_rng();

        let powers = setup::<F, G>(bytes.len(), rng)?;

        let blocks = full!(bytes, powers, encoding_mat);

        assert!(verify(
            &recode(&blocks[2..=3], rng).unwrap().unwrap(),
            &powers
        )?);
        assert!(verify(
            &recode(&[blocks[3].clone(), blocks[5].clone()], rng)
                .unwrap()
                .unwrap(),
            &powers
        )?);

        Ok(())
    }

    fn end_to_end_template<F, G, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<F>,
    ) -> Result<(), KomodoError>
    where
        F: PrimeField,
        G: CurveGroup<ScalarField = F>,
        P: DenseUVPolynomial<F>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let rng = &mut test_rng();

        let powers = setup::<F, G>(bytes.len(), rng)?;

        let blocks = full!(bytes, powers, encoding_mat);

        let shards: Vec<Shard<F>> = blocks.iter().map(|b| b.shard.clone()).collect();

        assert_eq!(bytes, decode(shards).unwrap());

        Ok(())
    }

    fn end_to_end_with_recoding_template<F, G, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<F>,
    ) -> Result<(), KomodoError>
    where
        F: PrimeField,
        G: CurveGroup<ScalarField = F>,
        P: DenseUVPolynomial<F>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let rng = &mut test_rng();

        let powers = setup::<F, G>(bytes.len(), rng)?;

        let blocks = full!(bytes, powers, encoding_mat);

        let b_0_1 = recode(&blocks[0..=1], rng).unwrap().unwrap();
        let shards = vec![
            b_0_1.shard,
            blocks[2].shard.clone(),
            blocks[3].shard.clone(),
        ];
        assert_eq!(bytes, decode(shards).unwrap());

        let b_0_1 = recode(&[blocks[0].clone(), blocks[1].clone()], rng)
            .unwrap()
            .unwrap();
        let shards = vec![
            blocks[0].shard.clone(),
            blocks[1].shard.clone(),
            b_0_1.shard,
        ];
        assert!(decode(shards).is_err());

        let b_0_1 = recode(&blocks[0..=1], rng).unwrap().unwrap();
        let b_2_3 = recode(&blocks[2..=3], rng).unwrap().unwrap();
        let b_1_4 = recode(&[blocks[1].clone(), blocks[4].clone()], rng)
            .unwrap()
            .unwrap();
        let shards = vec![b_0_1.shard, b_2_3.shard, b_1_4.shard];
        assert_eq!(bytes, decode(shards).unwrap());

        let fully_recoded_shards = (0..3)
            .map(|_| recode(&blocks[0..=2], rng).unwrap().unwrap().shard)
            .collect();
        assert_eq!(bytes, decode(fully_recoded_shards).unwrap());

        Ok(())
    }

    // NOTE: this is part of an experiment, to be honest, to be able to see how
    // much these tests could be refactored and simplified
    fn run_template<F, P, Fun>(test: Fun)
    where
        F: PrimeField,
        Fun: Fn(&[u8], &Matrix<F>) -> Result<(), KomodoError>,
        P: DenseUVPolynomial<F>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let mut rng = ark_std::test_rng();

        let (k, n) = (3, 6_usize);

        let bytes = bytes();

        let test_case = format!("TEST | data: {} bytes, k: {}, n: {}", bytes.len(), k, n);

        test(&bytes, &Matrix::random(k, n, &mut rng)).unwrap_or_else(|_| {
            panic!("verification failed for bls12-381 and random encoding matrix\n{test_case}")
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
            panic!("verification failed for bls12-381 and Vandermonde encoding matrix\n{test_case}")
        });
    }

    #[test]
    fn verification() {
        run_template::<Fr, DensePolynomial<Fr>, _>(
            verify_template::<Fr, G1Projective, DensePolynomial<Fr>>,
        );
    }

    #[test]
    fn verify_with_errors() {
        run_template::<Fr, DensePolynomial<Fr>, _>(
            verify_with_errors_template::<Fr, G1Projective, DensePolynomial<Fr>>,
        );
    }

    #[test]
    fn verify_recoding() {
        run_template::<Fr, DensePolynomial<Fr>, _>(
            verify_recoding_template::<Fr, G1Projective, DensePolynomial<Fr>>,
        );
    }

    #[test]
    fn end_to_end() {
        run_template::<Fr, DensePolynomial<Fr>, _>(
            end_to_end_template::<Fr, G1Projective, DensePolynomial<Fr>>,
        );
    }

    #[test]
    fn end_to_end_with_recoding() {
        run_template::<Fr, DensePolynomial<Fr>, _>(
            end_to_end_with_recoding_template::<Fr, G1Projective, DensePolynomial<Fr>>,
        );
    }
}
