use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_serialize::CanonicalSerialize;
use ark_std::ops::Div;
use rand::{thread_rng, Rng};
use rs_merkle::Hasher;

pub(crate) fn random_bytes(n: usize, rng: &mut impl Rng) -> Vec<u8> {
    (0..n).map(|_| rng.gen::<u8>()).collect()
}

fn run<const N: usize, F: PrimeField, H: Hasher, P>(
    nb_bytes: usize,
    k: usize,
    n: usize,
    bf: usize,
    rpo: usize,
    q: usize,
    rng: &mut impl Rng,
) where
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    <H as rs_merkle::Hasher>::Hash: AsRef<[u8]> + CanonicalSerialize,
{
    let bytes = random_bytes(nb_bytes, rng);

    let evaluations = komodo::fri::evaluate::<F>(&bytes, k, n);

    let shards = komodo::fri::encode::<F>(&bytes, &evaluations, k);

    let builder = dragoonfri::frida::FridaBuilder::<F, H>::new::<N, _>(
        &evaluations,
        dragoonfri::rng::FriChallenger::<H>::default(),
        bf,
        rpo,
        q,
    );

    let commitment = komodo::fri::commit(builder.clone());

    let proofs = komodo::fri::prove::<F, H>(builder.clone(), &(0..n).collect::<Vec<_>>());

    for (shard, proof) in shards.iter().zip(proofs.iter()) {
        assert!(komodo::fri::verify::<N, F, H, P>(shard, &commitment, proof, n, q,).is_ok())
    }

    let decoded = komodo::fri::decode::<F>(
        &shards.clone().into_iter().enumerate().collect::<Vec<_>>(),
        n,
    );

    assert_eq!(hex::encode(bytes), hex::encode(decoded));
}

fn main() {
    run::<2, ark_bls12_381::Fr, dragoonfri::algorithms::Sha3_256, DensePolynomial<ark_bls12_381::Fr>>(
        1024,
        4,
        8,
        2,
        1,
        50,
        &mut thread_rng(),
    );
}
