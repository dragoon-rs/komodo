//! Komodo: Cryptographically-proven Erasure Coding
use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::ops::Div;
use ark_std::rand::RngCore;

use tracing::{debug, info};

pub mod error;
pub mod fec;
pub mod field;
pub mod fs;
pub mod linalg;
pub mod zk;

use crate::{
    error::KomodoError,
    fec::Shard,
    zk::{Commitment, Powers},
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
    let elements = field::split_data_into_field_elements(bytes, k);
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
        build,
        error::KomodoError,
        fec::{decode, encode, Shard},
        linalg::Matrix,
        prove, recode, verify,
        zk::{setup, Commitment},
    };

    fn bytes() -> Vec<u8> {
        include_bytes!("../tests/dragoon_133x133.png").to_vec()
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
