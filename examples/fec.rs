use clap::{arg, Parser, ValueEnum};
use dragoonfri_test_utils::Fq;
use komodo::algebra::linalg;
use komodo::fec;
use rand::{Rng, RngCore, SeedableRng};

use ark_ff::PrimeField;
use std::time::Instant;

#[path = "utils/time.rs"]
mod time;

fn random_loss<T>(shards: &mut Vec<T>, k: usize, rng: &mut impl Rng) {
    // Randomly drop some shards until k are left
    while shards.len() > k {
        let i = rng.gen_range(0..shards.len());
        shards.remove(i);
    }
}

#[derive(ValueEnum, Debug, Clone)]
enum FiniteField {
    FP128,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    data_size: usize,

    #[arg(long, default_value = "1234")]
    seed: u64,

    #[arg(short)]
    k: usize,

    #[arg(short)]
    n: usize,

    #[arg(long, default_value = "fp128")]
    finite_field: FiniteField,
}

fn run<F: PrimeField>(bytes: &[u8], k: usize, n: usize, seed: u64) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let matrix = linalg::Matrix::random(k, n, &mut rng);
    let mut shards = timeit_and_print!("encoding", fec::encode, bytes, &matrix).unwrap();
    random_loss(&mut shards, k, &mut rng);
    let recovered = timeit_and_print!("decoding", fec::decode::<F>, &shards).unwrap();
    assert_eq!(bytes, recovered);
}

fn main() {
    let args = Args::parse();

    let mut rng = rand::rngs::StdRng::seed_from_u64(args.seed);

    let mut bytes = vec![0u8; args.data_size];
    rng.fill_bytes(&mut bytes);

    let (k, n) = (args.k, args.n);
    match args.finite_field {
        FiniteField::FP128 => run::<Fq>(&bytes, k, n, args.seed),
    }
}
