use std::time::Duration;

use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_std::ops::Div;

use rand::thread_rng;

use komodo::{algebra::linalg::Matrix, fec, semi_avid, zk};

use crate::{name_some_pair, FECParams};

pub(crate) fn bench<F, G, P>(
    nb_bytes: usize,
    fec_params: FECParams,
) -> Vec<(&'static str, Option<Duration>)>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut rng = thread_rng();
    let bytes = crate::random::random_bytes(nb_bytes, &mut rng);

    let powers = zk::setup::<F, G>(zk::nb_elements_in_setup::<F>(bytes.len()), &mut rng).unwrap();
    let encoding_mat = Matrix::random(fec_params.k, fec_params.n, &mut rng);
    let shards = fec::encode(&bytes, &encoding_mat).unwrap();

    let plnk::TimeWithValue {
        t: t_prove_k,
        v: proofs,
    } = plnk::timeit(|| semi_avid::prove(&bytes, &powers, encoding_mat.height).unwrap());

    let plnk::TimeWithValue {
        t: t_build_n,
        v: blocks,
    } = plnk::timeit(|| semi_avid::build(&shards, &proofs));

    let plnk::TimeWithValue { t: t_verify_n, .. } = plnk::timeit(|| {
        for block in &blocks {
            assert!(semi_avid::verify(block, &powers).unwrap());
        }
    });

    vec![
        name_some_pair!(t_prove_k),
        name_some_pair!(t_build_n),
        name_some_pair!(t_verify_n),
    ]
}
