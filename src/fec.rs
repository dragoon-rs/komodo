use std::ops::{Add, Mul};

use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{One, Zero};

use crate::field;
use crate::linalg::{LinalgError, Matrix};

#[derive(Debug, Default, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct LinearCombinationElement<E: Pairing> {
    pub index: u32,
    pub weight: E::ScalarField,
}

#[derive(Debug, Default, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Shard<E: Pairing> {
    pub k: u32,
    pub linear_combination: Vec<LinearCombinationElement<E>>,
    pub hash: Vec<u8>,
    pub bytes: Vec<u8>,
    pub size: usize,
}

impl<E: Pairing> Shard<E> {
    pub fn mul(&self, alpha: E::ScalarField) -> Self {
        let bytes = if alpha.is_zero() {
            vec![0u8; self.bytes.len()]
        } else if alpha.is_one() {
            self.bytes.to_vec()
        } else {
            let elements = field::split_data_into_field_elements::<E>(&self.bytes, 1, true)
                .iter()
                .map(|e| e.mul(alpha))
                .collect::<Vec<_>>();

            field::merge_elements_into_bytes::<E>(&elements, false)
        };

        Shard {
            k: self.k,
            linear_combination: self
                .linear_combination
                .iter()
                .map(|l| LinearCombinationElement {
                    index: l.index,
                    weight: l.weight.mul(alpha),
                })
                .collect(),
            hash: self.hash.clone(),
            bytes,
            size: self.size,
        }
    }

    pub fn combine(&self, alpha: E::ScalarField, other: &Self, beta: E::ScalarField) -> Self {
        if alpha.is_zero() {
            return other.clone();
        } else if beta.is_zero() {
            return self.clone();
        }

        let elements = {
            let elements_self = field::split_data_into_field_elements::<E>(&self.bytes, 1, true);
            let elements_other = field::split_data_into_field_elements::<E>(&other.bytes, 1, true);

            elements_self
                .iter()
                .zip(elements_other.iter())
                .map(|(es, eo)| es.mul(alpha).add(eo.mul(beta)))
                .collect::<Vec<_>>()
        };

        let mut linear_combination = vec![];
        for lce in &self.linear_combination {
            linear_combination.push(LinearCombinationElement {
                index: lce.index,
                weight: lce.weight.mul(alpha),
            });
        }
        for lce in &other.linear_combination {
            linear_combination.push(LinearCombinationElement {
                index: lce.index,
                weight: lce.weight.mul(beta),
            });
        }

        Shard {
            k: self.k,
            linear_combination,
            hash: self.hash.clone(),
            bytes: field::merge_elements_into_bytes::<E>(&elements, false),
            size: self.size,
        }
    }
}

pub fn decode<E: Pairing>(blocks: Vec<Shard<E>>) -> Result<Vec<u8>, LinalgError> {
    let k = blocks[0].k;

    if blocks.len() < k as usize {
        return Err(LinalgError::Other("too few shards".to_string()));
    }

    let points: Vec<_> = blocks
        .iter()
        .take(k as usize)
        .map(|b| {
            E::ScalarField::from_le_bytes_mod_order(
                // TODO: use the real linear combination
                &(b.linear_combination[0].index as u64).to_le_bytes(),
            )
        })
        .collect();

    let shards = Matrix::from_vec_vec(
        blocks
            .iter()
            .take(k as usize)
            .map(|b| field::split_data_into_field_elements::<E>(&b.bytes, 1, true))
            .collect(),
    )?
    .transpose();

    let source_shards = shards
        .mul(&Matrix::vandermonde(&points, k as usize).invert()?)?
        .transpose()
        .elements;

    let mut bytes = field::merge_elements_into_bytes::<E>(&source_shards, true);
    bytes.resize(blocks[0].size, 0);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Bls12_381;
    use ark_ec::pairing::Pairing;
    use ark_ff::PrimeField;
    use ark_std::One;
    use rs_merkle::algorithms::Sha256;
    use rs_merkle::Hasher;

    use crate::{
        fec::{decode, LinearCombinationElement, Shard},
        field,
        linalg::Matrix,
    };

    const BYTES: [u8; 2810] = [
        99, 111, 110, 115, 116, 32, 75, 79, 77, 79, 68, 79, 95, 66, 73, 78, 65, 82, 89, 32, 61, 32,
        34, 46, 47, 116, 97, 114, 103, 101, 116, 47, 114, 101, 108, 101, 97, 115, 101, 47, 107,
        111, 109, 111, 100, 111, 34, 10, 10, 100, 101, 102, 32, 34, 110, 117, 45, 99, 111, 109,
        112, 108, 101, 116, 101, 32, 108, 111, 103, 45, 108, 101, 118, 101, 108, 115, 34, 32, 91,
        93, 58, 32, 110, 111, 116, 104, 105, 110, 103, 32, 45, 62, 32, 108, 105, 115, 116, 60, 115,
        116, 114, 105, 110, 103, 62, 32, 123, 10, 32, 32, 32, 32, 91, 10, 32, 32, 32, 32, 32, 32,
        32, 32, 34, 84, 82, 65, 67, 69, 34, 10, 32, 32, 32, 32, 32, 32, 32, 32, 34, 68, 69, 66, 85,
        71, 34, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 34, 73, 78, 70, 79, 34, 44, 10, 32, 32, 32,
        32, 32, 32, 32, 32, 34, 87, 65, 82, 78, 34, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 34, 69,
        82, 82, 79, 82, 34, 44, 10, 32, 32, 32, 32, 93, 10, 125, 10, 10, 100, 101, 102, 32, 114,
        117, 110, 45, 107, 111, 109, 111, 100, 111, 32, 91, 10, 32, 32, 32, 32, 97, 114, 103, 115,
        58, 32, 114, 101, 99, 111, 114, 100, 60, 98, 121, 116, 101, 115, 58, 32, 115, 116, 114,
        105, 110, 103, 44, 32, 107, 58, 32, 105, 110, 116, 44, 32, 110, 58, 32, 105, 110, 116, 44,
        32, 100, 111, 95, 103, 101, 110, 101, 114, 97, 116, 101, 95, 112, 111, 119, 101, 114, 115,
        58, 32, 98, 111, 111, 108, 44, 32, 112, 111, 119, 101, 114, 115, 95, 102, 105, 108, 101,
        58, 32, 112, 97, 116, 104, 44, 32, 100, 111, 95, 114, 101, 99, 111, 110, 115, 116, 114,
        117, 99, 116, 95, 100, 97, 116, 97, 58, 32, 98, 111, 111, 108, 44, 32, 100, 111, 95, 118,
        101, 114, 105, 102, 121, 95, 98, 108, 111, 99, 107, 115, 58, 32, 98, 111, 111, 108, 44, 32,
        98, 108, 111, 99, 107, 95, 102, 105, 108, 101, 115, 58, 32, 108, 105, 115, 116, 60, 115,
        116, 114, 105, 110, 103, 62, 62, 44, 10, 32, 32, 32, 32, 45, 45, 108, 111, 103, 45, 108,
        101, 118, 101, 108, 58, 32, 115, 116, 114, 105, 110, 103, 44, 10, 93, 58, 32, 110, 111,
        116, 104, 105, 110, 103, 32, 45, 62, 32, 97, 110, 121, 32, 123, 10, 32, 32, 32, 32, 119,
        105, 116, 104, 45, 101, 110, 118, 32, 123, 82, 85, 83, 84, 95, 76, 79, 71, 58, 32, 36, 108,
        111, 103, 95, 108, 101, 118, 101, 108, 125, 32, 123, 10, 32, 32, 32, 32, 32, 32, 32, 32,
        108, 101, 116, 32, 114, 101, 115, 32, 61, 32, 100, 111, 32, 123, 10, 32, 32, 32, 32, 32,
        32, 32, 32, 32, 32, 32, 32, 94, 36, 75, 79, 77, 79, 68, 79, 95, 66, 73, 78, 65, 82, 89, 32,
        40, 91, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 36, 97, 114,
        103, 115, 46, 98, 121, 116, 101, 115, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
        32, 32, 32, 32, 36, 97, 114, 103, 115, 46, 107, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
        32, 32, 32, 32, 32, 32, 36, 97, 114, 103, 115, 46, 110, 10, 32, 32, 32, 32, 32, 32, 32, 32,
        32, 32, 32, 32, 32, 32, 32, 32, 40, 36, 97, 114, 103, 115, 46, 100, 111, 95, 103, 101, 110,
        101, 114, 97, 116, 101, 95, 112, 111, 119, 101, 114, 115, 32, 124, 32, 105, 110, 116, 111,
        32, 115, 116, 114, 105, 110, 103, 41, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
        32, 32, 32, 32, 36, 97, 114, 103, 115, 46, 112, 111, 119, 101, 114, 115, 95, 102, 105, 108,
        101, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 40, 36, 97, 114,
        103, 115, 46, 100, 111, 95, 114, 101, 99, 111, 110, 115, 116, 114, 117, 99, 116, 95, 100,
        97, 116, 97, 32, 124, 32, 105, 110, 116, 111, 32, 115, 116, 114, 105, 110, 103, 41, 10, 32,
        32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 40, 36, 97, 114, 103, 115, 46,
        100, 111, 95, 118, 101, 114, 105, 102, 121, 95, 98, 108, 111, 99, 107, 115, 32, 124, 32,
        105, 110, 116, 111, 32, 115, 116, 114, 105, 110, 103, 41, 10, 32, 32, 32, 32, 32, 32, 32,
        32, 32, 32, 32, 32, 93, 32, 124, 32, 97, 112, 112, 101, 110, 100, 32, 36, 97, 114, 103,
        115, 46, 98, 108, 111, 99, 107, 95, 102, 105, 108, 101, 115, 41, 10, 32, 32, 32, 32, 32,
        32, 32, 32, 125, 32, 124, 32, 99, 111, 109, 112, 108, 101, 116, 101, 10, 10, 32, 32, 32,
        32, 32, 32, 32, 32, 112, 114, 105, 110, 116, 32, 36, 114, 101, 115, 46, 115, 116, 100, 111,
        117, 116, 10, 32, 32, 32, 32, 32, 32, 32, 32, 36, 114, 101, 115, 46, 115, 116, 100, 101,
        114, 114, 32, 124, 32, 102, 114, 111, 109, 32, 106, 115, 111, 110, 10, 32, 32, 32, 32, 125,
        10, 125, 10, 10, 101, 120, 112, 111, 114, 116, 32, 100, 101, 102, 32, 34, 107, 111, 109,
        111, 100, 111, 32, 98, 117, 105, 108, 100, 34, 32, 91, 93, 32, 123, 10, 32, 32, 32, 32, 94,
        99, 97, 114, 103, 111, 32, 98, 117, 105, 108, 100, 32, 45, 45, 112, 97, 99, 107, 97, 103,
        101, 32, 107, 111, 109, 111, 100, 111, 32, 45, 45, 114, 101, 108, 101, 97, 115, 101, 10,
        125, 10, 10, 101, 120, 112, 111, 114, 116, 32, 100, 101, 102, 32, 34, 107, 111, 109, 111,
        100, 111, 32, 115, 101, 116, 117, 112, 34, 32, 91, 10, 32, 32, 32, 32, 98, 121, 116, 101,
        115, 58, 32, 115, 116, 114, 105, 110, 103, 44, 10, 32, 32, 32, 32, 45, 45, 112, 111, 119,
        101, 114, 115, 45, 102, 105, 108, 101, 58, 32, 112, 97, 116, 104, 32, 61, 32, 34, 112, 111,
        119, 101, 114, 115, 46, 98, 105, 110, 34, 44, 10, 32, 32, 32, 32, 45, 45, 108, 111, 103,
        45, 108, 101, 118, 101, 108, 58, 32, 115, 116, 114, 105, 110, 103, 64, 34, 110, 117, 45,
        99, 111, 109, 112, 108, 101, 116, 101, 32, 108, 111, 103, 45, 108, 101, 118, 101, 108, 115,
        34, 32, 61, 32, 34, 73, 78, 70, 79, 34, 10, 93, 58, 32, 110, 111, 116, 104, 105, 110, 103,
        32, 45, 62, 32, 110, 111, 116, 104, 105, 110, 103, 32, 123, 10, 32, 32, 32, 32, 114, 117,
        110, 45, 107, 111, 109, 111, 100, 111, 32, 45, 45, 108, 111, 103, 45, 108, 101, 118, 101,
        108, 32, 36, 108, 111, 103, 95, 108, 101, 118, 101, 108, 32, 123, 10, 32, 32, 32, 32, 32,
        32, 32, 32, 98, 121, 116, 101, 115, 58, 32, 36, 98, 121, 116, 101, 115, 44, 10, 32, 32, 32,
        32, 32, 32, 32, 32, 107, 58, 32, 48, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 110, 58, 32,
        48, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 100, 111, 95, 103, 101, 110, 101, 114, 97, 116,
        101, 95, 112, 111, 119, 101, 114, 115, 58, 32, 116, 114, 117, 101, 44, 10, 32, 32, 32, 32,
        32, 32, 32, 32, 112, 111, 119, 101, 114, 115, 95, 102, 105, 108, 101, 58, 32, 36, 112, 111,
        119, 101, 114, 115, 95, 102, 105, 108, 101, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 100,
        111, 95, 114, 101, 99, 111, 110, 115, 116, 114, 117, 99, 116, 95, 100, 97, 116, 97, 58, 32,
        102, 97, 108, 115, 101, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 100, 111, 95, 118, 101,
        114, 105, 102, 121, 95, 98, 108, 111, 99, 107, 115, 58, 32, 102, 97, 108, 115, 101, 44, 10,
        32, 32, 32, 32, 32, 32, 32, 32, 98, 108, 111, 99, 107, 95, 102, 105, 108, 101, 115, 58, 32,
        91, 93, 44, 10, 32, 32, 32, 32, 125, 10, 125, 10, 10, 101, 120, 112, 111, 114, 116, 32,
        100, 101, 102, 32, 34, 107, 111, 109, 111, 100, 111, 32, 112, 114, 111, 118, 101, 34, 32,
        91, 10, 32, 32, 32, 32, 98, 121, 116, 101, 115, 58, 32, 115, 116, 114, 105, 110, 103, 44,
        10, 32, 32, 32, 32, 45, 45, 102, 101, 99, 45, 112, 97, 114, 97, 109, 115, 58, 32, 114, 101,
        99, 111, 114, 100, 60, 107, 58, 32, 105, 110, 116, 44, 32, 110, 58, 32, 105, 110, 116, 62,
        44, 10, 32, 32, 32, 32, 45, 45, 112, 111, 119, 101, 114, 115, 45, 102, 105, 108, 101, 58,
        32, 112, 97, 116, 104, 32, 61, 32, 34, 112, 111, 119, 101, 114, 115, 46, 98, 105, 110, 34,
        44, 10, 32, 32, 32, 32, 45, 45, 108, 111, 103, 45, 108, 101, 118, 101, 108, 58, 32, 115,
        116, 114, 105, 110, 103, 64, 34, 110, 117, 45, 99, 111, 109, 112, 108, 101, 116, 101, 32,
        108, 111, 103, 45, 108, 101, 118, 101, 108, 115, 34, 32, 61, 32, 34, 73, 78, 70, 79, 34,
        10, 93, 58, 32, 110, 111, 116, 104, 105, 110, 103, 32, 45, 62, 32, 108, 105, 115, 116, 60,
        115, 116, 114, 105, 110, 103, 62, 32, 123, 10, 32, 32, 32, 32, 114, 117, 110, 45, 107, 111,
        109, 111, 100, 111, 32, 45, 45, 108, 111, 103, 45, 108, 101, 118, 101, 108, 32, 36, 108,
        111, 103, 95, 108, 101, 118, 101, 108, 32, 123, 10, 32, 32, 32, 32, 32, 32, 32, 32, 98,
        121, 116, 101, 115, 58, 32, 36, 98, 121, 116, 101, 115, 44, 10, 32, 32, 32, 32, 32, 32, 32,
        32, 107, 58, 32, 36, 102, 101, 99, 95, 112, 97, 114, 97, 109, 115, 46, 107, 44, 10, 32, 32,
        32, 32, 32, 32, 32, 32, 110, 58, 32, 36, 102, 101, 99, 95, 112, 97, 114, 97, 109, 115, 46,
        110, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 100, 111, 95, 103, 101, 110, 101, 114, 97,
        116, 101, 95, 112, 111, 119, 101, 114, 115, 58, 32, 102, 97, 108, 115, 101, 44, 10, 32, 32,
        32, 32, 32, 32, 32, 32, 112, 111, 119, 101, 114, 115, 95, 102, 105, 108, 101, 58, 32, 36,
        112, 111, 119, 101, 114, 115, 95, 102, 105, 108, 101, 44, 10, 32, 32, 32, 32, 32, 32, 32,
        32, 100, 111, 95, 114, 101, 99, 111, 110, 115, 116, 114, 117, 99, 116, 95, 100, 97, 116,
        97, 58, 32, 102, 97, 108, 115, 101, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 100, 111, 95,
        118, 101, 114, 105, 102, 121, 95, 98, 108, 111, 99, 107, 115, 58, 32, 102, 97, 108, 115,
        101, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 98, 108, 111, 99, 107, 95, 102, 105, 108, 101,
        115, 58, 32, 91, 93, 44, 10, 32, 32, 32, 32, 125, 10, 125, 10, 10, 101, 120, 112, 111, 114,
        116, 32, 100, 101, 102, 32, 34, 107, 111, 109, 111, 100, 111, 32, 118, 101, 114, 105, 102,
        121, 34, 32, 91, 10, 32, 32, 32, 32, 46, 46, 46, 98, 108, 111, 99, 107, 115, 58, 32, 112,
        97, 116, 104, 44, 10, 32, 32, 32, 32, 45, 45, 112, 111, 119, 101, 114, 115, 45, 102, 105,
        108, 101, 58, 32, 112, 97, 116, 104, 32, 61, 32, 34, 112, 111, 119, 101, 114, 115, 46, 98,
        105, 110, 34, 44, 10, 32, 32, 32, 32, 45, 45, 108, 111, 103, 45, 108, 101, 118, 101, 108,
        58, 32, 115, 116, 114, 105, 110, 103, 64, 34, 110, 117, 45, 99, 111, 109, 112, 108, 101,
        116, 101, 32, 108, 111, 103, 45, 108, 101, 118, 101, 108, 115, 34, 32, 61, 32, 34, 73, 78,
        70, 79, 34, 10, 93, 58, 32, 110, 111, 116, 104, 105, 110, 103, 32, 45, 62, 32, 116, 97, 98,
        108, 101, 60, 98, 108, 111, 99, 107, 58, 32, 115, 116, 114, 105, 110, 103, 44, 32, 115,
        116, 97, 116, 117, 115, 58, 32, 105, 110, 116, 62, 32, 123, 10, 32, 32, 32, 32, 114, 117,
        110, 45, 107, 111, 109, 111, 100, 111, 32, 45, 45, 108, 111, 103, 45, 108, 101, 118, 101,
        108, 32, 36, 108, 111, 103, 95, 108, 101, 118, 101, 108, 32, 123, 10, 32, 32, 32, 32, 32,
        32, 32, 32, 98, 121, 116, 101, 115, 58, 32, 34, 34, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32,
        107, 58, 32, 48, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 110, 58, 32, 48, 44, 10, 32, 32,
        32, 32, 32, 32, 32, 32, 100, 111, 95, 103, 101, 110, 101, 114, 97, 116, 101, 95, 112, 111,
        119, 101, 114, 115, 58, 32, 102, 97, 108, 115, 101, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32,
        112, 111, 119, 101, 114, 115, 95, 102, 105, 108, 101, 58, 32, 36, 112, 111, 119, 101, 114,
        115, 95, 102, 105, 108, 101, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 100, 111, 95, 114,
        101, 99, 111, 110, 115, 116, 114, 117, 99, 116, 95, 100, 97, 116, 97, 58, 32, 102, 97, 108,
        115, 101, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 100, 111, 95, 118, 101, 114, 105, 102,
        121, 95, 98, 108, 111, 99, 107, 115, 58, 32, 116, 114, 117, 101, 44, 10, 32, 32, 32, 32,
        32, 32, 32, 32, 98, 108, 111, 99, 107, 95, 102, 105, 108, 101, 115, 58, 32, 36, 98, 108,
        111, 99, 107, 115, 44, 10, 32, 32, 32, 32, 125, 10, 125, 10, 10, 101, 120, 112, 111, 114,
        116, 32, 100, 101, 102, 32, 34, 107, 111, 109, 111, 100, 111, 32, 114, 101, 99, 111, 110,
        115, 116, 114, 117, 99, 116, 34, 32, 91, 10, 32, 32, 32, 32, 46, 46, 46, 98, 108, 111, 99,
        107, 115, 58, 32, 112, 97, 116, 104, 44, 10, 32, 32, 32, 32, 45, 45, 108, 111, 103, 45,
        108, 101, 118, 101, 108, 58, 32, 115, 116, 114, 105, 110, 103, 64, 34, 110, 117, 45, 99,
        111, 109, 112, 108, 101, 116, 101, 32, 108, 111, 103, 45, 108, 101, 118, 101, 108, 115, 34,
        32, 61, 32, 34, 73, 78, 70, 79, 34, 10, 93, 58, 32, 110, 111, 116, 104, 105, 110, 103, 32,
        45, 62, 32, 108, 105, 115, 116, 60, 105, 110, 116, 62, 32, 123, 10, 32, 32, 32, 32, 114,
        117, 110, 45, 107, 111, 109, 111, 100, 111, 32, 45, 45, 108, 111, 103, 45, 108, 101, 118,
        101, 108, 32, 36, 108, 111, 103, 95, 108, 101, 118, 101, 108, 32, 123, 10, 32, 32, 32, 32,
        32, 32, 32, 32, 98, 121, 116, 101, 115, 58, 32, 34, 34, 44, 10, 32, 32, 32, 32, 32, 32, 32,
        32, 107, 58, 32, 48, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 110, 58, 32, 48, 44, 10, 32,
        32, 32, 32, 32, 32, 32, 32, 100, 111, 95, 103, 101, 110, 101, 114, 97, 116, 101, 95, 112,
        111, 119, 101, 114, 115, 58, 32, 102, 97, 108, 115, 101, 44, 10, 32, 32, 32, 32, 32, 32,
        32, 32, 112, 111, 119, 101, 114, 115, 95, 102, 105, 108, 101, 58, 32, 34, 34, 44, 10, 32,
        32, 32, 32, 32, 32, 32, 32, 100, 111, 95, 114, 101, 99, 111, 110, 115, 116, 114, 117, 99,
        116, 95, 100, 97, 116, 97, 58, 32, 116, 114, 117, 101, 44, 10, 32, 32, 32, 32, 32, 32, 32,
        32, 100, 111, 95, 118, 101, 114, 105, 102, 121, 95, 98, 108, 111, 99, 107, 115, 58, 32,
        102, 97, 108, 115, 101, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 98, 108, 111, 99, 107, 95,
        102, 105, 108, 101, 115, 58, 32, 36, 98, 108, 111, 99, 107, 115, 44, 10, 32, 32, 32, 32,
        125, 10, 125, 10,
    ];

    #[allow(clippy::expect_fun_call)]
    fn decoding_template<E: Pairing>(data: &[u8], k: usize, n: usize) {
        let hash = Sha256::hash(data).to_vec();

        let points: Vec<E::ScalarField> = (0..n)
            .map(|i| E::ScalarField::from_le_bytes_mod_order(&[i as u8]))
            .collect();
        let encoding = Matrix::vandermonde(&points, k);

        let source_shards = Matrix::from_vec_vec(
            field::split_data_into_field_elements::<E>(data, 1, false)
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
            .map(|(i, s)| Shard {
                k: k as u32,
                linear_combination: vec![LinearCombinationElement {
                    index: i as u32,
                    weight: E::ScalarField::one(),
                }],
                hash: hash.clone(),
                bytes: field::merge_elements_into_bytes::<E>(s, false),
                size: data.len(),
            })
            .collect();

        assert_eq!(
            data,
            decode::<E>(shards).expect(&format!("could not decode shards ({} bytes)", data.len()))
        );
    }

    #[test]
    fn decoding() {
        decoding_template::<Bls12_381>(&BYTES[..63], 3, 5);
    }

    fn create_fake_shard<E: Pairing>(
        linear_combination: &[LinearCombinationElement<E>],
        bytes: &[u8],
    ) -> Shard<E> {
        let mut bytes = bytes.to_vec();
        bytes.resize(32, 0);

        Shard {
            k: 0,
            linear_combination: linear_combination.to_vec(),
            hash: vec![],
            bytes,
            size: 0,
        }
    }

    fn recoding_template<E: Pairing>() {
        let a: Shard<E> = create_fake_shard(
            &[LinearCombinationElement {
                index: 0,
                weight: E::ScalarField::one(),
            }],
            &[1, 2, 3],
        );
        let b: Shard<E> = create_fake_shard(
            &[LinearCombinationElement {
                index: 1,
                weight: E::ScalarField::one(),
            }],
            &[4, 5, 6],
        );

        assert_eq!(
            a.mul(E::ScalarField::from_le_bytes_mod_order(&[2])),
            create_fake_shard(
                &[LinearCombinationElement {
                    index: 0,
                    weight: E::ScalarField::from_le_bytes_mod_order(&[2]),
                }],
                &[2, 4, 6],
            )
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
                    LinearCombinationElement {
                        index: 0,
                        weight: E::ScalarField::from_le_bytes_mod_order(&[3]),
                    },
                    LinearCombinationElement {
                        index: 1,
                        weight: E::ScalarField::from_le_bytes_mod_order(&[5]),
                    }
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
                    LinearCombinationElement {
                        index: 0,
                        weight: E::ScalarField::from_le_bytes_mod_order(&[6]),
                    },
                    LinearCombinationElement {
                        index: 1,
                        weight: E::ScalarField::from_le_bytes_mod_order(&[10]),
                    },
                    LinearCombinationElement {
                        index: 0,
                        weight: E::ScalarField::from_le_bytes_mod_order(&[4]),
                    }
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
