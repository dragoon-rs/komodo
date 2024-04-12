use std::time::Duration;

use ark_ec::{pairing::Pairing, CurveGroup};
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_poly_commit::kzg10::{self, KZG10};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use ark_std::ops::Div;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use komodo::zk::{self, Powers};

fn setup_template<F, G, P>(c: &mut Criterion, degree: usize, curve: &str)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut rand::thread_rng();

    c.bench_function(&format!("setup (komodo) {} on {}", degree, curve), |b| {
        b.iter(|| zk::setup::<F, G>(degree, rng).unwrap())
    });
}

fn ark_setup_template<E, P>(c: &mut Criterion, degree: usize, curve: &str)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut rand::thread_rng();

    c.bench_function(
        &format!("setup (arkworks) {} bytes on {}", degree, curve),
        |b| {
            b.iter(|| {
                let setup = KZG10::<E, P>::setup(degree, false, rng).unwrap();
                let powers_of_g = setup.powers_of_g[..=degree].to_vec();
                let powers_of_gamma_g = (0..=degree).map(|i| setup.powers_of_gamma_g[&i]).collect();
                kzg10::Powers::<E> {
                    powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
                    powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
                }
            })
        },
    );
}

fn serde_template<F, G, P>(c: &mut Criterion, degree: usize, curve: &str)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut group = c.benchmark_group("setup");

    let rng = &mut rand::thread_rng();

    let setup = zk::setup::<F, G>(degree, rng).unwrap();

    group.bench_function(
        &format!("serializing with compression {} on {}", degree, curve),
        |b| {
            b.iter(|| {
                let mut serialized = vec![0; setup.serialized_size(Compress::Yes)];
                setup
                    .serialize_with_mode(&mut serialized[..], Compress::Yes)
                    .unwrap();
            })
        },
    );

    group.bench_function(
        &format!("serializing with no compression {} on {}", degree, curve),
        |b| {
            b.iter(|| {
                let mut serialized = vec![0; setup.serialized_size(Compress::No)];
                setup
                    .serialize_with_mode(&mut serialized[..], Compress::No)
                    .unwrap();
            })
        },
    );

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
            r#"["id": "{} degree serialized with {} and {} on {}", "size": {}"#,
            degree,
            match compress {
                Compress::Yes => "compression",
                Compress::No => "no compression",
            },
            match validate {
                Validate::Yes => "validation",
                Validate::No => "no validation",
            },
            curve,
            serialized.len(),
        );

        group.bench_function(
            &format!(
                "deserializing with {} and {} {} on {}",
                match compress {
                    Compress::Yes => "compression",
                    Compress::No => "no compression",
                },
                match validate {
                    Validate::Yes => "validation",
                    Validate::No => "no validation",
                },
                degree,
                curve
            ),
            |b| {
                b.iter(|| {
                    Powers::<F, G>::deserialize_with_mode(&serialized[..], compress, validate)
                })
            },
        );
    }

    group.finish();
}

fn setup(c: &mut Criterion) {
    fn aux<F: PrimeField, G: CurveGroup<ScalarField = F>>(
        c: &mut Criterion,
        degree: usize,
        curve: &str,
    ) {
        setup_template::<F, G, DensePolynomial<F>>(c, black_box(degree), curve);
    }

    for n in [1, 2, 4, 8, 16] {
        aux::<ark_bls12_381::Fr, ark_bls12_381::G1Projective>(c, n, "BLS-12-381");
        aux::<ark_bn254::Fr, ark_bn254::G1Projective>(c, n, "BN-254");
        aux::<ark_pallas::Fr, ark_pallas::Projective>(c, n, "PALLAS");
    }
}

fn serde(c: &mut Criterion) {
    fn aux<F: PrimeField, G: CurveGroup<ScalarField = F>>(
        c: &mut Criterion,
        degree: usize,
        curve: &str,
    ) {
        serde_template::<F, G, DensePolynomial<F>>(c, black_box(degree), curve);
    }

    for n in [1, 2, 4, 8, 16] {
        aux::<ark_bls12_381::Fr, ark_bls12_381::G1Projective>(c, n, "BLS-12-381");
        aux::<ark_bn254::Fr, ark_bn254::G1Projective>(c, n, "BN-254");
        aux::<ark_pallas::Fr, ark_pallas::Projective>(c, n, "PALLAS");
    }
}

fn ark_setup(c: &mut Criterion) {
    fn aux<E: Pairing>(c: &mut Criterion, degree: usize, curve: &str) {
        ark_setup_template::<E, DensePolynomial<E::ScalarField>>(c, black_box(degree), curve);
    }

    for n in [1, 2, 4, 8, 16] {
        aux::<ark_bls12_381::Bls12_381>(c, n, "BLS-12-381");
        aux::<ark_bn254::Bn254>(c, n, "BN-254");
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs_f32(0.5))
        .sample_size(10);
    targets = setup, ark_setup, serde
);
criterion_main!(benches);
