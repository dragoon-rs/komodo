use ark_bls12_381::{Fr, G1Projective};
use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_serialize::{CanonicalSerialize, Compress, Validate};
use ark_std::{ops::Div, test_rng};

use komodo::zk;

fn setup_template<F, G, P>(nb_bytes: usize)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut test_rng();

    let setup = zk::setup::<_, F, G>(zk::nb_elements_in_setup::<F>(nb_bytes), rng).unwrap();

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
            std::any::type_name::<F>(),
            serialized.len(),
        );
    }
}

fn main() {
    for n in [1, 2, 4, 8, 16] {
        setup_template::<Fr, G1Projective, DensePolynomial<Fr>>(n * 1024);
    }
}
