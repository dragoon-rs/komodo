use std::io::prelude::*;
use std::ops::Div;
use std::path::Path;
use std::process::exit;
use std::{fs::File, path::PathBuf};

use ark_bls12_381::Bls12_381;
use ark_ec::pairing::Pairing;
use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::kzg10::Powers;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use komodo::recode;
use rs_merkle::algorithms::Sha256;
use rs_merkle::Hasher;
use tracing::{debug, info, warn};

use komodo::{
    encode,
    fec::{decode, Shard},
    setup, verify, Block,
};

type UniPoly12_381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

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
    let block_hashes = std::env::args().skip(11).collect::<Vec<_>>();

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
        block_hashes,
    )
}

fn throw_error(code: i32, message: &str) {
    eprint!("{}", message);
    exit(code);
}

fn generate_powers(n: usize, powers_file: &PathBuf) -> Result<(), std::io::Error> {
    info!("generating new powers");
    let powers = setup::random::<Bls12_381, UniPoly12_381>(n).unwrap_or_else(|_| {
        throw_error(3, "could not generate random trusted setup");
        unreachable!()
    });

    info!("serializing powers");
    let mut serialized = vec![0; powers.serialized_size(COMPRESS)];
    powers
        .serialize_with_mode(&mut serialized[..], COMPRESS)
        .unwrap_or_else(|_| throw_error(3, "could not serialize trusted setup"));

    info!("dumping powers into `{:?}`", powers_file);
    let mut file = File::create(powers_file)?;
    file.write_all(&serialized)?;

    Ok(())
}

fn read_block<E: Pairing>(block_hashes: &[String], block_dir: &Path) -> Vec<(String, Block<E>)> {
    block_hashes
        .iter()
        .map(|f| {
            let filename = block_dir.join(format!("{}.bin", f));
            let s = std::fs::read(filename).unwrap_or_else(|_| {
                throw_error(2, &format!("could not read block {}", f));
                unreachable!()
            });
            (
                f.clone(),
                Block::<E>::deserialize_with_mode(&s[..], COMPRESS, VALIDATE).unwrap_or_else(
                    |_| {
                        throw_error(2, &format!("could not deserialize block {}", f));
                        unreachable!()
                    },
                ),
            )
        })
        .collect::<Vec<_>>()
}

fn verify_blocks<E, P>(blocks: &[(String, Block<E>)], powers: Powers<E>)
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let res: Vec<_> = blocks
        .iter()
        .map(|(f, b)| {
            (
                f,
                verify::<E, P>(b, &powers).unwrap_or_else(|_| {
                    throw_error(
                        4,
                        &format!("verification failed unexpectedly for block {}", f),
                    );
                    unreachable!()
                }),
            )
        })
        .collect();

    eprint!("[");
    for (f, v) in res {
        eprint!("{{block: {:?}, status: {}}}", f, v);
    }
    eprint!("]");
}

fn dump_blocks<E: Pairing>(blocks: &[Block<E>], block_dir: &PathBuf) -> Result<(), std::io::Error> {
    info!("dumping blocks to `{:?}`", block_dir);
    let mut hashes = vec![];
    for (i, block) in blocks.iter().enumerate() {
        debug!("serializing block {}", i);
        let mut serialized = vec![0; block.serialized_size(COMPRESS)];
        block
            .serialize_with_mode(&mut serialized[..], COMPRESS)
            .unwrap_or_else(|_| throw_error(5, &format!("could not serialize block {}", i)));
        let repr = Sha256::hash(&serialized)
            .iter()
            .map(|x| format!("{:x}", x))
            .collect::<Vec<_>>()
            .join("");

        let filename = block_dir.join(format!("{}.bin", repr));
        std::fs::create_dir_all(block_dir)?;

        debug!("dumping serialized block to `{:?}`", filename);
        let mut file = File::create(&filename)?;
        file.write_all(&serialized)?;

        hashes.push(repr);
    }

    eprint!("[");
    for hash in &hashes {
        eprint!("{:?},", hash);
    }
    eprint!("]");

    Ok(())
}

fn main() {
    tracing_subscriber::fmt::try_init().expect("cannot init logger");

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
        block_hashes,
    ) = parse_args();

    let home_dir = PathBuf::from(&home_dir);
    let block_dir = home_dir.join("blocks/");
    let powers_file = home_dir.join("powers.bin");

    if do_generate_powers {
        generate_powers(nb_bytes, &powers_file)
            .unwrap_or_else(|e| throw_error(1, &format!("could not generate powers: {}", e)));

        exit(0);
    }

    if do_reconstruct_data {
        let blocks: Vec<Shard<Bls12_381>> = read_block::<Bls12_381>(&block_hashes, &block_dir)
            .iter()
            .cloned()
            .map(|b| b.1.shard)
            .collect();
        eprintln!(
            "{:?}",
            decode::<Bls12_381>(blocks, true).unwrap_or_else(|e| {
                throw_error(1, &format!("could not decode: {}", e));
                unreachable!()
            })
        );

        exit(0);
    }

    if do_combine_blocks {
        let blocks = read_block::<Bls12_381>(&block_hashes, &block_dir);
        if blocks.len() != 2 {
            throw_error(
                1,
                &format!("expected exactly 2 blocks, found {}", blocks.len()),
            );
        }

        dump_blocks(&[recode(&blocks[0].1, &blocks[1].1)], &block_dir)
            .unwrap_or_else(|e| throw_error(1, &format!("could not dump block: {}", e)));

        exit(0);
    }

    if do_inspect_blocks {
        let blocks = read_block::<Bls12_381>(&block_hashes, &block_dir);
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
        Powers::<Bls12_381>::deserialize_with_mode(&serialized[..], COMPRESS, VALIDATE)
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
        setup::random::<Bls12_381, UniPoly12_381>(nb_bytes).unwrap_or_else(|e| {
            throw_error(1, &format!("could not generate powers: {}", e));
            unreachable!()
        })
    };

    if do_verify_blocks {
        verify_blocks::<Bls12_381, UniPoly12_381>(
            &read_block::<Bls12_381>(&block_hashes, &block_dir),
            powers,
        );

        exit(0);
    }

    dump_blocks(
        &encode::<Bls12_381, UniPoly12_381>(&bytes, k, n, &powers).unwrap_or_else(|e| {
            throw_error(1, &format!("could not encode: {}", e));
            unreachable!()
        }),
        &block_dir,
    )
    .unwrap_or_else(|e| throw_error(1, &format!("could not dump blocks: {}", e)));
}
