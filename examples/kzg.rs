use ark_bls12_381::Bls12_381;
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::KZG10;
use ark_std::ops::Div;
use ark_std::test_rng;

use komodo::{algebra, algebra::linalg::Matrix, error::KomodoError, fec::encode, kzg, zk::trim};

fn run<E, P>() -> Result<(), KomodoError>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut test_rng();

    // the code parameters and the data to manipulate
    let (k, n) = (3, 6_usize);
    let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();

    // KZG+ needs a trusted setup to craft the proofs for each shard of encoded data. the bytes are
    // arranged in an $m \times k$ matrix, possibly involving padding, where $k$ is the number of
    // coefficients for each one of the $m$ polynomials
    let degree = bytes.len() / (E::ScalarField::MODULUS_BIT_SIZE as usize / 8);
    let params = KZG10::<E, P>::setup(degree, false, rng).expect("setup failed");
    let (powers, verifier_key) = trim(&params, degree);

    // build the $m$ polynomials from the data
    let elements = algebra::split_data_into_field_elements::<E::ScalarField>(&bytes, k);
    let mut polynomials = Vec::new();
    for chunk in elements.chunks(k) {
        polynomials.push(P::from_coefficients_vec(chunk.to_vec()))
    }

    // commit the polynomials
    let commitment = kzg::commit(&powers, &polynomials).unwrap();

    // encode the data with a Vandermonde encoding
    let encoding_points = &(0..n)
        .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
        .collect::<Vec<_>>();
    let encoding_mat = Matrix::vandermonde_unchecked(encoding_points, k);
    let shards = encode::<E::ScalarField>(&bytes, &encoding_mat)
        .unwrap_or_else(|_| panic!("could not encode"));

    // prove to each shard of encoded data
    let proofs = kzg::prove::<E, P>(&polynomials, &shards, encoding_points, &powers)
        .expect("KZG+ proof failed");

    // verify that all the shards are valid
    for (i, (shard, proof)) in shards.iter().zip(proofs.iter()).enumerate() {
        assert!(
            kzg::verify::<E, P>(
                shard,
                &commitment,
                proof,
                E::ScalarField::from_le_bytes_mod_order(&[i as u8]),
                &verifier_key,
            ),
            "could not verify block {}",
            i
        );
    }

    // verify a batch of shards at once
    assert!(
        kzg::batch_verify(
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
            &verifier_key
        )
        .unwrap(),
        "could not batch-verify blocks 1..3"
    );

    Ok(())
}

fn main() {
    run::<Bls12_381, DensePolynomial<<Bls12_381 as Pairing>::ScalarField>>().unwrap();
}
