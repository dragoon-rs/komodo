use ark_ff::PrimeField;
use ark_std::rand::Rng;

use komodo::{
    algebra,
    fec::{recode_with_coeffs, Shard},
};
use plnk::Bencher;

fn to_curve<F: PrimeField>(n: u128) -> F {
    F::from_le_bytes_mod_order(&n.to_le_bytes())
}

fn create_fake_shard<F: PrimeField>(nb_bytes: usize, k: usize) -> Shard<F> {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..(nb_bytes / k)).map(|_| rng.gen::<u8>()).collect();

    let linear_combination: Vec<F> = (0..k).map(|_| to_curve::<F>(rng.gen::<u128>())).collect();

    Shard {
        k: k as u32,
        linear_combination,
        hash: vec![],
        data: algebra::split_data_into_field_elements::<F>(&bytes, 1),
        size: 0,
    }
}

pub(crate) fn run<F: PrimeField>(b: &Bencher, nb_bytes: usize, k: usize, nb_shards: usize) {
    let shards: Vec<Shard<F>> = (0..nb_shards)
        .map(|_| create_fake_shard(nb_bytes, k))
        .collect();

    let mut rng = rand::thread_rng();
    let coeffs: Vec<F> = (0..nb_shards)
        .map(|_| to_curve::<F>(rng.gen::<u128>()))
        .collect();

    plnk::bench(
        b,
        crate::label! { bytes: nb_bytes, shards: nb_shards, k: k },
        || plnk::timeit(|| recode_with_coeffs(&shards, &coeffs)),
    );
}
