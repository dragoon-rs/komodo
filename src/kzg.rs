//! $\text{KZG}^+$: the multipolynomial and non-interactive extension of $\text{KZG}$
//! > - [Kate et al., 2010](https://link.springer.com/chapter/10.1007/978-3-642-17373-8_11)
//! (`10.1007/978-3-642-17373-8_11`; [PDF](https://www.cypherpunks.ca/~iang/pubs/PolyCommit-AsiaCrypt.pdf))
//! > - [Boneh et al., 2020](https://eprint.iacr.org/2020/081) ([PDF](https://eprint.iacr.org/2020/081.pdf))
//!
//! # The protocol
//! Here, we assume that the input data has been encoded with a _Reed-Solomon_ encoding, as can be
//! done with the [`crate::fec`] module.
//!
//! > **Note**
//! >
//! > In the following, the data $\Delta$ is arranged in an $m \times k$ matrix and $i$ will denote
//! > the number of a row and $j$ the number of a column
//! > - $0 \leq i \leq m - 1$
//! > - $0 \leq j \leq k - 1$
//! >
//! > Also, $H$ is a secure hash function and
//! > $E: \mathbb{G}_1 \times \mathbb{G}_2 \mapsto \mathbb{G}_T$ is the bilinear pairing mapping
//! > such as BLS12-381.
#![doc = simple_mermaid::mermaid!("kzg.mmd")]
//!
//! Conveniently, each one of the $n$ encoded shards is a linear combination of the $k$ source
//! shards. More precisely, it is the evaluation of the input data seen as a polynomial on some
//! evalution point.
//!
//! We would like to prove that this evaluation has been done correctly and not corrupted. More
//! formally, we want to prove that a shard $s$ is the evaluation of a polynomial $P$, the input
//! data, on some evaluation point $\alpha$.
//!
//! KZG+ will unfold as follows:
//! - the prover: evaluates $P$ on a secret point $\tau$ and generates a commitment $c$
//! - the prover: computes the quotient between $A(X) = P(X) - P(\alpha)$ and $B(X) = X - \alpha$.
//!   Because $A(X)$ has $\alpha$ as a root by definition, $A(X)$ is divisible by $B(X)$ and the
//!   result $Q(X) = \frac{A(X)}{B(X)}$ makes sense. A proof $\pi$ is then crafted by evaluting the
//!   polynomial $Q(X)$ on $\tau$
//! - the prover: share the commit $c$, the shard $s$ and its proof $\pi$ onto the network
//! - the verifier: verifies the validity of the commit $c$, the proof $\pi$ and the shard $s$ with
//!   a _pairing_ operator defined on an appropriate elliptic curve
//!
//! ## Some details
//! - each shard $s$ is associated to a unique evaluation point $\alpha$
//! - because $k$ is a fixed code parameter and the data can be of arbitrary size, the bytes are
//!   arranged in an $m \times k$ matrix of finite field elements. Then, instead of computing $m$
//!   proofs per shard, KZG+ will _aggregate_ the $m$ polynomials, one per row in the data, into a
//!   single polynomial $Q$. This is done by computing a random linear combination of the $m$ input
//!   polynomials.
//!
//! # Example
//! See the KZG example.
use ark_ec::{pairing::Pairing, AffineRepr};
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::{kzg10, PCRandomness};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError};
use ark_std::{ops::Div, Zero};
use rs_merkle::{algorithms::Sha256, Hasher};
use std::ops::{AddAssign, Index, Mul};

use crate::error::KomodoError;
use crate::fec::Shard;
use crate::{algebra, zk};

#[derive(Debug, Clone, Default, PartialEq, CanonicalDeserialize, CanonicalSerialize)]
pub struct Commitment<E: Pairing>(Vec<kzg10::Commitment<E>>);

impl<E: Pairing> Index<usize> for Commitment<E> {
    type Output = kzg10::Commitment<E>;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

#[derive(Debug, Clone, Default, PartialEq, CanonicalDeserialize, CanonicalSerialize)]
pub struct Proof<E: Pairing>(kzg10::Proof<E>);

/// Commits $m$ polynomials.
pub fn commit<E, P>(
    powers: &kzg10::Powers<E>,
    polynomials: &[P],
) -> Result<Commitment<E>, KomodoError>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let commitments = match zk::ark_commit(powers, polynomials) {
        Ok((commitments, _)) => commitments,
        Err(error) => return Err(KomodoError::Other(format!("Serialization: {}", error))),
    };
    Ok(Commitment(commitments))
}

/// Proves $n$ encoded shards by computing one proof for each of them.
pub fn prove<E, P>(
    polynomials: &[P],
    shards: &[Shard<E::ScalarField>],
    points: &[E::ScalarField],
    powers: &kzg10::Powers<E>,
) -> Result<Vec<Proof<E>>, KomodoError>
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
        for p in polynomials {
            let elt = p.evaluate(pt);
            if let Err(error) = elt.serialize_with_mode(&mut eval_bytes, Compress::Yes) {
                return Err(KomodoError::Other(format!("Serialization: {}", error)));
            }
        }

        let mut compressed_bytes = Vec::new();
        for el in &s.data {
            el.serialize_uncompressed(&mut compressed_bytes).unwrap();
        }
        let hash = Sha256::hash(&compressed_bytes);
        let r = E::ScalarField::from_le_bytes_mod_order(&hash);

        let r_vec = algebra::powers_of::<E>(r, polynomials.len());
        let poly_q = algebra::scalar_product_polynomial::<E, P>(&r_vec, polynomials);

        match kzg10::KZG10::<E, P>::open(
            powers,
            &poly_q,
            *pt,
            &kzg10::Randomness::<E::ScalarField, P>::empty(),
        ) {
            Ok(proof) => proofs.push(Proof(proof)),
            Err(error) => return Err(KomodoError::Other(format!("kzg open error: {}", error))),
        };
    }

    Ok(proofs)
}

fn compute_data_for_one_shard<E, P>(
    shard: &Shard<E::ScalarField>,
    commitment: &Commitment<E>,
) -> (E::ScalarField, E::G1)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut compressed_bytes = Vec::new();
    for el in &shard.data {
        el.serialize_uncompressed(&mut compressed_bytes).unwrap();
    }
    let hash = Sha256::hash(&compressed_bytes);

    let r = E::ScalarField::from_le_bytes_mod_order(&hash);
    let r_vec = algebra::powers_of::<E>(r, shard.data.len());

    // compute y and c
    let mut y = E::ScalarField::zero();
    let mut c = E::G1::zero();
    for (i, (shard, commit)) in shard.data.iter().zip(commitment.0.iter()).enumerate() {
        y.add_assign(shard.mul(r_vec[i]));
        c.add_assign(commit.0.mul(r_vec[i]));
    }

    (y, c)
}

/// For a given [`Shard`], verifies that the data has been correctly generated.
///
/// - transforms data bytes into $m$ polynomial evaluations
/// - computes the hash of the concatenation of these evaluations
/// - computes $y$ as a combination of the shards: $$y = \sum(r^i s_i)$$
/// - computes $c$ as a combination of the commitments: $$c = \sum(r^i c_i)$$
/// - checks that $$E(c - \[y\]_1, \[1\]_2) = E(\pi\_\alpha, \[\tau\]_2 - \[\alpha\]_2)$$
pub fn verify<E, P>(
    shard: &Shard<E::ScalarField>,
    commitment: &Commitment<E>,
    proof: &Proof<E>,
    pt: E::ScalarField,
    verifier_key: &kzg10::VerifierKey<E>,
) -> bool
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let (y, c) = compute_data_for_one_shard(shard, commitment);

    let p1 = c - verifier_key.g.mul(y);
    let inner = verifier_key.beta_h.into_group() - verifier_key.h.mul(&pt);

    E::pairing(p1, verifier_key.h) == E::pairing(proof.0.w, inner)
}

/// Verifies a bunch of shards at once using a single elliptic pairing.
///
/// Rather than checking
///     $$E(c - \[y\]_1, \[1\]_2) = E(\pi\_\alpha, \[\tau - \alpha\]_2)$$
/// for each shard individually (see [`verify`]),
/// we combine the shards and perform one pairing as follows:
///
/// > **Note**
/// > let's define
/// > - $m$ as the number of polynomials in the data
/// > - $k$ as the number of shards given
///
/// 1. compute $r$ as the hash of all the proofs
/// 2. for each shard $s_j$:
///    - compute $y_j = \sum_{i = 0}^m r^i s_{j,i}$
///    - compute $c_j = \sum_{i = 0}^m r^i c_i$
/// 3. combine a combination of proofs and $(y, c, \alpha)$ such as :
///    - $\Pi = \sum_{j = 0}^k r^j \pi_j$
///    - $\Alpha = \sum_{j = 0}^k r^j (c_j - \[y_j\]_1 + \alpha_j \pi_j)$
/// 4. check $E(\Pi, \[\tau\]_2) = E(\Alpha, \[1\]_2)$
pub fn batch_verify<E, P>(
    blocks: &[(Shard<E::ScalarField>, Proof<E>)],
    commitment: &Commitment<E>,
    pts: &[E::ScalarField],
    verifier_key: &kzg10::VerifierKey<E>,
) -> Result<bool, SerializationError>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut data = Vec::new();
    for (_, p) in blocks {
        p.0.w.serialize_with_mode(&mut data, Compress::Yes)?
    }
    let hash = Sha256::hash(data.as_slice());
    let r = E::ScalarField::from_le_bytes_mod_order(&hash);
    let r_vec = algebra::powers_of::<E>(r, blocks.len());

    let (proof_agg, inner_agg) = blocks.iter().zip(pts.iter()).enumerate().fold(
        (E::G1::zero(), E::G1::zero()),
        |(proof_acc, inner_acc), (i, ((s, p), pt))| {
            let (y, c) = compute_data_for_one_shard(s, commitment);
            (
                proof_acc + p.0.w * r_vec[i],
                inner_acc + (c - verifier_key.g * y + p.0.w * pt) * r_vec[i],
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

    use crate::{
        algebra::{self, linalg::Matrix},
        conversions::u32_to_u8_vec,
        fec::{self, encode},
        zk::trim,
    };

    type UniPoly381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

    fn bytes<E: Pairing>(k: usize, nb_polynomials: usize) -> Vec<u8> {
        let nb_bytes = k * nb_polynomials * (E::ScalarField::MODULUS_BIT_SIZE as usize / 8);
        include_bytes!("../assets/dragoon_133x133.png")[0..nb_bytes].to_vec()
    }

    #[allow(clippy::type_complexity)]
    fn test_setup<E, P>(
        bytes: &[u8],
        k: usize,
        n: usize,
    ) -> Result<
        (
            Vec<(fec::Shard<E::ScalarField>, super::Proof<E>)>,
            super::Commitment<E>,
            VerifierKey<E>,
        ),
        ark_poly_commit::Error,
    >
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let degree = bytes.len() / (E::ScalarField::MODULUS_BIT_SIZE as usize / 8);

        let rng = &mut test_rng();

        let params = KZG10::<E, P>::setup(degree, false, rng)?;
        let (powers, verifier_key) = trim(&params, degree);

        let elements = algebra::split_data_into_field_elements::<E::ScalarField>(bytes, k);
        let mut polynomials = Vec::new();
        for chunk in elements.chunks(k) {
            polynomials.push(P::from_coefficients_vec(chunk.to_vec()))
        }

        let commitment = super::commit(&powers, &polynomials).unwrap();

        let encoding_points = (0..n)
            .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
            .collect::<Vec<_>>();
        let encoding_mat = Matrix::vandermonde_unchecked(&encoding_points, k);
        let shards = encode::<E::ScalarField>(bytes, &encoding_mat)
            .unwrap_or_else(|_| panic!("could not encode"));

        let proofs = super::prove::<E, P>(&polynomials, &shards, &encoding_points, &powers)
            .expect("KZG+ proof failed");

        Ok((
            shards.into_iter().zip(proofs).collect::<Vec<_>>(),
            commitment,
            verifier_key,
        ))
    }

    fn verify_template<E, P>(bytes: &[u8], k: usize, n: usize) -> Result<(), ark_poly_commit::Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let (blocks, commitment, verifier_key) =
            test_setup::<E, P>(bytes, k, n).expect("proof failed for bls12-381");

        for (i, (s, p)) in blocks.iter().enumerate() {
            assert!(
                super::verify::<E, P>(
                    s,
                    &commitment,
                    p,
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
                &commitment,
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
        let (blocks, commitment, verifier_key) =
            test_setup::<E, P>(bytes, k, n).expect("proof failed for bls12-381");

        for (i, (s, p)) in blocks.iter().enumerate() {
            assert!(
                super::verify::<E, P>(
                    s,
                    &commitment,
                    p,
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
                &commitment,
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

        let (shard, mut proof) = blocks[0].clone();
        let a = E::ScalarField::from_le_bytes_mod_order(&123u128.to_le_bytes());
        proof.0.w = proof.0.w.mul(a.pow([4321_u64])).into();

        assert!(!super::verify::<E, P>(
            &shard,
            &commitment,
            &proof,
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
