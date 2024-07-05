// see `examples/benches/README.md`
use ark_ec::{pairing::Pairing, CurveGroup};
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_poly_commit::kzg10::{self, KZG10};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use ark_std::ops::Div;

use clap::{command, Parser, ValueEnum};
use komodo::zk::{self, Powers};
use plnk::Bencher;

fn setup_template<F, G, P>(b: &Bencher, degree: usize)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut rand::thread_rng();

    plnk::bench(b, &format!("degree {}", degree), || {
        plnk::timeit(|| zk::setup::<F, G>(degree, rng))
    });
}

fn ark_setup_template<E, P>(b: &Bencher, degree: usize)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let rng = &mut rand::thread_rng();

    plnk::bench(b, &format!("degree {}", degree), || {
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

fn setup<F: PrimeField, G: CurveGroup<ScalarField = F>>(
    degrees: &[usize],
    nb_measurements: usize,
    curve: &str,
) {
    for d in degrees {
        let b = plnk::Bencher::new(nb_measurements).with_name(format!("setup on {}", curve));
        setup_template::<F, G, DensePolynomial<F>>(&b, *d);
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

fn ark_setup<E: Pairing>(degrees: &[usize], nb_measurements: usize, curve: &str) {
    for d in degrees {
        let b = plnk::Bencher::new(nb_measurements).with_name(format!("ARK setup on {}", curve));
        ark_setup_template::<E, DensePolynomial<E::ScalarField>>(&b, *d);
    }
}

#[derive(ValueEnum, Clone, Hash, PartialEq, Eq)]
enum Curve {
    BLS12381,
    BN254,
    Pallas,
    EDOnMnt4298,
    CP6782,
    MNT4753,
    ARKBLS12381,
    ARKBN254,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// the polynomial degrees to measure the commit time on
    #[arg(num_args = 1.., value_delimiter = ' ')]
    degrees: Vec<usize>,

    /// the number of measurements to repeat each case, larger values will reduce the variance of
    /// the measurements
    #[arg(short, long)]
    nb_measurements: usize,

    #[arg(short, long, num_args=1.., value_delimiter = ',')]
    curves: Vec<Curve>,
}

fn main() {
    let cli = Cli::parse();

    for curve in cli.curves {
        match curve {
            Curve::ARKBN254 => {
                ark_setup::<ark_bn254::Bn254>(&cli.degrees, cli.nb_measurements, "BN254");
            }
            Curve::ARKBLS12381 => {
                ark_setup::<ark_bls12_381::Bls12_381>(
                    &cli.degrees,
                    cli.nb_measurements,
                    "BLS12-381",
                );
            }
            Curve::Pallas => {
                setup::<ark_pallas::Fr, ark_pallas::Projective>(
                    &cli.degrees,
                    cli.nb_measurements,
                    "PALLAS",
                );
            }
            Curve::BN254 => {
                setup::<ark_bn254::Fr, ark_bn254::G1Projective>(
                    &cli.degrees,
                    cli.nb_measurements,
                    "BN254",
                );
            }
            Curve::BLS12381 => {
                setup::<ark_bls12_381::Fr, ark_bls12_381::G1Projective>(
                    &cli.degrees,
                    cli.nb_measurements,
                    "BLS12-381",
                );
            }
            Curve::EDOnMnt4298 => {
                setup::<ark_ed_on_mnt4_298::Fr, ark_ed_on_mnt4_298::EdwardsProjective>(
                    &cli.degrees,
                    cli.nb_measurements,
                    "ED-MNT4-298",
                );
            }
            Curve::CP6782 => {
                setup::<ark_cp6_782::Fr, ark_cp6_782::G1Projective>(
                    &cli.degrees,
                    cli.nb_measurements,
                    "CP6-782",
                );
            }
            Curve::MNT4753 => {
                setup::<ark_mnt4_753::Fr, ark_mnt4_753::G1Projective>(
                    &cli.degrees,
                    cli.nb_measurements,
                    "MNT4-753",
                );
            }
        }
    }

    // NOTE: this is disabled for now because it takes so much time...
    // serde(&cli.degrees, cli.nb_measurements);
}
