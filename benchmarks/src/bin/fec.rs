// see `examples/benches/README.md`
use ark_ff::PrimeField;

use clap::{arg, command, Parser, ValueEnum};
use komodo::{fec, linalg::Matrix};
use plnk::Bencher;
use rand::{rngs::ThreadRng, thread_rng, Rng, RngCore};

fn random_bytes(n: usize, rng: &mut ThreadRng) -> Vec<u8> {
    (0..n).map(|_| rng.gen::<u8>()).collect()
}

fn build_encoding_mat<F: PrimeField>(
    k: usize,
    n: usize,
    encoding: &Encoding,
    rng: &mut impl RngCore,
) -> Matrix<F> {
    match encoding {
        Encoding::Random => Matrix::random(k, n, rng),
        Encoding::Vandermonde => {
            let points: Vec<F> = (0..n)
                .map(|i| F::from_le_bytes_mod_order(&i.to_le_bytes()))
                .collect();
            Matrix::vandermonde_unchecked(&points, k)
        }
    }
}

fn template<F: PrimeField>(b: &Bencher, nb_bytes: usize, k: usize, n: usize, encoding: &Encoding) {
    let mut rng = thread_rng();

    let encoding_mat = build_encoding_mat(k, n, encoding, &mut rng);

    plnk::bench(
        b,
        &format!(
            r#"{{"bytes": {}, "step": "encode", "k": {}, "n": {}}}"#,
            nb_bytes, k, n
        ),
        || {
            let bytes = random_bytes(nb_bytes, &mut rng);

            plnk::timeit(|| fec::encode::<F>(&bytes, &encoding_mat).unwrap())
        },
    );

    let encoding_mat = build_encoding_mat(k, k, encoding, &mut rng);

    plnk::bench(
        b,
        &format!(
            r#"{{"bytes": {}, "step": "decode", "k": {}, "n": {}}}"#,
            nb_bytes, k, n
        ),
        || {
            let bytes = random_bytes(nb_bytes, &mut rng);
            let shards = fec::encode::<F>(&bytes, &encoding_mat).unwrap();

            plnk::timeit(|| fec::decode::<F>(shards.clone()).unwrap())
        },
    );
}

#[derive(ValueEnum, Clone)]
enum Encoding {
    Vandermonde,
    Random,
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
    /// the sizes of the data to consider
    #[arg(num_args = 1.., value_delimiter = ' ')]
    sizes: Vec<usize>,

    #[arg(short, long)]
    encoding: Encoding,

    #[arg(short, long, num_args=1.., value_delimiter = ' ')]
    curves: Vec<Curve>,

    #[arg(short)]
    k: usize,
    #[arg(short)]
    n: usize,

    /// the number of measurements to repeat each case, larger values will reduce the variance of
    /// the measurements
    #[arg(long)]
    nb_measurements: usize,
}

fn main() {
    let cli = Cli::parse();

    let b = plnk::Bencher::new(cli.nb_measurements);

    for n in cli.sizes {
        for curve in &cli.curves {
            match curve {
                Curve::BLS12381 => template::<ark_bls12_381::Fr>(
                    &b.with_name("BLS12-381"),
                    n,
                    cli.k,
                    cli.n,
                    &cli.encoding,
                ),
                Curve::BN254 => {
                    template::<ark_bn254::Fr>(&b.with_name("BN254"), n, cli.k, cli.n, &cli.encoding)
                }
                Curve::Pallas => template::<ark_pallas::Fr>(
                    &b.with_name("PALLAS"),
                    n,
                    cli.k,
                    cli.n,
                    &cli.encoding,
                ),
            }
        }
    }
}
