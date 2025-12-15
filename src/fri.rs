use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_std::ops::Div;
use rs_merkle::algorithms::Sha256;
use rs_merkle::Hasher;
use std::rc::Rc;
use tracing::{debug, info};

use crate::{algebra, error::KomodoError, fec};
use dragoonfri::{
    frida::{FridaBuilder, FridaCommitment},
    interpolation::interpolate_polynomials,
    rng::FriChallenger,
    utils::{to_evaluations, HasherExt, MerkleProof},
};

/// representation of a block of proven data.
///
/// this is a wrapper around a [`fec::Shard`] with some additional cryptographic
/// information that allows to prove the integrity of said shard.
#[derive(Clone, PartialEq)]
pub struct Block<F: PrimeField, H: Hasher> {
    pub shard: fec::Shard<F>,
    pub proof: MerkleProof<H>,
    pub commit: Rc<FridaCommitment<F, H>>,
    position: usize,
}

pub fn evaluate<F: PrimeField>(bytes: &[u8], k: usize, n: usize) -> Vec<Vec<F>> {
    debug!("splitting bytes into rows");
    let elements: Vec<F> = algebra::split_data_into_field_elements(bytes, k);
    let rows = elements.chunks(k).map(|c| c.to_vec()).collect::<Vec<_>>();
    info!(
        "data is composed of {} rows and {} elements",
        rows.len(),
        elements.len()
    );

    rows.into_iter()
        .map(|r| to_evaluations(r, n))
        .collect::<Vec<_>>()
}

#[inline]
fn transpose<F: Copy>(v: &[Vec<F>]) -> Vec<Vec<F>> {
    let mut cols: Vec<_> = Vec::<Vec<F>>::with_capacity(v[0].len());
    for i in 0..v[0].len() {
        cols.push((0..v.len()).map(|j| v[j][i]).collect());
    }
    cols
}

pub fn encode<F: PrimeField>(bytes: &[u8], evaluations: &[Vec<F>], k: usize) -> Vec<fec::Shard<F>> {
    let hash = Sha256::hash(bytes);

    let n = evaluations[0].len();

    let t = transpose(evaluations);

    (0..n)
        .map(|i| fec::Shard {
            k: k as u32,
            linear_combination: vec![],
            hash: hash.to_vec(),
            data: t[i].clone(),
            size: bytes.len(),
        })
        .collect::<Vec<_>>()
}

pub fn prove<const N: usize, F: PrimeField, H: Hasher, P>(
    evaluations: &[Vec<F>],
    shards: &[fec::Shard<F>],
    blowup_factor: usize,
    remainder_plus_one: usize,
    nb_queries: usize,
) -> Result<Vec<Block<F, H>>, KomodoError>
where
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    <H as rs_merkle::Hasher>::Hash: AsRef<[u8]>,
{
    let builder = FridaBuilder::<F, H>::new::<N, _>(
        evaluations,
        FriChallenger::<H>::default(),
        blowup_factor,
        remainder_plus_one,
        nb_queries,
    );

    let commit = Rc::new(FridaCommitment::from(builder.clone()));

    Ok(shards
        .iter()
        .enumerate()
        .map(|(i, s)| Block {
            shard: s.clone(),
            proof: builder.prove_shards(&[i]),
            commit: commit.clone(),
            position: i,
        })
        .collect())
}

pub fn verify<const N: usize, F: PrimeField, H: Hasher, P>(
    block: &Block<F, H>,
    domain_size: usize,
    nb_queries: usize,
) -> Result<(), KomodoError>
where
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    <H as rs_merkle::Hasher>::Hash: AsRef<[u8]>,
{
    block
        .commit
        .verify::<N, _>(
            FriChallenger::<H>::default(),
            nb_queries,
            block.shard.k as usize,
            domain_size,
        )
        .unwrap();

    assert!(block.proof.verify(
        block.commit.tree_root(),
        &[block.position],
        &[H::hash_item(&block.shard.data)],
        domain_size,
    ));

    Ok(())
}

pub fn decode<F: PrimeField, H: Hasher>(blocks: &[Block<F, H>], n: usize) -> Vec<u8> {
    let w = F::get_root_of_unity(n as u64).unwrap();

    let t_shards = transpose(
        &blocks
            .iter()
            .map(|b| b.shard.data.clone())
            .collect::<Vec<_>>(),
    );
    let positions = blocks
        .iter()
        .map(|b| w.pow([b.position as u64]))
        .collect::<Vec<_>>();
    let source_shards = interpolate_polynomials(&t_shards, &positions)
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let mut bytes = algebra::merge_elements_into_bytes(&source_shards);
    bytes.resize(blocks[0].shard.size, 0);
    bytes
}

#[cfg(test)]
mod tests {
    use ark_ff::PrimeField;
    use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
    use ark_serialize::CanonicalSerialize;
    use ark_std::ops::Div;
    use rs_merkle::Hasher;

    use ark_bls12_381::Fr as F_BLS12_381;
    use dragoonfri::{
        algorithms::{Blake3, Sha3_256, Sha3_512},
        dynamic_folding_factor,
    };
    use dragoonfri_test_utils::Fq as F_128;

    use crate::error::KomodoError;

    use super::{decode, encode, evaluate, prove, verify};

    fn bytes() -> Vec<u8> {
        include_bytes!("../assets/dragoon_133x133.png").to_vec()
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
        let evaluations = evaluate::<F>(bytes, k, n);

        let evals = evaluations.clone();
        let shards = encode::<F>(bytes, &evals, k);

        let blocks = prove::<N, F, H, P>(&evaluations, &shards, bf, rpo, q).unwrap();

        for b in &blocks {
            verify::<N, F, H, P>(b, n, q).unwrap();
        }

        assert_eq!(decode::<F, H>(&blocks[0..k], n), bytes);

        Ok(())
    }

    macro_rules! run {
        ($n:tt, $f:ident, $h:ident) => {
            dynamic_folding_factor!(
                let N = $n => run::<N, $f, $h, DensePolynomial<$f>>
            )
        }
    }

    #[test]
    fn end_to_end() {
        for (ff, k, n, bf, rpo, q) in [(2, 4, 8, 2, 1, 50), (2, 4, 8, 2, 2, 50)] {
            let _ = run!(ff, F_128, Blake3)(&bytes(), k, n, bf, rpo, q);
            let _ = run!(ff, F_128, Sha3_256)(&bytes(), k, n, bf, rpo, q);
            let _ = run!(ff, F_128, Sha3_512)(&bytes(), k, n, bf, rpo, q);
            let _ = run!(ff, F_BLS12_381, Blake3)(&bytes(), k, n, bf, rpo, q);
            let _ = run!(ff, F_BLS12_381, Sha3_256)(&bytes(), k, n, bf, rpo, q);
            let _ = run!(ff, F_BLS12_381, Sha3_512)(&bytes(), k, n, bf, rpo, q);
        }
    }
}
