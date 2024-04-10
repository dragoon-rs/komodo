use std::time::Instant;

use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_std::ops::Div;

use komodo::zk;

fn run<F, G, P>(degrees: &Vec<usize>, curve: &str)
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    eprintln!("curve: {}", curve);
    let rng = &mut rand::thread_rng();

    let max_degree = *degrees.iter().max().unwrap_or(&0);

    eprint!("building trusted setup for degree {}... ", max_degree);
    let setup = zk::setup::<_, F, G>(max_degree, rng).unwrap();
    eprintln!("done");

    for (i, degree) in degrees.iter().enumerate() {
        eprint!("     d: {} [{}/{}]\r", degree, i + 1, degrees.len());
        let polynomial = P::rand(*degree, rng);

        let start_time = Instant::now();
        let _ = zk::commit(&setup, &polynomial);
        let end_time = Instant::now();

        println!(
            "{}: {} -> {}",
            curve,
            degree,
            end_time.duration_since(start_time).as_nanos()
        );
    }
    eprintln!();
}

/// ## example
/// ```rust
/// measure!(ark_pallas, degrees, G1=Projective, name="PALLAS");
/// ```
/// will produce
/// ```rust
/// run::<ark_pallas::Fr, ark_pallas::Projective, DensePolynomial<ark_pallas::Fr>, ThreadRng>(&degrees, "PALLAS");
/// ```
macro_rules! measure {
    ($c:ident, $d:ident, G1=$g:ident, name=$n:expr) => {
        run::<$c::Fr, $c::$g, DensePolynomial<$c::Fr>>(&$d, $n);
    };
}

fn main() {
    let n = 20;

    let mut degrees = Vec::with_capacity(n);
    let mut cur = 1;
    for _ in 1..n {
        degrees.push(cur);
        cur *= 2;
    }

    measure!(ark_pallas, degrees, G1 = Projective, name = "PALLAS");
    measure!(
        ark_bls12_381,
        degrees,
        G1 = G1Projective,
        name = "BLS12-381"
    );
    measure!(ark_bn254, degrees, G1 = G1Projective, name = "BN-254");
    measure!(ark_secp256k1, degrees, G1 = Projective, name = "SECP256-K1");
    measure!(ark_secp256r1, degrees, G1 = Projective, name = "SECP256-R1");
    measure!(ark_vesta, degrees, G1 = Projective, name = "VESTA");
}
