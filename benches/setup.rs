use std::ops::Div;

use ark_bls12_381::{Fr, G1Projective};
use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;

use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use ark_std::test_rng;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use komodo::zk::{self, Powers};

type UniPoly12_381 = DensePolynomial<Fr>;

fn setup_template<F, G, P>(c: &mut Criterion, nb_bytes: usize)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut group = c.benchmark_group("setup");

    let rng = &mut test_rng();

    group.bench_function(
        &format!("setup {} on {}", nb_bytes, std::any::type_name::<F>()),
        |b| b.iter(|| zk::setup::<_, F, G>(nb_bytes, rng).unwrap()),
    );

    let setup = zk::setup::<_, F, G>(nb_bytes, rng).unwrap();

    group.bench_function(
        &format!(
            "serializing with compression {} on {}",
            nb_bytes,
            std::any::type_name::<F>()
        ),
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
        &format!(
            "serializing with no compression {} on {}",
            nb_bytes,
            std::any::type_name::<F>()
        ),
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
            r#"["id": "{} bytes serialized with {} and {} on {}", "size": {}"#,
            nb_bytes,
            match compress {
                Compress::Yes => "compression",
                Compress::No => "no compression",
            },
            match validate {
                Validate::Yes => "validation",
                Validate::No => "no validation",
            },
            std::any::type_name::<F>(),
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
                nb_bytes,
                std::any::type_name::<F>()
            ),
            |b| {
                b.iter(|| {
                    Powers::<Fr, G1Projective>::deserialize_with_mode(
                        &serialized[..],
                        compress,
                        validate,
                    )
                })
            },
        );
    }

    group.finish();
}

fn setup(c: &mut Criterion) {
    for n in [1, 2, 4, 8, 16] {
        setup_template::<Fr, G1Projective, UniPoly12_381>(c, black_box(n * 1024));
    }
}

criterion_group!(benches, setup);
criterion_main!(benches);
