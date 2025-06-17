use ark_ec::{pairing::Pairing, CurveGroup};
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::{self, KZG10};
use ark_std::ops::Div;

use indicatif::ProgressBar;
use komodo::zk;
use rand::thread_rng;

pub(crate) fn build<F, G, P>(degree: usize, setup_pb: &ProgressBar) -> plnk::FnTimed<()>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let setup = zk::setup::<F, G>(degree, &mut thread_rng()).unwrap();

    crate::update_progress_bar_with_serializable_items!(setup_pb : setup);

    plnk::closure! {
        let polynomial = P::rand(degree, &mut thread_rng());
        crate::timeit_and_discard_output! { zk::commit(&setup, &polynomial) }
    }
}

pub(crate) fn ark_build<E, P>(degree: usize, setup_pb: &ProgressBar) -> plnk::FnTimed<()>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let setup = {
        let setup = KZG10::<E, P>::setup(degree, false, &mut thread_rng()).unwrap();
        let powers_of_g = setup.powers_of_g[..=degree].to_vec();
        let powers_of_gamma_g = (0..=degree).map(|i| setup.powers_of_gamma_g[&i]).collect();
        kzg10::Powers::<E> {
            powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
            powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
        }
    };
    setup_pb.inc(1);

    plnk::closure! {
        let polynomial = P::rand(degree, &mut thread_rng());
        crate::timeit_and_discard_output! { KZG10::commit(&setup, &polynomial, None, None) }
    }
}
