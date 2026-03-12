use std::time::Duration;

use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_std::ops::Div;

use rand::Rng;

use komodo::{algebra::linalg::Matrix, fec, semi_avid, zk};

use crate::FECParams;

pub(crate) struct SemiAVIDResult {
    t_commit_k: Option<Duration>,
    t_verify_n: Option<Duration>,
}

impl From<SemiAVIDResult> for Vec<(&'static str, Option<u128>)> {
    fn from(value: SemiAVIDResult) -> Self {
        vec![
            ("t_commit_k", value.t_commit_k.map(|v| v.as_nanos())),
            ("t_verify_n", value.t_verify_n.map(|v| v.as_nanos())),
        ]
    }
}

impl SemiAVIDResult {
    /// Sets all fields to [`None`].
    fn empty() -> Self {
        SemiAVIDResult {
            t_commit_k: None,
            t_verify_n: None,
        }
    }
}

pub(crate) fn bench<F, G, P>(
    nb_bytes: usize,
    fec_params: FECParams,
    rng: &mut impl Rng,
) -> SemiAVIDResult
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let bytes = crate::random::random_bytes(nb_bytes, rng);

    let powers = zk::setup::<F, G>(zk::nb_elements_in_setup::<F>(bytes.len()), rng)
        .expect("komodo::zk::setup");
    let encoding_mat = Matrix::random(fec_params.k, fec_params.n, rng);
    let shards = fec::encode(&bytes, &encoding_mat).expect("komodo::fec::encode");

    let plnk::TimeWithValue {
        t: t_commit_k,
        v: commitment,
    } = plnk::timeit(|| {
        semi_avid::commit(&bytes, &powers, encoding_mat.height).expect("komodo::semi_avid::prove")
    });

    let plnk::TimeWithValue {
        t: t_verify_n,
        v: ok,
    } = plnk::timeit(|| {
        let mut ok = true;
        for (i, shard) in shards.iter().enumerate() {
            if !semi_avid::verify(shard, &commitment, &powers)
                .unwrap_or_else(|_| panic!("komodo::semi_avid::verify({})", i))
            {
                ok = false;
            }
        }
        ok
    });
    if !ok {
        return SemiAVIDResult::empty();
    }

    SemiAVIDResult {
        t_commit_k: Some(t_commit_k),
        t_verify_n: Some(t_verify_n),
    }
}
