use ark_ec::{pairing::Pairing, CurveGroup};
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::{self, KZG10};
use ark_std::ops::Div;

use komodo::zk;

pub(crate) fn run<F, G, P>(degree: usize, nb_measurements: usize, curve: &str)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut rand::thread_rng();
    let b = plnk::Bencher::new(nb_measurements).with_name(crate::label! { curve: curve });

    eprint!("building trusted setup for degree {degree}... ");
    let setup = zk::setup::<F, G>(degree, rng).unwrap();
    eprintln!("done");

    plnk::bench(&b, crate::label! { degree: degree }, || {
        let polynomial = P::rand(degree, rng);
        plnk::timeit(|| zk::commit(&setup, &polynomial))
    });
}

pub(crate) fn ark_run<E, P>(degree: usize, nb_measurements: usize, curve: &str)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut rand::thread_rng();
    let b = plnk::Bencher::new(nb_measurements).with_name(crate::label! { curve: curve });

    eprint!("building trusted setup for degree {degree}... ");
    let setup = {
        let setup = KZG10::<E, P>::setup(degree, false, rng).unwrap();
        let powers_of_g = setup.powers_of_g[..=degree].to_vec();
        let powers_of_gamma_g = (0..=degree).map(|i| setup.powers_of_gamma_g[&i]).collect();
        kzg10::Powers::<E> {
            powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
            powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
        }
    };
    eprintln!("done");

    plnk::bench(&b, crate::label! { degree: degree }, || {
        let polynomial = P::rand(degree, rng);
        plnk::timeit(|| KZG10::commit(&setup, &polynomial, None, None))
    })
}
