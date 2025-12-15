use ark_poly::univariate::DensePolynomial;
use clap::{arg, Parser, ValueEnum};
use dragoonfri::algorithms::Sha3_512;
use dragoonfri_test_utils::Fq;
use komodo::algebra::linalg;
use komodo::{fec, fri};
use rand::{Rng, RngCore, SeedableRng};

use ark_ff::PrimeField;
use komodo::fec::Shard;
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
enum Coding {
    Matrix,
    Fft,
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

    #[arg(long, default_value = "matrix")]
    coding: Coding,
}

fn encode_fft<F: PrimeField>(bytes: &[u8], k: usize, n: usize) -> Vec<Shard<F>> {
    let evaluations = fri::evaluate::<F>(bytes, k, n);
    fri::encode::<F>(bytes, &evaluations, k)
}

fn run<F: PrimeField>(bytes: &[u8], k: usize, n: usize, seed: u64, coding: Coding) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    match coding {
        Coding::Matrix => {
            let matrix = linalg::Matrix::random(k, n, &mut rng);
            let mut shards = timeit_and_print!("encoding", fec::encode, bytes, &matrix).unwrap();
            random_loss(&mut shards, k, &mut rng);
            let recovered = timeit_and_print!("decoding", fec::decode::<F>, &shards).unwrap();
            assert_eq!(bytes, recovered);
        }
        Coding::Fft => {
            assert_eq!(n.count_ones(), 1, "n must be a power of 2");
            assert_eq!(k.count_ones(), 1, "k must be a power of 2");
            let shards = timeit_and_print!("encoding", encode_fft::<F>, bytes, k, n);

            let evaluations = fri::evaluate::<F>(bytes, k, n);
            let mut blocks =
                fri::prove::<2, F, Sha3_512, DensePolynomial<F>>(&evaluations, &shards, 2, 2, 1)
                    .unwrap();

            random_loss(&mut blocks, k, &mut rng);

            let recovered = timeit_and_print!("decoding", fri::decode::<F, Sha3_512>, &blocks, n);
            assert_eq!(
                bytes, recovered,
                "decoded data does not match original data"
            );
        }
    }
}

fn main() {
    let args = Args::parse();

    let mut rng = rand::rngs::StdRng::seed_from_u64(args.seed);

    let mut bytes = vec![0u8; args.data_size];
    rng.fill_bytes(&mut bytes);

    let (k, n) = (args.k, args.n);
    match args.finite_field {
        FiniteField::FP128 => run::<Fq>(&bytes, k, n, args.seed, args.coding),
    }
}
