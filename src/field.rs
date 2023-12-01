use ark_ec::pairing::Pairing;
use ark_ff::{BigInteger, PrimeField};
use ark_poly::DenseUVPolynomial;
use ark_std::ops::Div;
use ark_std::One;

/// split a sequence of raw bytes into valid field elements
///
/// [`split_data_into_field_elements`] supports padding the output vector of
/// elements by giving a number that needs to divide the length of the vector.
pub(crate) fn split_data_into_field_elements<E: Pairing>(
    bytes: &[u8],
    modulus: usize,
    one_more: bool,
) -> Vec<E::ScalarField> {
    let bytes_per_element = if one_more {
        (E::ScalarField::MODULUS_BIT_SIZE as usize) / 8 + 1
    } else {
        (E::ScalarField::MODULUS_BIT_SIZE as usize) / 8
    };

    let mut elements = Vec::new();
    for chunk in bytes.chunks(bytes_per_element) {
        elements.push(E::ScalarField::from_le_bytes_mod_order(chunk));
    }

    if elements.len() % modulus != 0 {
        elements.resize(
            (elements.len() / modulus + 1) * modulus,
            E::ScalarField::one(),
        );
    }

    elements
}

pub(crate) fn merge_elements_into_bytes<E: Pairing>(elements: &[E::ScalarField]) -> Vec<u8> {
    let mut bytes = vec![];
    for e in elements {
        bytes.append(&mut e.into_bigint().to_bytes_le());
    }

    bytes
}

// create a set of polynomials containing k coefficients (#polynomials = |elements| / k)
//
// # Implementation
// as Dragoon uses FEC encoding to share data over a network of peers, we have
// some contraints on the way we compute the polynomials from the data.
//
// with a *(k, n)* code, the output of the encoding mixes all the coefficients
// of the original data.
// more specifically, all the constant coefficients come first, then the ones
// of *X*, then *X^2*, and so forth.
// this is where interleaving comes in handy! but let's take an example to
// understand the algorithm below.
//
// ## Example
// let's say we have 12 elements, namely *(e_0, e_1, ..., e_11)*, and we want to
// use a *(4, n)* code.
// we will then have 3 polynomials with 4 coefficients each:
// - *P_0 = e_0 + e_3 X + e_6 X^2 + e_9  X^3 = [e_0, e_3, e_6, e_9]*
// - *P_1 = e_1 + e_4 X + e_7 X^2 + e_10 X^3 = [e_1, e_4, e_7, e_10]*
// - *P_2 = e_2 + e_5 X + e_8 X^2 + e_11 X^3 = [e_2, e_5, e_8, e_11]*
//
// we can see that in each polynomial, the indices on *e_j* satisfy:
//     *j % 3 == i*
//   where *i* is the index of the polynomial.
//
// and we have:
// - *P_0*: 0, 3, 6 and 9 all satifsy *j % 3 == 0*
// - *P_1*: 1, 4, 7 and 10 all satifsy *j % 3 == 1*
// - *P_2*: 2, 5, 8 and 11 all satifsy *j % 3 == 2*
pub(crate) fn build_interleaved_polynomials<E, P>(
    elements: &[E::ScalarField],
    nb_polynomials: usize,
) -> Option<Vec<P>>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    if elements.len() % nb_polynomials != 0 {
        return None;
    }

    let mut polynomials = Vec::new();
    for i in 0..nb_polynomials {
        let coefficients = elements
            .iter()
            .enumerate()
            .filter(|(j, _)| j % nb_polynomials == i)
            .map(|(_, v)| *v)
            .collect::<Vec<_>>();
        polynomials.push(P::from_coefficients_vec(coefficients));
    }

    Some(polynomials)
}

#[cfg(test)]
mod tests {
    use std::ops::Div;

    use ark_bls12_381::Bls12_381;
    use ark_ec::pairing::Pairing;
    use ark_ff::PrimeField;
    use ark_poly::univariate::DensePolynomial;
    use ark_poly::DenseUVPolynomial;
    use ark_std::{test_rng, UniformRand, Zero};

    use crate::field;

    type UniPoly381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

    fn bytes() -> Vec<u8> {
        include_bytes!("../tests/dragoon_32x32.png").to_vec()
    }

    fn split_data_template<E: Pairing>(bytes: &[u8], modulus: usize, exact_length: Option<usize>) {
        let elements = field::split_data_into_field_elements::<E>(bytes, modulus, false);
        assert!(
            elements.len() % modulus == 0,
            "number of elements should be divisible by {}, found {}",
            modulus,
            elements.len()
        );

        if let Some(length) = exact_length {
            assert!(
                elements.len() == length,
                "number of elements should be exactly {}, found {}",
                length,
                elements.len()
            );
        }

        assert!(!elements.iter().any(|&e| e == E::ScalarField::zero()));
    }

    #[test]
    fn split_data() {
        split_data_template::<Bls12_381>(&bytes(), 1, None);
        split_data_template::<Bls12_381>(&bytes(), 8, None);
        split_data_template::<Bls12_381>(&[], 1, None);
        split_data_template::<Bls12_381>(&[], 8, None);

        let nb_bytes = 11 * (<Bls12_381 as Pairing>::ScalarField::MODULUS_BIT_SIZE as usize / 8);
        split_data_template::<Bls12_381>(&bytes()[..nb_bytes], 1, Some(11));
        split_data_template::<Bls12_381>(&bytes()[..nb_bytes], 8, Some(16));

        let nb_bytes =
            11 * (<Bls12_381 as Pairing>::ScalarField::MODULUS_BIT_SIZE as usize / 8) - 10;
        split_data_template::<Bls12_381>(&bytes()[..nb_bytes], 1, Some(11));
        split_data_template::<Bls12_381>(&bytes()[..nb_bytes], 8, Some(16));
    }

    fn build_interleaved_polynomials_template<E, P>(
        nb_elements: usize,
        m: usize,
        expected: Option<Vec<Vec<usize>>>,
    ) where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let rng = &mut test_rng();

        let elements = (0..nb_elements)
            .map(|_| E::ScalarField::rand(rng))
            .collect::<Vec<_>>();

        let actual = field::build_interleaved_polynomials::<E, P>(&elements, m);

        let expected = if let Some(expected) = expected {
            Some(
                expected
                    .iter()
                    .map(|r| {
                        P::from_coefficients_vec(r.iter().map(|&i| elements[i]).collect::<Vec<_>>())
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn build_interleaved_polynomials() {
        build_interleaved_polynomials_template::<Bls12_381, UniPoly381>(
            12,
            2,
            Some(vec![vec![0, 2, 4, 6, 8, 10], vec![1, 3, 5, 7, 9, 11]]),
        );
        build_interleaved_polynomials_template::<Bls12_381, UniPoly381>(
            12,
            3,
            Some(vec![vec![0, 3, 6, 9], vec![1, 4, 7, 10], vec![2, 5, 8, 11]]),
        );
        build_interleaved_polynomials_template::<Bls12_381, UniPoly381>(
            12,
            4,
            Some(vec![
                vec![0, 4, 8],
                vec![1, 5, 9],
                vec![2, 6, 10],
                vec![3, 7, 11],
            ]),
        );
        build_interleaved_polynomials_template::<Bls12_381, UniPoly381>(
            12,
            6,
            Some(vec![
                vec![0, 6],
                vec![1, 7],
                vec![2, 8],
                vec![3, 9],
                vec![4, 10],
                vec![5, 11],
            ]),
        );

        build_interleaved_polynomials_template::<Bls12_381, UniPoly381>(12, 5, None);
        build_interleaved_polynomials_template::<Bls12_381, UniPoly381>(12, 7, None);
        build_interleaved_polynomials_template::<Bls12_381, UniPoly381>(12, 34, None);
    }
}
