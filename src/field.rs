use ark_ec::pairing::Pairing;
use ark_ff::{BigInteger, PrimeField};
use ark_std::One;

/// split a sequence of raw bytes into valid field elements
///
/// [`split_data_into_field_elements`] supports padding the output vector of
/// elements by giving a number that needs to divide the length of the vector.
pub fn split_data_into_field_elements<E: Pairing>(
    bytes: &[u8],
    modulus: usize,
) -> Vec<E::ScalarField> {
    let bytes_per_element = (E::ScalarField::MODULUS_BIT_SIZE as usize) / 8;

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

/// merges elliptic curve elements back into a sequence of bytes
///
/// this is the inverse operation of [`split_data_into_field_elements`].
pub(crate) fn merge_elements_into_bytes<E: Pairing>(elements: &[E::ScalarField]) -> Vec<u8> {
    let mut bytes = vec![];
    for e in elements {
        let mut b = e.into_bigint().to_bytes_le();
        b.pop();
        bytes.append(&mut b);
    }

    bytes
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Bls12_381;
    use ark_ec::pairing::Pairing;
    use ark_ff::PrimeField;
    use ark_std::Zero;

    use crate::field::{self, merge_elements_into_bytes};

    fn bytes() -> Vec<u8> {
        include_bytes!("../tests/dragoon_32x32.png").to_vec()
    }

    fn split_data_template<E: Pairing>(bytes: &[u8], modulus: usize, exact_length: Option<usize>) {
        let test_case = format!(
            "TEST | modulus: {}, exact_length: {:?}",
            modulus, exact_length
        );

        let elements = field::split_data_into_field_elements::<E>(bytes, modulus);
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
            !elements.iter().any(|&e| e == E::ScalarField::zero()),
            "elements should not contain any 0\n{test_case}"
        );
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

    fn split_and_merge_template<E: Pairing>(bytes: &[u8], modulus: usize) {
        let elements = field::split_data_into_field_elements::<E>(bytes, modulus);
        let mut actual = merge_elements_into_bytes::<E>(&elements);
        actual.resize(bytes.len(), 0);
        assert_eq!(bytes, actual, "TEST | modulus: {modulus}");
    }

    #[test]
    fn split_and_merge() {
        split_and_merge_template::<Bls12_381>(&bytes(), 1);
        split_and_merge_template::<Bls12_381>(&bytes(), 8);
        split_and_merge_template::<Bls12_381>(&bytes(), 64);
        split_and_merge_template::<Bls12_381>(&bytes(), 4096);
    }
}
