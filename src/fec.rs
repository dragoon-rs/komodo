//! A module to encode, recode and decode shards of data with [FEC] methods.
//!
//! In all the following, $(k, n)$ codes will be described, where $k$ is the number of source
//! shards and $n$ is the number of encoded shards.
//!
//! The _code ratio_ is defined as $\rho = \frac{k}{n}$.
//!
//! ## Example
//! In the following example, a file is encoded and decoded back.
//!
//! The dotted circle in between "_dissemination_" and "_gathering_" represents the "_life_" of the
//! shards, e.g. them being shared between peers on a network, recoded or lost.
#![doc = simple_mermaid::mermaid!("fec.mmd")]
//! In the end, [FEC] methods guarantee that $F^* = F$, as long as at least $k$ linearly
//! independant shards are gathered before decoding.
//!
//! [FEC]: https://en.wikipedia.org/wiki/Error_correction_code

use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::RngCore;

use rs_merkle::{algorithms::Sha256, Hasher};

use crate::{algebra, algebra::linalg::Matrix, error::KomodoError};

/// Representation of a [FEC] shard of data.
///
/// [FEC]: https://en.wikipedia.org/wiki/Error_correction_code
#[derive(Debug, Default, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Shard<F: PrimeField> {
    /// the code parameter, required to decode
    pub k: u32,
    /// tells the decoder how the shard was constructed with respect to the original source shards.
    ///
    /// This effectively allows support for _recoding_.
    ///
    /// If we denote the $k$ source shards by $(s\_i)\_\{0 \le i \lt k\}$, the linear combination by $k$
    /// coefficients $(\alpha_i)_{0 \le i \lt k}$ and $s$ the shard itself, then
    ///
    /// $$ s = \sum\limits_{i = 0}^{k - 1} \alpha_i s_i$$
    pub linear_combination: Vec<F>,
    /// the hash of the original data, used for validation
    pub hash: Vec<u8>,
    /// the shard itself
    pub data: Vec<F>,
    /// the size of the original data, used for padding
    pub size: usize,
}

impl<F: PrimeField> Shard<F> {
    /// Computes the linear combination between two [`Shard`]s.
    ///
    /// If we denote the [`Shard`] itself and the other [`Shard`] by $s$ and $o$ respectively, the
    /// output is
    /// $$ \alpha s + \beta o $$
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
                .collect(),
            size: self.size,
        }
    }
}

/// Computes the linear combination between an arbitrary number of [`Shard`]s.
///
/// > **Note**
/// >
/// > This is basically a multi-[`Shard`] wrapper around [`Shard::recode_with`].
/// >
/// > [`recode_with_coeffs`] will return [`None`] if the number of shards
/// > is not the same as the number of coefficients or if no shards are provided.
///
/// If the shards are the $(s \_i)\_\{1 \le i \le n\}$ and the coefficients the
/// $(\alpha\_i)\_\{1 \le i \le n\}$, then the output will be
///
/// $$ \sum\limits_{i = 1}^{n} \alpha_i s_i$$
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

/// Computes a recoded shard from an arbitrary set of compatible shards.
///
/// Coefficients will be drawn at random, one for each shard.
///
/// If the shards appear to come from different data, e.g. if $k$ is not the
/// same or the hash of the data is different, an error will be returned.
///
/// > **Note**
/// >
/// > This is a wrapper around [`recode_with_coeffs`].
pub fn recode_random<F: PrimeField>(
    shards: &[Shard<F>],
    rng: &mut impl RngCore,
) -> Result<Option<Shard<F>>, KomodoError> {
    for (i, (s1, s2)) in shards.iter().zip(shards.iter().skip(1)).enumerate() {
        if s1.k != s2.k {
            return Err(KomodoError::IncompatibleShards {
                key: "k".to_string(),
                index: i,
                left: s1.k.to_string(),
                right: s2.k.to_string(),
            });
        }
        if s1.hash != s2.hash {
            return Err(KomodoError::IncompatibleShards {
                key: "hash".to_string(),
                index: i,
                left: format!("{:?}", s1.hash),
                right: format!("{:?}", s2.hash),
            });
        }
        if s1.size != s2.size {
            return Err(KomodoError::IncompatibleShards {
                key: "size".to_string(),
                index: i,
                left: s1.size.to_string(),
                right: s2.size.to_string(),
            });
        }
    }

    let coeffs = shards.iter().map(|_| F::rand(rng)).collect::<Vec<_>>();
    Ok(recode_with_coeffs(shards, &coeffs))
}

/// Applies a given encoding matrix to some data to generate encoded shards.
///
/// We arrange the source shards to be encoded in an $m \times k$ matrix $S$, i.e. $k$ shards of
/// length $m$. The encoding matrix $M$ then is a $k \times n$ matrix and the encoded shards are
/// the $n$ columns of
///
/// $$E = S M$$
///
/// > **Note**
/// >
/// > The input data and the encoding matrix should have compatible shapes,
/// > otherwise, an error might be thrown to the caller.
///
/// Padding might be applied depending on the size of the data compared to the size of the encoding
/// matrix, see [`algebra::split_data_into_field_elements`].
///
/// This is the inverse of [`decode`].
pub fn encode<F: PrimeField>(
    data: &[u8],
    encoding_mat: &Matrix<F>,
) -> Result<Vec<Shard<F>>, KomodoError> {
    let hash = Sha256::hash(data).to_vec();

    let k = encoding_mat.height;

    let source_shards = Matrix::from_vec_vec(
        algebra::split_data_into_field_elements(data, k)
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

/// Reconstructs the original data from a set of encoded, possibly recoded, shards.
///
/// Let's assume at least $k$ linearly independant shards have been retrieved and put in a matrix
/// $\hat{E}$. We use the [linear combination][`Shard::linear_combination`] of each shard to
/// reconstruct the columns of the square submatrix $\hat{M}$ that has been used to encode these
/// shards. Then the reconstructed source shards $\hat{S}$ are given by
///
/// $$\hat{S} = \hat{M}^{-1} \hat{E}$$
///
/// > **Note**
/// >
/// > This function might fail in a variety of cases
/// > - if there are too few shards
/// > - if there are linear dependencies between shards
///
/// This is the inverse of [`encode`].
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

    let mut bytes = algebra::merge_elements_into_bytes(&source_shards);
    bytes.resize(shards[0].size, 0);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Fr;
    use ark_ff::PrimeField;

    use crate::{
        algebra,
        algebra::linalg::Matrix,
        fec::{decode, encode, recode_random, Shard},
    };

    use itertools::Itertools;
    use rand::seq::SliceRandom;

    use super::recode_with_coeffs;

    type LC = Vec<usize>;
    type LCExclusion = Vec<usize>;

    fn bytes() -> Vec<u8> {
        include_bytes!("../assets/dragoon_32x32.png").to_vec()
    }

    fn to_curve<F: PrimeField>(n: u128) -> F {
        F::from_le_bytes_mod_order(&n.to_le_bytes())
    }

    /// `contains_one_of(x, set)` is true iif `x` fully contains one of the lists from `set`
    ///
    /// > **Note**
    /// >
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
        assert!(contains_one_of(&[0, 4, 5], &[vec![2, 3, 5], vec![4, 5]]));
    }

    fn try_all_decoding_combinations<F: PrimeField>(
        data: &[u8],
        shards: &[Shard<F>],
        k: usize,
        n: usize,
        test_case: &str,
        limit: Option<usize>,
        should_not_be_decodable: Vec<LCExclusion>,
    ) {
        let there_are_recoded_shards = shards.len() > n;

        for c in shards
            .iter()
            .cloned()
            .enumerate()
            .combinations(k)
            .take(limit.unwrap_or(usize::MAX))
        {
            let is: Vec<usize> = c.iter().map(|(i, _)| *i).collect();
            if there_are_recoded_shards {
                let contains_recoded_shards = *is.iter().max().unwrap() < n;
                if contains_recoded_shards {
                    continue;
                }
            }

            let pretty_is = is
                .iter()
                .map(|&i| {
                    #[allow(clippy::comparison_chain)]
                    if i == n {
                        "(n)".into()
                    } else if i > n {
                        format!("(n + {})", i - n)
                    } else {
                        format!("{}", i)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let pretty_is = format!("[{pretty_is}]");

            let actual = decode::<F>(c.iter().map(|(_, s)| s).cloned().collect());

            if contains_one_of(&is, &should_not_be_decodable) {
                assert!(
                    actual.is_err(),
                    "should not decode with {} {test_case}",
                    pretty_is
                );
                continue;
            }

            assert!(
                actual.is_ok(),
                "could not decode with {} {test_case}",
                pretty_is
            );

            assert_eq!(
                data,
                actual.unwrap(),
                "bad decoded data with {} {test_case}",
                pretty_is,
            );
        }
    }

    fn end_to_end_template<F: PrimeField>(data: &[u8], k: usize, n: usize) {
        let mut rng = ark_std::test_rng();
        let test_case = format!("TEST | data: {} bytes, k: {}, n: {}", data.len(), k, n);

        let shards = encode::<F>(data, &Matrix::random(k, n, &mut rng))
            .unwrap_or_else(|_| panic!("could not encode {test_case}"));

        try_all_decoding_combinations(data, &shards, k, n, &test_case, None, vec![]);
    }

    fn end_to_end_with_recoding_template<F: PrimeField>(
        data: &[u8],
        k: usize,
        n: usize,
        recoding_steps: Vec<LC>,
        should_not_be_decodable: Vec<LCExclusion>,
        name: &str,
    ) {
        let mut rng = ark_std::test_rng();
        let test_case = format!(
            "TEST | data: {} bytes, k: {}, n: {}, scenario: {}",
            data.len(),
            k,
            n,
            name
        );

        let mut shards = encode::<F>(data, &Matrix::random(k, n, &mut rng))
            .unwrap_or_else(|_| panic!("could not encode {test_case}"));

        for step in recoding_steps {
            let shards_to_recode: Vec<_> = shards
                .iter()
                .cloned()
                .enumerate()
                .filter_map(|(i, s)| if step.contains(&i) { Some(s) } else { None })
                .collect();
            shards.push(recode_random(&shards_to_recode, &mut rng).unwrap().unwrap());
        }

        try_all_decoding_combinations(
            data,
            &shards,
            k,
            n,
            &test_case,
            None,
            should_not_be_decodable,
        );
    }

    #[test]
    fn end_to_end() {
        let bytes = bytes();

        let ks = [3, 5];
        let n = 5;

        for k in ks {
            end_to_end_template::<Fr>(&bytes, k, n);
        }
    }

    #[test]
    fn end_to_end_with_recoding() {
        let bytes = bytes();

        fn get_scenarii(n: usize) -> Vec<(String, Vec<LC>, Vec<LCExclusion>)> {
            vec![
                // ```mermaid
                // graph TD;
                //     a[n+1]; b[n+2]; c[n+3];
                //
                //     1;
                //     3-->a; 5-->a;
                //     2-->b; 4-->b;
                //     a-->c; b-->c;
                // ```
                (
                    "simple".into(),
                    vec![
                        vec![2, 4],       // =  n
                        vec![1, 3],       // = (n + 1)
                        vec![n, (n + 1)], // = (n + 2) = ((2, 4), (1, 3))
                    ],
                    vec![
                        vec![2, 4, n],
                        //
                        vec![1, 3, (n + 1)],
                        vec![n, (n + 1), (n + 2)],
                        vec![1, 3, n, (n + 2)],
                        vec![2, 4, (n + 1), (n + 2)],
                        vec![1, 2, 3, 4, (n + 2)],
                    ],
                ),
                // ```mermaid
                // graph TD;
                //     a[n+1]; b[n+2];
                //
                //     1-->a; a-->b;
                //     2; 3; 4; 5;
                // ```
                (
                    "chain".into(),
                    vec![
                        vec![0],   // = (n) = (0)
                        vec![(n)], // = (n + 1) = (0)
                    ],
                    vec![vec![0, (n)], vec![0, (n + 1)], vec![(n), (n + 1)]],
                ),
            ]
        }

        for (k, n) in [(3, 5), (5, 5), (8, 10)] {
            for (name, steps, should_not_decode) in get_scenarii(n) {
                end_to_end_with_recoding_template::<Fr>(
                    &bytes,
                    k,
                    n,
                    steps,
                    should_not_decode,
                    &name,
                );
            }
        }
    }

    //   (encode) | (select k) |     (recode)    | (decode)
    //            *
    // *          *            *     * ... *     * \
    // *  ------> * ---------> * --> * ... * --> *  |--> ?
    // *          *            *     * ... *     * /
    //            *                  \__(#steps)_/
    //
    // k          n            k     k     k     k
    fn long_full_end_to_end_with_recoding_template<F: PrimeField>(
        data: &[u8],
        k: usize,
        n: usize,
        nb_steps: usize,
    ) {
        let mut rng = ark_std::test_rng();
        let test_case = format!("TEST | data: {} bytes, k: {}, n: {}", data.len(), k, n,);

        let mut shards = encode::<F>(data, &Matrix::random(k, n, &mut rng))
            .unwrap_or_else(|_| panic!("could not encode {test_case}"));
        shards.shuffle(&mut rng);
        shards.truncate(k);

        for _ in 0..nb_steps {
            shards = (0..k)
                .map(|_| recode_random(&shards, &mut rng).unwrap().unwrap())
                .collect();
        }

        let actual = decode::<F>(shards).unwrap_or_else(|_| panic!("could not decode {test_case}"));
        assert_eq!(data, actual, "bad decoded data with {test_case}",);
    }

    #[test]
    fn long_full_end_to_end_with_recoding() {
        let bytes = bytes();

        for (k, n) in [(3, 5), (5, 5), (8, 10)] {
            for nb_steps in [10, 20, 100] {
                long_full_end_to_end_with_recoding_template::<Fr>(&bytes, k, n, nb_steps);
            }
        }
    }

    fn create_fake_shard<F: PrimeField>(linear_combination: &[F], bytes: &[u8]) -> Shard<F> {
        Shard {
            k: 2,
            linear_combination: linear_combination.to_vec(),
            hash: vec![],
            data: algebra::split_data_into_field_elements(bytes, 1),
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
