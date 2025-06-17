use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::{Commitment, KZG10};
use ark_std::ops::Div;

use indicatif::ProgressBar;
use komodo::{
    algebra::{self, linalg::Matrix},
    fec, kzg, zk,
};
use rand::thread_rng;

use crate::random::random_bytes;

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum Step {
    Setup,
    Encoding,
    Commit,
    Proof,
}

#[allow(clippy::type_complexity)]
fn setup_bench<E, P>(
    nb_bytes: usize,
    k: usize,
    n: usize,
    step: Step,
    maybe_bar: Option<&ProgressBar>,
) -> (
    ark_poly_commit::kzg10::Powers<'static, E>,
    ark_poly_commit::kzg10::VerifierKey<E>,
    Vec<P>,
    Option<Vec<fec::Shard<<E as Pairing>::ScalarField>>>,
    Option<Vec<<E as Pairing>::ScalarField>>,
    Option<Vec<Commitment<E>>>,
    Option<Vec<kzg::Block<E>>>,
)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let msg = crate::pretty::bar::dimmed(maybe_bar, None);

    let rng = &mut thread_rng();

    let bytes = random_bytes(nb_bytes, rng);

    crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} setup")));
    let degree = bytes.len() / (E::ScalarField::MODULUS_BIT_SIZE as usize / 8);
    let params = KZG10::<E, P>::setup(degree, false, rng).unwrap();
    let (powers, verifier_key) = zk::trim(params, degree);

    crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} polynomials")));
    let elements = algebra::split_data_into_field_elements::<E::ScalarField>(&bytes, k);
    let mut polynomials = Vec::new();
    for chunk in elements.chunks(k) {
        polynomials.push(P::from_coefficients_vec(chunk.to_vec()))
    }

    let (shards, encoding_points) = if step >= Step::Encoding {
        crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} encoding")));
        let encoding_points = &(0..n)
            .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
            .collect::<Vec<_>>();
        let encoding_mat = Matrix::vandermonde_unchecked(encoding_points, k);
        (
            Some(fec::encode::<E::ScalarField>(&bytes, &encoding_mat).unwrap()),
            Some(encoding_points.clone()),
        )
    } else {
        (None, None)
    };

    let commits = if step >= Step::Commit {
        crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} commitment")));
        let (res, _) = kzg::commit(&powers, &polynomials).unwrap();
        Some(res)
    } else {
        None
    };

    let blocks = if step >= Step::Proof {
        crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} proof")));
        Some(
            kzg::prove(
                commits.clone().unwrap(),
                polynomials.clone(),
                shards.clone().unwrap(),
                encoding_points.clone().unwrap(),
                powers.clone(),
            )
            .unwrap(),
        )
    } else {
        None
    };

    crate::pretty::bar::normal(maybe_bar, msg);

    (
        powers,
        verifier_key,
        polynomials,
        shards,
        encoding_points,
        commits,
        blocks,
    )
}

pub(crate) fn build<E, P>(k: usize, n: usize, nb_bytes: usize) -> Vec<(String, plnk::FnTimed<()>)>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    vec![
        (
            "commit".to_string(),
            plnk::closure!(bar, {
                let (powers, _, polynomials, _, _, _, _) =
                    setup_bench::<E, P>(nb_bytes, k, n, Step::Setup, Some(&bar));

                crate::timeit_and_discard_output! {
                    kzg::commit(&powers, &polynomials).unwrap();
                }
            }),
        ),
        (
            "proof".to_string(),
            plnk::closure!(bar, {
                let (powers, _, polynomials, shards, encoding_points, commits, _) =
                    setup_bench::<E, P>(nb_bytes, k, n, Step::Encoding, Some(&bar));

                crate::timeit_and_discard_output! {
                    kzg::prove(
                        commits.clone().unwrap(),
                        polynomials.clone(),
                        shards.clone().unwrap(),
                        encoding_points.clone().unwrap(),
                        powers.clone(),
                    )
                    .unwrap();
                }
            }),
        ),
        (
            "verify".to_string(),
            plnk::closure!(bar, {
                let (_, verifier_key, _, _, _, _, blocks) =
                    setup_bench::<E, P>(nb_bytes, k, n, Step::Proof, Some(&bar));

                crate::timeit_and_discard_output! {
                    for (i, block) in blocks.clone().unwrap().iter().enumerate() {
                        assert!(kzg::verify(
                            block,
                            E::ScalarField::from_le_bytes_mod_order(&[i as u8]),
                            &verifier_key,
                        ),);
                    }
                }
            }),
        ),
    ]
}
