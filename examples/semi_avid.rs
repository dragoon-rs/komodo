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
    fec::{self, decode, encode, recode_random},
    semi_avid::{commit, verify},
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

    // Semi-AVID needs a _trusted setup_ to commit and verify shards of encoded data
    eprint!("creating trusted setup... ");
    let powers = setup::<F, G>(bytes.len(), &mut rng)?;
    eprintln!("done");

    // encode the data with a _random_ encoding
    eprint!("building shards... ");
    let encoding_mat = &Matrix::random(k, n, &mut rng);
    let shards = encode(&bytes, encoding_mat)?;
    eprintln!("done");
    eprint!("committing data... ");
    let commitment = commit(&bytes, &powers, encoding_mat.height)?;
    eprintln!("done");

    // verify that all the shards are valid
    eprint!("verifying shards... ");
    for shard in &shards {
        assert!(verify(shard, &commitment, &powers)?);
    }

    // corrupt the first shard...
    let mut serialized = vec![0; shards[0].serialized_size(Compress::No)];
    shards[0]
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
    let shard: fec::Shard<F> =
        fec::Shard::deserialize_with_mode(&serialized[..], Compress::No, Validate::No).unwrap();
    // ... and make sure it is not valid anymore
    assert!(!verify(&shard, &commitment, &powers)?);

    eprintln!("all good");

    // some recoding examples:
    // - let's denote the original shards by $(s_i)_{0 \leq i \lt n}$
    // - if shard $s$ is the result of recoding shards $s_i$ and $s_j$, then we write
    // $s = s_i + s_j$
    eprint!("some recoding scenarii... ");

    // successfully decode the data with the following shards
    // - $s_0 + s_1$
    // - $s_2$
    // - $s_3$
    //
    // > **Note**
    // >
    // > it works because $s_0$, $s_1$, $s_2$ and $s_3$ are all linearly independent and thus $s_0
    // + s_1$, $s_2$ and $s_3$ are as well
    let s_0_1 = recode_random(&shards[0..=1], &mut rng).unwrap().unwrap();
    let shards = vec![s_0_1, shards[2].clone(), shards[3].clone()];
    assert_eq!(bytes, decode(&shards).unwrap());

    // fail to decode the data with the following shards
    // - $s_0$
    // - $s_1$
    // - $s_0 + s_1$
    //
    // > **Note**
    // >
    // > it fails because $s_0 + s_1$ is lineary dependent on $s_0$ and $s_1$
    let s_0_1 = recode_random(&[shards[0].clone(), shards[1].clone()], &mut rng)
        .unwrap()
        .unwrap();
    let shards = vec![shards[0].clone(), shards[1].clone(), s_0_1];
    assert!(decode(&shards).is_err());

    // successfully decode the data with the following shards
    // - $s_0 + s_1$
    // - $s_2 + s_3$
    // - $s_1 + s_4$
    let s_0_1 = recode_random(&shards[0..=1], &mut rng).unwrap().unwrap();
    let s_2_3 = recode_random(&shards[2..=3], &mut rng).unwrap().unwrap();
    let s_1_4 = recode_random(&[shards[1].clone(), shards[4].clone()], &mut rng)
        .unwrap()
        .unwrap();
    let shards = vec![s_0_1, s_2_3, s_1_4];
    assert_eq!(bytes, decode(&shards).unwrap());

    // successfully decode the data with the following shards
    // - $s_0 + s_1 + s_2$
    // - $s_0 + s_1 + s_2$
    // - $s_0 + s_1 + s_2$
    //
    // > **Note**
    // >
    // > it works, even though all three recoded shards come from the same original ones, because
    // > the linear combinations that generate the recoded shards are random and different each
    // > time. because the finite field used is so large, we end up with linearly independent shards
    let fully_recoded_shards = (0..3)
        .map(|_| recode_random(&shards[0..=2], &mut rng).unwrap().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(bytes, decode(&fully_recoded_shards).unwrap());

    eprintln!("all good");

    Ok(())
}

fn main() {
    run::<Fr, G1Projective, DensePolynomial<Fr>>().unwrap();
}
