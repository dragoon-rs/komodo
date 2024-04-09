use ark_bls12_381::{Fr, G1Projective};
use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_serialize::{CanonicalSerialize, Compress, Validate};
use ark_std::ops::Div;

use komodo::zk;

fn setup_template<F, G, P>(degree: usize)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut rand::thread_rng();

    eprintln!("degree: {}", degree);

    let setup = zk::setup::<_, F, G>(degree, rng).unwrap();

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
            degree,
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
