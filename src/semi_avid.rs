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
//! >
//! > see the Semi-AVID example for a fully-typed code.
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
    const CURVE_NAME: &str = "bls12-381";
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

    use super::{build, prove, recode, verify, Block};

    fn bytes() -> Vec<u8> {
        include_bytes!("../assets/dragoon_133x133.png").to_vec()
    }

    macro_rules! full {
        ($b:ident, $p:ident, $m:ident) => {
            build::<F, G, P>(&encode($b, $m)?, &prove($b, &$p, $m.height)?)
        };
    }

    /// verify all `n` blocks
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

    /// attack a block by alterring one part of its proof
    fn attack<F, G>(block: Block<F, G>, c: usize, base: u128, pow: u64) -> Block<F, G>
    where
        F: PrimeField,
        G: CurveGroup<ScalarField = F>,
    {
        let mut block = block;
        // modify a field in the struct b to corrupt the block proof without corrupting the data serialization
        let a = F::from_le_bytes_mod_order(&base.to_le_bytes());
        let mut commits: Vec<G> = block.proof.iter().map(|c| c.0.into()).collect();
        commits[c] = commits[c].mul(a.pow([pow]));
        block.proof = commits.iter().map(|&c| Commitment(c.into())).collect();

        block
    }

    /// verify all `n` blocks and then make sure an attacked block does not verify
    fn verify_with_errors_template<F, G, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<F>,
        attacks: Vec<(usize, usize, u128, u64)>,
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

        for (b, c, base, pow) in attacks {
            assert!(!verify(&attack(blocks[b].clone(), c, base, pow), &powers)?);
        }

        Ok(())
    }

    /// make sure recoded blocks still verify correctly
    fn verify_recoding_template<F, G, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<F>,
        recodings: Vec<Vec<usize>>,
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

        let min_nb_blocks = recodings.clone().into_iter().flatten().max().unwrap() + 1;
        assert!(
            blocks.len() >= min_nb_blocks,
            "not enough blocks, expected {}, found {}",
            min_nb_blocks,
            blocks.len()
        );

        for bs in recodings {
            assert!(verify(
                &recode(
                    &bs.iter().map(|&i| blocks[i].clone()).collect::<Vec<_>>(),
                    rng
                )
                .unwrap()
                .unwrap(),
                &powers
            )?);
        }

        Ok(())
    }

    /// encode and decode with all `n` shards
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

    /// encode and try to decode with recoded shards
    fn end_to_end_with_recoding_template<F, G, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<F>,
        recodings: Vec<(Vec<Vec<usize>>, bool)>,
    ) -> Result<(), KomodoError>
    where
        F: PrimeField,
        G: CurveGroup<ScalarField = F>,
        P: DenseUVPolynomial<F>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let rng = &mut test_rng();

        let max_k = recodings.iter().map(|(rs, _)| rs.len()).min().unwrap();
        assert!(
            encoding_mat.height <= max_k,
            "too many source shards, expected at most {}, found {}",
            max_k,
            encoding_mat.height
        );

        let powers = setup::<F, G>(bytes.len(), rng)?;

        let blocks = full!(bytes, powers, encoding_mat);

        let min_n = recodings
            .iter()
            .flat_map(|(rs, _)| rs.iter().flatten())
            .max()
            .unwrap()
            + 1;
        assert!(
            blocks.len() >= min_n,
            "not enough blocks, expected {}, found {}",
            min_n,
            blocks.len()
        );

        for (rs, pass) in recodings {
            let recoded_shards = rs
                .iter()
                .map(|bs| {
                    if bs.len() == 1 {
                        blocks[bs[0]].clone().shard
                    } else {
                        recode(
                            &bs.iter().map(|&i| blocks[i].clone()).collect::<Vec<_>>(),
                            rng,
                        )
                        .unwrap()
                        .unwrap()
                        .shard
                    }
                })
                .collect();
            if pass {
                assert_eq!(
                    bytes,
                    decode(recoded_shards).unwrap(),
                    "should decode with {:?}",
                    rs
                );
            } else {
                assert!(
                    decode(recoded_shards).is_err(),
                    "should not decode with {:?}",
                    rs
                );
            }
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
                vec![(0, 0, 123u128, 4321u64)],
            )
        });
    }

    #[test]
    fn verify_recoding() {
        run_template::<Fr, DensePolynomial<Fr>, _>(3, 6, |b, m| {
            verify_recoding_template::<Fr, G1Projective, DensePolynomial<Fr>>(
                b,
                m,
                vec![vec![2, 3], vec![3, 5]],
            )
        });
    }

    #[test]
    fn end_to_end() {
        run_template::<Fr, DensePolynomial<Fr>, _>(
            3,
            6,
            end_to_end_template::<Fr, G1Projective, DensePolynomial<Fr>>,
        );
    }

    #[test]
    fn end_to_end_with_recoding() {
        run_template::<Fr, DensePolynomial<Fr>, _>(3, 6, |b, m| {
            end_to_end_with_recoding_template::<Fr, G1Projective, DensePolynomial<Fr>>(
                b,
                m,
                vec![
                    (vec![vec![0, 1], vec![2], vec![3]], true),
                    (vec![vec![0, 1], vec![0], vec![1]], false),
                    (vec![vec![0, 1], vec![2, 3], vec![1, 4]], true),
                    (vec![vec![0, 1, 2], vec![0, 1, 2], vec![0, 1, 2]], true),
                ],
            )
        });
    }
}
