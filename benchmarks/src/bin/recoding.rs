// see `examples/benches/README.md`
use ark_ff::PrimeField;
use ark_std::rand::Rng;

use clap::{arg, command, Parser, ValueEnum};
use komodo::{
    algebra,
    fec::{recode_with_coeffs, Shard},
};
use plnk::Bencher;

fn to_curve<F: PrimeField>(n: u128) -> F {
    F::from_le_bytes_mod_order(&n.to_le_bytes())
}

fn create_fake_shard<F: PrimeField>(nb_bytes: usize, k: usize) -> Shard<F> {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..(nb_bytes / k)).map(|_| rng.gen::<u8>()).collect();

    let linear_combination: Vec<F> = (0..k).map(|_| to_curve::<F>(rng.gen::<u128>())).collect();

    Shard {
        k: k as u32,
        linear_combination,
        hash: vec![],
        data: algebra::split_data_into_field_elements::<F>(&bytes, 1),
        size: 0,
    }
}

fn bench_template<F: PrimeField>(b: &Bencher, nb_bytes: usize, k: usize, nb_shards: usize) {
    let shards: Vec<Shard<F>> = (0..nb_shards)
        .map(|_| create_fake_shard(nb_bytes, k))
        .collect();

    let mut rng = rand::thread_rng();
    let coeffs: Vec<F> = (0..nb_shards)
        .map(|_| to_curve::<F>(rng.gen::<u128>()))
        .collect();

    plnk::bench(
        b,
        &format!(
            r#"{{"bytes": {}, "shards": {}, "k": {}}}"#,
            nb_bytes, nb_shards, k
        ),
        || plnk::timeit(|| recode_with_coeffs(&shards, &coeffs)),
    );
}

#[derive(ValueEnum, Clone, Hash, PartialEq, Eq)]
enum Curve {
    BLS12381,
    BN254,
    Pallas,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(num_args = 1.., value_delimiter = ' ')]
    bytes: Vec<usize>,

    #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
    shards: Vec<usize>,

    #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
    ks: Vec<usize>,

    #[arg(short, long, num_args=1.., value_delimiter = ' ')]
    curves: Vec<Curve>,

    /// the number of measurements to repeat each case, larger values will reduce the variance of
    /// the measurements
    #[arg(short, long)]
    nb_measurements: usize,
}

fn main() {
    let cli = Cli::parse();

    let bencher = plnk::Bencher::new(cli.nb_measurements);

    for b in cli.bytes {
        for s in &cli.shards {
            for k in &cli.ks {
                for curve in &cli.curves {
                    match curve {
                        Curve::BLS12381 => bench_template::<ark_bls12_381::Fr>(
                            &bencher.with_name("BLS12-381"),
                            b,
                            *k,
                            *s,
                        ),
                        Curve::BN254 => {
                            bench_template::<ark_bn254::Fr>(&bencher.with_name("BN254"), b, *k, *s)
                        }
                        Curve::Pallas => bench_template::<ark_pallas::Fr>(
                            &bencher.with_name("PALLAS"),
                            b,
                            *k,
                            *s,
                        ),
                    }
                }
            }
        }
    }
}
