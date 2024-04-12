// see `benches/README.md`
use std::time::Duration;

use ark_ff::PrimeField;
use ark_std::rand::Rng;

use criterion::{criterion_group, criterion_main, Criterion};

use komodo::{
    fec::{recode_with_coeffs, Shard},
    field,
};

fn to_curve<F: PrimeField>(n: u128) -> F {
    F::from_le_bytes_mod_order(&n.to_le_bytes())
}

fn create_fake_shard<F: PrimeField>(nb_bytes: usize, k: usize) -> Shard<F> {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..nb_bytes).map(|_| rng.gen::<u8>()).collect();

    let linear_combination: Vec<F> = (0..k).map(|_| to_curve::<F>(rng.gen::<u128>())).collect();

    Shard {
        k: k as u32,
        linear_combination,
        hash: vec![],
        data: field::split_data_into_field_elements::<F>(&bytes, 1),
        size: 0,
    }
}

fn bench_template<F: PrimeField>(
    c: &mut Criterion,
    nb_bytes: usize,
    k: usize,
    nb_shards: usize,
    curve: &str,
) {
    let shards: Vec<Shard<F>> = (0..nb_shards)
        .map(|_| create_fake_shard(nb_bytes, k))
        .collect();

    let mut rng = rand::thread_rng();
    let coeffs: Vec<F> = (0..nb_shards)
        .map(|_| to_curve::<F>(rng.gen::<u128>()))
        .collect();

    c.bench_function(
        &format!(
            "recoding {} bytes and {} shards with k = {} on {}",
            nb_bytes, nb_shards, k, curve
        ),
        |b| b.iter(|| recode_with_coeffs(&shards, &coeffs)),
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    for nb_bytes in [1, 1_024, 1_024 * 1_024] {
        for nb_shards in [2, 4, 8, 16] {
            for k in [2, 4, 8, 16] {
                bench_template::<ark_bls12_381::Fr>(c, nb_bytes, k, nb_shards, "BLS-12-381");
                bench_template::<ark_bn254::Fr>(c, nb_bytes, k, nb_shards, "BN-254");
                bench_template::<ark_pallas::Fr>(c, nb_bytes, k, nb_shards, "PALLAS");
            }
        }
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs_f32(0.5))
        .sample_size(10);
    targets = criterion_benchmark
);
criterion_main!(benches);
