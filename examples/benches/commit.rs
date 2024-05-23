// see `examples/benches/README.md`
use ark_ec::{pairing::Pairing, CurveGroup};
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_poly_commit::kzg10::{self, KZG10};
use ark_std::ops::Div;

use clap::{arg, command, Parser, ValueEnum};
use komodo::zk;

fn run<F, G, P>(degrees: &Vec<usize>, curve: &str, nb_measurements: usize)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    eprintln!("curve: {}", curve);
    let rng = &mut rand::thread_rng();
    let b = plnk::Bencher::new(nb_measurements).with_name(curve);

    let max_degree = *degrees.iter().max().unwrap_or(&0);

    eprint!("building trusted setup for degree {}... ", max_degree);
    let setup = zk::setup::<F, G>(max_degree, rng).unwrap();
    eprintln!("done");

    for degree in degrees {
        plnk::bench(&b, &format!("degree {}", degree), || {
            let polynomial = P::rand(*degree, rng);
            plnk::timeit(|| zk::commit(&setup, &polynomial))
        });
    }
}

fn ark_run<E, P>(degrees: &Vec<usize>, curve: &str, nb_measurements: usize)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    eprintln!("curve: {}", curve);
    let rng = &mut rand::thread_rng();
    let b = plnk::Bencher::new(nb_measurements).with_name(curve);

    let max_degree = *degrees.iter().max().unwrap_or(&0);

    eprint!("building trusted setup for degree {}... ", max_degree);
    let setup = {
        let setup = KZG10::<E, P>::setup(max_degree, false, rng).unwrap();
        let powers_of_g = setup.powers_of_g[..=max_degree].to_vec();
        let powers_of_gamma_g = (0..=max_degree)
            .map(|i| setup.powers_of_gamma_g[&i])
            .collect();
        kzg10::Powers::<E> {
            powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
            powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
        }
    };
    eprintln!("done");

    for degree in degrees {
        plnk::bench(&b, &format!("degree {}", degree), || {
            let polynomial = P::rand(*degree, rng);
            plnk::timeit(|| KZG10::commit(&setup, &polynomial, None, None))
        })
    }
}

/// ## example
/// ### non-pairing curves
/// ```rust
/// measure!(ark_pallas, degrees, 10, G1=Projective, name="PALLAS");
/// ```
/// will produce
/// ```rust
/// run::<ark_pallas::Fr, ark_pallas::Projective, DensePolynomial<ark_pallas::Fr>>(&degrees, "PALLAS", 10);
/// ```
///
/// ### pairing-friendly curves
/// ```rust
/// measure!(
///     ark_bls12_381,
///     degrees,
///     10,
///     G1 = G1Projective,
///     E = Bls12_381,
///     name = "BLS12-381"
/// )
/// ```
/// will produce
/// ```rust
/// ark_run::<ark_bls12_381::Bls12_381, DensePolynomial<<ark_bls12_381::Bls12_381 as Pairing>::ScalarField>>(&degrees, "BLS12-381", 10)
/// ```
macro_rules! measure {
    ($c:ident, $d:ident, $m:expr, G1=$g:ident, name=$n:expr) => {
        run::<$c::Fr, $c::$g, DensePolynomial<$c::Fr>>(&$d, $n, $m)
    };
    ($c:ident, $d:ident, $m:expr, G1=$g:ident, E=$e:ident, name=$n:expr) => {
        ark_run::<$c::$e, DensePolynomial<<$c::$e as Pairing>::ScalarField>>(
            &$d,
            concat!($n, "-ark"),
            $m,
        )
    };
}

#[derive(ValueEnum, Clone, Hash, PartialEq, Eq)]
enum Curve {
    BLS12381,
    BN254,
    Pallas,
    ARKBLS12381,
    ARKBN254,
    SECP256K1,
    SECP256R1,
    Vesta,
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
    let degrees = cli.degrees;

    for curve in cli.curves {
        match curve {
            Curve::Pallas => {
                measure!(
                    ark_pallas,
                    degrees,
                    cli.nb_measurements,
                    G1 = Projective,
                    name = "PALLAS"
                )
            }
            Curve::ARKBLS12381 => {
                measure!(
                    ark_bls12_381,
                    degrees,
                    cli.nb_measurements,
                    G1 = G1Projective,
                    E = Bls12_381,
                    name = "BLS12-381"
                )
            }
            Curve::ARKBN254 => {
                measure!(
                    ark_bn254,
                    degrees,
                    cli.nb_measurements,
                    G1 = G1Projective,
                    E = Bn254,
                    name = "BN254"
                )
            }
            Curve::BLS12381 => {
                measure!(
                    ark_bls12_381,
                    degrees,
                    cli.nb_measurements,
                    G1 = G1Projective,
                    name = "BLS12-381"
                )
            }
            Curve::BN254 => {
                measure!(
                    ark_bn254,
                    degrees,
                    cli.nb_measurements,
                    G1 = G1Projective,
                    name = "BN254"
                )
            }
            Curve::SECP256K1 => {
                measure!(
                    ark_secp256k1,
                    degrees,
                    cli.nb_measurements,
                    G1 = Projective,
                    name = "SECP256-K1"
                )
            }
            Curve::SECP256R1 => {
                measure!(
                    ark_secp256r1,
                    degrees,
                    cli.nb_measurements,
                    G1 = Projective,
                    name = "SECP256-R1"
                )
            }
            Curve::Vesta => {
                measure!(
                    ark_vesta,
                    degrees,
                    cli.nb_measurements,
                    G1 = Projective,
                    name = "VESTA"
                )
            }
        }
    }
}
