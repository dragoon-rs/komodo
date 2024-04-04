use ark_bls12_381::Fr;
use ark_ff::PrimeField;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use komodo::linalg::Matrix;

fn inverse_template<F: PrimeField>(c: &mut Criterion, n: usize) {
    let matrix = Matrix::<F>::random(n, n);

    c.bench_function(
        &format!("inverse {}x{} on {}", n, n, std::any::type_name::<F>()),
        |b| b.iter(|| matrix.invert().unwrap()),
    );
}

fn inverse(c: &mut Criterion) {
    for n in [10, 15, 20, 30, 40, 60, 80, 120, 160, 240, 320] {
        inverse_template::<Fr>(c, black_box(n));
    }
}

fn transpose_template<F: PrimeField>(c: &mut Criterion, n: usize) {
    let matrix = Matrix::<F>::random(n, n);

    c.bench_function(
        &format!("transpose {}x{} on {}", n, n, std::any::type_name::<F>()),
        |b| b.iter(|| matrix.transpose()),
    );
}

fn transpose(c: &mut Criterion) {
    for n in [10, 15, 20, 30, 40, 60, 80, 120, 160, 240, 320] {
        transpose_template::<Fr>(c, black_box(n));
    }
}

fn mul_template<F: PrimeField>(c: &mut Criterion, n: usize) {
    let mat_a = Matrix::<F>::random(n, n);
    let mat_b = Matrix::<F>::random(n, n);

    c.bench_function(
        &format!("mul {}x{} on {}", n, n, std::any::type_name::<F>()),
        |b| b.iter(|| mat_a.mul(&mat_b)),
    );
}

fn mul(c: &mut Criterion) {
    for n in [10, 15, 20, 30, 40, 60, 80, 120, 160, 240, 320] {
        mul_template::<Fr>(c, black_box(n));
    }
}

criterion_group!(benches, inverse, transpose, mul);
criterion_main!(benches);
