use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_std::ops::Div;
use rs_merkle::algorithms::Sha256;
use rs_merkle::Hasher;
use tracing::{debug, info};

use crate::{algebra, error::KomodoError, fec};
use dragoonfri::{
    frida::{FridaBuilder, FridaCommitment},
    interpolation::interpolate_polynomials,
    rng::FriChallenger,
    utils::{to_evaluations, HasherExt, MerkleProof},
};

#[derive(Clone, PartialEq)]
pub struct Commitment<F: PrimeField, H: Hasher>(pub FridaCommitment<F, H>);

#[derive(Clone, PartialEq)]
pub struct Proof<H: Hasher> {
    pub path: MerkleProof<H>,
    pub position: usize,
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

pub fn commit<F: PrimeField, H: Hasher>(builder: FridaBuilder<F, H>) -> Commitment<F, H> {
    Commitment(FridaCommitment::from(builder))
}

pub fn prove<F: PrimeField, H: Hasher>(
    builder: FridaBuilder<F, H>,
    positions: &[usize],
) -> Vec<Proof<H>> {
    positions
        .iter()
        .map(|i| Proof {
            path: builder.prove_shards(&[*i]),
            position: *i,
        })
        .collect()
}

pub fn verify<const N: usize, F: PrimeField, H: Hasher, P>(
    shard: &fec::Shard<F>,
    commitment: &Commitment<F, H>,
    proof: &Proof<H>,
    domain_size: usize,
    nb_queries: usize,
) -> Result<(), KomodoError>
where
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    <H as rs_merkle::Hasher>::Hash: AsRef<[u8]>,
{
    commitment
        .0
        .verify::<N, _>(
            FriChallenger::<H>::default(),
            nb_queries,
            shard.k as usize,
            domain_size,
        )
        .unwrap();

    assert!(proof.path.verify(
        commitment.0.tree_root(),
        &[proof.position],
        &[H::hash_item(&shard.data)],
        domain_size,
    ));

    Ok(())
}

pub fn decode<F: PrimeField>(blocks: &[(usize, fec::Shard<F>)], n: usize) -> Vec<u8> {
    let w = F::get_root_of_unity(n as u64).unwrap();

    let t_shards = transpose(
        &blocks
            .iter()
            .map(|(_, s)| s.data.clone())
            .collect::<Vec<_>>(),
    );
    let positions = blocks
        .iter()
        .map(|(p, _)| w.pow([*p as u64]))
        .collect::<Vec<_>>();
    let source_shards = interpolate_polynomials(&t_shards, &positions)
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let mut bytes = algebra::merge_elements_into_bytes(&source_shards);
    bytes.resize(blocks[0].1.size, 0);
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
        frida::FridaBuilder,
        rng::FriChallenger,
    };
    use dragoonfri_test_utils::Fq as F_128;

    use crate::error::KomodoError;

    use super::{commit, decode, encode, evaluate, prove, verify};

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

        let builder = FridaBuilder::<F, H>::new::<N, _>(
            &evaluations,
            FriChallenger::<H>::default(),
            bf,
            rpo,
            q,
        );

        let commitment = commit(builder.clone());
        let proofs = prove::<F, H>(builder, &(0..n).collect::<Vec<_>>());

        for (shard, proof) in shards.iter().zip(proofs.iter()) {
            verify::<N, F, H, P>(shard, &commitment, proof, n, q).unwrap();
        }

        assert_eq!(
            decode::<F>(&shards.into_iter().enumerate().collect::<Vec<_>>(), n),
            bytes
        );

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
