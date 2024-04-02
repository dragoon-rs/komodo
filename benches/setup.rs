use std::ops::Div;

use ark_bls12_381::Bls12_381;
use ark_ec::pairing::Pairing;
use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;

use ark_poly_commit::kzg10::Powers;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

type UniPoly12_381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

fn setup_template<E, P>(c: &mut Criterion, nb_bytes: usize)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut group = c.benchmark_group("setup");

    group.bench_function(
        &format!("setup {} on {}", nb_bytes, std::any::type_name::<E>()),
        |b| b.iter(|| komodo::setup::random::<E, P>(nb_bytes).unwrap()),
    );

    let setup = komodo::setup::random::<E, P>(nb_bytes).unwrap();

    group.bench_function(
        &format!(
            "serializing with compression {} on {}",
            nb_bytes,
            std::any::type_name::<E>()
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
            std::any::type_name::<E>()
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
            std::any::type_name::<E>(),
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
                std::any::type_name::<E>()
            ),
            |b| {
                b.iter(|| {
                    Powers::<Bls12_381>::deserialize_with_mode(&serialized[..], compress, validate)
                })
            },
        );
    }

    group.finish();
}

fn setup(c: &mut Criterion) {
    for n in [1, 2, 4, 8, 16] {
        setup_template::<Bls12_381, UniPoly12_381>(c, black_box(n * 1024));
    }
}

criterion_group!(benches, setup);
criterion_main!(benches);
