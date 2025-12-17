use std::time::Duration;

use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_serialize::CanonicalSerialize;
use ark_std::ops::Div;
use rand::Rng;
use rs_merkle::Hasher;

use crate::FECParams;

pub(crate) struct FRIParams {
    pub bf: usize,
    pub rpo: usize,
    pub q: usize,
}

pub(crate) struct FRIResult {
    t_evaluate_kn: Option<Duration>,
    t_encode_n: Option<Duration>,
    t_prove_n: Option<Duration>,
    t_verify_n: Option<Duration>,
    t_decode_k: Option<Duration>,
}

impl From<FRIResult> for Vec<(&'static str, Option<u128>)> {
    fn from(value: FRIResult) -> Self {
        vec![
            ("t_evaluate_kn", value.t_evaluate_kn.map(|v| v.as_nanos())),
            ("t_encode_n", value.t_encode_n.map(|v| v.as_nanos())),
            ("t_prove_n", value.t_prove_n.map(|v| v.as_nanos())),
            ("t_verify_n", value.t_verify_n.map(|v| v.as_nanos())),
            ("t_decode_k", value.t_decode_k.map(|v| v.as_nanos())),
        ]
    }
}

impl FRIResult {
    /// Sets all fields to [`None`].
    fn empty() -> Self {
        FRIResult {
            t_evaluate_kn: None,
            t_encode_n: None,
            t_prove_n: None,
            t_verify_n: None,
            t_decode_k: None,
        }
    }
}

pub(crate) fn bench<const N: usize, F: PrimeField, H: Hasher, P>(
    nb_bytes: usize,
    fec_params: FECParams,
    fri_params: FRIParams,
    rng: &mut impl Rng,
) -> FRIResult
where
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    <H as rs_merkle::Hasher>::Hash: AsRef<[u8]> + CanonicalSerialize,
{
    let bytes = crate::random::random_bytes(nb_bytes, rng);

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
        .expect("komodo::fri::prove")
    });

    let plnk::TimeWithValue {
        t: t_verify_n,
        v: ok,
    } = plnk::timeit(|| {
        let mut ok = true;
        for b in &blocks {
            if komodo::fri::verify::<N, F, H, P>(b, fec_params.n, fri_params.q).is_err() {
                ok = false;
            }
        }
        ok
    });
    if !ok {
        return FRIResult::empty();
    }

    let plnk::TimeWithValue {
        t: t_decode_k,
        v: decoded,
    } = plnk::timeit(|| komodo::fri::decode::<F, H>(&blocks[0..fec_params.k], fec_params.n));

    if hex::encode(bytes) != hex::encode(decoded) {
        return FRIResult::empty();
    }

    FRIResult {
        t_evaluate_kn: Some(t_evaluate_kn),
        t_encode_n: Some(t_encode_n),
        t_prove_n: Some(t_prove_n),
        t_verify_n: Some(t_verify_n),
        t_decode_k: Some(t_decode_k),
    }
}
