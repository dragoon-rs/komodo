use std::time::Duration;

use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::KZG10;
use ark_std::ops::Div;

use komodo::{algebra, algebra::linalg::Matrix, fec::encode, kzg, zk::trim};
use rand::thread_rng;

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

    let degree = fec_params.k - 1;
    let params = KZG10::<E, P>::setup(degree, false, &mut rng).expect("setup failed");
    let (powers, verifier_key) = trim(&params, degree);

    let elements = algebra::split_data_into_field_elements::<E::ScalarField>(&bytes, fec_params.k);
    let mut polynomials = Vec::new();
    for chunk in elements.chunks(fec_params.k) {
        polynomials.push(P::from_coefficients_vec(chunk.to_vec()))
    }

    let plnk::TimeWithValue {
        t: t_commit_m,
        v: (commitments, _),
    } = plnk::timeit(|| kzg::commit(&powers, &polynomials).unwrap());

    let encoding_points = &(0..fec_params.n)
        .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
        .collect::<Vec<_>>();
    let encoding_mat = Matrix::vandermonde_unchecked(encoding_points, fec_params.k);
    let shards = encode::<E::ScalarField>(&bytes, &encoding_mat).unwrap();

    let plnk::TimeWithValue {
        t: t_prove_n,
        v: blocks,
    } = plnk::timeit(|| {
        kzg::prove::<E, P>(
            &commitments,
            &polynomials,
            &shards,
            encoding_points,
            &powers,
        )
        .unwrap()
    });

    let plnk::TimeWithValue { t: t_verify_n, .. } = plnk::timeit(|| {
        for (i, block) in blocks.iter().enumerate() {
            assert!(kzg::verify::<E, P>(
                block,
                E::ScalarField::from_le_bytes_mod_order(&[i as u8]),
                &verifier_key,
            ),);
        }
    });

    let plnk::TimeWithValue {
        t: t_verify_batch_3,
        ..
    } = plnk::timeit(|| {
        assert!(kzg::batch_verify(
            &blocks[1..3],
            &[
                E::ScalarField::from_le_bytes_mod_order(&[1]),
                E::ScalarField::from_le_bytes_mod_order(&[2]),
                E::ScalarField::from_le_bytes_mod_order(&[3]),
            ],
            &verifier_key
        )
        .unwrap(),)
    });

    vec![
        name_some_pair!(t_commit_m),
        name_some_pair!(t_prove_n),
        name_some_pair!(t_verify_n),
        name_some_pair!(t_verify_batch_3),
    ]
}
