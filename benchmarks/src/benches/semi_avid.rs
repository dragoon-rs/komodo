use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_std::ops::Div;

use komodo::{
    algebra::linalg::Matrix,
    fec::encode,
    semi_avid::{build, prove, verify},
    zk::setup,
};
use plnk::Bencher;
use rand::thread_rng;

use crate::random::random_bytes;

pub(crate) fn run<F, G, P>(b: &Bencher, k: usize, n: usize, nb_bytes: usize)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut rng = thread_rng();

    let bytes = random_bytes(nb_bytes, &mut rng);

    let powers = setup::<F, G>(bytes.len(), &mut rng).unwrap();

    let encoding_mat = &Matrix::random(k, n, &mut rng);
    let shards = encode(&bytes, encoding_mat).unwrap();

    let proof = plnk::bench_and_return(
        b,
        plnk::label! { bytes: nb_bytes, step: "commit", k: k, n: n },
        || plnk::timeit_and_return(|| prove(&bytes, &powers, encoding_mat.height).unwrap()),
    )
    .unwrap();

    let blocks = plnk::bench_and_return(
        b,
        plnk::label! { bytes: nb_bytes, step: "proof", k: k, n: n },
        || plnk::timeit_and_return(|| build::<F, G, P>(&shards, &proof)),
    )
    .unwrap();

    plnk::bench(
        b,
        plnk::label! { bytes: nb_bytes, step: "verify", k: k, n: n },
        || {
            plnk::timeit(|| {
                for block in &blocks {
                    assert!(verify(block, &powers).unwrap());
                }
            })
        },
    );
}
