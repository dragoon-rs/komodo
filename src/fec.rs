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
    };

    const DATA: &[u8] = b"f\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0o\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0o\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0b\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0a\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0r\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0b\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0a\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0z\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

    const K: usize = 3;

    const SHARDS: [[u32; K]; 7] = [
        [102u32, 111u32, 111u32],
        [298u32, 305u32, 347u32],
        [690u32, 693u32, 827u32],
        [1278u32, 1275u32, 1551u32],
        [2062u32, 2051u32, 2519u32],
        [3042u32, 3021u32, 3731u32],
        [4218u32, 4185u32, 5187u32],
    ];
    const LOST_SHARDS: [usize; 2] = [4, 0];

    fn decoding_template<E: Pairing>() {
        let hash = Sha256::hash(DATA).to_vec();

        let mut shards = SHARDS
            .iter()
            .map(|r| {
                Some(
                    r.iter()
                        .map(|s| E::ScalarField::from_le_bytes_mod_order(&s.to_le_bytes()))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        for i in LOST_SHARDS {
            shards[i] = None;
        }

        let mut blocks = Vec::new();
        for (i, shard) in shards.iter().enumerate() {
            if let Some(bytes) = shard {
                blocks.push(Shard {
                    k: K as u32,
                    linear_combination: vec![LinearCombinationElement {
                        index: i as u32,
                        weight: E::ScalarField::one(),
                    }],
                    hash: hash.clone(),
                    bytes: field::merge_elements_into_bytes::<E>(bytes, false),
                    size: DATA.len(),
                });
            }
        }

        assert_eq!(DATA, decode::<E>(blocks).unwrap())
    }

    #[test]
    fn decoding() {
        decoding_template::<Bls12_381>();
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
