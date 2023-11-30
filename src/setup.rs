use std::ops::Div;

use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::{Powers, UniversalParams, VerifierKey, KZG10};
use ark_std::test_rng;

/// Specializes the public parameters for a given maximum degree `d` for polynomials
/// `d` should be less that `pp.max_degree()`.
///
/// > see [`ark-poly-commit::kzg10::tests::KZG10`](https://github.com/jdetchart/poly-commit/blob/master/src/kzg10/mod.rs#L509)
pub fn trim<E: Pairing>(
    pp: UniversalParams<E>,
    supported_degree: usize,
) -> (Powers<'static, E>, VerifierKey<E>) {
    let powers_of_g = pp.powers_of_g[..=supported_degree].to_vec();
    let powers_of_gamma_g = (0..=supported_degree)
        .map(|i| pp.powers_of_gamma_g[&i])
        .collect();

    let powers = Powers {
        powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
        powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
    };
    let vk = VerifierKey {
        g: pp.powers_of_g[0],
        gamma_g: pp.powers_of_gamma_g[&0],
        h: pp.h,
        beta_h: pp.beta_h,
        prepared_h: pp.prepared_h.clone(),
        prepared_beta_h: pp.prepared_beta_h.clone(),
    };

    (powers, vk)
}

pub fn random<E, P>(nb_bytes: usize) -> Result<Powers<'static, E>, ark_poly_commit::Error>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let degree = nb_bytes / (E::ScalarField::MODULUS_BIT_SIZE as usize / 8);

    let rng = &mut test_rng();

    let params = KZG10::<E, P>::setup(degree, false, rng)?;
    let (powers, _) = trim(params, degree);

    Ok(powers)
}
