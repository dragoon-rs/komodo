use std::ops::{Add, Mul};

use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use reed_solomon_erasure::{Error, Field as GF, ReedSolomonNonSystematic};

use crate::field;

#[derive(Debug, Default, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct LinearCombinationElement {
    pub index: u32,
    pub weight: u32,
}

#[derive(Debug, Default, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Shard {
    pub k: u32,
    pub linear_combination: Vec<LinearCombinationElement>,
    pub hash: Vec<u8>,
    pub bytes: Vec<u8>,
    pub size: usize,
}

impl Shard {
    pub fn mul<E: Pairing>(&self, alpha: u32) -> Self {
        let bytes = match alpha {
            0 => vec![0u8; self.bytes.len()],
            1 => self.bytes.to_vec(),
            _ => {
                let alpha = E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(alpha));

                let elements = field::split_data_into_field_elements::<E>(&self.bytes, 1)
                    .iter()
                    .map(|e| e.mul(alpha))
                    .collect::<Vec<_>>();

                field::merge_elements_into_bytes::<E>(&elements)
            }
        };

        Shard {
            k: self.k,
            linear_combination: self
                .linear_combination
                .iter()
                .map(|l| LinearCombinationElement {
                    index: l.index,
                    weight: l.weight * alpha,
                })
                .collect(),
            hash: self.hash.clone(),
            bytes,
            size: self.size,
        }
    }

    pub fn combine<E: Pairing>(&self, alpha: u32, other: &Self, beta: u32) -> Self {
        if alpha == 0 {
            return other.clone();
        } else if beta == 0 {
            return self.clone();
        }

        let elements = {
            let alpha = E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(alpha));
            let beta = E::ScalarField::from_le_bytes_mod_order(&u32_to_u8_vec(beta));

            let elements_self = field::split_data_into_field_elements::<E>(&self.bytes, 1);
            let elements_other = field::split_data_into_field_elements::<E>(&other.bytes, 1);

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
                weight: lce.weight * alpha,
            });
        }
        for lce in &other.linear_combination {
            linear_combination.push(LinearCombinationElement {
                index: lce.index,
                weight: lce.weight * beta,
            });
        }

        Shard {
            k: self.k,
            linear_combination,
            hash: self.hash.clone(),
            bytes: field::merge_elements_into_bytes::<E>(&elements),
            size: self.size,
        }
    }
}

fn u32_to_u8_vec(num: u32) -> Vec<u8> {
    vec![
        (num & 0xFF) as u8,
        ((num >> 8) & 0xFF) as u8,
        ((num >> 16) & 0xFF) as u8,
        ((num >> 24) & 0xFF) as u8,
    ]
}

pub fn decode<F: GF>(blocks: Vec<Shard>) -> Result<Vec<u8>, Error> {
    let k = blocks[0].k;
    let n = blocks
        .iter()
        // FIXME: this is incorrect
        .map(|b| b.linear_combination[0].index)
        .max()
        .unwrap_or(0)
        + 1;

    if blocks.len() < k as usize {
        return Err(Error::TooFewShards);
    }

    let mut shards: Vec<Option<Vec<F::Elem>>> = Vec::with_capacity(n as usize);
    shards.resize(n as usize, None);
    for block in &blocks {
        // FIXME: this is incorrect
        shards[block.linear_combination[0].index as usize] = Some(F::deserialize(&block.bytes));
    }

    ReedSolomonNonSystematic::<F>::vandermonde(k as usize, n as usize)?.reconstruct(&mut shards)?;
    let elements: Vec<_> = shards.iter().filter_map(|x| x.clone()).flatten().collect();

    let mut data = F::into_data(elements.as_slice());
    data.resize(blocks[0].size, 0);
    Ok(data)
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Bls12_381;
    use ark_ec::pairing::Pairing;
    use ark_ff::PrimeField;
    use reed_solomon_erasure::galois_prime::Field as GF;
    use rs_merkle::algorithms::Sha256;
    use rs_merkle::Hasher;

    use crate::{
        fec::{decode, u32_to_u8_vec, LinearCombinationElement, Shard},
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
    const LOST_SHARDS: [usize; 3] = [1, 3, 6];

    fn to_big_int_from_bytes(i: &[u8]) -> <Bls12_381 as Pairing>::ScalarField {
        <Bls12_381 as Pairing>::ScalarField::from_le_bytes_mod_order(i)
    }

    #[test]
    fn decoding() {
        let hash = Sha256::hash(DATA).to_vec();

        let mut shards = SHARDS
            .iter()
            .map(|r| {
                Some(
                    r.iter()
                        .map(|s| to_big_int_from_bytes(&s.to_le_bytes()))
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
                        weight: 1,
                    }],
                    hash: hash.clone(),
                    bytes: field::merge_elements_into_bytes::<Bls12_381>(&bytes),
                    size: DATA.len(),
                });
            }
        }

        assert_eq!(DATA, decode::<GF>(blocks).unwrap())
    }

    #[test]
    fn u32_to_u8_conversion() {
        assert_eq!(u32_to_u8_vec(0u32), vec![0u8, 0u8, 0u8, 0u8]);
        assert_eq!(u32_to_u8_vec(1u32), vec![1u8, 0u8, 0u8, 0u8]);
        assert_eq!(u32_to_u8_vec(256u32), vec![0u8, 1u8, 0u8, 0u8]);
        assert_eq!(u32_to_u8_vec(65536u32), vec![0u8, 0u8, 1u8, 0u8]);
        assert_eq!(u32_to_u8_vec(16777216u32), vec![0u8, 0u8, 0u8, 1u8]);
    }

    fn create_fake_shard(linear_combination: &[LinearCombinationElement], bytes: &[u8]) -> Shard {
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

    #[test]
    fn recoding() {
        let a = create_fake_shard(
            &[LinearCombinationElement {
                index: 0,
                weight: 1,
            }],
            &[1, 2, 3],
        );
        let b = create_fake_shard(
            &[LinearCombinationElement {
                index: 1,
                weight: 1,
            }],
            &[4, 5, 6],
        );

        assert_eq!(
            a.mul::<Bls12_381>(2),
            create_fake_shard(
                &[LinearCombinationElement {
                    index: 0,
                    weight: 2,
                }],
                &[2, 4, 6],
            )
        );

        let c = a.combine::<Bls12_381>(3, &b, 5);

        assert_eq!(
            c,
            create_fake_shard(
                &[
                    LinearCombinationElement {
                        index: 0,
                        weight: 3,
                    },
                    LinearCombinationElement {
                        index: 1,
                        weight: 5,
                    }
                ],
                &[23, 31, 39]
            )
        );

        assert_eq!(
            c.combine::<Bls12_381>(2, &a, 4),
            create_fake_shard(
                &[
                    LinearCombinationElement {
                        index: 0,
                        weight: 6,
                    },
                    LinearCombinationElement {
                        index: 1,
                        weight: 10,
                    },
                    LinearCombinationElement {
                        index: 0,
                        weight: 4,
                    }
                ],
                &[50, 70, 90],
            )
        );
    }
}
