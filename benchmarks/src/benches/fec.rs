use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use clap::ValueEnum;
use dragoonfri::algorithms::Sha3_512;
use plnk::Bencher;
use rand::{thread_rng, Rng, RngCore};

use komodo::{algebra::linalg::Matrix, fec, fri};

use crate::random::random_bytes;

#[derive(ValueEnum, Clone)]
pub(crate) enum Encoding {
    Vandermonde,
    Random,
    Fft,
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

pub(crate) fn run<F: PrimeField>(
    b: &Bencher,
    nb_bytes: usize,
    k: usize,
    n: usize,
    encoding: &Encoding,
) {
    let mut rng = thread_rng();

    match encoding {
        Encoding::Fft => {
            assert_eq!(n.count_ones(), 1, "n must be a power of 2");
            assert_eq!(k.count_ones(), 1, "k must be a power of 2");
            let bytes = random_bytes(nb_bytes, &mut rng);
            let mut shards: Vec<fec::Shard<F>> = vec![];
            plnk::bench(
                b,
                crate::label! { nb_bytes: nb_bytes, step: "encode", method: "fft", k: k, n: n },
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
                crate::label! { nb_bytes: nb_bytes, step: "decode", method: "fft", k: k, n: n },
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
                crate::label! { nb_bytes: nb_bytes, step: "encode", method: "matrix", k: k, n: n },
                || {
                    let bytes = random_bytes(nb_bytes, &mut rng);

                    plnk::timeit(|| fec::encode::<F>(&bytes, &encoding_mat).unwrap())
                },
            );

            let encoding_mat = build_encoding_mat(k, k, encoding, &mut rng);

            plnk::bench(
                b,
                crate::label! { nb_bytes: nb_bytes, step: "decode", method: "matrix", k: k, n: n },
                || {
                    let bytes = random_bytes(nb_bytes, &mut rng);
                    let shards = fec::encode::<F>(&bytes, &encoding_mat).unwrap();

                    plnk::timeit(|| fec::decode::<F>(shards.clone()).unwrap())
                },
            );
        }
    }
}
