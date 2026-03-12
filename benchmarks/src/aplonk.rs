use ark_ec::{pairing::Pairing, AffineRepr};
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use rand::Rng;
use std::{ops::Div, time::Duration};

use komodo::{
    algebra::{self, linalg::Matrix},
    aplonk, fec, zk,
};

use crate::FECParams;

pub(crate) struct AplonkResult {
    t_commit_m: Option<Duration>,
    t_prove_n: Option<Duration>,
    t_verify_n: Option<Duration>,
}

impl From<AplonkResult> for Vec<(&'static str, Option<u128>)> {
    fn from(value: AplonkResult) -> Self {
        vec![
            ("t_commit_m", value.t_commit_m.map(|v| v.as_nanos())),
            ("t_prove_n", value.t_prove_n.map(|v| v.as_nanos())),
            ("t_verify_n", value.t_verify_n.map(|v| v.as_nanos())),
        ]
    }
}

impl AplonkResult {
    /// Sets all fields to [`None`].
    fn empty() -> Self {
        AplonkResult {
            t_commit_m: None,
            t_prove_n: None,
            t_verify_n: None,
        }
    }
}

pub(crate) fn bench<E, P>(
    nb_bytes: usize,
    fec_params: FECParams,
    rng: &mut impl Rng,
) -> AplonkResult
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let bytes = crate::random::random_bytes(nb_bytes, rng);

    let elements = algebra::split_data_into_field_elements::<E::ScalarField>(&bytes, fec_params.k);
    let mut polynomials = Vec::new();
    for chunk in elements.chunks(fec_params.k) {
        polynomials.push(P::from_coefficients_vec(chunk.to_vec()))
    }

    if polynomials.len() <= 1 {
        return AplonkResult::empty();
    }

    let degree = fec_params.k - 1;
    let params = aplonk::setup::<E, P>(degree, polynomials.len()).expect("komodo::aplonk::setup");
    let (_, vk_psi) = zk::trim(&params.kzg.clone(), degree);

    let vk = aplonk::VerifierKey {
        vk_psi,
        tau_1: params.ipa.tau_1,
        g_1: params.kzg.powers_of_g[0].into_group(),
        g_2: params.kzg.h.into_group(),
    };

    let plnk::TimeWithValue {
        t: t_commit_m,
        v: commitment,
    } = plnk::timeit(|| aplonk::commit(&polynomials, &params).expect("komodo::aplonk::commit"));

    let encoding_points = (0..fec_params.n)
        .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
        .collect::<Vec<_>>();
    let encoding_mat = Matrix::vandermonde_unchecked(&encoding_points, fec_params.k);
    let shards = fec::encode::<E::ScalarField>(&bytes, &encoding_mat).expect("komodo::fec::encode");

    let plnk::TimeWithValue {
        t: t_prove_n,
        v: proofs,
    } = plnk::timeit(|| {
        aplonk::prove::<E, P>(&commitment, &polynomials, &encoding_points, &params)
            .expect("komodo::aplonk::prove")
    });

    let plnk::TimeWithValue {
        t: t_verify_n,
        v: ok,
    } = plnk::timeit(|| {
        let mut ok = true;
        for (shard, proof) in shards.iter().zip(proofs.iter()) {
            let alpha = shard.linear_combination[1]; // Vandermonde coefficient
            if !aplonk::verify::<E, P>(shard, &commitment, proof, alpha, &vk)
                .unwrap_or_else(|_| panic!("komodo::aplonk::verify({})", alpha))
            {
                ok = false;
            }
        }
        ok
    });
    if !ok {
        return AplonkResult::empty();
    }

    AplonkResult {
        t_commit_m: Some(t_commit_m),
        t_prove_n: Some(t_prove_n),
        t_verify_n: Some(t_verify_n),
    }
}
