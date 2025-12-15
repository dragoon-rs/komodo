//! Manipulate elements from finite field $\mathbb{F}_p$.
#[cfg(feature = "aplonk")]
use ark_ec::pairing::PairingOutput;
use ark_ff::{BigInteger, PrimeField};
#[cfg(any(feature = "kzg", feature = "aplonk"))]
use {
    ark_ec::pairing::Pairing,
    ark_poly::DenseUVPolynomial,
    ark_std::One,
    std::ops::{Div, Mul},
};

pub mod linalg;

/// Splits a sequence of raw bytes into valid field elements in $\mathbb{F}_p$.
///
/// The size of the output vector is a multiple of the provided `modulus` argument.
///
/// If necessary, the output vector is padded with $1$ in $\mathbb{F}_p$.
///
/// # Example
/// In the following example $\mathbb{F}_p$ is a small finite field with prime order $2^{16} + 1$ and which
/// requires only two bytes to represent elements.
///
/// 1. splitting `0x02000300`, which contains $4$ bytes, will result in two elements of $\mathbb{F}_p$, i.e. $2$
///    and $3$
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
/// 2. splitting `0x0200030004000500`, which contains $8$ bytes, and asking for a multiple of $3$
///    elements, will result in $6$ elements of $\mathbb{F}_p$, i.e. $2$, $3$, $4$ and $5$ which come from the data and
///    two padding elements, set to $1$.
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

/// Merges elements of $\mathbb{F}_p$ back into a sequence of bytes.
///
/// > **Note**
/// >
/// > This is the inverse operation of [`split_data_into_field_elements`].
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
/// Computes the linear combination of polynomials.
///
/// [`scalar_product_polynomial`] computes the linear combination $P$ of $n$
/// polynomials $(P_i) \in \mathbb{F}_p\[X\]^n \sim \texttt{lhs}$ with
/// coefficients $(c_i) \in \mathbb{F}_p^n \sim \texttt{rhs}$ as
///
/// $$P(X) = \sum\limits_{i = 0}^{n - 1} c_i P_i(X)$$
///
/// ## Preconditions
/// - `lhs` and `rhs` should contain the same number of elements.
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
/// Computes the "_scalar product_" between vectors of elements in $\mathbb{G}_1$ and in $\mathbb{G}_2$ respectively.
///
/// [`scalar_product_pairing`] computes the "_pairing combination_" $c$ of $(a_i) \in \mathbb{G}_1^n \sim \texttt{lhs}$ and
/// $(b_i) \in \mathbb{G}_2^n \sim \texttt{rhs}$ as
///
/// $$ c = \sum\limits_{i = 0}^{n - 1} E(a_i, b_i) $$
///
/// where $E$ is a [bilinear mapping] from $\mathbb{G}_1 \times \mathbb{G}_2 \rightarrow \mathbb{G}_T$.
///
/// ## Preconditions
/// - `lhs` and `rhs` should contain the same number of elements.
///
/// [bilinear mapping]: <https://en.wikipedia.org/wiki/Bilinear_map>
pub(super) fn scalar_product_pairing<E: Pairing>(lhs: &[E::G1], rhs: &[E::G2]) -> PairingOutput<E> {
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| E::pairing(l, r))
        .sum()
}

#[cfg(feature = "aplonk")]
/// Computes the [scalar product] between vectors of elements of a finite field $\mathbb{F}_p$
/// associated with a "_pairing-friendly_" [elliptic curve] $(\mathbb{G}_1, \mathbb{G}_2, \mathbb{G}_T)$.
///
/// [`scalar_product`] computes the [scalar product] $c$ of $(a_i) \in \mathbb{F}_p^n \sim \texttt{lhs}$ and
/// $(b_i) \in \mathbb{F}_p^n \sim \texttt{rhs}$ as
///
/// $$ c = a \cdot b = \sum\limits_{i = 0}^{n - 1} a_i b_i $$
///
/// ## Preconditions
/// - `lhs` and `rhs` should contain the same number of elements.
///
/// [scalar product]: <https://en.wikipedia.org/wiki/Dot_product>
/// [elliptic curve]: <https://en.wikipedia.org/wiki/Elliptic_curve>
pub(super) fn scalar_product<E: Pairing>(
    lhs: &[E::ScalarField],
    rhs: &[E::ScalarField],
) -> E::ScalarField {
    lhs.iter().zip(rhs.iter()).map(|(l, r)| l.mul(r)).sum()
}

#[cfg(feature = "aplonk")]
/// Computes a linear combination of elements of a finite field $\mathbb{F}_p$ associated with a
/// "_pairing-friendly_" [elliptic curve] $(\mathbb{G}_1, \mathbb{G}_2, \mathbb{G}_T)$.
///
/// [`scalar_product_g1`] computes the linear combination $c$ of the $(a_i) \in \mathbb{G}_1^n \sim \texttt{lhs}$
/// with coefficients $(c_i) \in \mathbb{F}_p^n \sim \texttt{rhs}$ as
///
/// $$ c = \sum\limits_{i = 0}^{n - 1} c_i a_i $$
///
/// > **Note**
/// >
/// > [`scalar_product_g1`] is the same as [`scalar_product`], but with elements from $\mathbb{G}_1$.
///
/// ## Preconditions
/// - `lhs` and `rhs` should contain the same number of elements.
///
/// [elliptic curve]: <https://en.wikipedia.org/wiki/Elliptic_curve>
pub(super) fn scalar_product_g1<E: Pairing>(lhs: &[E::G1], rhs: &[E::ScalarField]) -> E::G1 {
    lhs.iter().zip(rhs.iter()).map(|(l, r)| l.mul(r)).sum()
}

#[cfg(feature = "aplonk")]
/// Computes a linear combination of elements of a finite field $\mathbb{F}_p$ associated with a
/// "_pairing-friendly_" [elliptic curve] $(\mathbb{G}_1, \mathbb{G}_2, \mathbb{G}_T)$.
///
/// [`scalar_product_g2`] computes the linear combination $c$ of the $(a_i) \in \mathbb{G}_2^n \sim \texttt{lhs}$
/// with coefficients $(c_i) \in \mathbb{F}_p^n \sim \texttt{rhs}$ as
///
/// $$ c = \sum\limits_{i = 0}^{n - 1} c_i a_i $$
///
/// > **Note**
/// >
/// > [`scalar_product_g2`] is the same as [`scalar_product`], but with elements from $\mathbb{G}_2$.
///
/// [elliptic curve]: <https://en.wikipedia.org/wiki/Elliptic_curve>
pub(super) fn scalar_product_g2<E: Pairing>(lhs: &[E::G2], rhs: &[E::ScalarField]) -> E::G2 {
    lhs.iter().zip(rhs.iter()).map(|(l, r)| l.mul(r)).sum()
}

#[cfg(feature = "aplonk")]
pub(super) mod vector {
    use ark_ff::Zero;

    /// Returns $(0, ..., 0) \in \mathbb{F}_p^n$.
    pub fn zero<Z: Zero + Clone>(capacity: usize) -> Vec<Z> {
        let mut vector = Vec::with_capacity(capacity);
        vector.resize(capacity, Z::zero());

        vector
    }
}

/// Computes the successive powers of a scalar $r$ in a field $\mathbb{F}_p$ associated with a
/// "_pairing-friendly_" [elliptic curve] $(\mathbb{G}_1, \mathbb{G}_2, \mathbb{G}_T)$.
///
/// [`powers_of`] will compute a vector $R$ from a scalar $r \in \mathbb{F}_p$ as
///
/// $$ R = (1, r, r^2, ..., r^{n-1}) $$
///
/// where $n$ is the desired number of powers.
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

        let res = super::powers_of::<E>(r, POWER + 1);
        assert_eq!(res.len(), POWER + 1);
        assert_eq!(res.last().unwrap(), &r.pow([POWER as u64]));
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

        fn vec_to_elements<E: Pairing>(elements: &[u8]) -> Vec<E::ScalarField> {
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
                P::from_coefficients_vec(vec_to_elements::<E>(&[1])),
                P::from_coefficients_vec(vec_to_elements::<E>(&[0, 1])),
                P::from_coefficients_vec(vec_to_elements::<E>(&[0, 0, 1])),
                P::from_coefficients_vec(vec_to_elements::<E>(&[0, 0, 0, 1])),
            ];
            let coeffs = vec_to_elements::<E>(&[2, 3, 4, 5]);

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
        fn scalar_template<E: Pairing>(lhs: &[u8], rhs: &[u8], result: u8) {
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
            scalar_template::<Bls12_381>(&[1, 2], &[3, 4], 11);
            scalar_template::<Bls12_381>(&[5, 6], &[7, 8], 83);
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
