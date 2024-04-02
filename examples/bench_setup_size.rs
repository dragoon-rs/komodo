use std::ops::Div;

use ark_bls12_381::Bls12_381;
use ark_ec::pairing::Pairing;
use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;

use ark_serialize::{CanonicalSerialize, Compress, Validate};

type UniPoly12_381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

fn setup_template<E, P>(nb_bytes: usize)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let setup = komodo::setup::random::<E, P>(nb_bytes).unwrap();

    for (compress, validate) in [
        (Compress::Yes, Validate::Yes),
        (Compress::Yes, Validate::No),
        (Compress::No, Validate::Yes),
        (Compress::No, Validate::No),
    ] {
        let mut serialized = vec![0; setup.serialized_size(compress)];
        setup
            .serialize_with_mode(&mut serialized[..], compress)
            .unwrap();

        println!(
            r#"{{"reason": "benchmark-complete", "id": "serialized size with {} and {} {} on {}", "mean": {}}}"#,
            match compress {
                Compress::Yes => "compression",
                Compress::No => "no compression",
            },
            match validate {
                Validate::Yes => "validation",
                Validate::No => "no validation",
            },
            nb_bytes,
            std::any::type_name::<E>(),
            serialized.len(),
        );
    }
}

fn main() {
    for n in [1, 2, 4, 8, 16] {
        setup_template::<Bls12_381, UniPoly12_381>(n * 1024);
    }
}
