use ark_ec::{pairing::Pairing, CurveGroup};
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_poly_commit::kzg10::{self, KZG10};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use ark_std::ops::Div;

use komodo::zk::{self, Powers};
use plnk::Bencher;

pub(crate) fn run<F, G, P>(degree: usize, nb_measurements: usize, curve: &str)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let b = plnk::Bencher::new(nb_measurements).with_name(plnk::label! { curve: curve });
    let rng = &mut rand::thread_rng();

    plnk::bench(&b, plnk::label! { degree: degree }, || {
        plnk::timeit(|| zk::setup::<F, G>(degree, rng))
    });
}

pub(crate) fn ark_run<E, P>(degree: usize, nb_measurements: usize, curve: &str)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let b = plnk::Bencher::new(nb_measurements)
        .with_name(plnk::label! { curve: format!("{curve}-ark") });
    let rng = &mut rand::thread_rng();

    plnk::bench(&b, plnk::label! { degree: degree }, || {
        plnk::timeit(|| {
            let setup = KZG10::<E, P>::setup(degree, false, rng).unwrap();
            let powers_of_g = setup.powers_of_g[..=degree].to_vec();
            let powers_of_gamma_g = (0..=degree).map(|i| setup.powers_of_gamma_g[&i]).collect();
            kzg10::Powers::<E> {
                powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
                powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
            }
        })
    });
}

#[allow(dead_code)]
fn serde_template<F, G, P>(b: &Bencher, degree: usize)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut rand::thread_rng();

    let setup = zk::setup::<F, G>(degree, rng).unwrap();

    plnk::bench(
        b,
        &format!("serializing with compression {}", degree),
        || {
            plnk::timeit(|| {
                let mut serialized = vec![0; setup.serialized_size(Compress::Yes)];
                setup
                    .serialize_with_mode(&mut serialized[..], Compress::Yes)
                    .unwrap();
            })
        },
    );

    plnk::bench(
        b,
        &format!("serializing with no compression {}", degree),
        || {
            plnk::timeit(|| {
                let mut serialized = vec![0; setup.serialized_size(Compress::No)];
                setup
                    .serialize_with_mode(&mut serialized[..], Compress::No)
                    .unwrap();
            })
        },
    );

    for (compress, validate) in [
        (Compress::Yes, Validate::Yes),
        (Compress::Yes, Validate::No),
        (Compress::No, Validate::Yes),
        (Compress::No, Validate::No),
    ] {
        let mut serialized = vec![0; setup.serialized_size(compress)];
        setup
            .serialize_with_mode(&mut serialized[..], compress)
            .unwrap();

        plnk::bench(
            b,
            &format!(
                "deserializing with {} and {} {}",
                match compress {
                    Compress::Yes => "compression",
                    Compress::No => "no compression",
                },
                match validate {
                    Validate::Yes => "validation",
                    Validate::No => "no validation",
                },
                degree,
            ),
            || {
                plnk::timeit(|| {
                    Powers::<F, G>::deserialize_with_mode(&serialized[..], compress, validate)
                })
            },
        );
    }
}

#[allow(dead_code)]
fn serde(degrees: &[usize], nb_measurements: usize) {
    fn aux<F: PrimeField, G: CurveGroup<ScalarField = F>>(
        degree: usize,
        curve: &str,
        nb_measurements: usize,
    ) {
        let b =
            plnk::Bencher::new(nb_measurements).with_name(format!("serialization on {}", curve));
        serde_template::<F, G, DensePolynomial<F>>(&b, degree);
    }

    for d in degrees {
        aux::<ark_bls12_381::Fr, ark_bls12_381::G1Projective>(*d, "BLS12-381", nb_measurements);
        aux::<ark_bn254::Fr, ark_bn254::G1Projective>(*d, "BN254", nb_measurements);
        aux::<ark_pallas::Fr, ark_pallas::Projective>(*d, "PALLAS", nb_measurements);
        aux::<ark_vesta::Fr, ark_vesta::Projective>(*d, "VESTA", nb_measurements);
    }
}
