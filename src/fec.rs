use std::cmp::max;
use std::ops::{Add, Mul};

use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::Zero;

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
    pub fn mul(&self, alpha: E::ScalarField) -> Self {
        Self {
            k: self.k,
            linear_combination: self
                .linear_combination
                .iter()
                .map(|e| e.mul(alpha))
                .collect(),
            hash: self.hash.clone(),
            bytes: self.bytes.iter().map(|e| e.mul(alpha)).collect(),
            size: self.size,
        }
    }

    pub fn combine(&self, alpha: E::ScalarField, other: &Self, beta: E::ScalarField) -> Self {
        if alpha.is_zero() {
            return other.clone();
        } else if beta.is_zero() {
            return self.clone();
        }

        let mut linear_combination = Vec::new();
        linear_combination.resize(
            max(
                self.linear_combination.len(),
                other.linear_combination.len(),
            ),
            E::ScalarField::zero(),
        );
        for (i, l) in self.linear_combination.iter().enumerate() {
            linear_combination[i] += l.mul(alpha);
        }
        for (i, l) in other.linear_combination.iter().enumerate() {
            linear_combination[i] += l.mul(beta);
        }

        Shard {
            k: self.k,
            linear_combination,
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

pub fn decode<E: Pairing>(blocks: Vec<Shard<E>>, transpose: bool) -> Result<Vec<u8>, KomodoError> {
    let k = blocks[0].k;

    if blocks.len() < k as usize {
        return Err(KomodoError::TooFewShards(blocks.len(), k as usize));
    }

    let points: Vec<_> = blocks
        .iter()
        .take(k as usize)
        .map(|b| {
            // TODO: use the real linear combination
            let first_non_zero = b
                .linear_combination
                .iter()
                .enumerate()
                .filter(|(_, l)| !l.is_zero())
                .collect::<Vec<_>>()[0];
            E::ScalarField::from_le_bytes_mod_order(&(first_non_zero.0 as u64).to_le_bytes())
        })
        .collect();

    let shards = Matrix::from_vec_vec(
        blocks
            .iter()
            .take(k as usize)
            .map(|b| b.bytes.clone())
            .collect(),
    )?
    .transpose();

    let source_shards = shards.mul(&Matrix::vandermonde(&points, k as usize).invert()?)?;
    let source_shards = if transpose {
        source_shards.transpose().elements
    } else {
        source_shards.elements
    };

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
    use rs_merkle::algorithms::Sha256;
    use rs_merkle::Hasher;

    use crate::{
        fec::{decode, Shard},
        field,
        linalg::Matrix,
    };

    fn bytes() -> Vec<u8> {
        include_bytes!("../tests/dragoon_32x32.png").to_vec()
    }

    #[allow(clippy::expect_fun_call)]
    fn decoding_template<E: Pairing>(data: &[u8], k: usize, n: usize) {
        let hash = Sha256::hash(data).to_vec();

        let points: Vec<E::ScalarField> = (0..n)
            .map(|i| E::ScalarField::from_le_bytes_mod_order(&[i as u8]))
            .collect();
        let encoding = Matrix::vandermonde(&points, k);

        let source_shards = Matrix::from_vec_vec(
            field::split_data_into_field_elements::<E>(data, k)
                .chunks(k)
                .map(|c| c.to_vec())
                .collect(),
        )
        .expect(&format!(
            "could not build source shard matrix ({} bytes)",
            data.len()
        ));

        let shards = source_shards
            .mul(&encoding)
            .expect(&format!("could not encode shards ({} bytes)", data.len()))
            .transpose()
            .elements
            .chunks(source_shards.height)
            .enumerate()
            .map(|(i, s)| {
                let mut linear_combination = Vec::new();
                linear_combination.resize(i + 1, E::ScalarField::zero());
                linear_combination[i] = E::ScalarField::one();

                Shard {
                    k: k as u32,
                    linear_combination,
                    hash: hash.clone(),
                    bytes: s.to_vec(),
                    size: data.len(),
                }
            })
            .collect();

        assert_eq!(
            data,
            decode::<E>(shards, false)
                .expect(&format!("could not decode shards ({} bytes)", data.len()))
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

    fn create_fake_shard<E: Pairing>(
        linear_combination: &[E::ScalarField],
        bytes: &[u8],
    ) -> Shard<E> {
        let bytes = field::split_data_into_field_elements::<E>(bytes, 1);
        Shard {
            k: 0,
            linear_combination: linear_combination.to_vec(),
            hash: vec![],
            bytes,
            size: 0,
        }
    }

    fn recoding_template<E: Pairing>() {
        let a: Shard<E> = create_fake_shard(&[E::ScalarField::one()], &[1, 2, 3]);
        let b: Shard<E> =
            create_fake_shard(&[E::ScalarField::zero(), E::ScalarField::one()], &[4, 5, 6]);

        assert_eq!(
            a.mul(E::ScalarField::from_le_bytes_mod_order(&[2])),
            create_fake_shard(&[E::ScalarField::from_le_bytes_mod_order(&[2])], &[2, 4, 6],)
        );

        let c = a.combine(
            E::ScalarField::from_le_bytes_mod_order(&[3]),
            &b,
            E::ScalarField::from_le_bytes_mod_order(&[5]),
        );

        assert_eq!(
            c,
            create_fake_shard(
                &[
                    E::ScalarField::from_le_bytes_mod_order(&[3]),
                    E::ScalarField::from_le_bytes_mod_order(&[5]),
                ],
                &[23, 31, 39]
            )
        );

        assert_eq!(
            c.combine(
                E::ScalarField::from_le_bytes_mod_order(&[2]),
                &a,
                E::ScalarField::from_le_bytes_mod_order(&[4]),
            ),
            create_fake_shard(
                &[
                    E::ScalarField::from_le_bytes_mod_order(&[10]),
                    E::ScalarField::from_le_bytes_mod_order(&[10]),
                ],
                &[50, 70, 90],
            )
        );
    }

    #[test]
    fn recoding() {
        recoding_template::<Bls12_381>();
    }
}
