use std::time::Duration;

use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_serialize::CanonicalSerialize;
use ark_std::ops::Div;
use rand::thread_rng;
use rs_merkle::Hasher;

use crate::{name_some_pair, FECParams};

pub(crate) struct FRIParams {
    pub bf: usize,
    pub rpo: usize,
    pub q: usize,
}

pub(crate) fn bench<const N: usize, F: PrimeField, H: Hasher, P>(
    nb_bytes: usize,
    fec_params: FECParams,
    fri_params: FRIParams,
) -> Vec<(&'static str, Option<Duration>)>
where
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    <H as rs_merkle::Hasher>::Hash: AsRef<[u8]> + CanonicalSerialize,
{
    let mut rng = thread_rng();
    let bytes = crate::random::random_bytes(nb_bytes, &mut rng);

    let plnk::TimeWithValue {
        t: t_evaluate_kn,
        v: evaluations,
    } = plnk::timeit(|| komodo::fri::evaluate::<F>(&bytes, fec_params.k, fec_params.n));

    let plnk::TimeWithValue {
        t: t_encode_n,
        v: shards,
    } = plnk::timeit(|| komodo::fri::encode::<F>(&bytes, &evaluations, fec_params.k));

    let plnk::TimeWithValue {
        t: t_prove_n,
        v: blocks,
    } = plnk::timeit(|| {
        komodo::fri::prove::<N, F, H, P>(
            &evaluations,
            &shards,
            fri_params.bf,
            fri_params.rpo,
            fri_params.q,
        )
        .unwrap()
    });

    let plnk::TimeWithValue { t: t_verify_n, .. } = plnk::timeit(|| {
        for b in &blocks {
            komodo::fri::verify::<N, F, H, P>(b, fec_params.n, fri_params.q).unwrap();
        }
    });

    let plnk::TimeWithValue {
        t: t_decode_k,
        v: decoded,
    } = plnk::timeit(|| komodo::fri::decode::<F, H>(&blocks[0..fec_params.k], fec_params.n));

    assert_eq!(hex::encode(bytes), hex::encode(decoded));

    vec![
        name_some_pair!(t_evaluate_kn),
        name_some_pair!(t_encode_n),
        name_some_pair!(t_prove_n),
        name_some_pair!(t_verify_n),
        name_some_pair!(t_decode_k),
    ]
}
