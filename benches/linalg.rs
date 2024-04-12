// see `benches/README.md`
use std::time::Duration;

use ark_ff::PrimeField;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use komodo::linalg::Matrix;

fn inverse_template<F: PrimeField>(c: &mut Criterion, n: usize, curve: &str) {
    let mut rng = rand::thread_rng();
    let matrix = Matrix::<F>::random(n, n, &mut rng);

    c.bench_function(&format!("inverse {}x{} on {}", n, n, curve), |b| {
        b.iter(|| matrix.invert().unwrap())
    });
}

fn inverse(c: &mut Criterion) {
    for n in [10, 15, 20, 30, 40, 60, 80, 120, 160, 240, 320] {
        inverse_template::<ark_bls12_381::Fr>(c, black_box(n), "BLS12-381");
        inverse_template::<ark_bn254::Fr>(c, black_box(n), "BN-254");
        inverse_template::<ark_pallas::Fr>(c, black_box(n), "PALLAS");
    }
}

fn transpose_template<F: PrimeField>(c: &mut Criterion, n: usize, curve: &str) {
    let mut rng = rand::thread_rng();
    let matrix = Matrix::<F>::random(n, n, &mut rng);

    c.bench_function(&format!("transpose {}x{} on {}", n, n, curve), |b| {
        b.iter(|| matrix.transpose())
    });
}

fn transpose(c: &mut Criterion) {
    for n in [10, 15, 20, 30, 40, 60, 80, 120, 160, 240, 320] {
        transpose_template::<ark_bls12_381::Fr>(c, black_box(n), "BLS-12-381");
        transpose_template::<ark_bn254::Fr>(c, black_box(n), "BN-254");
        transpose_template::<ark_pallas::Fr>(c, black_box(n), "PALLAS");
    }
}

fn mul_template<F: PrimeField>(c: &mut Criterion, n: usize, curve: &str) {
    let mut rng = rand::thread_rng();
    let mat_a = Matrix::<F>::random(n, n, &mut rng);
    let mat_b = Matrix::<F>::random(n, n, &mut rng);

    c.bench_function(&format!("mul {}x{} on {}", n, n, curve), |b| {
        b.iter(|| mat_a.mul(&mat_b))
    });
}

fn mul(c: &mut Criterion) {
    for n in [10, 15, 20, 30, 40, 60, 80, 120] {
        mul_template::<ark_bls12_381::Fr>(c, black_box(n), "BLS-12-381");
        mul_template::<ark_bn254::Fr>(c, black_box(n), "BN-254");
        mul_template::<ark_pallas::Fr>(c, black_box(n), "PALLAS");
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs_f32(0.5))
        .sample_size(10);
    targets = inverse, transpose, mul
);
criterion_main!(benches);
