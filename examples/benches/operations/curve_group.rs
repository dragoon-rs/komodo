// see `benches/README.md`
use std::time::{Duration, Instant};

use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use clap::{command, Parser};
use rand::RngCore;

fn bench(b: &Bencher, op: &str, thing: fn(&mut dyn RngCore) -> Duration) {
    let mut rng = ark_std::test_rng();

    let mut times = vec![];
    for i in 0..b.nb_measurements {
        eprint!(
            "{} on {} [{:>5}/{}]\r",
            op,
            b.name,
            i + 1,
            b.nb_measurements
        );

        times.push(thing(&mut rng).as_nanos());
    }
    eprintln!();

    println!(
        r#"{{op: "{}", curve: "{}", times: {:?}}}"#,
        op, b.name, times
    );
}

macro_rules! timeit {
    ($f:tt) => {{
        let start_time = Instant::now();
        #[allow(clippy::redundant_closure_call)]
        let _ = $f();
        Instant::now().duration_since(start_time)
    }};
}

#[derive(Clone)]
struct Bencher {
    nb_measurements: usize,
    name: String,
}

impl Bencher {
    fn new(nb_measurements: usize) -> Self {
        Self {
            nb_measurements,
            name: "".to_string(),
        }
    }

    fn with_name(&self, name: impl ToString) -> Self {
        let mut new = self.clone();
        new.name = name.to_string();
        new
    }
}

fn bench_template<F: PrimeField, G: CurveGroup<ScalarField = F>>(b: &Bencher) {
    bench(b, "random sampling", |rng| timeit!((|| G::rand(rng))));

    bench(b, "addition", |rng| {
        let g1 = G::rand(rng);
        let g2 = G::rand(rng);

        timeit!((|| g1 + g2))
    });

    bench(b, "substraction", |rng| {
        let g1 = G::rand(rng);
        let g2 = G::rand(rng);

        timeit!((|| g1 - g2))
    });

    bench(b, "double", |rng| {
        let g1 = G::rand(rng);

        timeit!((|| g1.double()))
    });

    bench(b, "scalar multiplication", |rng| {
        let g1 = G::rand(rng);
        let f1 = F::rand(rng);

        timeit!((|| g1.mul(f1)))
    });

    bench(b, "into affine", |rng| {
        let g1 = G::rand(rng);

        timeit!((|| g1.into_affine()))
    });

    bench(b, "from affine", |rng| {
        let g1_affine = G::rand(rng).into_affine();

        timeit!((|| Into::<G>::into(g1_affine)))
    });

    bench(b, "affine addition", |rng| {
        let g1_affine = G::rand(rng).into_affine();
        let g2_affine = G::rand(rng).into_affine();

        timeit!((|| g1_affine + g2_affine))
    });

    bench(b, "affine scalar multiplication", |rng| {
        let g1_affine = G::rand(rng).into_affine();
        let f1 = F::rand(rng);

        timeit!((|| g1_affine * f1))
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

    let bencher = Bencher::new(cli.nb_measurements);

    bench_template::<ark_bls12_381::Fr, ark_bls12_381::G1Projective>(
        &bencher.with_name("BLS12-381"),
    );
    bench_template::<ark_bn254::Fr, ark_bn254::G1Projective>(&bencher.with_name("BN-254"));
    bench_template::<ark_pallas::Fr, ark_pallas::Projective>(&bencher.with_name("PALLAS"));
}
