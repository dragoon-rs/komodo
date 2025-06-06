use ark_ec::pairing::{Pairing, PairingOutput};
use ark_ff::Field;
use ark_poly::DenseUVPolynomial;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use std::ops::{Add, Div, Mul};

use crate::algebra::{
    powers_of, scalar_product_g1, scalar_product_g2, scalar_product_pairing, vector,
};
use crate::aplonk::polynomial;
use crate::aplonk::transcript;
use crate::error::KomodoError;

/// Holds the setup parameters of the IPA stage of [aPlonk from [Ambrona et al.]][aPlonK].
///
/// This can be found in [aPlonk from [Ambrona et al.]][aPlonK] in
/// - page **13**. in Setup.1
/// - page **13**. in Setup.3
///
/// [aPlonk]: https://eprint.iacr.org/2022/1352.pdf
#[derive(Debug, Clone, PartialEq, Default, CanonicalSerialize, CanonicalDeserialize)]
pub struct Params<E: Pairing> {
    /// $\[\tau\]_1$ in the paper
    pub tau_1: E::G1,
    /// $\text{ck}_\tau$ in the paper
    pub ck_tau: Vec<E::G2>,
}

/// holds all the necessary pieces to prove the IPA stage of [aPlonk from [Ambrona et al.]][aPlonK]
/// this can be found in [aPlonk from [Ambrona et al.]][aPlonK] as
/// $\pi = ({L_G^j, R_G^j, L_r^j, R_r^j}_{j \in [\kappa]}, \mu^0, G^0)$ in
/// - page **15**. in IPA.Prove.10
///
/// > **Note**  
/// > the notations are the same as in the paper, only with all letters in lower
/// > case and the powers at the bottom, e.g. `l_g_j` instead of $L_G^j$, and
/// > with $G$ rename as `ck_tau`.
///
/// [aPlonk]: https://eprint.iacr.org/2022/1352.pdf
#[derive(Debug, Clone, Default, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub(super) struct Proof<E: Pairing> {
    pub l_g: Vec<PairingOutput<E>>,
    pub r_g: Vec<PairingOutput<E>>,
    pub l_r: Vec<E::G1>,
    pub r_r: Vec<E::G1>,
    mu_0: E::G1,
    pub ck_tau_0: E::G2,
}

// compute if a number is a power of two
//
// generated by ChatGPT
fn is_power_of_two(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

/// prove a sequence of commits with a modified IPA
///
/// > **Note**  
/// > when we say *page xx* or *\<name of algorithm\>*, we refer to the following
/// > paper: [aPlonk from [Ambrona et al.]][aPlonK]
///
/// the following algorithm
/// - uses the same notations as in the aPlonK paper, modulus the fancy LaTeX fonts
/// - marks the steps of the paper algorithm with `// 1.` or `// 7.`
///
/// > **Note**  
/// > here, we do not use the red steps of the algorithm page **15**, e.g.
/// > we did not have to write the step 8. at all.
///
/// Arguments:
/// - k: number of polynomials (must be a power of 2)
/// - ck_tau: commitment key of IPA containing *k* values from *G_2*
/// - c_g: sum of pairing of commit, i.e. *com_f* in *Commit-Polys* page **13**
/// - r: random scalar
/// - P: random linear combination of the commits
/// - mu: the actual commits of the *k* polynomials
///
/// [aPlonk]: https://eprint.iacr.org/2022/1352.pdf
pub(super) fn prove<E: Pairing>(
    k: usize,
    ck_tau: &[E::G2],
    c_g: PairingOutput<E>,
    r: E::ScalarField,
    p: E::G1,
    mu: &[E::G1],
) -> Result<(Proof<E>, Vec<E::ScalarField>), KomodoError> {
    if !is_power_of_two(k) {
        return Err(KomodoError::Other(format!(
            "PolynomialCountIpaError: expected $k$ to be a power of 2, found {}",
            k
        )));
    }
    let kappa = f64::log2(k as f64) as usize;
    let mut l_g = vector::zero::<PairingOutput<E>>(kappa);
    let mut l_r = vector::zero::<E::G1>(kappa);
    let mut r_g = vector::zero::<PairingOutput<E>>(kappa);
    let mut r_r = vector::zero::<E::G1>(kappa);
    let mut u = vector::zero::<E::ScalarField>(kappa);

    // 1.
    let mut mu = mu.to_vec();
    let mut r_vec = powers_of::<E>(r, k);
    let mut ck_tau = ck_tau.to_vec();
    let mut ts = match transcript::initialize(c_g, r, p) {
        Ok(transcript) => transcript,
        Err(error) => return Err(KomodoError::Other(format!("SerializationError: {}", error))),
    };

    // 2.
    for j in (0..kappa).rev() {
        let (mu_left, mu_right) = mu.split_at(mu.len() / 2);
        let (ck_tau_left, ck_tau_right) = ck_tau.split_at(ck_tau.len() / 2);
        let (r_left, r_right) = r_vec.split_at(r_vec.len() / 2);

        // 3.
        l_g[j] = scalar_product_pairing(mu_left, ck_tau_right);
        l_r[j] = scalar_product_g1::<E>(mu_left, r_right);

        // 4.
        r_g[j] = scalar_product_pairing(mu_right, ck_tau_left);
        r_r[j] = scalar_product_g1::<E>(mu_right, r_left);

        // 5.
        u[j] = match transcript::hash(l_g[j], r_g[j], l_r[j], r_r[j], &ts) {
            Ok(hash) => hash,
            Err(error) => return Err(KomodoError::Other(format!("SerializationError: {}", error))),
        };
        ts = match transcript::reset::<E>(u[j]) {
            Ok(transcript) => transcript,
            Err(error) => return Err(KomodoError::Other(format!("SerializationError: {}", error))),
        };

        let u_j_inv = if let Some(inverse) = u[j].inverse() {
            inverse
        } else {
            return Err(KomodoError::Other(format!(
                "EllipticInverseError: could not inverse {:?}",
                u[j],
            )));
        };

        // 6.
        mu = mu_left
            .iter()
            .zip(mu_right.iter())
            .map(|(l, r)| l.mul(u[j]) + r.mul(u_j_inv))
            .collect();
        // 7.
        ck_tau = ck_tau_left
            .iter()
            .zip(ck_tau_right.iter())
            .map(|(l, r)| l.mul(u_j_inv) + r.mul(u[j]))
            .collect();
        // 9.
        r_vec = r_left
            .iter()
            .zip(r_right.iter())
            .map(|(l, r)| l.mul(u_j_inv) + r.mul(u[j]))
            .collect();
    }

    // 10.
    Ok((
        Proof {
            l_g,
            r_g,
            l_r,
            r_r,
            mu_0: mu[0],
            ck_tau_0: ck_tau[0],
        },
        u,
    ))
}

/// verify the integrity of a proven sequence of commits with a modified IPA
///
/// > **Note**  
/// > when we say *page xx* or *\<name of algorithm\>*, we refer to the following
/// > paper: [aPlonk from [Ambrona et al.]][aPlonK]
///
/// the following algorithm
/// - uses the same notations as in the aPlonK paper, modulus the fancy LaTeX fonts
/// - marks the steps of the paper algorithm with `// 1.` or `// 7.`
///
/// > **Note**  
/// > here, we do not use the red steps of the algorithm page **15**, e.g.
/// > we did not have to write the step 7.2. at all.
///
/// Arguments:
/// - k: number of polynomials (must be a power of 2)
/// - ck_tau: commitment key of IPA containing *k* values from *G_2*
/// when set to `None`, will turn *IPA.Verify* into *IPA.Verify'*, without the
/// "scalar product" guard and which is the version of the algorithm used in
/// the *Open* algorithm of aPlonK.
/// - c_g: sum of pairing of commit, i.e. *com_f* in *Commit-Polys* page **13**
/// - r: random scalar
/// - P: random linear combination of the commits
/// - proof: the proof crafted by the *IPA.Prove* algorithm
///
/// [aPlonk]: https://eprint.iacr.org/2022/1352.pdf
pub(super) fn verify<E, P>(
    k: usize,
    ck_tau: Option<&Vec<E::G2>>, // this is an option to implement `IPA.Verify'`
    c_g: PairingOutput<E>,
    r: E::ScalarField,
    p: E::G1,
    proof: &Proof<E>,
) -> Result<bool, KomodoError>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    if !is_power_of_two(k) {
        return Err(KomodoError::Other(format!(
            "PolynomialCountIpaError: expected $k$ to be a power of 2, found {}",
            k,
        )));
    }
    let kappa = f64::log2(k as f64) as usize;
    let mut ts = match transcript::initialize(c_g, r, p) {
        Ok(transcript) => transcript,
        Err(error) => return Err(KomodoError::Other(format!("SerializationError: {}", error))),
    };
    let mut u = vector::zero::<E::ScalarField>(kappa);

    // 1.
    let l_g = &proof.l_g;
    let r_g = &proof.r_g;
    let l_r = &proof.l_r;
    let r_r = &proof.r_r;
    let mu_0 = proof.mu_0;
    let ck_tau_0 = proof.ck_tau_0;

    // 2.
    for j in (0..kappa).rev() {
        // 3.
        u[j] = match transcript::hash(l_g[j], r_g[j], l_r[j], r_r[j], &ts) {
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
            return Err(KomodoError::Other(format!(
                "EllipticInverseError: could not inverse {:?}",
                u_i,
            )));
        }
    }

    // 4.
    let g = polynomial::compute_g::<E, P>(k, kappa, &u, &u_inv);

    // 5.
    let r_0 = g.evaluate(&r);

    // 6.
    // implicit because the polynomial `g` *is* equivalent to its list of
    // coefficients, in the same order as in the paper.

    // 7.
    if let Some(ck_tau) = ck_tau {
        // implements `IPA.Verify'` without the guard
        if scalar_product_g2::<E>(ck_tau, g.coeffs()) != ck_tau_0 {
            return Ok(false);
        }
    }

    // 8.
    let r_sum: E::G1 = u
        .iter()
        .zip(u_inv.iter())
        .zip(l_r.iter().zip(r_r.iter()))
        .map(|((u_j, u_j_inv), (l_r_j, r_r_j))| {
            let lhs = l_r_j.mul(u_j.mul(u_j));
            let rhs = r_r_j.mul(u_j_inv.mul(u_j_inv));

            lhs.add(rhs)
        })
        .sum();
    let g_sum: PairingOutput<E> = u
        .iter()
        .zip(u_inv.iter())
        .zip(l_g.iter().zip(r_g.iter()))
        .map(|((u_j, u_j_inv), (l_g_j, r_g_j))| {
            let lhs = l_g_j.mul(u_j.mul(u_j));
            let rhs = r_g_j.mul(u_j_inv.mul(u_j_inv));

            lhs.add(rhs)
        })
        .sum();

    let lhs = mu_0.mul(r_0) == p.add(r_sum);
    let rhs = E::pairing(mu_0, ck_tau_0) == c_g.add(g_sum);

    Ok(lhs && rhs)
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Bls12_381;
    use ark_ec::pairing::{Pairing, PairingOutput};
    use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
    use ark_poly_commit::Error;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
    use ark_std::{test_rng, UniformRand};
    use std::ops::Div;

    use super::{is_power_of_two, Proof};
    use crate::algebra::{powers_of, scalar_product_g1, scalar_product_pairing};
    use crate::aplonk::setup;

    type UniPoly381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

    // generated by ChatGPT
    #[test]
    fn power_of_two() {
        // Powers of 2 should return true
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(8));
        assert!(is_power_of_two(16));

        // Non-powers of 2 should return false
        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(5));
        assert!(!is_power_of_two(10));
        assert!(!is_power_of_two(15));
    }

    #[allow(clippy::type_complexity)]
    fn test_setup<E, P>(
        k: usize,
        degree: usize,
    ) -> Result<
        (
            Vec<E::G2>,
            PairingOutput<E>,
            E::ScalarField,
            E::G1,
            Proof<E>,
        ),
        Error,
    >
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let rng = &mut test_rng();

        let mu: Vec<E::G1> = (0..k).map(|_| E::G1::rand(rng)).collect();

        let params = setup::<E, P>(degree, k)?;
        let ck_tau = params.ipa.ck_tau;

        let r = E::ScalarField::rand(rng);

        let c_g = scalar_product_pairing::<E>(&mu, &ck_tau);
        let p = scalar_product_g1::<E>(&mu, &powers_of::<E>(r, k));

        let (proof, _) = super::prove::<E>(k, &ck_tau, c_g, r, p, &mu).unwrap();

        Ok((ck_tau, c_g, r, p, proof))
    }

    fn verify_template<E, P>(k: usize, degree: usize) -> Result<(), Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let (ck_tau, c_g, r, p, proof) = test_setup::<E, P>(k, degree)?;

        assert!(
            super::verify::<E, P>(k, Some(&ck_tau), c_g, r, p, &proof).unwrap(),
            "IPA failed for bls12-381 and k = {}",
            k
        );

        Ok(())
    }

    fn verify_with_errors_template<E, P>(k: usize, degree: usize) -> Result<(), Error>
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let (ck_tau, c_g, r, p, proof) = test_setup::<E, P>(k, degree)?;

        let mut bytes = Vec::new();
        proof
            .serialize_with_mode(&mut bytes, Compress::Yes)
            .expect("Could not serialize the proof");
        bytes[10] += 1;
        let proof = Proof::deserialize_with_mode(&*bytes, Compress::Yes, Validate::No)
            .expect("Could not deserialize the corrupted proof");
        assert!(
            !super::verify::<E, P>(k, Some(&ck_tau), c_g, r, p, &proof).unwrap(),
            "IPA should fail for bls12-381, k = {} and a corrupted proof",
            k
        );

        Ok(())
    }

    const DEGREE_BOUND: usize = 32;

    #[test]
    fn verify() {
        for k in [2, 4, 8, 16, 32] {
            verify_template::<Bls12_381, UniPoly381>(k, DEGREE_BOUND).unwrap();
        }
    }

    #[test]
    fn verify_with_errors() {
        for k in [2, 4, 8, 16, 32] {
            verify_with_errors_template::<Bls12_381, UniPoly381>(k, DEGREE_BOUND).unwrap();
        }
    }
}
