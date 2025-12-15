use ark_ec::{pairing::Pairing, AffineRepr};
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use rand::thread_rng;
use std::{ops::Div, time::Duration};

use komodo::{
    algebra,
    algebra::linalg::Matrix,
    aplonk::{commit, prove, setup, verify},
    fec::encode,
    zk::trim,
};

use crate::{name_some_pair, FECParams};

pub(crate) fn bench<E, P>(
    nb_bytes: usize,
    fec_params: FECParams,
) -> Vec<(&'static str, Option<Duration>)>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut rng = thread_rng();
    let bytes = crate::random::random_bytes(nb_bytes, &mut rng);

    let elements = algebra::split_data_into_field_elements::<E::ScalarField>(&bytes, fec_params.k);
    let mut polynomials = Vec::new();
    for chunk in elements.chunks(fec_params.k) {
        polynomials.push(P::from_coefficients_vec(chunk.to_vec()))
    }

    if polynomials.len() <= 1 {
        return vec![
            ("t_commit_m", None),
            ("t_prove_n", None),
            ("t_verify_n", None),
        ];
    }

    let degree = fec_params.k - 1;
    let params = setup::<E, P>(degree, polynomials.len()).unwrap();
    let (_, vk_psi) = trim(&params.kzg, degree);

    let plnk::TimeWithValue {
        t: t_commit_m,
        v: commitment,
    } = plnk::timeit(|| commit(&polynomials, &params).unwrap());

    let encoding_points = (0..fec_params.n)
        .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
        .collect::<Vec<_>>();
    let encoding_mat = Matrix::vandermonde_unchecked(&encoding_points, fec_params.k);
    let shards = encode::<E::ScalarField>(&bytes, &encoding_mat).unwrap();

    let plnk::TimeWithValue {
        t: t_prove_n,
        v: blocks,
    } = plnk::timeit(|| {
        prove::<E, P>(
            commitment.clone(),
            &polynomials,
            &shards,
            &encoding_points,
            &params,
        )
        .unwrap()
    });

    let plnk::TimeWithValue { t: t_verify_n, .. } = plnk::timeit(|| {
        for (i, block) in blocks.iter().enumerate() {
            assert!(verify::<E, P>(
                block,
                E::ScalarField::from_le_bytes_mod_order(&[i as u8]),
                &vk_psi,
                params.ipa.tau_1,
                params.kzg.powers_of_g[0].into_group(),
                params.kzg.h.into_group(),
            )
            .unwrap());
        }
    });

    vec![
        name_some_pair!(t_commit_m),
        name_some_pair!(t_prove_n),
        name_some_pair!(t_verify_n),
    ]
}
