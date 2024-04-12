use std::time::Duration;

use ark_ec::{pairing::Pairing, CurveGroup};
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_poly_commit::kzg10::{Powers, KZG10};
use ark_std::ops::Div;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use komodo::zk;

fn commit_template<F, G, P>(c: &mut Criterion, degree: usize, curve: &str)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut rand::thread_rng();

    let setup = zk::setup::<F, G>(degree, rng).unwrap();
    let polynomial = P::rand(degree, rng);

    c.bench_function(&format!("commit (komodo) {} on {}", degree, curve), |b| {
        b.iter(|| zk::commit(&setup, &polynomial))
    });
}

fn ark_commit_template<E, P>(c: &mut Criterion, degree: usize, curve: &str)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut rand::thread_rng();

    let setup = KZG10::<E, P>::setup(degree, false, rng).unwrap();
    let powers_of_g = setup.powers_of_g[..=degree].to_vec();
    let powers_of_gamma_g = (0..=degree).map(|i| setup.powers_of_gamma_g[&i]).collect();
    let powers = Powers::<E> {
        powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
        powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
    };
    let polynomial = P::rand(degree, rng);

    c.bench_function(&format!("commit (arkworks) {} on {}", degree, curve), |b| {
        b.iter(|| KZG10::commit(&powers, &polynomial, None, None))
    });
}

fn commit(c: &mut Criterion) {
    fn aux<F: PrimeField, G: CurveGroup<ScalarField = F>>(
        c: &mut Criterion,
        degree: usize,
        curve: &str,
    ) {
        commit_template::<F, G, DensePolynomial<F>>(c, black_box(degree), curve);
    }

    for n in [1, 2, 4, 8, 16] {
        aux::<ark_bls12_381::Fr, ark_bls12_381::G1Projective>(c, n, "BLS12-381");
        aux::<ark_bn254::Fr, ark_bn254::G1Projective>(c, n, "BN-254");
        aux::<ark_pallas::Fr, ark_pallas::Projective>(c, n, "PALLAS");
    }
}

fn ark_commit(c: &mut Criterion) {
    fn aux<E: Pairing>(c: &mut Criterion, degree: usize, curve: &str) {
        ark_commit_template::<E, DensePolynomial<E::ScalarField>>(c, black_box(degree), curve);
    }

    for n in [1, 2, 4, 8, 16] {
        aux::<ark_bls12_381::Bls12_381>(c, n, "BLS12-381");
        aux::<ark_bn254::Bn254>(c, n, "BN-254");
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs_f32(0.5))
        .sample_size(10);
    targets = commit, ark_commit
);
criterion_main!(benches);
