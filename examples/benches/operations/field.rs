// see `benches/README.md`
use std::time::{Duration, Instant};

use ark_ff::PrimeField;
use clap::{arg, command, Parser};
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

fn bench_template<F: PrimeField>(b: &Bencher) {
    bench(b, "random sampling", |rng| timeit!((|| F::rand(rng))));

    bench(b, "addition", |rng| {
        let f1 = F::rand(rng);
        let f2 = F::rand(rng);

        timeit!((|| f1 + f2))
    });

    bench(b, "substraction", |rng| {
        let f1 = F::rand(rng);
        let f2 = F::rand(rng);

        timeit!((|| f1 - f2))
    });

    bench(b, "double", |rng| {
        let f1 = F::rand(rng);

        timeit!((|| f1.double()))
    });

    bench(b, "multiplication", |rng| {
        let f1 = F::rand(rng);
        let f2 = F::rand(rng);

        timeit!((|| f1 * f2))
    });

    bench(b, "square", |rng| {
        let f1 = F::rand(rng);

        timeit!((|| f1.square()))
    });

    bench(b, "inverse", |rng| {
        let f1 = F::rand(rng);

        timeit!((|| f1.inverse()))
    });

    bench(b, "legendre", |rng| {
        let f1 = F::rand(rng);

        timeit!((|| f1.legendre()))
    });

    bench(b, "sqrt", |rng| {
        let f1 = F::rand(rng);
        if f1.legendre().is_qr() {
            timeit!((|| f1.sqrt()))
        } else {
            Duration::default()
        }
    });

    bench(b, "exponentiation", |rng| {
        let f1 = F::rand(rng);

        timeit!((|| f1.pow(F::MODULUS)))
    });

    bench(b, "into bigint", |rng| {
        let f1 = F::rand(rng);

        timeit!((|| f1.into_bigint()))
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

    bench_template::<ark_bls12_381::Fr>(&bencher.with_name("BLS12-381"));
    bench_template::<ark_bn254::Fr>(&bencher.with_name("BN-254"));
    bench_template::<ark_pallas::Fr>(&bencher.with_name("PALLAS"));
}
