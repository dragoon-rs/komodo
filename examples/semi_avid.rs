use ark_bls12_381::{Fr, G1Projective};
use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use ark_std::{ops::Div, test_rng};

use komodo::{
    algebra::linalg::Matrix,
    error::KomodoError,
    fec::{decode, encode},
    semi_avid::{build, commit, recode, verify, Block},
    zk::setup,
};

fn run<F, G, P>() -> Result<(), KomodoError>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut rng = test_rng();

    // the code parameters and the data to manipulate
    let (k, n) = (3, 6_usize);
    let bytes = include_bytes!("../assets/dragoon_133x133.png").to_vec();
    eprintln!("loaded {} bytes of data", bytes.len());

    // Semi-AVID needs a _trusted setup_ to prove and verify blocks of encoded data
    eprint!("creating trusted setup... ");
    let powers = setup::<F, G>(bytes.len(), &mut rng)?;
    eprintln!("done");

    // encode and prove the data with a _random_ encoding
    eprint!("building blocks... ");
    let encoding_mat = &Matrix::random(k, n, &mut rng);
    let shards = encode(&bytes, encoding_mat)?;
    let commitment = commit(&bytes, &powers, encoding_mat.height)?;
    let blocks = build::<F, G, P>(&shards, &commitment);
    eprintln!("done");

    // verify that all the blocks are valid
    eprint!("verifying blocks... ");
    for block in &blocks {
        assert!(verify(block, &powers)?);
    }

    // corrupt the first block...
    let mut serialized = vec![0; blocks[0].serialized_size(Compress::No)];
    blocks[0]
        .serialize_with_mode(&mut serialized[..], Compress::No)
        .unwrap();
    // -> attack the `data` field of the [`komodo::fec::Shard`] structure
    let field_element_size = (F::MODULUS_BIT_SIZE as usize).div_ceil(8);
    const VEC_LEN_SIZE: usize = 8;
    const HASH_SIZE: usize = 32;
    const U32_SIZE: usize = 4;
    let data_start_index =
        U32_SIZE + VEC_LEN_SIZE + k * field_element_size + VEC_LEN_SIZE + HASH_SIZE + VEC_LEN_SIZE;
    serialized[data_start_index] = 0x00;
    let block: Block<F, G> =
        Block::deserialize_with_mode(&serialized[..], Compress::No, Validate::No).unwrap();
    // ... and make sure it is not valid anymore
    assert!(!verify(&block, &powers)?);

    eprintln!("all good");

    // some recoding examples:
    // - let's denote the original blocks by $(b_i)_{0 \leq i \lt n}$
    // - if block $b$ is the result of recoding blocks $b_i$ and $b_j$, then we write
    // $b = b_i + b_j$
    eprint!("some recoding scenarii... ");

    // successfully decode the data with the following blocks
    // - $b_0 + b_1$
    // - $b_2$
    // - $b_3$
    //
    // > **Note**
    // >
    // > it works because $b_0$, $b_1$, $b_2$ and $b_3$ are all linearly independent and thus $b_0
    // + b_1$, $b_2$ and $b_3$ are as well
    let b_0_1 = recode(&blocks[0..=1], &mut rng).unwrap().unwrap();
    let shards = vec![
        b_0_1.shard,
        blocks[2].shard.clone(),
        blocks[3].shard.clone(),
    ];
    assert_eq!(bytes, decode(&shards).unwrap());

    // fail to decode the data with the following blocks
    // - $b_0$
    // - $b_1$
    // - $b_0 + b_1$
    //
    // > **Note**
    // >
    // > it fails because $b_0 + b_1$ is lineary dependent on $b_0$ and $b_1$
    let b_0_1 = recode(&[blocks[0].clone(), blocks[1].clone()], &mut rng)
        .unwrap()
        .unwrap();
    let shards = vec![
        blocks[0].shard.clone(),
        blocks[1].shard.clone(),
        b_0_1.shard,
    ];
    assert!(decode(&shards).is_err());

    // successfully decode the data with the following blocks
    // - $b_0 + b_1$
    // - $b_2 + b_3$
    // - $b_1 + b_4$
    let b_0_1 = recode(&blocks[0..=1], &mut rng).unwrap().unwrap();
    let b_2_3 = recode(&blocks[2..=3], &mut rng).unwrap().unwrap();
    let b_1_4 = recode(&[blocks[1].clone(), blocks[4].clone()], &mut rng)
        .unwrap()
        .unwrap();
    let shards = vec![b_0_1.shard, b_2_3.shard, b_1_4.shard];
    assert_eq!(bytes, decode(&shards).unwrap());

    // successfully decode the data with the following blocks
    // - $b_0 + b_1 + b_2$
    // - $b_0 + b_1 + b_2$
    // - $b_0 + b_1 + b_2$
    //
    // > **Note**
    // >
    // > it works, even though all three recoded shards come from the same original ones, because
    // > the linear combinations that generate the recoded shards are random and different each
    // > time. because the finite field used is so large, we end up with linearly independent shards
    let fully_recoded_shards = (0..3)
        .map(|_| recode(&blocks[0..=2], &mut rng).unwrap().unwrap().shard)
        .collect::<Vec<_>>();
    assert_eq!(bytes, decode(&fully_recoded_shards).unwrap());

    eprintln!("all good");

    Ok(())
}

fn main() {
    run::<Fr, G1Projective, DensePolynomial<Fr>>().unwrap();
}
