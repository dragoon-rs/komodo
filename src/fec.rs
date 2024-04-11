//! a module to encode, recode and decode shards of data with FEC methods

use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

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
    pub fn combine(&self, alpha: F, other: &Self, beta: F) -> Self {
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
/// > this is basically a multi-[`Shard`] wrapper around [`Shard::combine`]
/// >
/// > returns [`None`] if number of shards is not the same as the number of
/// > coefficients or if no shards are provided.
pub fn combine<F: PrimeField>(shards: &[Shard<F>], coeffs: &[F]) -> Option<Shard<F>> {
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
            (acc_s.combine(acc_c, s, *c), F::one())
        });
    Some(s)
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
        field::split_data_into_field_elements::<F>(data, k)
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

    let mut bytes = field::merge_elements_into_bytes::<F>(&source_shards);
    bytes.resize(shards[0].size, 0);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Fr;
    use ark_ff::PrimeField;

    use crate::{
        fec::{decode, encode, Shard},
        field,
        linalg::Matrix,
    };

    use super::combine;

    fn bytes() -> Vec<u8> {
        include_bytes!("../tests/dragoon_32x32.png").to_vec()
    }

    fn to_curve<F: PrimeField>(n: u128) -> F {
        F::from_le_bytes_mod_order(&n.to_le_bytes())
    }

    fn end_to_end_template<F: PrimeField>(data: &[u8], k: usize, n: usize) {
        let mut rng = ark_std::test_rng();

        let test_case = format!("TEST | data: {} bytes, k: {}, n: {}", data.len(), k, n);
        assert_eq!(
            data,
            decode::<F>(encode(data, &Matrix::random(k, n, &mut rng)).unwrap()).unwrap(),
            "{test_case}"
        );
    }

    /// k should be at least 5
    fn end_to_end_with_recoding_template<F: PrimeField>(data: &[u8], k: usize, n: usize) {
        let mut rng = ark_std::test_rng();

        let mut shards = encode(data, &Matrix::random(k, n, &mut rng)).unwrap();
        shards[1] = shards[2].combine(to_curve::<F>(7), &shards[4], to_curve::<F>(6));
        shards[2] = shards[1].combine(to_curve::<F>(5), &shards[3], to_curve::<F>(4));
        assert_eq!(
            data,
            decode::<F>(shards).unwrap(),
            "TEST | data: {} bytes, k: {}, n: {}",
            data.len(),
            k,
            n
        );
    }

    // NOTE: this is part of an experiment, to be honest, to be able to see how
    // much these tests could be refactored and simplified
    fn run_template<F, Fun>(test: Fun)
    where
        F: PrimeField,
        Fun: Fn(&[u8], usize, usize),
    {
        let bytes = bytes();
        let (k, n) = (3, 5);

        let modulus_byte_size = F::MODULUS_BIT_SIZE as usize / 8;
        // NOTE: starting at `modulus_byte_size * (k - 1) + 1` to include at least _k_ elements
        for b in (modulus_byte_size * (k - 1) + 1)..bytes.len() {
            test(&bytes[..b], k, n);
        }
    }

    #[test]
    fn end_to_end() {
        run_template::<Fr, _>(end_to_end_template::<Fr>);
    }

    #[test]
    fn end_to_end_with_recoding() {
        run_template::<Fr, _>(end_to_end_with_recoding_template::<Fr>);
    }

    fn create_fake_shard<F: PrimeField>(linear_combination: &[F], bytes: &[u8]) -> Shard<F> {
        Shard {
            k: 2,
            linear_combination: linear_combination.to_vec(),
            hash: vec![],
            data: field::split_data_into_field_elements::<F>(bytes, 1),
            size: 0,
        }
    }

    fn recoding_template<F: PrimeField>() {
        let a: Shard<F> = create_fake_shard(&[F::one(), F::zero()], &[1, 2, 3]);
        let b: Shard<F> = create_fake_shard(&[F::zero(), F::one()], &[4, 5, 6]);

        let c = a.combine(to_curve::<F>(3), &b, to_curve::<F>(5));

        assert_eq!(
            c,
            create_fake_shard(&[to_curve::<F>(3), to_curve::<F>(5),], &[23, 31, 39])
        );

        assert_eq!(
            c.combine(to_curve::<F>(2), &a, to_curve::<F>(4),),
            create_fake_shard(&[to_curve::<F>(10), to_curve::<F>(10),], &[50, 70, 90],)
        );
    }

    #[test]
    fn recoding() {
        recoding_template::<Fr>();
    }

    fn combine_shards_template<F: PrimeField>() {
        let a = create_fake_shard::<F>(&[to_curve::<F>(1), to_curve::<F>(0)], &[1, 4, 7]);
        let b = create_fake_shard::<F>(&[to_curve::<F>(0), to_curve::<F>(2)], &[2, 5, 8]);
        let c = create_fake_shard::<F>(&[to_curve::<F>(3), to_curve::<F>(5)], &[3, 6, 9]);

        assert!(combine::<F>(&[], &[]).is_none());
        assert!(combine::<F>(
            &[a.clone(), b.clone(), c.clone()],
            &[to_curve::<F>(1), to_curve::<F>(2)]
        )
        .is_none());
        assert_eq!(
            combine::<F>(
                &[a, b, c],
                &[to_curve::<F>(1), to_curve::<F>(2), to_curve::<F>(3)]
            ),
            Some(create_fake_shard::<F>(
                &[to_curve::<F>(10), to_curve::<F>(19)],
                &[14, 32, 50]
            ))
        );
    }

    #[test]
    fn combine_shards() {
        combine_shards_template::<Fr>();
    }
}
