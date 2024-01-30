use std::ops::{Add, Mul};

use ark_ec::pairing::Pairing;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{One, Zero};
use rs_merkle::algorithms::Sha256;
use rs_merkle::Hasher;

use crate::error::KomodoError;
use crate::field;
use crate::linalg::Matrix;

#[derive(Debug, Default, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Shard<E: Pairing> {
    pub k: u32,
    pub linear_combination: Vec<E::ScalarField>,
    pub hash: Vec<u8>,
    pub bytes: Vec<E::ScalarField>,
    pub size: usize,
}

impl<E: Pairing> Shard<E> {
    pub fn combine(&self, alpha: E::ScalarField, other: &Self, beta: E::ScalarField) -> Self {
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
            bytes: self
                .bytes
                .iter()
                .zip(other.bytes.iter())
                .map(|(es, eo)| es.mul(alpha).add(eo.mul(beta)))
                .collect::<Vec<_>>(),
            size: self.size,
        }
    }
}

pub(super) fn combine<E: Pairing>(
    shards: &[Shard<E>],
    coeffs: &[E::ScalarField],
) -> Option<Shard<E>> {
    if shards.len() != coeffs.len() {
        return None;
    }
    if shards.is_empty() {
        return None;
    }

    let (s, _) = shards
        .iter()
        .zip(coeffs.iter())
        .skip(1)
        .fold((shards[0].clone(), coeffs[0]), |(acc_s, acc_c), (s, c)| {
            (acc_s.combine(acc_c, s, *c), E::ScalarField::one())
        });
    Some(s)
}

pub fn encode<E: Pairing>(
    data: &[u8],
    encoding_mat: &Matrix<E::ScalarField>,
) -> Result<Vec<Shard<E>>, KomodoError> {
    let hash = Sha256::hash(data).to_vec();

    let k = encoding_mat.height;

    let source_shards = Matrix::from_vec_vec(
        field::split_data_into_field_elements::<E>(data, k)
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
            bytes: s.to_vec(),
            size: data.len(),
        })
        .collect())
}

pub fn decode<E: Pairing>(blocks: Vec<Shard<E>>) -> Result<Vec<u8>, KomodoError> {
    let k = blocks[0].k;
    let np = blocks.len();

    if np < k as usize {
        return Err(KomodoError::TooFewShards(np, k as usize));
    }

    let encoding_mat = Matrix::from_vec_vec(
        blocks
            .iter()
            .map(|b| b.linear_combination.clone())
            .collect(),
    )?
    .truncate(Some(np - k as usize), None);

    let shards = Matrix::from_vec_vec(
        blocks
            .iter()
            .take(k as usize)
            .map(|b| b.bytes.clone())
            .collect(),
    )?;

    let source_shards = encoding_mat.invert()?.mul(&shards)?.transpose().elements;

    let mut bytes = field::merge_elements_into_bytes::<E>(&source_shards);
    bytes.resize(blocks[0].size, 0);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Bls12_381;
    use ark_ec::pairing::Pairing;
    use ark_ff::PrimeField;
    use ark_std::{One, Zero};

    use crate::{
        fec::{decode, encode, Shard},
        field,
        linalg::Matrix,
    };

    use super::combine;

    fn bytes() -> Vec<u8> {
        include_bytes!("../tests/dragoon_32x32.png").to_vec()
    }

    fn to_curve<E: Pairing>(n: u128) -> E::ScalarField {
        E::ScalarField::from_le_bytes_mod_order(&n.to_le_bytes())
    }

    fn decoding_template<E: Pairing>(data: &[u8], k: usize, n: usize) {
        let test_case = format!("TEST | data: {} bytes, k: {}, n: {}", data.len(), k, n);
        assert_eq!(
            data,
            decode::<E>(encode(data, &Matrix::random(k, n)).unwrap()).unwrap(),
            "{test_case}"
        );
    }

    #[test]
    fn decoding() {
        let bytes = bytes();
        let (k, n) = (3, 5);

        let modulus_byte_size = <Bls12_381 as Pairing>::ScalarField::MODULUS_BIT_SIZE as usize / 8;
        // NOTE: starting at `modulus_byte_size * (k - 1) + 1` to include at least _k_ elements
        for b in (modulus_byte_size * (k - 1) + 1)..bytes.len() {
            decoding_template::<Bls12_381>(&bytes[..b], k, n);
        }
    }

    fn decoding_with_recoding_template<E: Pairing>(data: &[u8], k: usize, n: usize) {
        let mut shards = encode(data, &Matrix::random(k, n)).unwrap();
        shards[1] = shards[2].combine(to_curve::<E>(7), &shards[4], to_curve::<E>(6));
        shards[2] = shards[1].combine(to_curve::<E>(5), &shards[3], to_curve::<E>(4));
        assert_eq!(
            data,
            decode::<E>(shards).unwrap(),
            "TEST | data: {} bytes, k: {}, n: {}",
            data.len(),
            k,
            n
        );
    }

    #[test]
    fn decoding_with_recoding() {
        let bytes = bytes();
        let (k, n) = (3, 5);

        let modulus_byte_size = <Bls12_381 as Pairing>::ScalarField::MODULUS_BIT_SIZE as usize / 8;
        // NOTE: starting at `modulus_byte_size * (k - 1) + 1` to include at least _k_ elements
        for b in (modulus_byte_size * (k - 1) + 1)..bytes.len() {
            decoding_with_recoding_template::<Bls12_381>(&bytes[..b], k, n);
        }
    }

    fn create_fake_shard<E: Pairing>(
        linear_combination: &[E::ScalarField],
        bytes: &[u8],
    ) -> Shard<E> {
        let bytes = field::split_data_into_field_elements::<E>(bytes, 1);
        Shard {
            k: 2,
            linear_combination: linear_combination.to_vec(),
            hash: vec![],
            bytes,
            size: 0,
        }
    }

    fn recoding_template<E: Pairing>() {
        let a: Shard<E> =
            create_fake_shard(&[E::ScalarField::one(), E::ScalarField::zero()], &[1, 2, 3]);
        let b: Shard<E> =
            create_fake_shard(&[E::ScalarField::zero(), E::ScalarField::one()], &[4, 5, 6]);

        let c = a.combine(to_curve::<E>(3), &b, to_curve::<E>(5));

        assert_eq!(
            c,
            create_fake_shard(&[to_curve::<E>(3), to_curve::<E>(5),], &[23, 31, 39])
        );

        assert_eq!(
            c.combine(to_curve::<E>(2), &a, to_curve::<E>(4),),
            create_fake_shard(&[to_curve::<E>(10), to_curve::<E>(10),], &[50, 70, 90],)
        );
    }

    #[test]
    fn recoding() {
        recoding_template::<Bls12_381>();
    }

    fn combine_shards_template<E: Pairing>() {
        let a = create_fake_shard::<E>(&[to_curve::<E>(1), to_curve::<E>(0)], &[1, 4, 7]);
        let b = create_fake_shard::<E>(&[to_curve::<E>(0), to_curve::<E>(2)], &[2, 5, 8]);
        let c = create_fake_shard::<E>(&[to_curve::<E>(3), to_curve::<E>(5)], &[3, 6, 9]);

        assert!(combine::<E>(&[], &[]).is_none());
        assert!(combine::<E>(
            &[a.clone(), b.clone(), c.clone()],
            &[to_curve::<E>(1), to_curve::<E>(2)]
        )
        .is_none());
        assert_eq!(
            combine::<E>(
                &[a, b, c],
                &[to_curve::<E>(1), to_curve::<E>(2), to_curve::<E>(3)]
            ),
            Some(create_fake_shard::<E>(
                &[to_curve::<E>(10), to_curve::<E>(19)],
                &[14, 32, 50]
            ))
        );
    }

    #[test]
    fn combine_shards() {
        combine_shards_template::<Bls12_381>();
    }
}
