use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use clap::ValueEnum;
use dragoonfri::algorithms::Sha3_512;
use rand::{thread_rng, Rng, RngCore};

use komodo::{algebra::linalg::Matrix, fec, fri};

use crate::random::random_bytes;

#[derive(ValueEnum, Clone, Debug)]
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

pub(crate) fn build<F: PrimeField>(
    nb_bytes: usize,
    k: usize,
    n: usize,
    encoding: &Encoding,
) -> Vec<(String, plnk::FnTimed<()>)> {
    match encoding {
        Encoding::Fft => {
            assert_eq!(n.count_ones(), 1, "n must be a power of 2");
            assert_eq!(k.count_ones(), 1, "k must be a power of 2");

            vec![
                (
                    "encode".to_string(),
                    plnk::closure! {
                        let bytes = random_bytes(nb_bytes, &mut thread_rng());
                        crate::timeit_and_discard_output! {
                            fri::encode::<F>(&bytes, fri::evaluate::<F>(&bytes, k, n), k);
                        }
                    },
                ),
                (
                    "decode".to_string(),
                    plnk::closure! {
                        let bytes = random_bytes(nb_bytes, &mut thread_rng());
                        let evaluations = fri::evaluate::<F>(&bytes, k, n);
                        let shards = fri::encode::<F>(&bytes, evaluations.clone(), k);

                        let mut blocks =
                            fri::prove::<2, F, Sha3_512, DensePolynomial<F>>(evaluations, shards, 2, 2, 1)
                                .unwrap();

                        random_loss(&mut blocks, k, &mut thread_rng());

                        plnk::timeit(|| {
                            fri::decode::<F, Sha3_512>(blocks.clone(), n);
                        })
                    },
                ),
            ]
        }
        _ => {
            let encoding_encode = encoding.clone();
            let encoding_decode = encoding.clone();
            vec![
                (
                    "encode".to_string(),
                    plnk::closure! {
                        let encoding_mat = build_encoding_mat(k, n, &encoding_encode, &mut thread_rng());

                        let bytes = random_bytes(nb_bytes, &mut thread_rng());
                        crate::timeit_and_discard_output! {
                            fec::encode::<F>(&bytes, &encoding_mat).unwrap();
                        }
                    },
                ),
                (
                    "decode".to_string(),
                    plnk::closure! {
                        let encoding_mat = build_encoding_mat(k, k, &encoding_decode, &mut thread_rng());

                        let bytes = random_bytes(nb_bytes, &mut thread_rng());
                        let shards = fec::encode::<F>(&bytes, &encoding_mat).unwrap();

                        crate::timeit_and_discard_output! {
                            fec::decode::<F>(shards.clone()).unwrap();
                        }
                    },
                ),
            ]
        }
    }
}
