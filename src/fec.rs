//! a module to encode, recode and decode shards of data with FEC methods

use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::RngCore;

use rs_merkle::{algorithms::Sha256, Hasher};

use crate::{error::KomodoError, field, linalg::Matrix};

/// representation of a FEC shard of data
///
/// - `k` is the code parameter, required to decode
/// - the _linear combination_ tells the decoded how the shard was constructed,
///   with respect to the original source shards => this effectively allows
///   support for _recoding_
/// - the hash and the size represent the original data
#[derive(Debug, Default, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Shard<F: PrimeField> {
    pub k: u32,
    pub linear_combination: Vec<F>,
    pub hash: Vec<u8>,
    pub data: Vec<F>,
    pub size: usize,
}

impl<F: PrimeField> Shard<F> {
    /// compute the linear combination between two [`Shard`]s
    pub fn recode_with(&self, alpha: F, other: &Self, beta: F) -> Self {
        if alpha.is_zero() {
            return other.clone();
        } else if beta.is_zero() {
            return self.clone();
        }

        Shard {
            k: self.k,
            linear_combination: self
                .linear_combination
                .iter()
                .zip(other.linear_combination.iter())
                .map(|(l, r)| l.mul(alpha) + r.mul(beta))
                .collect(),
            hash: self.hash.clone(),
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(es, eo)| es.mul(alpha) + eo.mul(beta))
                .collect::<Vec<_>>(),
            size: self.size,
        }
    }
}

/// compute the linear combination between an arbitrary number of [`Shard`]s
///
/// > **Note**
/// > this is basically a multi-[`Shard`] wrapper around [`Shard::recode_with`]
/// >
/// > returns [`None`] if number of shards is not the same as the number of
/// > coefficients or if no shards are provided.
pub fn recode_with_coeffs<F: PrimeField>(shards: &[Shard<F>], coeffs: &[F]) -> Option<Shard<F>> {
    if shards.len() != coeffs.len() {
        return None;
    }
    if shards.is_empty() {
        return None;
    }

    let (s, _) = shards
        .iter()
        .zip(coeffs)
        .skip(1)
        .fold((shards[0].clone(), coeffs[0]), |(acc_s, acc_c), (s, c)| {
            (acc_s.recode_with(acc_c, s, *c), F::one())
        });
    Some(s)
}

/// compute a recoded shard from an arbitrary set of shards
///
/// coefficients will be drawn at random, one for each shard.
///
/// if the shards appear to come from different data, e.g. if `k` is not the
/// same or the hash of the data is different, an error will be returned.
///
/// > **Note**
/// > this is a wrapper around [`recode_with_coeffs`].
pub fn recode_random<F: PrimeField>(
    shards: &[Shard<F>],
    rng: &mut impl RngCore,
) -> Result<Option<Shard<F>>, KomodoError> {
    for (i, (s1, s2)) in shards.iter().zip(shards.iter().skip(1)).enumerate() {
        if s1.k != s2.k {
            return Err(KomodoError::IncompatibleShards(format!(
                "k is not the same at {}: {} vs {}",
                i, s1.k, s2.k
            )));
        }
        if s1.hash != s2.hash {
            return Err(KomodoError::IncompatibleShards(format!(
                "hash is not the same at {}: {:?} vs {:?}",
                i, s1.hash, s2.hash
            )));
        }
        if s1.size != s2.size {
            return Err(KomodoError::IncompatibleShards(format!(
                "size is not the same at {}: {} vs {}",
                i, s1.size, s2.size
            )));
        }
    }

    let coeffs = shards.iter().map(|_| F::rand(rng)).collect::<Vec<_>>();
    Ok(recode_with_coeffs(shards, &coeffs))
}

/// applies a given encoding matrix to some data to generate encoded shards
///
/// > **Note**
/// > the input data and the encoding matrix should have compatible shapes,
/// > otherwise, an error might be thrown to the caller.
pub fn encode<F: PrimeField>(
    data: &[u8],
    encoding_mat: &Matrix<F>,
) -> Result<Vec<Shard<F>>, KomodoError> {
    let hash = Sha256::hash(data).to_vec();

    let k = encoding_mat.height;

    let source_shards = Matrix::from_vec_vec(
        field::split_data_into_field_elements(data, k)
            .chunks(k)
            .map(|c| c.to_vec())
            .collect(),
    )?;

    Ok(source_shards
        .mul(encoding_mat)?
        .transpose()
        .elements
        .chunks(source_shards.height)
        .enumerate()
        .map(|(j, s)| Shard {
            k: k as u32,
            linear_combination: encoding_mat.get_col(j).unwrap(),
            hash: hash.clone(),
            data: s.to_vec(),
            size: data.len(),
        })
        .collect())
}

/// reconstruct the original data from a set of encoded, possibly recoded,
/// shards
///
/// > **Note**
/// > this function might fail in a variety of cases
/// > - if there are too few shards
/// > - if there are linear dependencies between shards
pub fn decode<F: PrimeField>(shards: Vec<Shard<F>>) -> Result<Vec<u8>, KomodoError> {
    if shards.is_empty() {
        return Err(KomodoError::TooFewShards(0, 0));
    }

    let k = shards[0].k;
    let np = shards.len();

    if np < k as usize {
        return Err(KomodoError::TooFewShards(np, k as usize));
    }

    let encoding_mat = Matrix::from_vec_vec(
        shards
            .iter()
            .map(|b| b.linear_combination.clone())
            .collect(),
    )?
    .truncate(Some(np - k as usize), None);

    let shard_mat = Matrix::from_vec_vec(
        shards
            .iter()
            .take(k as usize)
            .map(|b| b.data.clone())
            .collect(),
    )?;

    let source_shards = encoding_mat.invert()?.mul(&shard_mat)?.transpose().elements;

    let mut bytes = field::merge_elements_into_bytes(&source_shards);
    bytes.resize(shards[0].size, 0);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Fr;
    use ark_ff::PrimeField;

    use crate::{
        fec::{decode, encode, recode_random, Shard},
        field,
        linalg::Matrix,
    };

    use itertools::Itertools;

    use super::recode_with_coeffs;

    fn bytes() -> Vec<u8> {
        include_bytes!("../tests/dragoon_32x32.png").to_vec()
    }

    fn to_curve<F: PrimeField>(n: u128) -> F {
        F::from_le_bytes_mod_order(&n.to_le_bytes())
    }

    /// `contains_one_of(x, set)` is true iif `x` fully contains one of the lists from `set`
    ///
    /// > **Note**  
    /// > see [`containment`] for some example
    fn contains_one_of(x: &[usize], set: &[Vec<usize>]) -> bool {
        set.iter().any(|y| y.iter().all(|z| x.contains(z)))
    }

    #[test]
    fn containment() {
        assert!(contains_one_of(&[1, 2, 3], &[vec![1, 2, 3]]));
        assert!(contains_one_of(
            &[3, 6, 8],
            &[vec![2, 4, 6], vec![1, 3, 7], vec![6, 7, 8], vec![3, 6, 8]],
        ));
        assert!(!contains_one_of(
            &[1, 6, 8],
            &[vec![2, 4, 6], vec![1, 3, 7], vec![6, 7, 8], vec![3, 6, 8]],
        ));
        assert!(contains_one_of(
            &[3, 6, 8, 9, 10],
            &[vec![2, 4, 6], vec![1, 3, 7], vec![6, 7, 8], vec![3, 6, 8]],
        ));
    }

    fn try_all_decoding_combinations<F: PrimeField>(
        data: &[u8],
        shards: &[Shard<F>],
        k: usize,
        test_case: &str,
        limit: Option<usize>,
        should_not_be_decodable: Vec<Vec<usize>>,
    ) {
        for c in shards
            .iter()
            .cloned()
            .enumerate()
            .combinations(k)
            .take(limit.unwrap_or(usize::MAX))
        {
            let s = c.iter().map(|(_, s)| s).cloned().collect();
            let is: Vec<usize> = c.iter().map(|(i, _)| i).cloned().collect();

            let actual = decode::<F>(s);

            if contains_one_of(&is, &should_not_be_decodable) {
                assert!(
                    actual.is_err(),
                    "should not decode with {:?} {test_case}",
                    is
                );
                continue;
            }

            assert!(actual.is_ok(), "could not decode with {:?} {test_case}", is);

            assert_eq!(
                data,
                actual.unwrap(),
                "bad decoded data with {:?} {test_case}",
                is,
            );
        }
    }

    fn end_to_end_template<F: PrimeField>(data: &[u8], k: usize, n: usize) {
        let mut rng = ark_std::test_rng();
        let test_case = format!("TEST | data: {} bytes, k: {}, n: {}", data.len(), k, n);

        let shards = encode::<F>(data, &Matrix::random(k, n, &mut rng))
            .unwrap_or_else(|_| panic!("could not encode {test_case}"));

        try_all_decoding_combinations(data, &shards, k, &test_case, None, vec![]);
    }

    fn end_to_end_with_recoding_template<F: PrimeField>(data: &[u8], k: usize, n: usize) {
        assert!(n >= 5, "n should be at least 5, found {}", n);

        let mut rng = ark_std::test_rng();
        let test_case = format!("TEST | data: {} bytes, k: {}, n: {}", data.len(), k, n);

        let mut shards = encode::<F>(data, &Matrix::random(k, n, &mut rng))
            .unwrap_or_else(|_| panic!("could not encode {test_case}"));

        let recoding_steps = [
            vec![2, 4],       // =  n
            vec![1, 3],       // = (n + 1)
            vec![n, (n + 1)], // = (n + 2) = ((2, 4), (1, 3))
            vec![0],          // = (n + 3) = (0)
            vec![(n + 3)],    // = (n + 4) = (0)
        ];
        let should_not_be_decodable = vec![
            vec![2, 4, n],
            vec![1, 3, (n + 1)],
            vec![n, (n + 1), (n + 2)],
            vec![1, 3, n, (n + 2)],
            vec![2, 4, (n + 1), (n + 2)],
            vec![1, 2, 3, 4, (n + 2)],
            vec![0, (n + 3)],
            vec![0, (n + 4)],
            vec![(n + 3), (n + 4)],
        ];

        for step in recoding_steps {
            let shards_to_recode: Vec<_> = shards
                .iter()
                .cloned()
                .enumerate()
                .filter_map(|(i, s)| if step.contains(&i) { Some(s) } else { None })
                .collect();
            shards.push(recode_random(&shards_to_recode, &mut rng).unwrap().unwrap());
        }

        try_all_decoding_combinations(data, &shards, k, &test_case, None, should_not_be_decodable);
    }

    #[test]
    fn end_to_end() {
        let bytes = bytes();

        for k in [3, 5] {
            for rho in [0.5, 0.33] {
                let n = (k as f64 / rho) as usize;
                end_to_end_template::<Fr>(&bytes, k, n);
            }
        }
    }

    #[test]
    fn end_to_end_with_recoding() {
        let bytes = bytes();

        for k in [3, 5] {
            for rho in [0.50, 0.33] {
                let n = (k as f64 / rho) as usize;
                end_to_end_with_recoding_template::<Fr>(&bytes, k, n);
            }
        }
    }

    fn create_fake_shard<F: PrimeField>(linear_combination: &[F], bytes: &[u8]) -> Shard<F> {
        Shard {
            k: 2,
            linear_combination: linear_combination.to_vec(),
            hash: vec![],
            data: field::split_data_into_field_elements(bytes, 1),
            size: 0,
        }
    }

    fn recoding_template<F: PrimeField>() {
        let a: Shard<F> = create_fake_shard(&[F::one(), F::zero()], &[1, 2, 3]);
        let b: Shard<F> = create_fake_shard(&[F::zero(), F::one()], &[4, 5, 6]);

        let c = a.recode_with(to_curve(3), &b, to_curve(5));

        assert_eq!(
            c,
            create_fake_shard(&[to_curve(3), to_curve(5),], &[23, 31, 39])
        );

        assert_eq!(
            c.recode_with(to_curve(2), &a, to_curve(4),),
            create_fake_shard(&[to_curve(10), to_curve(10),], &[50, 70, 90],)
        );
    }

    #[test]
    fn recoding() {
        recoding_template::<Fr>();
    }

    fn combine_shards_template<F: PrimeField>() {
        let a = create_fake_shard::<F>(&[to_curve(1), to_curve(0)], &[1, 4, 7]);
        let b = create_fake_shard::<F>(&[to_curve(0), to_curve(2)], &[2, 5, 8]);
        let c = create_fake_shard::<F>(&[to_curve(3), to_curve(5)], &[3, 6, 9]);

        assert!(recode_with_coeffs::<F>(&[], &[]).is_none());
        assert!(recode_with_coeffs(
            &[a.clone(), b.clone(), c.clone()],
            &[to_curve(1), to_curve(2)]
        )
        .is_none());
        assert_eq!(
            recode_with_coeffs(&[a, b, c], &[to_curve(1), to_curve(2), to_curve(3)]),
            Some(create_fake_shard(
                &[to_curve(10), to_curve(19)],
                &[14, 32, 50]
            ))
        );
    }

    #[test]
    fn combine_shards() {
        combine_shards_template::<Fr>();
    }
}
