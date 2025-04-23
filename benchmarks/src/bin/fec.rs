// see `examples/benches/README.md`
use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use benchmarks::fields::Fq128;
use clap::{arg, command, Parser, ValueEnum};
use dragoonfri::algorithms::Sha3_512;
use komodo::{algebra::linalg::Matrix, fec, fri};
use plnk::Bencher;
use rand::{rngs::ThreadRng, thread_rng, Rng, RngCore};

fn random_bytes(n: usize, rng: &mut ThreadRng) -> Vec<u8> {
    (0..n).map(|_| rng.gen::<u8>()).collect()
}

fn random_loss<T>(shards: &mut Vec<T>, k: usize, rng: &mut impl Rng) {
    // Randomly drop some shards until k are left
    while shards.len() > k {
        let i = rng.gen_range(0..shards.len());
        shards.remove(i);
    }
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
        _ => panic!("FFT encoding is not supported for matrix encoding"),
    }
}

fn template<F: PrimeField>(b: &Bencher, nb_bytes: usize, k: usize, n: usize, encoding: &Encoding) {
    let mut rng = thread_rng();

    match encoding {
        Encoding::Fft => {
            assert_eq!(n.count_ones(), 1, "n must be a power of 2");
            assert_eq!(k.count_ones(), 1, "k must be a power of 2");
            let bytes = random_bytes(nb_bytes, &mut rng);
            let mut shards: Vec<fec::Shard<F>> = vec![];
            plnk::bench(
                b,
                &format!(
                    r#"{{"bytes": {}, "step": "encode", "method": "fft", "k": {}, "n": {}}}"#,
                    nb_bytes, k, n
                ),
                || {
                    plnk::timeit(|| {
                        let evaluations = fri::evaluate::<F>(&bytes, k, n);
                        shards = fri::encode::<F>(&bytes, evaluations, k)
                    })
                },
            );

            let evaluations = fri::evaluate::<F>(&bytes, k, n);
            let mut blocks =
                fri::prove::<2, F, Sha3_512, DensePolynomial<F>>(evaluations, shards, 2, 2, 1)
                    .unwrap();

            random_loss(&mut blocks, k, &mut rng);

            plnk::bench(
                b,
                &format!(
                    r#"{{"bytes": {}, "step": "decode", "method":"fft" "k": {}, "n": {}}}"#,
                    nb_bytes, k, n
                ),
                || {
                    plnk::timeit(|| {
                        fri::decode::<F, Sha3_512>(blocks.clone(), n);
                    })
                },
            );
        }
        _ => {
            let encoding_mat = build_encoding_mat(k, n, encoding, &mut rng);

            plnk::bench(
                b,
                &format!(
                    r#"{{"bytes": {}, "step": "encode", "method": "matrix", "k": {}, "n": {}}}"#,
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
                    r#"{{"bytes": {}, "step": "decode", "method":"matrix", "k": {}, "n": {}}}"#,
                    nb_bytes, k, n
                ),
                || {
                    let bytes = random_bytes(nb_bytes, &mut rng);
                    let shards = fec::encode::<F>(&bytes, &encoding_mat).unwrap();

                    plnk::timeit(|| fec::decode::<F>(shards.clone()).unwrap())
                },
            );
        }
    }
}

#[derive(ValueEnum, Clone)]
enum Encoding {
    Vandermonde,
    Random,
    Fft,
}

#[derive(ValueEnum, Clone, Hash, PartialEq, Eq)]
enum Curve {
    BLS12381,
    BN254,
    Pallas,
    FP128,
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
                Curve::FP128 => {
                    template::<Fq128>(&b.with_name("FP128"), n, cli.k, cli.n, &cli.encoding)
                }
            }
        }
    }
}
