use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::KZG10;
use ark_std::ops::Div;

use komodo::{algebra, algebra::linalg::Matrix, fec::encode, kzg, zk::trim};
use plnk::Bencher;
use rand::thread_rng;

use crate::random::random_bytes;

pub(crate) fn run<E, P>(b: &Bencher, k: usize, n: usize, nb_bytes: usize)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut thread_rng();

    let bytes = random_bytes(nb_bytes, rng);

    let degree = bytes.len() / (E::ScalarField::MODULUS_BIT_SIZE as usize / 8);
    let params = KZG10::<E, P>::setup(degree, false, rng).unwrap();
    let (powers, verifier_key) = trim(params, degree);

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

    let (commits, _) = plnk::bench_and_return(
        b,
        plnk::label! { bytes: nb_bytes, step: "commit", k: k, n: n },
        || plnk::timeit_and_return(|| kzg::commit(&powers, &polynomials).unwrap()),
    )
    .unwrap();

    let blocks = plnk::bench_and_return(
        b,
        plnk::label! { bytes: nb_bytes, step: "prove", k: k, n: n },
        || {
            plnk::timeit_and_return(|| {
                kzg::prove::<E, P>(
                    commits.clone(),
                    polynomials.clone(),
                    shards.clone(),
                    encoding_points.clone(),
                    powers.clone(),
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
                    assert!(kzg::verify::<E, P>(
                        block,
                        E::ScalarField::from_le_bytes_mod_order(&[i as u8]),
                        &verifier_key,
                    ),);
                }
            })
        },
    );
}
