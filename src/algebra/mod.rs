//! Manipulate finite field elements
#[cfg(any(feature = "kzg", feature = "aplonk"))]
use ark_ec::pairing::Pairing;
#[cfg(feature = "aplonk")]
use ark_ec::pairing::PairingOutput;
use ark_ff::{BigInteger, PrimeField};
#[cfg(any(feature = "kzg", feature = "aplonk"))]
use ark_poly::DenseUVPolynomial;
#[cfg(any(feature = "kzg", feature = "aplonk"))]
use ark_std::One;
#[cfg(any(feature = "kzg", feature = "aplonk"))]
use std::ops::{Div, Mul};

pub mod linalg;

/// split a sequence of raw bytes into valid field elements
///
/// [`split_data_into_field_elements`] supports padding the output vector of
/// elements by giving a number that needs to divide the length of the vector.
///
/// # Example
/// In the following example `Fp` is a small finite field with prime order $65537$ and which
/// requires only two bytes to represent elements.
///
/// 1. splitting `0x02000300`, which contains 4 bytes, will result in two elements of `Fp`, i.e. 2
///    and 3
/// ```
/// # #[derive(ark_ff::MontConfig)]
/// # #[modulus = "65537"]
/// # #[generator = "3"]
/// # struct FpConfig_;
/// # type Fp = ark_ff::Fp64<ark_ff::MontBackend<FpConfig_, 1>>;
/// #
/// # use komodo::algebra::split_data_into_field_elements;
/// # use ark_ff::PrimeField;
/// # fn main() {
/// assert_eq!(
///     split_data_into_field_elements::<Fp>(&[2, 0, 3, 0], 1),
///     vec![Fp::from(2), Fp::from(3)],
/// );
/// # }
/// ```
/// 2. splitting `0x0200030004000500`, which contains 8 bytes, and asking for a multiple of 3
///    elements, will result in 6 elements of `Fp`, i.e. 2, 3, 4 and 5 which come from the data and
///    two padding elements, set to 1.
/// ```
/// # #[derive(ark_ff::MontConfig)]
/// # #[modulus = "65537"]
/// # #[generator = "3"]
/// # struct FpConfig_;
/// # type Fp = ark_ff::Fp64<ark_ff::MontBackend<FpConfig_, 1>>;
/// #
/// # use komodo::algebra::split_data_into_field_elements;
/// # use ark_ff::PrimeField;
/// # fn main() {
/// assert_eq!(
///     split_data_into_field_elements::<Fp>(&[2, 0, 3, 0, 4, 0, 5, 0], 3),
///     vec![
///         Fp::from(2),
///         Fp::from(3),
///         Fp::from(4),
///         Fp::from(5),
///         Fp::from(1),
///         Fp::from(1),
///     ],
/// );
/// # }
/// ```
pub fn split_data_into_field_elements<F: PrimeField>(bytes: &[u8], modulus: usize) -> Vec<F> {
    let bytes_per_element = (F::MODULUS_BIT_SIZE as usize - 1) / 8;

    let mut elements = Vec::new();
    for chunk in bytes.chunks(bytes_per_element) {
        elements.push(F::from_le_bytes_mod_order(chunk));
    }

    if elements.len() % modulus != 0 {
        elements.resize((elements.len() / modulus + 1) * modulus, F::one());
    }

    elements
}

/// merges elliptic curve elements back into a sequence of bytes
///
/// this is the inverse operation of [`split_data_into_field_elements`].
pub(crate) fn merge_elements_into_bytes<F: PrimeField>(elements: &[F]) -> Vec<u8> {
    let mut bytes = vec![];
    for e in elements {
        let mut b = e.into_bigint().to_bytes_le();
        b.pop();
        bytes.append(&mut b);
    }

    bytes
}

#[cfg(any(feature = "kzg", feature = "aplonk"))]
/// compute the linear combination of polynomials
///
/// if the _lhs_ are the coefficients, $(c_i)$ in a field $\mathbb{F}$, and the _rhs_ are the
/// polynomials, $(p_i)$ with coefficients in $\mathbb{F}$, then the result of this is
/// $$P(X) = \sum\limits_{i = 0}^{n - 1} c_i p_i(X)$$
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
/// compute the scalar product between vectors of elements in $G_1$ and in $G_2$ respectively
///
/// if the _lhs_ are the elements of $G_1$, $(a_i)$, and the _rhs_ are the ones from $G_2$, $(b_i)$,
/// then the result of this is
/// $$c = \sum\limits_{i = 0}^{n - 1} E(a_i, b_i)$$
/// where $E$ is a bilinear mapping from $G_1 \times G_2 \rightarrow G_T$
pub(super) fn scalar_product_pairing<E: Pairing>(lhs: &[E::G1], rhs: &[E::G2]) -> PairingOutput<E> {
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| E::pairing(l, r))
        .sum()
}

#[cfg(feature = "aplonk")]
/// compute the scalar product between vectors of elements of a finite field $\mathbb{F}$
///
/// if _lhs_ is the first vector, $(a_i)$, and _rhs_ is the second, $(b_i)$, then the result of this
/// is
/// $$c = \sum\limits_{i = 0}^{n - 1} a_i b_i$$
pub(super) fn scalar_product<E: Pairing>(
    lhs: &[E::ScalarField],
    rhs: &[E::ScalarField],
) -> E::ScalarField {
    lhs.iter().zip(rhs.iter()).map(|(l, r)| l.mul(r)).sum()
}

#[cfg(feature = "aplonk")]
/// see [`scalar_product`], but with _lhs_ a vector from $G_1$
pub(super) fn scalar_product_g1<E: Pairing>(lhs: &[E::G1], rhs: &[E::ScalarField]) -> E::G1 {
    lhs.iter().zip(rhs.iter()).map(|(l, r)| l.mul(r)).sum()
}

#[cfg(feature = "aplonk")]
/// see [`scalar_product`], but with _lhs_ a vector from $G_2$
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
#[cfg(any(feature = "kzg", feature = "aplonk"))]
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
    #[cfg(any(feature = "kzg", feature = "aplonk"))]
    use ark_bls12_381::Bls12_381;
    use ark_bls12_381::Fr;
    #[cfg(any(feature = "kzg", feature = "aplonk"))]
    use ark_ec::pairing::Pairing;
    #[cfg(any(feature = "kzg", feature = "aplonk"))]
    use ark_ff::Field;
    use ark_ff::PrimeField;
    #[cfg(any(feature = "kzg", feature = "aplonk"))]
    use ark_std::{test_rng, UniformRand};

    fn bytes() -> Vec<u8> {
        include_bytes!("../../assets/dragoon_32x32.png").to_vec()
    }

    fn split_data_template<F: PrimeField>(
        bytes: &[u8],
        modulus: usize,
        exact_length: Option<usize>,
    ) {
        let test_case = format!(
            "TEST | modulus: {}, exact_length: {:?}",
            modulus, exact_length
        );

        let elements = super::split_data_into_field_elements::<F>(bytes, modulus);
        assert!(
            elements.len() % modulus == 0,
            "number of elements should be divisible by {}, found {}\n{test_case}",
            modulus,
            elements.len(),
        );

        if let Some(length) = exact_length {
            assert!(
                elements.len() == length,
                "number of elements should be exactly {}, found {}\n{test_case}",
                length,
                elements.len(),
            );
        }

        assert!(
            !elements.iter().any(|&e| e == F::zero()),
            "elements should not contain any 0\n{test_case}"
        );
    }

    #[test]
    fn split_data() {
        split_data_template::<Fr>(&bytes(), 1, None);
        split_data_template::<Fr>(&bytes(), 8, None);
        split_data_template::<Fr>(&[], 1, None);
        split_data_template::<Fr>(&[], 8, None);

        const MODULUS_BYTE_SIZE: usize = Fr::MODULUS_BIT_SIZE as usize / 8;
        for n in (10 * MODULUS_BYTE_SIZE + 1)..(11 * MODULUS_BYTE_SIZE) {
            split_data_template::<Fr>(&bytes()[..n], 1, Some(11));
            split_data_template::<Fr>(&bytes()[..n], 8, Some(16));
        }
    }

    fn split_and_merge_template<F: PrimeField>(bytes: &[u8], modulus: usize) {
        let elements: Vec<F> = super::split_data_into_field_elements(bytes, modulus);
        let mut actual = super::merge_elements_into_bytes(&elements);
        actual.resize(bytes.len(), 0);
        assert_eq!(bytes, actual, "TEST | modulus: {modulus}");
    }

    #[test]
    fn split_and_merge() {
        for i in 0..12 {
            split_and_merge_template::<Fr>(&bytes(), 1 << i);
        }
    }

    #[cfg(any(feature = "kzg", feature = "aplonk"))]
    fn powers_of_template<E: Pairing>() {
        let rng = &mut test_rng();

        const POWER: usize = 10;
        let r = E::ScalarField::rand(rng);

        assert_eq!(
            super::powers_of::<E>(r, POWER + 1).last().unwrap(),
            &r.pow([POWER as u64])
        );
    }

    #[cfg(any(feature = "kzg", feature = "aplonk"))]
    #[test]
    fn powers_of() {
        powers_of_template::<Bls12_381>();
    }

    #[cfg(any(feature = "kzg", feature = "aplonk"))]
    mod scalar_product {
        use ark_bls12_381::Bls12_381;
        use ark_ec::pairing::Pairing;
        use ark_ff::PrimeField;
        use ark_poly::univariate::DensePolynomial;
        use ark_poly::DenseUVPolynomial;
        #[cfg(feature = "aplonk")]
        use ark_std::test_rng;
        #[cfg(feature = "aplonk")]
        use ark_std::UniformRand;
        #[cfg(feature = "aplonk")]
        use std::ops::Add;
        use std::ops::Div;

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

        #[cfg(feature = "aplonk")]
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

        #[cfg(feature = "aplonk")]
        #[test]
        fn scalar() {
            scalar_template::<Bls12_381>(vec![1, 2], vec![3, 4], 11);
            scalar_template::<Bls12_381>(vec![5, 6], vec![7, 8], 83);
        }

        #[cfg(feature = "aplonk")]
        #[ignore = "scalar_product_g1 is a clone of scalar_product"]
        #[test]
        fn g_1() {}

        #[cfg(feature = "aplonk")]
        #[ignore = "scalar_product_g2 is a clone of scalar_product"]
        #[test]
        fn g_2() {}

        #[cfg(feature = "aplonk")]
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

        #[cfg(feature = "aplonk")]
        #[test]
        fn pairing() {
            pairing_template::<Bls12_381>();
        }
    }
}
