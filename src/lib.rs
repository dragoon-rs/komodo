//! Komodo: Cryptographically-proven Erasure Coding
use std::ops::Div;

use ark_ec::pairing::Pairing;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::{Commitment, Powers, Randomness, KZG10};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{UniformRand, Zero};
use fec::combine;
use tracing::{debug, info};

mod error;
pub mod fec;
pub mod field;
pub mod fs;
pub mod linalg;
pub mod setup;

use error::KomodoError;

use crate::linalg::Matrix;

/// representation of a block of proven data.
///
/// this is a wrapper around a [`fec::Shard`] with some additional cryptographic
/// information that allows to prove the integrity of said shard.
#[derive(Debug, Default, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Block<E: Pairing> {
    pub shard: fec::Shard<E>,
    pub commit: Vec<Commitment<E>>,
}

impl<E: Pairing> std::fmt::Display for Block<E> {
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
        for commit in &self.commit {
            write!(f, r#""{}","#, commit.0)?;
        }
        write!(f, "]")?;
        write!(f, "}}")?;

        Ok(())
    }
}

/// compute the commitments and randomnesses of a set of polynomials
///
/// this function uses the commit scheme of KZG.
///
/// > **Note**
/// > - `powers` can be generated with functions like [`setup::random`]
/// > - if `polynomials` has length `n`, then [`commit`] will generate `n`
/// >   commits and `n` randomnesses.
#[allow(clippy::type_complexity)]
pub fn commit<E, P>(
    powers: &Powers<E>,
    polynomials: &[P],
) -> Result<(Vec<Commitment<E>>, Vec<Randomness<E::ScalarField, P>>), ark_poly_commit::Error>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut commits = Vec::new();
    let mut randomnesses = Vec::new();
    for polynomial in polynomials {
        let (commit, randomness) = KZG10::<E, P>::commit(powers, polynomial, None, None)?;
        commits.push(commit);
        randomnesses.push(randomness);
    }

    Ok((commits, randomnesses))
}

/// compute encoded and proven blocks of data from some data and an encoding
/// method
///
/// > **Note**
/// > this is a wrapper around [`fec::encode`].
pub fn encode<E, P>(
    bytes: &[u8],
    encoding_mat: &Matrix<E::ScalarField>,
    powers: &Powers<E>,
) -> Result<Vec<Block<E>>, ark_poly_commit::Error>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    info!("encoding and proving {} bytes", bytes.len());

    let k = encoding_mat.height;

    debug!("splitting bytes into polynomials");
    let elements = field::split_data_into_field_elements::<E>(bytes, k);
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
    let (commits, _) = commit(powers, &polynomials_to_commit)?;

    Ok(fec::encode(bytes, encoding_mat)
        .unwrap() // TODO: don't unwrap here
        .iter()
        .map(|s| Block {
            shard: s.clone(),
            commit: commits.clone(),
        })
        .collect::<Vec<_>>())
}

/// compute a recoded block from an arbitrary set of blocks
///
/// coefficients will be drawn at random, one for each block.
///
/// if the blocks appear to come from different data, e.g. if `k` is not the
/// same or the hash of the data is different, an error will be returned.
///
/// > **Note**
/// > this is a wrapper around [`fec::combine`].
pub fn recode<E: Pairing>(blocks: &[Block<E>]) -> Result<Option<Block<E>>, KomodoError> {
    let mut rng = rand::thread_rng();

    let coeffs = blocks
        .iter()
        .map(|_| E::ScalarField::rand(&mut rng))
        .collect::<Vec<_>>();

    for (i, (b1, b2)) in blocks.iter().zip(blocks.iter().skip(1)).enumerate() {
        if b1.shard.k != b2.shard.k {
            return Err(KomodoError::IncompatibleBlocks(format!(
                "k is not the same at {}: {} vs {}",
                i, b1.shard.k, b2.shard.k
            )));
        }
        if b1.shard.hash != b2.shard.hash {
            return Err(KomodoError::IncompatibleBlocks(format!(
                "hash is not the same at {}: {:?} vs {:?}",
                i, b1.shard.hash, b2.shard.hash
            )));
        }
        if b1.shard.size != b2.shard.size {
            return Err(KomodoError::IncompatibleBlocks(format!(
                "size is not the same at {}: {} vs {}",
                i, b1.shard.size, b2.shard.size
            )));
        }
        if b1.commit != b2.commit {
            return Err(KomodoError::IncompatibleBlocks(format!(
                "commits are not the same at {}: {:?} vs {:?}",
                i, b1.commit, b2.commit
            )));
        }
    }
    let shard = match combine(
        &blocks.iter().map(|b| b.shard.clone()).collect::<Vec<_>>(),
        &coeffs,
    ) {
        Some(s) => s,
        None => return Ok(None),
    };

    Ok(Some(Block {
        shard,
        commit: blocks[0].commit.clone(),
    }))
}

/// verify that a single block of encoded and proven data is valid
pub fn verify<E, P>(
    block: &Block<E>,
    verifier_key: &Powers<E>,
) -> Result<bool, ark_poly_commit::Error>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let elements = block.shard.data.clone();
    let polynomial = P::from_coefficients_vec(elements);
    let (commit, _) = KZG10::<E, P>::commit(verifier_key, &polynomial, None, None)?;

    let rhs = block
        .shard
        .linear_combination
        .iter()
        .enumerate()
        .map(|(i, w)| Into::<E::G1>::into(block.commit[i].0) * w)
        .sum();
    Ok(Into::<E::G1>::into(commit.0) == rhs)
}

#[cfg(test)]
mod tests {
    use std::ops::{Div, Mul};

    use ark_bls12_381::Bls12_381;
    use ark_ec::pairing::Pairing;
    use ark_ff::{Field, PrimeField};
    use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
    use ark_poly_commit::kzg10::Commitment;

    use crate::{
        encode,
        fec::{decode, Shard},
        linalg::Matrix,
        recode, setup, verify,
    };

    type UniPoly381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

    fn bytes() -> Vec<u8> {
        include_bytes!("../tests/dragoon_133x133.png").to_vec()
    }

    fn verify_template<E, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<E::ScalarField>,
    ) -> Result<(), ark_poly_commit::Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let powers = setup::random(bytes.len())?;
        let blocks = encode::<E, P>(bytes, encoding_mat, &powers)?;

        for block in &blocks {
            assert!(verify::<E, P>(block, &powers)?);
        }

        Ok(())
    }

    fn verify_with_errors_template<E, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<E::ScalarField>,
    ) -> Result<(), ark_poly_commit::Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let powers = setup::random(bytes.len())?;
        let blocks = encode::<E, P>(bytes, encoding_mat, &powers)?;

        for block in &blocks {
            assert!(verify::<E, P>(block, &powers)?);
        }

        let mut corrupted_block = blocks[0].clone();
        // modify a field in the struct b to corrupt the block proof without corrupting the data serialization
        let a = E::ScalarField::from_le_bytes_mod_order(&123u128.to_le_bytes());
        let mut commits: Vec<E::G1> = corrupted_block.commit.iter().map(|c| c.0.into()).collect();
        commits[0] = commits[0].mul(a.pow([4321_u64]));
        corrupted_block.commit = commits.iter().map(|&c| Commitment(c.into())).collect();

        assert!(!verify::<E, P>(&corrupted_block, &powers)?);

        Ok(())
    }

    fn verify_recoding_template<E, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<E::ScalarField>,
    ) -> Result<(), ark_poly_commit::Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let powers = setup::random(bytes.len())?;
        let blocks = encode::<E, P>(bytes, encoding_mat, &powers)?;

        assert!(verify::<E, P>(
            &recode(&blocks[2..=3]).unwrap().unwrap(),
            &powers
        )?);
        assert!(verify::<E, P>(
            &recode(&[blocks[3].clone(), blocks[5].clone()])
                .unwrap()
                .unwrap(),
            &powers
        )?);

        Ok(())
    }

    fn end_to_end_template<E, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<E::ScalarField>,
    ) -> Result<(), ark_poly_commit::Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let powers = setup::random(bytes.len())?;
        let blocks: Vec<Shard<E>> = encode::<E, P>(bytes, encoding_mat, &powers)?
            .iter()
            .map(|b| b.shard.clone())
            .collect();

        assert_eq!(bytes, decode::<E>(blocks).unwrap());

        Ok(())
    }

    fn end_to_end_with_recoding_template<E, P>(
        bytes: &[u8],
        encoding_mat: &Matrix<E::ScalarField>,
    ) -> Result<(), ark_poly_commit::Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let powers = setup::random(bytes.len())?;
        let blocks = encode::<E, P>(bytes, encoding_mat, &powers)?;

        let b_0_1 = recode(&blocks[0..=1]).unwrap().unwrap();
        let shards = vec![
            b_0_1.shard,
            blocks[2].shard.clone(),
            blocks[3].shard.clone(),
        ];
        assert_eq!(bytes, decode::<E>(shards).unwrap());

        let b_0_1 = recode(&[blocks[0].clone(), blocks[1].clone()])
            .unwrap()
            .unwrap();
        let shards = vec![
            blocks[0].shard.clone(),
            blocks[1].shard.clone(),
            b_0_1.shard,
        ];
        assert!(decode::<E>(shards).is_err());

        let b_0_1 = recode(&blocks[0..=1]).unwrap().unwrap();
        let b_2_3 = recode(&blocks[2..=3]).unwrap().unwrap();
        let b_1_4 = recode(&[blocks[1].clone(), blocks[4].clone()])
            .unwrap()
            .unwrap();
        let shards = vec![b_0_1.shard, b_2_3.shard, b_1_4.shard];
        assert_eq!(bytes, decode::<E>(shards).unwrap());

        let fully_recoded_shards = (0..3)
            .map(|_| recode(&blocks[0..=2]).unwrap().unwrap().shard)
            .collect();
        assert_eq!(bytes, decode::<E>(fully_recoded_shards).unwrap());

        Ok(())
    }

    // NOTE: this is part of an experiment, to be honest, to be able to see how
    // much these tests could be refactored and simplified
    fn run_template<E, T, P, F>(test: F)
    where
        E: Pairing,
        T: Field,
        F: Fn(&[u8], &Matrix<T>) -> Result<(), ark_poly_commit::Error>,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let (k, n) = (3, 6);

        let bytes = bytes();
        let encoding_mat = Matrix::random(k, n);

        let test_case = format!("TEST | data: {} bytes, k: {}, n: {}", bytes.len(), k, n);

        test(&bytes, &encoding_mat)
            .unwrap_or_else(|_| panic!("verification failed for bls12-381\n{test_case}"));
    }

    #[test]
    fn verification() {
        run_template::<Bls12_381, _, UniPoly381, _>(verify_template::<Bls12_381, UniPoly381>);
    }

    #[test]
    fn verify_with_errors() {
        run_template::<Bls12_381, _, UniPoly381, _>(
            verify_with_errors_template::<Bls12_381, UniPoly381>,
        );
    }

    #[test]
    fn verify_recoding() {
        run_template::<Bls12_381, _, UniPoly381, _>(
            verify_recoding_template::<Bls12_381, UniPoly381>,
        );
    }

    #[test]
    fn end_to_end() {
        run_template::<Bls12_381, _, UniPoly381, _>(end_to_end_template::<Bls12_381, UniPoly381>);
    }

    #[test]
    fn end_to_end_with_recoding() {
        run_template::<Bls12_381, _, UniPoly381, _>(
            end_to_end_with_recoding_template::<Bls12_381, UniPoly381>,
        );
    }
}
