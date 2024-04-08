use std::path::{Path, PathBuf};
use std::process::exit;

use ark_bls12_381::{Fr, G1Projective};
use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_serialize::{CanonicalDeserialize, Compress, Validate};
use ark_std::ops::Div;

use anyhow::Result;
use ark_std::rand::RngCore;
use tracing::{info, warn};

use komodo::{
    encode,
    error::KomodoError,
    fec::{decode, Shard},
    fs,
    linalg::Matrix,
    recode, verify,
    zk::{self, Powers},
    Block,
};

const COMPRESS: Compress = Compress::Yes;
const VALIDATE: Validate = Validate::Yes;

#[allow(clippy::type_complexity)]
fn parse_args() -> (
    Vec<u8>,
    usize,
    usize,
    bool,
    String,
    bool,
    bool,
    bool,
    bool,
    usize,
    String,
    Vec<String>,
) {
    let bytes_path = std::env::args()
        .nth(1)
        .expect("expected path to bytes as first positional argument");
    let bytes = if bytes_path.is_empty() {
        vec![]
    } else {
        std::fs::read(bytes_path).unwrap()
    };
    let k: usize = std::env::args()
        .nth(2)
        .expect("expected k as second positional argument")
        .parse()
        .expect("could not parse k as an int");
    let n: usize = std::env::args()
        .nth(3)
        .expect("expected n as third positional argument")
        .parse()
        .expect("could not parse n as an int");
    let do_generate_powers: bool = std::env::args()
        .nth(4)
        .expect("expected do_generate_powers as fourth positional argument")
        .parse()
        .expect("could not parse do_generate_powers as a bool");
    let home_dir = std::env::args()
        .nth(5)
        .expect("expected home_dir as fifth positional argument");
    let do_reconstruct_data: bool = std::env::args()
        .nth(6)
        .expect("expected do_reconstruct_data as sixth positional argument")
        .parse()
        .expect("could not parse do_reconstruct_data as a bool");
    let do_verify_blocks: bool = std::env::args()
        .nth(7)
        .expect("expected do_verify_blocks as seventh positional argument")
        .parse()
        .expect("could not parse do_verify_blocks as a bool");
    let do_combine_blocks: bool = std::env::args()
        .nth(8)
        .expect("expected do_combine_blocks as eigth positional argument")
        .parse()
        .expect("could not parse do_combine_blocks as a bool");
    let do_inspect_blocks: bool = std::env::args()
        .nth(9)
        .expect("expected do_inspect_blocks as ninth positional argument")
        .parse()
        .expect("could not parse do_inspect_blocks as a bool");
    let nb_bytes: usize = std::env::args()
        .nth(10)
        .expect("expected nb_bytes as 10th positional argument")
        .parse()
        .expect("could not parse nb_bytes as a usize");
    let encoding_method = std::env::args()
        .nth(11)
        .expect("expected encoding_method as 11th positional argument");
    let block_hashes = std::env::args().skip(12).collect::<Vec<_>>();

    (
        bytes,
        k,
        n,
        do_generate_powers,
        home_dir,
        do_reconstruct_data,
        do_verify_blocks,
        do_combine_blocks,
        do_inspect_blocks,
        nb_bytes,
        encoding_method,
        block_hashes,
    )
}

fn throw_error(code: i32, message: &str) {
    eprint!("{}", message);
    exit(code);
}

pub fn generate_random_powers<F, G, P, R>(
    n: usize,
    powers_dir: &Path,
    powers_filename: Option<&str>,
    rng: &mut R,
) -> Result<()>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    R: RngCore,
{
    info!("generating new powers");
    let powers = zk::setup::<_, F, G>(zk::nb_elements_in_setup::<F>(n), rng)?;

    fs::dump(&powers, powers_dir, powers_filename, COMPRESS)?;

    Ok(())
}

pub fn verify_blocks<F, G, P>(
    blocks: &[(String, Block<F, G>)],
    powers: Powers<F, G>,
) -> Result<(), KomodoError>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let res = blocks
        .iter()
        .map(|(f, b)| Ok((f, verify::<F, G, P>(b, &powers)?)))
        .collect::<Result<Vec<(&String, bool)>, KomodoError>>()?;

    eprint!("[");
    for (f, v) in res {
        eprint!("{{block: {:?}, status: {}}}", f, v);
    }
    eprint!("]");
    Ok(())
}

fn main() {
    tracing_subscriber::fmt::try_init().expect("cannot init logger");

    let mut rng = rand::thread_rng();

    let (
        bytes,
        k,
        n,
        do_generate_powers,
        home_dir,
        do_reconstruct_data,
        do_verify_blocks,
        do_combine_blocks,
        do_inspect_blocks,
        nb_bytes,
        encoding_method,
        block_hashes,
    ) = parse_args();

    let home_dir = PathBuf::from(&home_dir);
    let block_dir = home_dir.join("blocks/");
    let powers_dir = home_dir;
    let powers_filename = "powers";
    let powers_file = powers_dir.join(powers_filename);

    if do_generate_powers {
        generate_random_powers::<Fr, G1Projective, DensePolynomial<Fr>, _>(
            nb_bytes,
            &powers_dir,
            Some(powers_filename),
            &mut rng,
        )
        .unwrap_or_else(|e| throw_error(1, &format!("could not generate powers: {}", e)));

        exit(0);
    }

    if do_reconstruct_data {
        let blocks: Vec<Shard<Fr>> =
            fs::read_blocks::<Fr, G1Projective>(&block_hashes, &block_dir, COMPRESS, VALIDATE)
                .unwrap_or_else(|e| {
                    throw_error(1, &format!("could not read blocks: {}", e));
                    unreachable!()
                })
                .iter()
                .cloned()
                .map(|b| b.1.shard)
                .collect();
        eprintln!(
            "{:?}",
            decode::<Fr>(blocks).unwrap_or_else(|e| {
                throw_error(1, &format!("could not decode: {}", e));
                unreachable!()
            })
        );

        exit(0);
    }

    if do_combine_blocks {
        let blocks =
            fs::read_blocks::<Fr, G1Projective>(&block_hashes, &block_dir, COMPRESS, VALIDATE)
                .unwrap_or_else(|e| {
                    throw_error(1, &format!("could not read blocks: {}", e));
                    unreachable!()
                });

        let formatted_output = fs::dump_blocks(
            &[recode(
                &blocks.iter().map(|(_, b)| b).cloned().collect::<Vec<_>>(),
                &mut rng,
            )
            .unwrap_or_else(|e| {
                throw_error(1, &format!("could not encode block: {}", e));
                unreachable!()
            })
            .unwrap_or_else(|| {
                throw_error(1, "could not recode block (list of blocks is likely empty)");
                unreachable!()
            })],
            &block_dir,
            COMPRESS,
        )
        .unwrap_or_else(|e| {
            throw_error(1, &format!("could not dump block: {}", e));
            unreachable!()
        });

        eprint!("{}", formatted_output);

        exit(0);
    }

    if do_inspect_blocks {
        let blocks =
            fs::read_blocks::<Fr, G1Projective>(&block_hashes, &block_dir, COMPRESS, VALIDATE)
                .unwrap_or_else(|e| {
                    throw_error(1, &format!("could not read blocks: {}", e));
                    unreachable!()
                });
        eprint!("[");
        for (_, block) in &blocks {
            eprint!("{},", block);
        }
        eprintln!("]");

        exit(0);
    }

    info!("reading powers from file `{:?}`", powers_file);
    let powers = if let Ok(serialized) = std::fs::read(&powers_file) {
        info!("deserializing the powers from `{:?}`", powers_file);
        Powers::<Fr, G1Projective>::deserialize_with_mode(&serialized[..], COMPRESS, VALIDATE)
            .unwrap_or_else(|e| {
                throw_error(
                    1,
                    &format!("could not deserialize powers from {:?}: {}", powers_file, e),
                );
                unreachable!()
            })
    } else {
        warn!("could not read powers from `{:?}`", powers_file);
        info!("regenerating temporary powers");
        zk::setup::<_, Fr, G1Projective>(zk::nb_elements_in_setup::<Fr>(nb_bytes), &mut rng)
            .unwrap_or_else(|e| {
                throw_error(1, &format!("could not generate powers: {}", e));
                unreachable!()
            })
    };

    if do_verify_blocks {
        verify_blocks::<Fr, G1Projective, DensePolynomial<Fr>>(
            &fs::read_blocks::<Fr, G1Projective>(&block_hashes, &block_dir, COMPRESS, VALIDATE)
                .unwrap_or_else(|e| {
                    throw_error(1, &format!("could not read blocks: {}", e));
                    unreachable!()
                }),
            powers,
        )
        .unwrap_or_else(|e| {
            throw_error(1, &format!("Failed to verify blocks: {}", e));
            unreachable!()
        });

        exit(0);
    }

    let encoding_mat = match encoding_method.as_str() {
        "vandermonde" => {
            let points: Vec<Fr> = (0..n)
                .map(|i| Fr::from_le_bytes_mod_order(&i.to_le_bytes()))
                .collect();
            Matrix::vandermonde(&points, k)
        }
        "random" => Matrix::random(k, n, &mut rng),
        m => {
            throw_error(1, &format!("invalid encoding method: {}", m));
            unreachable!()
        }
    };

    let formatted_output = fs::dump_blocks(
        &encode::<Fr, G1Projective, DensePolynomial<Fr>>(&bytes, &encoding_mat, &powers)
            .unwrap_or_else(|e| {
                throw_error(1, &format!("could not encode: {}", e));
                unreachable!()
            }),
        &block_dir,
        COMPRESS,
    )
    .unwrap_or_else(|e| {
        throw_error(1, &format!("could not dump blocks: {}", e));
        unreachable!()
    });

    eprint!("{}", formatted_output);
}
