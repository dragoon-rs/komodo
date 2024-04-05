use ark_bls12_381::Fr;
use ark_ff::PrimeField;

use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;

use komodo::{
    fec::{combine, Shard},
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

fn bench_template<F: PrimeField>(c: &mut Criterion, nb_bytes: usize, k: usize, nb_shards: usize) {
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
            nb_bytes,
            nb_shards,
            k,
            std::any::type_name::<F>()
        ),
        |b| b.iter(|| combine(&shards, &coeffs)),
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    for nb_bytes in [1, 1_024, 1_024 * 1_024] {
        for nb_shards in [2, 4, 8, 16] {
            for k in [2, 4, 8, 16] {
                bench_template::<Fr>(c, nb_bytes, k, nb_shards);
            }
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
