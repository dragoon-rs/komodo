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
    t_build: Option<Duration>,
    t_commit: Option<Duration>,
    t_prove_n: Option<Duration>,
    t_verify_n: Option<Duration>,
    t_decode_k: Option<Duration>,
}

impl From<FRIResult> for Vec<(&'static str, Option<u128>)> {
    fn from(value: FRIResult) -> Self {
        vec![
            ("t_evaluate_kn", value.t_evaluate_kn.map(|v| v.as_nanos())),
            ("t_encode_n", value.t_encode_n.map(|v| v.as_nanos())),
            ("t_build", value.t_build.map(|v| v.as_nanos())),
            ("t_commit", value.t_commit.map(|v| v.as_nanos())),
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
            t_build: None,
            t_commit: None,
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
        t: t_build,
        v: builder,
    } = plnk::timeit(|| {
        dragoonfri::frida::FridaBuilder::<F, H>::new::<N, _>(
            &evaluations,
            dragoonfri::rng::FriChallenger::<H>::default(),
            fri_params.bf,
            fri_params.rpo,
            fri_params.q,
        )
    });

    let plnk::TimeWithValue {
        t: t_commit,
        v: commitment,
    } = plnk::timeit(|| komodo::fri::commit(builder.clone()));

    let plnk::TimeWithValue {
        t: t_prove_n,
        v: proofs,
    } = plnk::timeit(|| {
        komodo::fri::prove::<F, H>(builder.clone(), &(0..fec_params.n).collect::<Vec<_>>())
    });

    let plnk::TimeWithValue {
        t: t_verify_n,
        v: ok,
    } = plnk::timeit(|| {
        let mut ok = true;
        for (shard, proof) in shards.iter().zip(proofs.iter()) {
            if komodo::fri::verify::<N, F, H, P>(
                shard,
                &commitment,
                proof,
                fec_params.n,
                fri_params.q,
            )
            .is_err()
            {
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
    } = plnk::timeit(|| {
        komodo::fri::decode::<F>(
            &shards.clone().into_iter().enumerate().collect::<Vec<_>>(),
            fec_params.n,
        )
    });

    if hex::encode(bytes) != hex::encode(decoded) {
        return FRIResult::empty();
    }

    FRIResult {
        t_evaluate_kn: Some(t_evaluate_kn),
        t_encode_n: Some(t_encode_n),
        t_build: Some(t_build),
        t_commit: Some(t_commit),
        t_prove_n: Some(t_prove_n),
        t_verify_n: Some(t_verify_n),
        t_decode_k: Some(t_decode_k),
    }
}
