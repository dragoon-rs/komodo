use std::time::Duration;

use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::KZG10;
use ark_std::ops::Div;

use komodo::{algebra, algebra::linalg::Matrix, fec, kzg, zk};
use rand::Rng;

use crate::FECParams;

pub(crate) struct KZGResult {
    t_commit_m: Option<Duration>,
    t_prove_n: Option<Duration>,
    t_verify_n: Option<Duration>,
    t_verify_batch_3: Option<Duration>,
}

impl From<KZGResult> for Vec<(&'static str, Option<u128>)> {
    fn from(value: KZGResult) -> Self {
        vec![
            ("t_commit_m", value.t_commit_m.map(|v| v.as_nanos())),
            ("t_prove_n", value.t_prove_n.map(|v| v.as_nanos())),
            ("t_verify_n", value.t_verify_n.map(|v| v.as_nanos())),
            (
                "t_verify_batch_3",
                value.t_verify_batch_3.map(|v| v.as_nanos()),
            ),
        ]
    }
}

impl KZGResult {
    /// Sets all fields to [`None`].
    fn empty() -> Self {
        KZGResult {
            t_commit_m: None,
            t_prove_n: None,
            t_verify_n: None,
            t_verify_batch_3: None,
        }
    }
}

pub(crate) fn bench<E, P>(nb_bytes: usize, fec_params: FECParams, rng: &mut impl Rng) -> KZGResult
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let bytes = crate::random::random_bytes(nb_bytes, rng);

    let degree = fec_params.k - 1;
    let params = KZG10::<E, P>::setup(degree, false, rng)
        .expect("ark_poly_commit::kzg10::KZG10::<E, P>::setup");
    let (powers, verifier_key) = zk::trim(&params, degree);

    let elements = algebra::split_data_into_field_elements::<E::ScalarField>(&bytes, fec_params.k);
    let mut polynomials = Vec::new();
    for chunk in elements.chunks(fec_params.k) {
        polynomials.push(P::from_coefficients_vec(chunk.to_vec()))
    }

    let plnk::TimeWithValue {
        t: t_commit_m,
        v: commitment,
    } = plnk::timeit(|| kzg::commit(&powers, &polynomials).expect("komodo::kzg::commit"));

    let encoding_points = &(0..fec_params.n)
        .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
        .collect::<Vec<_>>();
    let encoding_mat = Matrix::vandermonde_unchecked(encoding_points, fec_params.k);
    let shards = fec::encode::<E::ScalarField>(&bytes, &encoding_mat).expect("komodo::fec::encode");

    let plnk::TimeWithValue {
        t: t_prove_n,
        v: proofs,
    } = plnk::timeit(|| {
        kzg::prove::<E, P>(&polynomials, &shards, encoding_points, &powers)
            .expect("komodo::kzg::prove")
    });

    let plnk::TimeWithValue {
        t: t_verify_n,
        v: ok,
    } = plnk::timeit(|| {
        let mut ok = true;
        for (s, p) in shards.iter().zip(proofs.iter()) {
            let alpha = s.linear_combination[1]; // Vandermonde coefficient
            if !kzg::verify::<E, P>(s, &commitment, p, alpha, &verifier_key) {
                ok = false;
            }
        }
        ok
    });
    if !ok {
        return KZGResult::empty();
    }

    let plnk::TimeWithValue {
        t: t_verify_batch_3,
        v: ok,
    } = plnk::timeit(|| {
        let mut ok = true;
        if !kzg::batch_verify(
            &[
                (shards[0].clone(), proofs[0].clone()),
                (shards[1].clone(), proofs[1].clone()),
                (shards[2].clone(), proofs[2].clone()),
            ],
            &commitment,
            &[
                E::ScalarField::from_le_bytes_mod_order(&[1]),
                E::ScalarField::from_le_bytes_mod_order(&[2]),
                E::ScalarField::from_le_bytes_mod_order(&[3]),
            ],
            &verifier_key,
        )
        .expect("komodo::kzg::batch_verify(1, 2, 3)")
        {
            ok = false;
        }
        ok
    });
    if !ok {
        return KZGResult::empty();
    }

    KZGResult {
        t_commit_m: Some(t_commit_m),
        t_prove_n: Some(t_prove_n),
        t_verify_n: Some(t_verify_n),
        t_verify_batch_3: Some(t_verify_batch_3),
    }
}
