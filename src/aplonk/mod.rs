//! aPlonK: an extension of KZG+ where commits are _folded_ into one
//!
//! > references:
//! > - [Ambrona et al., 2022](https://link.springer.com/chapter/10.1007/978-3-031-41326-1_11)
use ark_ec::{
    pairing::{Pairing, PairingOutput},
    AffineRepr,
};
use ark_ff::{Field, PrimeField};
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::{
    kzg10::{self, Randomness, KZG10},
    PCRandomness,
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress};
use ark_std::{test_rng, One, UniformRand};
use rs_merkle::algorithms::Sha256;
use rs_merkle::Hasher;
use std::marker::PhantomData;
use std::ops::{Div, Mul};

use crate::{
    algebra,
    error::KomodoError,
    fec::Shard,
    zk::{ark_commit, trim},
};

mod ipa;
mod polynomial;
mod transcript;

#[derive(Debug, Clone, Default, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Block<E: Pairing> {
    pub shard: Shard<E::ScalarField>,
    com_f: PairingOutput<E>,
    v_hat: E::ScalarField,
    mu_hat: E::G1,
    kzg_proof: kzg10::Proof<E>,
    ipa_proof: ipa::Proof<E>,
    aplonk_proof: E::G2,
}

/// /!\ [`Commitment`] is not [`CanonicalDeserialize`] because `P` is not [`Send`].
#[derive(Debug, Clone, Default, PartialEq, CanonicalSerialize)]
pub struct Commitment<E, P>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    _engine: PhantomData<E>,
    _poly: PhantomData<P>,
}

/// /!\ [`SetupParams`] is not [`Default`] because [`kzg10::UniversalParams`] is not [`Default`].
#[derive(Debug, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct SetupParams<E: Pairing> {
    pub kzg: kzg10::UniversalParams<E>,
    pub ipa: ipa::Params<E>,
}

#[derive(Debug, Clone, Default, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct VerifierKey<E: Pairing> {
    pub vk_psi: kzg10::VerifierKey<E>,
    pub tau_1: E::G1,
    pub g1: E::G1,
    pub g2: E::G2,
}

/// creates a combination of a trusted KZG and an IPA setup for [[aPlonk]]
///
/// > **Note**  
/// > this is an almost perfect translation of the *Setup* algorithm in page
/// > **13** of [aPlonk from [Ambrona et al.]][aPlonK]
///
/// [aPlonk]: https://eprint.iacr.org/2022/1352.pdf
pub fn setup<E, P>(
    degree_bound: usize,
    nb_polynomials: usize,
) -> Result<SetupParams<E>, ark_poly_commit::Error>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut test_rng();
    let params = KZG10::<E, P>::setup(degree_bound, true, rng)?;

    let g_1 = params.powers_of_g[0];
    let g_2 = params.h;

    let tau = E::ScalarField::rand(rng);
    let ck_tau = algebra::powers_of::<E>(tau, nb_polynomials)
        .iter()
        .map(|t| g_2.mul(t))
        .collect();

    Ok(SetupParams {
        kzg: params,
        ipa: ipa::Params {
            ck_tau,
            tau_1: g_1.mul(tau),
        },
    })
}

pub fn commit<E, P>(
    polynomials: Vec<P>,
    setup: SetupParams<E>,
) -> Result<(Vec<E::G1>, PairingOutput<E>), KomodoError>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let supported_degree = polynomials.iter().map(|p| p.degree()).max().unwrap_or(0);

    if setup.ipa.ck_tau.len() < polynomials.len() {
        return Err(KomodoError::Other("setup error".to_string()));
    }

    let (powers, _) = trim(setup.kzg, supported_degree);

    if powers.powers_of_g.len() <= supported_degree {
        return Err(KomodoError::Other("setup error".to_string()));
    }

    // commit.1.
    let mu = match ark_commit(&powers, &polynomials) {
        Ok((mu, _)) => mu,
        Err(error) => return Err(KomodoError::Other(error.to_string())),
    };
    let mu: Vec<E::G1> = mu.iter().map(|c| c.0.into_group()).collect();

    // commit.2.
    let com_f: PairingOutput<E> = mu
        .iter()
        .enumerate()
        .map(|(i, c)| E::pairing(c, setup.ipa.ck_tau[i]))
        .sum();

    Ok((mu, com_f))
}

pub fn prove<E, P>(
    commit: (Vec<E::G1>, PairingOutput<E>),
    polynomials: Vec<P>,
    shards: Vec<Shard<E::ScalarField>>,
    points: Vec<E::ScalarField>,
    params: SetupParams<E>,
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

    let (mu, com_f) = commit;

    let supported_degree = polynomials.iter().map(|p| p.degree()).max().unwrap_or(0);
    let (powers, _) = trim(params.kzg, supported_degree);

    // open
    let mut proofs = Vec::new();
    for (s, pt) in shards.iter().zip(points.iter()) {
        let v_hat_elements = polynomials
            .iter()
            .map(|p| p.evaluate(pt))
            .collect::<Vec<E::ScalarField>>();

        // open.3.1.
        let mut r_bytes = vec![];
        if let Err(error) = com_f.serialize_with_mode(&mut r_bytes, Compress::Yes) {
            return Err(KomodoError::Other(format!("serialization: {}", error)));
        }
        if let Err(error) = pt.serialize_with_mode(&mut r_bytes, Compress::Yes) {
            return Err(KomodoError::Other(format!("serialization: {}", error)));
        }
        // FIXME: hash *com_v* here
        let hash = Sha256::hash(r_bytes.as_slice());
        let r = E::ScalarField::from_le_bytes_mod_order(&hash);

        // open.3.2.
        let r_vec = algebra::powers_of::<E>(r, polynomials.len());
        // open.3.3
        let f = algebra::scalar_product_polynomial::<E, P>(&r_vec, &polynomials);
        // open.3.4.
        let mu_hat: E::G1 = algebra::scalar_product_g1::<E>(&mu, &r_vec);
        // open.3.5.
        let v_hat: E::ScalarField = algebra::scalar_product::<E>(&v_hat_elements, &r_vec);

        // open.4.
        let kzg_proof = match KZG10::<E, P>::open(
            &powers,
            &f,
            *pt,
            &Randomness::<E::ScalarField, P>::empty(),
        ) {
            Ok(proof) => proof,
            Err(error) => return Err(KomodoError::Other(format!("ark error: {}", error))),
        };

        // open.5.
        // we do no need this step as we already share the shards on the network

        // open.6.
        let (ipa_proof, u) =
            match ipa::prove(polynomials.len(), &params.ipa.ck_tau, com_f, r, mu_hat, &mu) {
                Ok(proof) => proof,
                Err(error) => return Err(error),
            };
        let mut u_inv = Vec::new();
        for u_i in &u {
            if let Some(inverse) = u_i.inverse() {
                u_inv.push(inverse)
            } else {
                return Err(KomodoError::Other("EllipticInverseError".to_string()));
            }
        }

        // open.7.1.
        let kappa = f64::log2(polynomials.len() as f64) as usize;
        let g = polynomial::compute_g::<E, P>(polynomials.len(), kappa, &u, &u_inv);
        // open.7.2.
        let mut rho_bytes = vec![];
        if let Err(error) = ipa_proof.serialize_with_mode(&mut rho_bytes, Compress::Yes) {
            return Err(KomodoError::Other(format!("SerializationError: {}", error)));
        }
        let rho = E::ScalarField::from_le_bytes_mod_order(&Sha256::hash(rho_bytes.as_slice()));
        // open.7.3.
        // implicit in the computation of the witness polynomial

        // open.8.1.
        let h = match KZG10::<E, P>::compute_witness_polynomial(
            &g,
            rho,
            &Randomness::<E::ScalarField, P>::empty(),
        ) {
            Ok((h, _)) => h,
            Err(error) => return Err(KomodoError::Other(format!("ArkError: {}", error))),
        };
        // open.8.2.
        let aplonk_proof = h
            .coeffs()
            .iter()
            .enumerate()
            .map(|(i, hi)| params.ipa.ck_tau[i].mul(hi))
            .sum();

        // open.9.
        proofs.push(Block {
            shard: s.clone(),
            com_f,
            v_hat,
            mu_hat,
            kzg_proof,
            ipa_proof,
            aplonk_proof,
        });
    }

    Ok(proofs)
}

pub fn verify<E, P>(
    block: &Block<E>,
    pt: E::ScalarField,
    vk_psi: &kzg10::VerifierKey<E>,
    tau_1: E::G1,
    g_1: E::G1,
    g_2: E::G2,
) -> Result<bool, KomodoError>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    // check.1.
    let mut bytes = vec![];
    if let Err(error) = block.com_f.serialize_with_mode(&mut bytes, Compress::Yes) {
        return Err(KomodoError::Other(format!("SerializationError: {}", error)));
    }
    if let Err(error) = pt.serialize_with_mode(&mut bytes, Compress::Yes) {
        return Err(KomodoError::Other(format!("SerializationError: {}", error)));
    }
    // FIXME: hash *com_v* here
    let hash = Sha256::hash(bytes.as_slice());
    let r = E::ScalarField::from_le_bytes_mod_order(&hash);

    // check.2.
    let p1 = block.mu_hat - vk_psi.g.mul(block.v_hat);
    let inner = vk_psi.beta_h.into_group() - vk_psi.h.mul(&pt);
    if E::pairing(p1, vk_psi.h) != E::pairing(block.kzg_proof.w, inner) {
        return Ok(false);
    }

    // TODO: missing part of the aplonk algorithm
    // check.3.

    let nb_polynomials = block.shard.size
        / (block.shard.k as usize)
        / (E::ScalarField::MODULUS_BIT_SIZE as usize / 8);

    // check.4.
    if !ipa::verify(
        nb_polynomials,
        None, // we call *IPA.Verify'* here
        block.com_f,
        r,
        block.mu_hat,
        &block.ipa_proof,
    )? {
        return Ok(false);
    }

    // check.5.1.
    let mut bytes = vec![];
    if let Err(error) = block
        .ipa_proof
        .serialize_with_mode(&mut bytes, Compress::Yes)
    {
        return Err(KomodoError::Other(format!("SerializationError: {}", error)));
    }
    let hash = Sha256::hash(bytes.as_slice());
    let rho = E::ScalarField::from_le_bytes_mod_order(&hash);

    let kappa = f64::log2(nb_polynomials as f64) as usize;
    let mut ts = match transcript::initialize(block.com_f, r, block.mu_hat) {
        Ok(transcript) => transcript,
        Err(error) => return Err(KomodoError::Other(format!("SerializationError: {}", error))),
    };
    let mut u = algebra::vector::zero::<E::ScalarField>(kappa);
    for j in (0..kappa).rev() {
        u[j] = match transcript::hash(
            block.ipa_proof.l_g[j],
            block.ipa_proof.r_g[j],
            block.ipa_proof.l_r[j],
            block.ipa_proof.r_r[j],
            &ts,
        ) {
            Ok(hash) => hash,
            Err(error) => return Err(KomodoError::Other(format!("SerializationError: {}", error))),
        };
        ts = match transcript::reset::<E>(u[j]) {
            Ok(transcript) => transcript,
            Err(error) => return Err(KomodoError::Other(format!("SerializationError: {}", error))),
        };
    }

    let mut u_inv = Vec::new();
    for u_i in &u {
        if let Some(inverse) = u_i.inverse() {
            u_inv.push(inverse)
        } else {
            return Err(KomodoError::Other("EllipticInverseError".to_string()));
        }
    }

    // check.5.2.
    let g = polynomial::compute_g::<E, P>(nb_polynomials, kappa, &u, &u_inv);
    let v_rho = g.evaluate(&rho);

    // check.6.
    let lhs = E::pairing(tau_1 - g_1.mul(rho), block.aplonk_proof);
    let rhs = E::pairing(
        g_1.mul(E::ScalarField::one()),
        block.ipa_proof.ck_tau_0 - g_2.mul(v_rho),
    );
    let b_tau = lhs == rhs;

    // check.7.
    // the formula is implicit because here
    //     - b_psi has passed in check.2.
    //     - b_v is skipped for now
    //     - b_IPA has passed in check.4.
    Ok(b_tau)
}

#[cfg(test)]
mod tests {
    use super::{commit, prove, setup, Block};
    use crate::{conversions::u32_to_u8_vec, fec::encode, field, linalg::Matrix, zk::trim};

    use ark_bls12_381::Bls12_381;
    use ark_ec::{pairing::Pairing, AffineRepr};
    use ark_ff::{Field, PrimeField};
    use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
    use ark_poly_commit::kzg10;
    use std::ops::{Div, MulAssign};

    type UniPoly381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

    fn bytes<E: Pairing>(k: usize, nb_polynomials: usize) -> Vec<u8> {
        let nb_bytes = k * nb_polynomials * (E::ScalarField::MODULUS_BIT_SIZE as usize / 8);
        include_bytes!("../../assets/dragoon_133x133.png")[0..nb_bytes].to_vec()
    }

    #[allow(clippy::type_complexity)]
    fn test_setup<E, P>(
        bytes: &[u8],
        k: usize,
        n: usize,
    ) -> Result<
        (
            Vec<Block<E>>,
            (kzg10::VerifierKey<E>, E::G1),
            (E::G1, E::G2),
        ),
        ark_poly_commit::Error,
    >
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let degree = k - 1;
        let vector_length_bound =
            bytes.len() / (E::ScalarField::MODULUS_BIT_SIZE as usize / 8) / (degree + 1);

        let params = setup::<E, P>(degree, vector_length_bound)?;
        let (_, vk_psi) = trim(params.kzg.clone(), degree);

        let elements = field::split_data_into_field_elements::<E::ScalarField>(bytes, k);
        let mut polynomials = Vec::new();
        for chunk in elements.chunks(k) {
            polynomials.push(P::from_coefficients_vec(chunk.to_vec()))
        }

        let commit = commit(polynomials.clone(), params.clone()).unwrap();

        let encoding_points = &(0..n)
            .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
            .collect::<Vec<_>>();
        let encoding_mat = Matrix::vandermonde_unchecked(encoding_points, k);
        let shards = encode::<E::ScalarField>(bytes, &encoding_mat)
            .unwrap_or_else(|_| panic!("could not encode"));

        let blocks = prove::<E, P>(
            commit,
            polynomials,
            shards,
            encoding_points.clone(),
            params.clone(),
        )
        .unwrap();

        Ok((
            blocks,
            (vk_psi, params.ipa.tau_1),
            (
                params.kzg.powers_of_g[0].into_group(),
                params.kzg.h.into_group(),
            ),
        ))
    }

    fn verify_template<E, P>(bytes: &[u8], k: usize, n: usize) -> Result<(), ark_poly_commit::Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let (blocks, (vk_psi, tau_1), (g_1, g_2)) =
            test_setup::<E, P>(bytes, k, n).expect("proof failed for bls12-381");

        for (i, block) in blocks.iter().enumerate() {
            assert!(super::verify::<E, P>(
                block,
                E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(i as u32)),
                &vk_psi,
                tau_1,
                g_1,
                g_2
            )
            .unwrap());
        }

        Ok(())
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
        let (blocks, (vk_psi, tau_1), (g_1, g_2)) =
            test_setup::<E, P>(bytes, k, n).expect("proof failed for bls12-381");

        for (i, block) in blocks.iter().enumerate() {
            let mut b = block.clone();
            // modify a field in the struct b to corrupt the block proof without corrupting the data serialization
            let a = E::ScalarField::from_le_bytes_mod_order(&[123]);
            b.ipa_proof.l_r[0].mul_assign(a.pow([4321_u64]));

            assert!(
                !super::verify::<E, P>(
                    &b,
                    E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(i as u32)),
                    &vk_psi,
                    tau_1,
                    g_1,
                    g_2
                )
                .unwrap(),
                "aPlonK should fail for bls12-381, k = {} and a corrupted block",
                k
            );
        }

        Ok(())
    }

    #[test]
    fn verify_2() {
        verify_template::<Bls12_381, UniPoly381>(&bytes::<Bls12_381>(4, 2), 4, 6)
            .expect("verification failed for bls12-381");
    }

    #[test]
    fn verify_4() {
        verify_template::<Bls12_381, UniPoly381>(&bytes::<Bls12_381>(4, 4), 4, 6)
            .expect("verification failed for bls12-381");
    }

    #[test]
    fn verify_8() {
        verify_template::<Bls12_381, UniPoly381>(&bytes::<Bls12_381>(4, 8), 4, 6)
            .expect("verification failed for bls12-381");
    }

    #[test]
    fn verify_with_errors_2() {
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes::<Bls12_381>(4, 2), 4, 6)
            .expect("verification failed for bls12-381");
    }

    #[test]
    fn verify_with_errors_4() {
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes::<Bls12_381>(4, 4), 4, 6)
            .expect("verification failed for bls12-381");
    }

    #[test]
    fn verify_with_errors_8() {
        verify_with_errors_template::<Bls12_381, UniPoly381>(&bytes::<Bls12_381>(4, 8), 4, 6)
            .expect("verification failed for bls12-381");
    }

    // TODO: implement padding for aPlonK
    #[ignore = "padding not supported by aPlonK"]
    #[test]
    fn verify_with_padding_test() {
        let bytes = bytes::<Bls12_381>(4, 2);
        verify_template::<Bls12_381, UniPoly381>(&bytes[0..(bytes.len() - 33)], 4, 6)
            .expect("verification failed for bls12-381 with padding");
    }
}
