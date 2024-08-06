use ark_ec::pairing::{Pairing, PairingOutput};
use ark_poly::DenseUVPolynomial;
use ark_std::One;
use std::ops::{Div, Mul};

#[cfg(feature = "kzg")]
pub(crate) fn scalar_product_polynomial<E, P>(lhs: &[E::ScalarField], rhs: &[P]) -> P
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut polynomial = P::from_coefficients_vec(Vec::new());
    for (p, s) in rhs.iter().zip(lhs.iter()) {
        let coefficients: Vec<E::ScalarField> = p
            .coeffs()
            .iter()
            .map(|coefficient| coefficient.mul(s))
            .collect();
        polynomial = polynomial.add(P::from_coefficients_vec(coefficients));
    }

    polynomial
}

#[cfg(feature = "aplonk")]
pub(super) fn scalar_product_pairing<E: Pairing>(lhs: &[E::G1], rhs: &[E::G2]) -> PairingOutput<E> {
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| E::pairing(l, r))
        .sum()
}

#[cfg(feature = "aplonk")]
pub(super) fn scalar_product<E: Pairing>(
    lhs: &[E::ScalarField],
    rhs: &[E::ScalarField],
) -> E::ScalarField {
    lhs.iter().zip(rhs.iter()).map(|(l, r)| l.mul(r)).sum()
}

#[cfg(feature = "aplonk")]
pub(super) fn scalar_product_g1<E: Pairing>(lhs: &[E::G1], rhs: &[E::ScalarField]) -> E::G1 {
    lhs.iter().zip(rhs.iter()).map(|(l, r)| l.mul(r)).sum()
}

#[cfg(feature = "aplonk")]
pub(super) fn scalar_product_g2<E: Pairing>(lhs: &[E::G2], rhs: &[E::ScalarField]) -> E::G2 {
    lhs.iter().zip(rhs.iter()).map(|(l, r)| l.mul(r)).sum()
}

#[cfg(feature = "aplonk")]
pub(super) mod vector {
    use ark_ff::Zero;

    /// return [0, 0, ..., 0] of size *n* on some group
    pub fn zero<Z: Zero + Clone>(capacity: usize) -> Vec<Z> {
        let mut vector = Vec::with_capacity(capacity);
        vector.resize(capacity, Z::zero());

        vector
    }
}

/// compute the successive powers of a scalar group element
///
/// if the scalar number is called *r*, then [`powers_of`] will return the
/// following vector:
///         [1, r, r^2, ..., r^(n-1)]
/// where *n* is the number of powers
pub(crate) fn powers_of<E: Pairing>(step: E::ScalarField, nb_powers: usize) -> Vec<E::ScalarField> {
    let mut powers = Vec::with_capacity(nb_powers);
    powers.push(E::ScalarField::one());
    for j in 1..nb_powers {
        powers.push(powers[j - 1].mul(step));
    }

    powers
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Bls12_381;
    use ark_ec::pairing::Pairing;
    use ark_ff::Field;
    use ark_std::test_rng;
    use ark_std::UniformRand;

    fn powers_of_template<E: Pairing>() {
        let rng = &mut test_rng();

        const POWER: usize = 10;
        let r = E::ScalarField::rand(rng);

        assert_eq!(
            super::powers_of::<E>(r, POWER + 1).last().unwrap(),
            &r.pow([POWER as u64])
        );
    }

    #[test]
    fn powers_of() {
        powers_of_template::<Bls12_381>();
    }

    mod scalar_product {
        use ark_bls12_381::Bls12_381;
        use ark_ec::pairing::Pairing;
        use ark_ff::PrimeField;
        use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
        use ark_std::test_rng;
        use ark_std::UniformRand;
        use std::ops::{Add, Div};

        type UniPoly381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

        fn vec_to_elements<E: Pairing>(elements: Vec<u8>) -> Vec<E::ScalarField> {
            elements
                .iter()
                .map(|&x| E::ScalarField::from_le_bytes_mod_order(&[x]))
                .collect()
        }

        fn polynomial_template<E, P>()
        where
            E: Pairing,
            P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
            for<'a, 'b> &'a P: Div<&'b P, Output = P>,
        {
            let polynomials = vec![
                P::from_coefficients_vec(vec_to_elements::<E>(vec![1])),
                P::from_coefficients_vec(vec_to_elements::<E>(vec![0, 1])),
                P::from_coefficients_vec(vec_to_elements::<E>(vec![0, 0, 1])),
                P::from_coefficients_vec(vec_to_elements::<E>(vec![0, 0, 0, 1])),
            ];
            let coeffs = vec_to_elements::<E>(vec![2, 3, 4, 5]);

            assert_eq!(
                super::super::scalar_product_polynomial::<E, P>(&coeffs, &polynomials),
                P::from_coefficients_vec(coeffs)
            )
        }

        #[test]
        fn polynomial() {
            polynomial_template::<Bls12_381, UniPoly381>();
        }

        fn scalar_template<E: Pairing>(lhs: Vec<u8>, rhs: Vec<u8>, result: u8) {
            let lhs = lhs
                .iter()
                .map(|x| E::ScalarField::from_le_bytes_mod_order(&[*x]))
                .collect::<Vec<_>>();
            let rhs = rhs
                .iter()
                .map(|x| E::ScalarField::from_le_bytes_mod_order(&[*x]))
                .collect::<Vec<_>>();
            let result = E::ScalarField::from_le_bytes_mod_order(&[result]);

            assert_eq!(super::super::scalar_product::<E>(&lhs, &rhs), result);
        }

        #[test]
        fn scalar() {
            scalar_template::<Bls12_381>(vec![1, 2], vec![3, 4], 11);
            scalar_template::<Bls12_381>(vec![5, 6], vec![7, 8], 83);
        }

        #[ignore = "scalar_product_g1 is a clone of scalar_product"]
        #[test]
        fn g_1() {}

        #[ignore = "scalar_product_g2 is a clone of scalar_product"]
        #[test]
        fn g_2() {}

        fn pairing_template<E: Pairing>() {
            let rng = &mut test_rng();

            let g_1 = E::G1::rand(rng);
            let g_2 = E::G2::rand(rng);

            let pairing = E::pairing(g_1, g_2);
            let two_pairings = pairing.add(pairing);

            assert_eq!(
                super::super::scalar_product_pairing::<E>(&[g_1, g_1], &[g_2, g_2]),
                two_pairings
            );
        }

        #[test]
        fn pairing() {
            pairing_template::<Bls12_381>();
        }
    }
}
