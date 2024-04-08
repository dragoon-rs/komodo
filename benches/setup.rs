use ark_bls12_381::{Bls12_381, Fr, G1Projective};
use ark_ec::{pairing::Pairing, CurveGroup};
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_poly_commit::kzg10::{self, KZG10};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use ark_std::{ops::Div, test_rng};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use komodo::zk::{self, Powers};

fn setup_template<F, G, P>(c: &mut Criterion, nb_bytes: usize)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut group = c.benchmark_group("setup");

    let rng = &mut test_rng();

    let degree = zk::nb_elements_in_setup::<F>(nb_bytes);

    group.bench_function(
        &format!(
            "setup (komodo) {} on {}",
            nb_bytes,
            std::any::type_name::<F>()
        ),
        |b| b.iter(|| zk::setup::<_, F, G>(degree, rng).unwrap()),
    );

    let setup = zk::setup::<_, F, G>(zk::nb_elements_in_setup::<F>(nb_bytes), rng).unwrap();

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

fn ark_setup_template<E, P>(c: &mut Criterion, nb_bytes: usize)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut test_rng();

    let degree = zk::nb_elements_in_setup::<E::ScalarField>(nb_bytes);

    c.bench_function(
        &format!(
            "setup (arkworks) {} bytes on {}",
            nb_bytes,
            std::any::type_name::<E>()
        ),
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

fn setup(c: &mut Criterion) {
    for n in [1, 2, 4, 8, 16] {
        setup_template::<Fr, G1Projective, DensePolynomial<Fr>>(c, black_box(n * 1024));
    }
}

fn ark_setup(c: &mut Criterion) {
    for n in [1, 2, 4, 8, 16] {
        ark_setup_template::<Bls12_381, DensePolynomial<<Bls12_381 as Pairing>::ScalarField>>(
            c,
            black_box(n * 1024),
        );
    }
}

criterion_group!(benches, setup, ark_setup);
criterion_main!(benches);
