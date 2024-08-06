//! KZG+: the multipolynomial and non-interactive extension of KZG
//!
//! > references:
//! > - [Kate et al., 2010](https://link.springer.com/chapter/10.1007/978-3-642-17373-8_11)
//! > - [Boneh et al., 2020](https://eprint.iacr.org/2020/081)
use ark_ec::{pairing::Pairing, AffineRepr};
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::{kzg10, PCRandomness};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError};
use ark_std::{ops::Div, Zero};
use rs_merkle::{algorithms::Sha256, Hasher};
use std::ops::{AddAssign, Mul};

use crate::algebra;
use crate::error::KomodoError;
use crate::fec::Shard;

pub use crate::zk::ark_commit as commit;

#[derive(Debug, Clone, Default, PartialEq, CanonicalDeserialize, CanonicalSerialize)]
pub struct Block<E: Pairing> {
    pub shard: Shard<E::ScalarField>,
    commit: Vec<kzg10::Commitment<E>>,
    proof: kzg10::Proof<E>,
}

/// this function splits the data (bytes) into k shards and generates n shards with a proof for each.
/// First, generate m polynomials with k coefficients (meaning degree is k-1)
/// Then, for each shard (n):
///     evaluate the m polynomials in one point (alpha)
///     compute a hash of the concatenated evaluations
///     compute Q(X) = sum_{i=0}^{m-1}{r^i * P_i(X)}
///     prove this polynomial with KZG10
///     store in the n Block the proof, the m commits and the m P_i evaluations
pub fn prove<E, P>(
    commits: Vec<kzg10::Commitment<E>>,
    polynomials: Vec<P>,
    shards: Vec<Shard<E::ScalarField>>,
    points: Vec<E::ScalarField>,
    powers: kzg10::Powers<E>,
) -> Result<Vec<Block<E>>, KomodoError>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    assert_eq!(
        shards.len(),
        points.len(),
        "should have same number of shards and evaluation points, found {} and {} respectively",
        shards.len(),
        points.len()
    );

    // step 3. each shard will contain an evaluation of each polynomial
    // in i (the alpha corresponding to the matrix column)
    // and the commit of each polynomials
    // compute a random combination of the polynomials and compute a proof for this polynomial
    let mut proofs = Vec::new();
    for (s, pt) in shards.iter().zip(points.iter()) {
        let mut eval_bytes = vec![];
        for p in &polynomials {
            let elt = p.evaluate(pt);
            if let Err(error) = elt.serialize_with_mode(&mut eval_bytes, Compress::Yes) {
                return Err(KomodoError::Other(error.to_string()));
            }
        }

        let mut compressed_bytes = Vec::new();
        for el in &s.data {
            el.serialize_uncompressed(&mut compressed_bytes).unwrap();
        }
        let hash = Sha256::hash(&compressed_bytes);
        let r = E::ScalarField::from_le_bytes_mod_order(&hash);

        let r_vec = algebra::powers_of::<E>(r, polynomials.len());
        let poly_q = algebra::scalar_product_polynomial::<E, P>(&r_vec, &polynomials);

        match kzg10::KZG10::<E, P>::open(
            &powers,
            &poly_q,
            *pt,
            &kzg10::Randomness::<E::ScalarField, P>::empty(),
        ) {
            Ok(proof) => proofs.push(Block {
                shard: s.clone(),
                commit: commits.clone(),
                proof,
            }),
            Err(error) => return Err(KomodoError::Other(error.to_string())),
        };
    }

    Ok(proofs)
}

fn compute_data_for_one_shard<E, P>(block: &Block<E>) -> (E::ScalarField, E::G1)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let data = &block.shard.data;

    let mut compressed_bytes = Vec::new();
    for el in data {
        el.serialize_uncompressed(&mut compressed_bytes).unwrap();
    }
    let hash = Sha256::hash(&compressed_bytes);

    let r = E::ScalarField::from_le_bytes_mod_order(&hash);
    let r_vec = algebra::powers_of::<E>(r, data.len());

    // compute y and c
    let mut y = E::ScalarField::zero();
    let mut c = E::G1::zero();
    for (i, (shard, commit)) in data.iter().zip(block.commit.iter()).enumerate() {
        y.add_assign(shard.mul(r_vec[i]));
        c.add_assign(commit.0.mul(r_vec[i]));
    }

    (y, c)
}

/// for a given Block, verify that the data has been correctly generated
/// First, transform data bytes into m polynomial evaluation
/// compute the hash of the concatenation of these evaluations
/// compute y as a combination of the shards: y = sum(r^i * Shard_i) for i=[0..m[
/// compute c as a combination of the commitments: c = sum(r^i * Commit_i) for i=[0..m[
/// Check if e(c - yG1,G2) == e(proof,(T-alpha)G2)
pub fn verify<E, P>(
    block: &Block<E>,
    pt: E::ScalarField,
    verifier_key: &kzg10::VerifierKey<E>,
) -> bool
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let (y, c) = compute_data_for_one_shard(block);

    let p1 = c - verifier_key.g.mul(y);
    let inner = verifier_key.beta_h.into_group() - verifier_key.h.mul(&pt);

    E::pairing(p1, verifier_key.h) == E::pairing(block.proof.w, inner)
}

/// verify a bunch of blocks at once using a single elliptic pairing.
///
/// Rather than checking
///     e(c - yG_1, G_2) = e(proof, (\tau - \alpha)G_2)
/// for each block individually (see [`verify`]),
/// we combine the blocks and perform one pairing as follows:
///
/// > **Note**
/// > let's define
/// > - $m$ as the number of polynomials in the data
/// > - $k$ as the number of blocks given
///
/// 1. compute $r$ as the hash of all the proofs
/// 2. for each block b_i:
///    - compute y_i = sum_{j=[0..m[}(r^j * Shard_j)
///    - compute c_i = sum_{j=[0..m[}(r^j * Commit_j)
/// 3. combine a combination of proofs and (y, c, \alpha) such as :
///    proof_agg = sum_{i=[0..k[}(r^i * proof_i)
///    inner_agg = sum_{i=[0..k[}(r^i * (c_i - y_i G_1 + alpha_i * proof_i))
/// 4. check e(proof_agg, \tau G_2) = e(inner_agg, G_2)
pub fn batch_verify<E, P>(
    blocks: &[Block<E>],
    pts: &[E::ScalarField],
    verifier_key: &kzg10::VerifierKey<E>,
) -> Result<bool, SerializationError>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut data = Vec::new();
    for b in blocks {
        b.proof.w.serialize_with_mode(&mut data, Compress::Yes)?
    }
    let hash = Sha256::hash(data.as_slice());
    let r = E::ScalarField::from_le_bytes_mod_order(&hash);
    let r_vec = algebra::powers_of::<E>(r, blocks.len());

    let (proof_agg, inner_agg) = blocks.iter().zip(pts.iter()).enumerate().fold(
        (E::G1::zero(), E::G1::zero()),
        |(proof_acc, inner_acc), (i, (block, pt))| {
            let (y, c) = compute_data_for_one_shard(block);
            (
                proof_acc + block.proof.w * r_vec[i],
                inner_acc + (c - verifier_key.g * y + block.proof.w * pt) * r_vec[i],
            )
        },
    );

    // e(sum(r^i * proof_i, T * g2) = e(sum(r^i * (commit_i  - y_i * g1 + alpha_i * proof_i)),g2)
    Ok(E::pairing(proof_agg, verifier_key.beta_h)
        == E::pairing(inner_agg, verifier_key.h.into_group()))
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Bls12_381;
    use ark_ec::pairing::Pairing;
    use ark_ff::{Field, PrimeField};
    use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
    use ark_poly_commit::kzg10::{VerifierKey, KZG10};
    use ark_std::test_rng;
    use std::ops::{Div, Mul};

    use crate::{conversions::u32_to_u8_vec, fec::encode, field, linalg::Matrix, zk::trim};

    type UniPoly381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

    fn bytes<E: Pairing>(k: usize, nb_polynomials: usize) -> Vec<u8> {
        let nb_bytes = k * nb_polynomials * (E::ScalarField::MODULUS_BIT_SIZE as usize / 8);
        include_bytes!("../assets/dragoon_133x133.png")[0..nb_bytes].to_vec()
    }

    fn test_setup<E, P>(
        bytes: &[u8],
        k: usize,
        n: usize,
    ) -> Result<(Vec<super::Block<E>>, VerifierKey<E>), ark_poly_commit::Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let degree = bytes.len() / (E::ScalarField::MODULUS_BIT_SIZE as usize / 8);

        let rng = &mut test_rng();

        let params = KZG10::<E, P>::setup(degree, false, rng)?;
        let (powers, verifier_key) = trim(params, degree);

        let elements = field::split_data_into_field_elements::<E::ScalarField>(bytes, k);
        let mut polynomials = Vec::new();
        for chunk in elements.chunks(k) {
            polynomials.push(P::from_coefficients_vec(chunk.to_vec()))
        }

        let (commits, _) = super::commit(&powers, &polynomials).unwrap();

        let encoding_points = &(0..n)
            .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
            .collect::<Vec<_>>();
        let encoding_mat = Matrix::vandermonde_unchecked(encoding_points, k);
        let shards = encode::<E::ScalarField>(bytes, &encoding_mat)
            .unwrap_or_else(|_| panic!("could not encode"));

        let blocks = super::prove::<E, P>(
            commits,
            polynomials,
            shards,
            encoding_points.clone(),
            powers,
        )
        .expect("KZG+ proof failed");

        Ok((blocks, verifier_key))
    }

    fn verify_template<E, P>(bytes: &[u8], k: usize, n: usize) -> Result<(), ark_poly_commit::Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let (blocks, verifier_key) =
            test_setup::<E, P>(bytes, k, n).expect("proof failed for bls12-381");

        for (i, block) in blocks.iter().enumerate() {
            assert!(
                super::verify::<E, P>(
                    block,
                    E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(i as u32)),
                    &verifier_key,
                ),
                "could not verify block {}",
                i
            );
        }

        assert!(
            super::batch_verify(
                &blocks[1..3],
                &[
                    E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(1)),
                    E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(2)),
                    E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(3)),
                ],
                &verifier_key
            )
            .unwrap(),
            "could not batch-verify blocks 1..3"
        );

        Ok(())
    }

    #[test]
    fn verify_2() {
        let bytes = bytes::<Bls12_381>(4, 2);
        verify_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
        verify_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 33)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    #[test]
    fn verify_4() {
        let bytes = bytes::<Bls12_381>(4, 4);
        verify_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
        verify_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 33)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    #[test]
    fn verify_6() {
        let bytes = bytes::<Bls12_381>(4, 6);
        verify_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
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
        let (blocks, verifier_key) =
            test_setup::<E, P>(bytes, k, n).expect("proof failed for bls12-381");

        for (i, block) in blocks.iter().enumerate() {
            assert!(
                super::verify::<E, P>(
                    block,
                    E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(i as u32)),
                    &verifier_key,
                ),
                "could not verify block {}",
                i
            );
        }

        assert!(
            super::batch_verify(
                &blocks[1..3],
                &[
                    E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(1)),
                    E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(2)),
                    E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(3)),
                ],
                &verifier_key
            )
            .unwrap(),
            "could not batch-verify blocks 1..3"
        );

        let mut corrupted_block = blocks[0].clone();
        let a = E::ScalarField::from_le_bytes_mod_order(&123u128.to_le_bytes());
        corrupted_block.proof.w = corrupted_block.proof.w.mul(a.pow([4321_u64])).into();

        assert!(!super::verify::<E, P>(
            &corrupted_block,
            E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(0u32)),
            &verifier_key,
        ));

        Ok(())
    }

    #[test]
    fn verify_with_errors_2() {
        let bytes = bytes::<Bls12_381>(4, 2);
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 33)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    #[test]
    fn verify_with_errors_4() {
        let bytes = bytes::<Bls12_381>(4, 4);
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 33)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }

    #[test]
    fn verify_with_errors_6() {
        let bytes = bytes::<Bls12_381>(4, 6);
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes, 4, 6)
            .expect("verification failed for bls12-381");
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 33)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }
}
