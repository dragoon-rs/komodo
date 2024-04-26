// see `examples/benches/README.md`
use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_serialize::{CanonicalSerialize, Compress};
use ark_std::ops::Div;

use komodo::zk;

fn setup_template<F, G, P>(degree: usize, curve: &str)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut rand::thread_rng();

    eprintln!("degree: {}", degree);

    let setup = zk::setup::<F, G>(degree, rng).unwrap();

    for compress in [Compress::Yes, Compress::No] {
        println!(
            r#"{{"reason": "benchmark-complete", "id": "serialized size with {} {} on {}", "mean": {}}}"#,
            match compress {
                Compress::Yes => "compression",
                Compress::No => "no compression",
            },
            degree,
            curve,
            setup.serialized_size(compress),
        );
    }
}

fn main() {
    fn aux<F: PrimeField, G: CurveGroup<ScalarField = F>>(degree: usize, curve: &str) {
        setup_template::<F, G, DensePolynomial<F>>(degree, curve);
    }

    for n in [1, 2, 4, 8, 16] {
        aux::<ark_bls12_381::Fr, ark_bls12_381::G1Projective>(n, "BLS12-381");
        aux::<ark_bn254::Fr, ark_bn254::G1Projective>(n, "BN-254");
        aux::<ark_pallas::Fr, ark_pallas::Projective>(n, "PALLAS");
    }
}
