use ark_ff::PrimeField;
use komodo::algebra::linalg::Matrix;

pub(crate) fn inverse<F: PrimeField>(n: usize) -> plnk::FnTimed<()> {
    plnk::closure! {
        let matrix = Matrix::<F>::random(n, n, &mut rand::thread_rng());
        crate::timeit_and_discard_output! { matrix.invert().unwrap() }
    }
}

pub(crate) fn transpose<F: PrimeField>(n: usize) -> plnk::FnTimed<()> {
    plnk::closure! {
        let matrix = Matrix::<F>::random(n, n, &mut rand::thread_rng());
        crate::timeit_and_discard_output! { matrix.transpose() }
    }
}

pub(crate) fn multiply<F: PrimeField>(n: usize) -> plnk::FnTimed<()> {
    plnk::closure! {
        let mat_a = Matrix::<F>::random(n, n, &mut rand::thread_rng());
        let mat_b = Matrix::<F>::random(n, n, &mut rand::thread_rng());
        crate::timeit_and_discard_output! { mat_a.mul(&mat_b).unwrap() }
    }
}
