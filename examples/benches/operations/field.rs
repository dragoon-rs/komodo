// see `benches/README.md`
use std::time::{Duration, Instant};

use ark_ff::PrimeField;
use clap::{arg, command, Parser};

fn bench_template<F: PrimeField>(b: &mut plnk::Bencher) {
    plnk::bench(b, "random sampling", |rng| plnk::timeit!((|| F::rand(rng))));

    plnk::bench(b, "addition", |rng| {
        let f1 = F::rand(rng);
        let f2 = F::rand(rng);

        plnk::timeit!((|| f1 + f2))
    });

    plnk::bench(b, "substraction", |rng| {
        let f1 = F::rand(rng);
        let f2 = F::rand(rng);

        plnk::timeit!((|| f1 - f2))
    });

    plnk::bench(b, "double", |rng| {
        let f1 = F::rand(rng);

        plnk::timeit!((|| f1.double()))
    });

    plnk::bench(b, "multiplication", |rng| {
        let f1 = F::rand(rng);
        let f2 = F::rand(rng);

        plnk::timeit!((|| f1 * f2))
    });

    plnk::bench(b, "square", |rng| {
        let f1 = F::rand(rng);

        plnk::timeit!((|| f1.square()))
    });

    plnk::bench(b, "inverse", |rng| {
        let f1 = F::rand(rng);

        plnk::timeit!((|| f1.inverse()))
    });

    plnk::bench(b, "legendre", |rng| {
        let f1 = F::rand(rng);

        plnk::timeit!((|| f1.legendre()))
    });

    plnk::bench(b, "sqrt", |rng| {
        let f1 = F::rand(rng);
        if f1.legendre().is_qr() {
            plnk::timeit!((|| f1.sqrt()))
        } else {
            Duration::default()
        }
    });

    plnk::bench(b, "exponentiation", |rng| {
        let f1 = F::rand(rng);

        plnk::timeit!((|| f1.pow(F::MODULUS)))
    });

    plnk::bench(b, "into bigint", |rng| {
        let f1 = F::rand(rng);

        plnk::timeit!((|| f1.into_bigint()))
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

    bench_template::<ark_bls12_381::Fr>(&mut bencher.with_name("BLS12-381"));
    bench_template::<ark_bn254::Fr>(&mut bencher.with_name("BN-254"));
    bench_template::<ark_pallas::Fr>(&mut bencher.with_name("PALLAS"));
}
