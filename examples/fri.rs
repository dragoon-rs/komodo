use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;
use ark_serialize::CanonicalSerialize;
use ark_std::ops::Div;
use clap::{Parser, ValueEnum};
use rs_merkle::Hasher;
use std::time::Instant;

use ark_bls12_381::Fr as F_BLS12_381;
use dragoonfri_test_utils::Fq as F_128;

use dragoonfri::{
    algorithms::{Blake3, Sha3_256, Sha3_512},
    dynamic_folding_factor,
};
use komodo::error::KomodoError;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// measure the time it takes to apply a function on a set of arguments and returns the result of
/// the call
///
/// ```rust
/// fn add(a: i32, b: i32) { a + b }
/// let (res, time) = timeit!(add, 1, 2);
/// ```
/// will be the same as
/// ```rust
/// fn add(a: i32, b: i32) { a + b }
/// let (res, time) = {
///     let start = Instant::now();
///     let res = add(1, 2);
///     let time = start.elapsed();
///     (res, time)
/// };
/// ```
macro_rules! timeit {
    ($func:expr, $( $args:expr ),*) => {{
        let start = Instant::now();
        let res = $func( $( $args ),* );
        let time = start.elapsed();
        (res, time)
    }};
}

/// same as [`timeit`] but prints a name and the time at the end directly
///
/// ```rust
/// fn add(a: i32, b: i32) { a + b }
/// let res = timeit_and_print!("addition", add, 1, 2);
/// ```
/// will be the same as
/// ```rust
/// fn add(a: i32, b: i32) { a + b }
/// let res = {
///     print!("addition: ");
///     let start = Instant::now();
///     let res = add(1, 2);
///     let time = start.elapsed();
///     println!("{}", time.as_nanos());
///     res
/// };
/// ```
macro_rules! timeit_and_print {
    ($name: expr, $func:expr, $( $args:expr ),*) => {{
        print!("{}: ", $name);
        let (res, time) = timeit!($func, $($args),*);
        println!("{}", time.as_nanos());
        res
    }};
}

fn run<const N: usize, F: PrimeField, H: Hasher, P>(
    bytes: &[u8],
    k: usize,
    n: usize,
    bf: usize,
    rpo: usize,
    q: usize,
) -> Result<(), KomodoError>
where
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    <H as rs_merkle::Hasher>::Hash: AsRef<[u8]> + CanonicalSerialize,
{
    let evaluations = timeit_and_print!("evaluating", komodo::fri::evaluate::<F>, &bytes, k, n);

    let evals = evaluations.clone();
    let shards = timeit_and_print!("encoding", komodo::fri::encode::<F>, &bytes, evals, k);

    let blocks = timeit_and_print!(
        "proving",
        komodo::fri::prove::<N, F, H, P>,
        evaluations,
        shards,
        bf,
        rpo,
        q
    );

    let blocks = blocks.unwrap();

    let proofs: usize = blocks.iter().map(|b| b.proof.compressed_size()).sum();
    let commits: usize = blocks.iter().map(|b| b.commit.compressed_size()).sum();
    println!("proofs: {}", proofs);
    println!("commits: {}", commits);

    print!("verifying: ");
    let time: std::time::Duration = blocks
        .iter()
        .cloned()
        .map(|b| {
            let (res, time) = timeit!(komodo::fri::verify::<N, F, H, P>, b, n, q);
            res.unwrap();
            time
        })
        .sum();
    println!("{}", time.as_nanos());

    let decoded = timeit_and_print!(
        "decoding",
        komodo::fri::decode::<F, H>,
        blocks[0..k].to_vec(),
        n
    );

    assert_eq!(hex::encode(bytes), hex::encode(decoded));

    Ok(())
}

#[derive(ValueEnum, Debug, Clone)]
enum Hash {
    BLAKE3,
    SHA3_256,
    SHA3_512,
}

#[derive(ValueEnum, Debug, Clone)]
enum FiniteField {
    FP128,
    BLS12_381,
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
    #[arg(short, long)]
    blowup_factor: usize,

    #[arg(short, long)]
    remainder_degree_plus_one: usize,
    #[arg(short, long)]
    folding_factor: usize,
    #[arg(short, long)]
    nb_queries: usize,

    #[arg(long)]
    hash: Hash,
    #[arg(long)]
    finite_field: FiniteField,
}

macro_rules! foo {
    ($n:ident, $f:ident, $h:ident) => {
        dynamic_folding_factor!(
            let N = $n => run::<N, $f, $h, DensePolynomial<$f>>
        )
    }
}

fn generate_data(size: usize, seed: u64) -> Vec<u8> {
    let mut rnd = StdRng::seed_from_u64(seed);
    (0..size).map(|_| rnd.gen()).collect()
}

fn main() {
    let args = Args::parse();

    let bytes = generate_data(args.data_size, args.seed);
    println!("loaded {} bytes of data", bytes.len());

    let ff = args.folding_factor;
    let f = match args.finite_field {
        FiniteField::FP128 => match args.hash {
            Hash::BLAKE3 => foo!(ff, F_128, Blake3),
            Hash::SHA3_256 => foo!(ff, F_128, Sha3_256),
            Hash::SHA3_512 => foo!(ff, F_128, Sha3_512),
        },
        FiniteField::BLS12_381 => match args.hash {
            Hash::BLAKE3 => foo!(ff, F_BLS12_381, Blake3),
            Hash::SHA3_256 => foo!(ff, F_BLS12_381, Sha3_256),
            Hash::SHA3_512 => foo!(ff, F_BLS12_381, Sha3_512),
        },
    };
    f(
        &bytes,
        args.k,
        args.k * args.blowup_factor,
        args.blowup_factor,
        args.remainder_degree_plus_one,
        args.nb_queries,
    )
    .unwrap()
}
