use ark_bls12_381::Bls12_381;
use ark_ec::{pairing::Pairing, AffineRepr};
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use std::ops::Div;

use komodo::{
    algebra,
    algebra::linalg::Matrix,
    aplonk::{commit, prove, setup, verify},
    error::KomodoError,
    fec::encode,
    zk::trim,
};

fn run<E, P>() -> Result<(), KomodoError>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    // the code parameters and the data to manipulate
    let (k, n) = (3, 6_usize);
    // NOTE: the size of the data needs to be a "power of 2" multiple of the finite field element
    // size
    let nb_bytes = k * 2 * (E::ScalarField::MODULUS_BIT_SIZE as usize / 8);
    let bytes = include_bytes!("../assets/dragoon_133x133.png")[0..nb_bytes].to_vec();

    // aPlonK needs a trusted setup to craft the proofs for each shard of encoded data. the bytes
    // are arranged in an $m \times k$ matrix, possibly involving padding, where $k$ is the number
    // of coefficients for each one of the $m$ polynomials
    let degree = k - 1;
    let vector_length_bound =
        bytes.len() / (E::ScalarField::MODULUS_BIT_SIZE as usize / 8) / (degree + 1);
    let params = setup::<E, P>(degree, vector_length_bound).expect("setup failed");
    let (_, vk_psi) = trim(&params.kzg, degree);

    // build the $m$ polynomials from the data
    let elements = algebra::split_data_into_field_elements::<E::ScalarField>(&bytes, k);
    let mut polynomials = Vec::new();
    for chunk in elements.chunks(k) {
        polynomials.push(P::from_coefficients_vec(chunk.to_vec()))
    }

    // commit the polynomials
    let commit = commit(&polynomials, &params).unwrap();

    // encode the data with a Vandermonde encoding
    let encoding_points = (0..n)
        .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
        .collect::<Vec<_>>();
    let encoding_mat = Matrix::vandermonde_unchecked(&encoding_points, k);
    let shards = encode::<E::ScalarField>(&bytes, &encoding_mat)
        .unwrap_or_else(|_| panic!("could not encode"));

    // craft and attach one proof to each shard of encoded data
    let blocks = prove::<E, P>(commit, &polynomials, &shards, &encoding_points, &params).unwrap();

    // verify that all the shards are valid
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

    Ok(())
}

fn main() {
    run::<Bls12_381, DensePolynomial<<Bls12_381 as Pairing>::ScalarField>>().unwrap();
}
