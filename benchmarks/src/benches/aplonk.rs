use ark_ec::{pairing::Pairing, AffineRepr};
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use plnk::Bencher;
use rand::thread_rng;
use std::ops::Div;

use komodo::{
    algebra,
    algebra::linalg::Matrix,
    aplonk::{commit, prove, setup, verify},
    fec::encode,
    zk::trim,
};

use crate::random::random_bytes;

pub(crate) fn run<E, P>(b: &Bencher, k: usize, n: usize, nb_bytes: usize)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut thread_rng();

    let bytes = random_bytes(nb_bytes, rng);

    let degree = k - 1;
    let vector_length_bound =
        bytes.len() / (E::ScalarField::MODULUS_BIT_SIZE as usize / 8) / (degree + 1);
    let params = setup::<E, P>(degree, vector_length_bound).unwrap();
    let (_, vk_psi) = trim(params.kzg.clone(), degree);

    let elements = algebra::split_data_into_field_elements::<E::ScalarField>(&bytes, k);
    let mut polynomials = Vec::new();
    for chunk in elements.chunks(k) {
        polynomials.push(P::from_coefficients_vec(chunk.to_vec()))
    }

    let encoding_points = &(0..n)
        .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
        .collect::<Vec<_>>();
    let encoding_mat = Matrix::vandermonde_unchecked(encoding_points, k);
    let shards = encode::<E::ScalarField>(&bytes, &encoding_mat).unwrap();

    let commit = plnk::bench_and_return(
        b,
        plnk::label! { bytes: nb_bytes, step: "commit", k: k, n: n },
        || plnk::timeit_and_return(|| commit(polynomials.clone(), params.clone()).unwrap()),
    )
    .unwrap();

    let blocks = plnk::bench_and_return(
        b,
        plnk::label! { bytes: nb_bytes, step: "prove", k: k, n: n },
        || {
            plnk::timeit_and_return(|| {
                prove::<E, P>(
                    commit.clone(),
                    polynomials.clone(),
                    shards.clone(),
                    encoding_points.clone(),
                    params.clone(),
                )
                .unwrap()
            })
        },
    )
    .unwrap();

    plnk::bench(
        b,
        plnk::label! { bytes: nb_bytes, step: "verify", k: k, n: n },
        || {
            plnk::timeit(|| {
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
            })
        },
    );
}
