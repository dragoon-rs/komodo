use ark_bls12_381::{Bls12_381, Fr, G1Projective};
use ark_ec::{pairing::Pairing, CurveGroup};
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_std::ops::Div;

use ark_poly_commit::kzg10::{Powers, KZG10};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use komodo::zk;

fn commit_template<F, G, P>(c: &mut Criterion, degree: usize)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut rand::thread_rng();

    let setup = zk::setup::<_, F, G>(degree, rng).unwrap();
    let polynomial = P::rand(degree, rng);

    c.bench_function(
        &format!(
            "commit (komodo) {} on {}",
            degree,
            std::any::type_name::<F>()
        ),
        |b| b.iter(|| zk::commit(&setup, &polynomial)),
    );
}

fn ark_commit_template<E, P>(c: &mut Criterion, degree: usize)
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

    c.bench_function(
        &format!(
            "commit (arkworks) {} on {}",
            degree,
            std::any::type_name::<E>()
        ),
        |b| b.iter(|| KZG10::commit(&powers, &polynomial, None, None)),
    );
}

fn commit(c: &mut Criterion) {
    for n in [1, 2, 4, 8, 16] {
        commit_template::<Fr, G1Projective, DensePolynomial<Fr>>(c, black_box(n * 1024));
    }
}

fn ark_commit(c: &mut Criterion) {
    for n in [1, 2, 4, 8, 16] {
        ark_commit_template::<Bls12_381, DensePolynomial<<Bls12_381 as Pairing>::ScalarField>>(
            c,
            black_box(n * 1024),
        );
    }
}

criterion_group!(benches, commit, ark_commit);
criterion_main!(benches);
