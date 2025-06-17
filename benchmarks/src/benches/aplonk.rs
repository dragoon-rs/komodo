use ark_ec::{
    pairing::{Pairing, PairingOutput},
    AffineRepr,
};
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use indicatif::ProgressBar;
use rand::thread_rng;
use std::ops::Div;

use komodo::{
    algebra::{self, linalg::Matrix},
    aplonk, fec, zk,
};

use crate::random::random_bytes;

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum Step {
    Setup,
    Encoding,
    Commit,
    Proof,
    Verify,
}

#[allow(clippy::type_complexity)]
fn setup_bench<E, P>(
    nb_bytes: usize,
    k: usize,
    n: usize,
    step: Step,
    maybe_bar: Option<&ProgressBar>,
) -> (
    aplonk::SetupParams<E>,
    Vec<P>,
    Option<Vec<fec::Shard<<E as Pairing>::ScalarField>>>,
    Option<Vec<<E as Pairing>::ScalarField>>,
    Option<(Vec<<E as Pairing>::G1>, PairingOutput<E>)>,
    Option<Vec<aplonk::Block<E>>>,
    Option<ark_poly_commit::kzg10::VerifierKey<E>>,
)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let msg = crate::pretty::bar::dimmed(maybe_bar, None);

    let rng = &mut thread_rng();

    let bytes = random_bytes(nb_bytes, rng);

    let degree = k - 1;

    crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} polynomials")));
    let elements = algebra::split_data_into_field_elements::<E::ScalarField>(&bytes, degree + 1);
    let mut polynomials = Vec::new();
    for chunk in elements.chunks(degree + 1) {
        polynomials.push(P::from_coefficients_vec(chunk.to_vec()))
    }

    crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} setup")));
    let params = aplonk::setup::<E, P>(degree, polynomials.len()).unwrap();

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

    let commit = if step >= Step::Commit {
        crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} commit")));
        Some(aplonk::commit(polynomials.clone(), params.clone()).unwrap())
    } else {
        None
    };

    let blocks = if step >= Step::Proof {
        crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} proof")));
        Some(
            aplonk::prove(
                commit.clone().unwrap(),
                polynomials.clone(),
                shards.clone().unwrap(),
                encoding_points.clone().unwrap(),
                params.clone(),
            )
            .unwrap(),
        )
    } else {
        None
    };

    let vk_psi = if step >= Step::Verify {
        crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} trim")));
        let (_, vk_psi) = zk::trim(params.kzg.clone(), degree);
        Some(vk_psi)
    } else {
        None
    };

    crate::pretty::bar::normal(maybe_bar, msg);

    (
        params,
        polynomials,
        shards,
        encoding_points,
        commit,
        blocks,
        vk_psi,
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
                let (params, polynomials, _, _, _, _, _) =
                    setup_bench::<E, P>(nb_bytes, k, n, Step::Setup, Some(&bar));

                crate::timeit_and_discard_output! {
                    aplonk::commit(polynomials.clone(), params.clone()).unwrap();
                }
            }),
        ),
        (
            "proof".to_string(),
            plnk::closure!(bar, {
                let (params, polynomials, shards, encoding_points, commit, _, _) =
                    setup_bench::<E, P>(nb_bytes, k, n, Step::Commit, Some(&bar));

                crate::timeit_and_discard_output! {
                    aplonk::prove(
                        commit.clone().unwrap(),
                        polynomials.clone(),
                        shards.clone().unwrap(),
                        encoding_points.clone().unwrap(),
                        params.clone(),
                    )
                    .unwrap();
                }
            }),
        ),
        (
            "verify".to_string(),
            plnk::closure!(bar, {
                let (params, _, _, _, _, blocks, vk_psi) =
                    setup_bench::<E, P>(nb_bytes, k, n, Step::Verify, Some(&bar));

                crate::timeit_and_discard_output! {
                    for (i, block) in blocks.clone().unwrap().iter().enumerate() {
                        assert!(aplonk::verify(
                            block,
                            E::ScalarField::from_le_bytes_mod_order(&[i as u8]),
                            &vk_psi.clone().unwrap(),
                            params.ipa.tau_1,
                            params.kzg.powers_of_g[0].into_group(),
                            params.kzg.h.into_group(),
                        )
                        .unwrap());
                    }
                }
            }),
        ),
    ]
}
