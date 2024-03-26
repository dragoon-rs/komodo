use ark_bls12_381::Bls12_381;
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;

use rand::Rng;

use komodo::{
    fec::{combine, Shard},
    field,
};

use criterion::{criterion_group, criterion_main, Criterion};

fn to_curve<E: Pairing>(n: u128) -> E::ScalarField {
    E::ScalarField::from_le_bytes_mod_order(&n.to_le_bytes())
}

fn create_fake_shard<E: Pairing>(nb_bytes: usize, k: usize) -> Shard<E> {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..nb_bytes).map(|_| rng.gen::<u8>()).collect();

    let linear_combination: Vec<E::ScalarField> =
        (0..k).map(|_| to_curve::<E>(rng.gen::<u128>())).collect();

    Shard {
        k: k as u32,
        linear_combination,
        hash: vec![],
        data: field::split_data_into_field_elements::<E>(&bytes, 1),
        size: 0,
    }
}

fn bench_template<E: Pairing>(c: &mut Criterion, nb_bytes: usize, k: usize, nb_shards: usize) {
    let shards: Vec<Shard<E>> = (0..nb_shards)
        .map(|_| create_fake_shard(nb_bytes, k))
        .collect();

    let mut rng = rand::thread_rng();
    let coeffs: Vec<E::ScalarField> = (0..nb_shards)
        .map(|_| to_curve::<E>(rng.gen::<u128>()))
        .collect();

    c.bench_function(
        &format!(
            "recoding {} bytes and {} shards with k = {}",
            nb_bytes, nb_shards, k
        ),
        |b| b.iter(|| combine(&shards, &coeffs)),
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    for nb_bytes in [1, 1_024, 1_024 * 1_024] {
        for nb_shards in [2, 4, 8, 16] {
            for k in [2, 4, 8, 16] {
                bench_template::<Bls12_381>(c, nb_bytes, k, nb_shards);
            }
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
