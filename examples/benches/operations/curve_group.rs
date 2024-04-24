// see `benches/README.md`
use std::time::Instant;

use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use clap::{command, Parser};

fn bench_template<F: PrimeField, G: CurveGroup<ScalarField = F>>(b: &mut plnk::Bencher) {
    plnk::bench(b, "random sampling", |rng| plnk::timeit!((|| G::rand(rng))));

    plnk::bench(b, "addition", |rng| {
        let g1 = G::rand(rng);
        let g2 = G::rand(rng);

        plnk::timeit!((|| g1 + g2))
    });

    plnk::bench(b, "substraction", |rng| {
        let g1 = G::rand(rng);
        let g2 = G::rand(rng);

        plnk::timeit!((|| g1 - g2))
    });

    plnk::bench(b, "double", |rng| {
        let g1 = G::rand(rng);

        plnk::timeit!((|| g1.double()))
    });

    plnk::bench(b, "scalar multiplication", |rng| {
        let g1 = G::rand(rng);
        let f1 = F::rand(rng);

        plnk::timeit!((|| g1.mul(f1)))
    });

    plnk::bench(b, "into affine", |rng| {
        let g1 = G::rand(rng);

        plnk::timeit!((|| g1.into_affine()))
    });

    plnk::bench(b, "from affine", |rng| {
        let g1_affine = G::rand(rng).into_affine();

        plnk::timeit!((|| Into::<G>::into(g1_affine)))
    });

    plnk::bench(b, "affine addition", |rng| {
        let g1_affine = G::rand(rng).into_affine();
        let g2_affine = G::rand(rng).into_affine();

        plnk::timeit!((|| g1_affine + g2_affine))
    });

    plnk::bench(b, "affine scalar multiplication", |rng| {
        let g1_affine = G::rand(rng).into_affine();
        let f1 = F::rand(rng);

        plnk::timeit!((|| g1_affine * f1))
    });
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// the number of measurements to repeat each case, larger values will reduce the variance of
    /// the measurements
    #[arg(short, long, default_value_t = 10)]
    nb_measurements: usize,
}

fn main() {
    let cli = Cli::parse();

    let bencher = plnk::Bencher::new(cli.nb_measurements, ark_std::rand::thread_rng());

    bench_template::<ark_bls12_381::Fr, ark_bls12_381::G1Projective>(
        &mut bencher.with_name("BLS12-381"),
    );
    bench_template::<ark_bn254::Fr, ark_bn254::G1Projective>(&mut bencher.with_name("BN-254"));
    bench_template::<ark_pallas::Fr, ark_pallas::Projective>(&mut bencher.with_name("PALLAS"));
}
