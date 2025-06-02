use ark_ff::PrimeField;

use komodo::algebra::linalg::Matrix;
use plnk::Bencher;

pub(crate) fn run_inverse<F: PrimeField>(b: &Bencher, n: usize) {
    let mut rng = rand::thread_rng();
    let matrix = Matrix::<F>::random(n, n, &mut rng);

    plnk::bench(b, crate::label! { operation: "inverse" }, || {
        plnk::timeit(|| matrix.invert())
    });
}

pub(crate) fn run_transpose<F: PrimeField>(b: &Bencher, n: usize) {
    let mut rng = rand::thread_rng();
    let matrix = Matrix::<F>::random(n, n, &mut rng);

    plnk::bench(b, crate::label! { operation: "transpose" }, || {
        plnk::timeit(|| matrix.transpose())
    });
}

pub(crate) fn run_multiply<F: PrimeField>(b: &Bencher, n: usize) {
    let mut rng = rand::thread_rng();
    let mat_a = Matrix::<F>::random(n, n, &mut rng);
    let mat_b = Matrix::<F>::random(n, n, &mut rng);

    plnk::bench(b, crate::label! { operation: "multiply" }, || {
        plnk::timeit(|| mat_a.mul(&mat_b))
    });
}
