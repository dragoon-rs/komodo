use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_std::ops::Div;

use indicatif::ProgressBar;
use komodo::{algebra::linalg::Matrix, fec, semi_avid, zk};
use rand::thread_rng;

use crate::random::random_bytes;

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum Step {
    Setup,
    Encoding,
    Proof,
    Build,
}

#[allow(clippy::type_complexity)]
fn setup_bench<F, G, P>(
    nb_bytes: usize,
    k: usize,
    n: usize,
    step: Step,
    maybe_bar: Option<&ProgressBar>,
) -> (
    Vec<u8>,
    komodo::zk::Powers<F, G>,
    Matrix<F>,
    Option<Vec<fec::Shard<F>>>,
    Option<Vec<zk::Commitment<F, G>>>,
    Option<Vec<semi_avid::Block<F, G>>>,
)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let msg = crate::pretty::bar::dimmed(maybe_bar, None);

    let mut rng = thread_rng();
    let bytes = random_bytes(nb_bytes, &mut rng);

    crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} setup")));
    let powers = zk::setup::<F, G>(zk::nb_elements_in_setup::<F>(bytes.len()), &mut rng).unwrap();
    let encoding_mat = Matrix::random(k, n, &mut rng);

    let shards = if step >= Step::Encoding {
        crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} encoding")));
        Some(fec::encode(&bytes, &encoding_mat).unwrap())
    } else {
        None
    };

    let proofs = if step >= Step::Proof {
        crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} proof")));
        Some(semi_avid::prove(&bytes, &powers, encoding_mat.height).unwrap())
    } else {
        None
    };

    let blocks = if step >= Step::Build {
        crate::pretty::bar::dimmed(maybe_bar, msg.clone().map(|m| format!("{m} build")));
        Some(semi_avid::build(
            &shards.clone().unwrap(),
            &proofs.clone().unwrap(),
        ))
    } else {
        None
    };

    crate::pretty::bar::normal(maybe_bar, msg);

    (bytes, powers, encoding_mat, shards, proofs, blocks)
}

pub(crate) fn build<F, G, P>(
    k: usize,
    n: usize,
    nb_bytes: usize,
) -> Vec<(String, plnk::FnTimed<()>)>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    vec![
        (
            "commit".to_string(),
            plnk::closure!(bar, {
                let (bytes, powers, encoding_mat, _, _, _) =
                    setup_bench::<F, G, P>(nb_bytes, k, n, Step::Setup, Some(&bar));

                crate::timeit_and_discard_output! {
                    semi_avid::prove(&bytes, &powers, encoding_mat.height);
                }
            }),
        ),
        (
            "proof".to_string(),
            plnk::closure!(bar, {
                let (_, _, _, shards, proofs, _) =
                    setup_bench::<F, G, P>(nb_bytes, k, n, Step::Proof, Some(&bar));

                crate::timeit_and_discard_output! {
                    semi_avid::build(&shards.clone().unwrap(), &proofs.clone().unwrap());
                }
            }),
        ),
        (
            "verify".to_string(),
            plnk::closure!(bar, {
                let (_, powers, _, _, _, blocks) =
                    setup_bench::<F, G, P>(nb_bytes, k, n, Step::Build, Some(&bar));

                crate::timeit_and_discard_output! {
                    for block in &blocks.clone().unwrap() {
                        assert!(semi_avid::verify(block, &powers).unwrap());
                    }
                }
            }),
        ),
    ]
}
