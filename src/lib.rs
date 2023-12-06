use std::ops::{Div, Mul};

use ark_ec::pairing::Pairing;
use ark_ff::{Field, PrimeField};
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::{Commitment, Powers, Randomness, KZG10};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::One;
use fec::LinearCombinationElement;
use rs_merkle::algorithms::Sha256;
use rs_merkle::Hasher;
use tracing::{debug, info};

mod error;
pub mod fec;
mod field;
mod linalg;
pub mod setup;

#[derive(Debug, Default, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Block<E: Pairing> {
    pub shard: fec::Shard<E>,
    commit: Vec<Commitment<E>>,
    m: usize,
}

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

pub fn prove<E, P>(
    commits: Vec<Commitment<E>>,
    hash: [u8; 32],
    nb_bytes: usize,
    polynomials: Vec<P>,
    points: &[E::ScalarField],
) -> Result<Vec<Block<E>>, ark_poly_commit::Error>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let k = polynomials[0].coeffs().len();

    let evaluations = points
        .iter()
        .map(|point| polynomials.iter().map(|p| p.evaluate(point)).collect())
        .collect::<Vec<Vec<E::ScalarField>>>();

    let mut proofs = Vec::new();
    for (i, row) in evaluations.iter().enumerate() {
        proofs.push(Block {
            shard: fec::Shard {
                k: k as u32,
                linear_combination: vec![LinearCombinationElement {
                    index: i as u32,
                    weight: E::ScalarField::one(),
                }],
                hash: hash.to_vec(),
                bytes: field::merge_elements_into_bytes::<E>(row, false),
                size: nb_bytes,
            },
            commit: commits.clone(),
            m: polynomials.len(),
        })
    }

    Ok(proofs)
}

pub fn encode<E, P>(
    bytes: &[u8],
    k: usize,
    n: usize,
    powers: &Powers<E>,
) -> Result<Vec<Block<E>>, ark_poly_commit::Error>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    info!("encoding and proving {} bytes", bytes.len());

    debug!("splitting bytes into polynomials");
    let elements = field::split_data_into_field_elements::<E>(bytes, k, false);
    let nb_polynomials = elements.len() / k;
    let polynomials = match field::build_interleaved_polynomials::<E, P>(&elements, nb_polynomials)
    {
        Some(polynomials) => polynomials,
        None => return Err(ark_poly_commit::Error::IncorrectInputLength(
            format!(
                "padding_not_supported: vector of elements ({}) should be divisible by the desired number of polynomials ({})",
                elements.len(),
                nb_polynomials
            )
        )),
    };
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

    debug!("creating the {} evaluation points", n);
    let points: Vec<E::ScalarField> = (0..n)
        .map(|i| E::ScalarField::from_le_bytes_mod_order(&[i as u8]))
        .collect();

    debug!("hashing the {} bytes with SHA-256", bytes.len());
    let hash = Sha256::hash(bytes);

    debug!(
        "proving the {} bytes and the {} polynomials",
        bytes.len(),
        polynomials.len()
    );
    prove::<E, P>(commits, hash, bytes.len(), polynomials, &points)
}

pub fn verify<E, P>(
    block: &Block<E>,
    verifier_key: &Powers<E>,
) -> Result<bool, ark_poly_commit::Error>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let elements = field::split_data_into_field_elements::<E>(&block.shard.bytes, 1, true);
    let polynomial = P::from_coefficients_vec(elements);
    let (commit, _) = KZG10::<E, P>::commit(verifier_key, &polynomial, None, None)?;

    let rhs = block
        .shard
        .linear_combination
        .iter()
        .map(|lce| {
            let alpha = E::ScalarField::from_le_bytes_mod_order(&[lce.index as u8]);

            let f: E::G1 = block
                .commit
                .iter()
                .enumerate()
                .map(|(j, c)| {
                    let commit: E::G1 = c.0.into();
                    commit.mul(alpha.pow([j as u64]))
                })
                .sum();
            f * lce.weight
        })
        .sum();
    Ok(Into::<E::G1>::into(commit.0) == rhs)
}

pub fn batch_verify<E, P>(
    blocks: &[Block<E>],
    verifier_key: &Powers<E>,
) -> Result<bool, ark_poly_commit::Error>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    for block in blocks {
        if !verify(block, verifier_key)? {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use std::ops::{Div, Mul};

    use ark_bls12_381::Bls12_381;
    use ark_ec::pairing::Pairing;
    use ark_ff::{Field, PrimeField};
    use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
    use ark_poly_commit::kzg10::Commitment;
    use ark_std::One;

    use crate::{batch_verify, encode, setup, verify, Block};

    type UniPoly381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

    fn bytes<E: Pairing>(k: usize, nb_polynomials: usize) -> Vec<u8> {
        let nb_bytes = k * nb_polynomials * (E::ScalarField::MODULUS_BIT_SIZE as usize / 8);
        include_bytes!("../tests/dragoon_133x133.png")[0..nb_bytes].to_vec()
    }

    fn verify_template<E, P>(bytes: &[u8], k: usize, n: usize) -> Result<(), ark_poly_commit::Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let powers = setup::random(bytes.len())?;
        let blocks = encode::<E, P>(bytes, k, n, &powers)?;

        for block in &blocks {
            assert!(verify::<E, P>(block, &powers)?);
        }

        assert!(batch_verify(&blocks[1..3], &powers)?);

        Ok(())
    }

    #[test]
    fn verify_2() {
        let bytes = bytes::<Bls12_381>(4, 2);
        verify_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
        verify_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 10)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    #[test]
    fn verify_4() {
        let bytes = bytes::<Bls12_381>(4, 4);
        verify_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
        verify_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 10)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    #[test]
    fn verify_6() {
        let bytes = bytes::<Bls12_381>(4, 6);
        verify_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
        verify_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 10)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    #[ignore = "Semi-AVID-PR does not support large padding"]
    #[test]
    fn verify_with_large_padding_2() {
        let bytes = bytes::<Bls12_381>(4, 2);
        verify_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 33)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    #[ignore = "Semi-AVID-PR does not support large padding"]
    #[test]
    fn verify_with_large_padding_4() {
        let bytes = bytes::<Bls12_381>(4, 4);
        verify_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 33)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    #[ignore = "Semi-AVID-PR does not support large padding"]
    #[test]
    fn verify_with_large_padding_6() {
        let bytes = bytes::<Bls12_381>(4, 6);
        verify_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 33)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    fn verify_with_errors_template<E, P>(
        bytes: &[u8],
        k: usize,
        n: usize,
    ) -> Result<(), ark_poly_commit::Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let powers = setup::random(bytes.len())?;
        let blocks = encode::<E, P>(bytes, k, n, &powers)?;

        for block in &blocks {
            assert!(verify::<E, P>(block, &powers)?);
        }

        let mut corrupted_block = blocks[0].clone();
        // modify a field in the struct b to corrupt the block proof without corrupting the data serialization
        let a = E::ScalarField::from_le_bytes_mod_order(&[123]);
        let mut commits: Vec<E::G1> = corrupted_block.commit.iter().map(|c| c.0.into()).collect();
        commits[0] = commits[0].mul(a.pow([4321_u64]));
        corrupted_block.commit = commits.iter().map(|&c| Commitment(c.into())).collect();

        assert!(!verify::<E, P>(&corrupted_block, &powers)?);

        // let's build some blocks containing errors
        let mut blocks_with_errors = Vec::new();

        let b3 = blocks.get(3).unwrap();
        blocks_with_errors.push(Block {
            shard: b3.shard.clone(),
            commit: b3.commit.clone(),
            m: b3.m,
        });
        assert!(batch_verify(blocks_with_errors.as_slice(), &powers)?);

        blocks_with_errors.push(corrupted_block);
        assert!(!batch_verify(blocks_with_errors.as_slice(), &powers)?);

        Ok(())
    }

    #[test]
    fn verify_with_errors_2() {
        let bytes = bytes::<Bls12_381>(4, 2);
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 10)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    #[test]
    fn verify_with_errors_4() {
        let bytes = bytes::<Bls12_381>(4, 4);
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 10)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    #[test]
    fn verify_with_errors_6() {
        let bytes = bytes::<Bls12_381>(4, 6);
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 10)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    fn verify_recoding_template<E, P>(
        bytes: &[u8],
        k: usize,
        n: usize,
    ) -> Result<(), ark_poly_commit::Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let powers = setup::random(bytes.len())?;
        let blocks = encode::<E, P>(bytes, k, n, &powers)?;

        let block = Block {
            shard: blocks[0].shard.mul(E::ScalarField::one()),
            commit: blocks[0].commit.clone(),
            m: blocks[0].m,
        };
        assert!(verify::<E, P>(&block, &powers)?);

        let block = Block {
            shard: blocks[3]
                .shard
                .mul(E::ScalarField::from_le_bytes_mod_order(&[2u8])),
            commit: blocks[3].commit.clone(),
            m: blocks[3].m,
        };
        assert!(verify::<E, P>(&block, &powers)?);

        let block = Block {
            shard: blocks[3].shard.combine(
                E::ScalarField::from_le_bytes_mod_order(&[2u8]),
                &blocks[2].shard,
                E::ScalarField::from_le_bytes_mod_order(&[5u8]),
            ),
            commit: blocks[3].commit.clone(),
            m: blocks[3].m,
        };
        assert!(verify::<E, P>(&block, &powers)?);

        let block = Block {
            shard: blocks[3].shard.combine(
                E::ScalarField::from_le_bytes_mod_order(&[3u8]),
                &blocks[5].shard,
                E::ScalarField::from_le_bytes_mod_order(&[4u8]),
            ),
            commit: block.commit.clone(),
            m: block.m,
        };
        assert!(verify::<E, P>(&block, &powers)?);

        Ok(())
    }

    #[test]
    fn verify_recoding_2() {
        let bytes = bytes::<Bls12_381>(4, 2);
        verify_recoding_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
        verify_recoding_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 10)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }
}
