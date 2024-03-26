//! create and interact with ZK trusted setups
use std::ops::Div;

use anyhow::Result;

use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::{Powers, UniversalParams, VerifierKey, KZG10};
use ark_std::test_rng;

/// Specializes the public parameters for a given maximum degree `d` for polynomials
///
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

/// build a random trusted setup for a given number of bytes
///
/// `nb_bytes` will be divided by the "_modulus size_" of the elliptic curve to
/// get the number of powers of the secret to generate, e.g. creating a trusted
/// setup for 10kib on BLS-12-381 requires 331 powers of $\tau$.
///
/// /!\ Should be used only for tests, not for any real world usage. /!\
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

#[cfg(test)]
mod tests {
    use std::ops::Div;

    use ark_bls12_381::Bls12_381;
    use ark_ec::pairing::Pairing;
    use ark_ff::PrimeField;
    use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};

    use super::random;

    type UniPoly381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

    /// computes $a / b$ rounded to the integer above
    ///
    /// > **Note**
    /// > co-authored by ChatGPT
    fn ceil_divide(a: usize, b: usize) -> usize {
        (a + b - 1) / b
    }

    #[test]
    fn test_ceil_divide() {
        assert_eq!(ceil_divide(10, 2), 5);
        assert_eq!(ceil_divide(10, 3), 4);
        assert_eq!(ceil_divide(10, 6), 2);
    }

    fn random_setup_size_template<E, P>(nb_bytes: usize)
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let powers = random::<E, P>(nb_bytes);
        assert!(powers.is_ok());

        assert_eq!(
            powers.unwrap().powers_of_g.to_vec().len(),
            ceil_divide(nb_bytes, E::ScalarField::MODULUS_BIT_SIZE as usize / 8)
        );
    }

    #[test]
    fn random_setup_size() {
        random_setup_size_template::<Bls12_381, UniPoly381>(10 * 1_024);
    }
}
