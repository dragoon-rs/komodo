// see `benches/README.md`
use std::time::Duration;

use ark_ff::PrimeField;

use criterion::{criterion_group, criterion_main, Criterion};

fn random_sampling_template<F: PrimeField>(c: &mut Criterion, curve: &str) {
    let mut rng = ark_std::test_rng();

    c.bench_function(&format!("random sampling on {}", curve), |b| {
        b.iter(|| {
            let _ = F::rand(&mut rng);
        });
    });
}

fn additive_group_template<F: PrimeField>(c: &mut Criterion, curve: &str) {
    let mut rng = ark_std::test_rng();

    c.bench_function(&format!("addition on {}", curve), |b| {
        b.iter(|| {
            let f1 = F::rand(&mut rng);
            let f2 = F::rand(&mut rng);
            f1 + f2
        });
    });

    c.bench_function(&format!("substraction on {}", curve), |b| {
        b.iter(|| {
            let f1 = F::rand(&mut rng);
            let f2 = F::rand(&mut rng);
            f1 - f2
        });
    });

    c.bench_function(&format!("double on {}", curve), |b| {
        b.iter(|| {
            let f1 = F::rand(&mut rng);
            f1.double()
        });
    });

    c.bench_function(&format!("multiplication on {}", curve), |b| {
        b.iter(|| {
            let f1 = F::rand(&mut rng);
            let f2 = F::rand(&mut rng);
            f1 * f2
        });
    });
}

fn field_template<F: PrimeField>(c: &mut Criterion, curve: &str) {
    let mut rng = ark_std::test_rng();

    c.bench_function(&format!("square on {}", curve), |b| {
        b.iter(|| {
            let f1 = F::rand(&mut rng);
            f1.square()
        });
    });

    c.bench_function(&format!("inverse on {}", curve), |b| {
        b.iter(|| {
            let f1 = F::rand(&mut rng);
            f1.inverse()
        });
    });

    c.bench_function(&format!("legendre on {}", curve), |b| {
        b.iter(|| {
            let f1 = F::rand(&mut rng);
            f1.legendre()
        });
    });

    c.bench_function(&format!("sqrt on {}", curve), |b| {
        b.iter(|| {
            let f1 = F::rand(&mut rng);
            if f1.legendre().is_qr() {
                let _ = f1.sqrt();
            }
        });
    });
}

fn prime_field_template<F: PrimeField>(c: &mut Criterion, curve: &str) {
    let mut rng = ark_std::test_rng();

    c.bench_function(&format!("exponentiation on {}", curve), |b| {
        b.iter(|| {
            let f1 = F::rand(&mut rng);
            f1.pow(F::MODULUS)
        });
    });
}

fn conversions_template<F: PrimeField>(c: &mut Criterion, curve: &str) {
    let mut rng = ark_std::test_rng();

    c.bench_function(&format!("bigint conversion on {}", curve), |b| {
        b.iter(|| {
            let f1 = F::rand(&mut rng);
            f1.into_bigint()
        });
    });
}

macro_rules! bench {
    ($c:ident, $b:ident, F=$f:ident, name=$n:expr) => {
        random_sampling_template::<$c::$f>($b, $n);
        additive_group_template::<$c::$f>($b, $n);
        field_template::<$c::$f>($b, $n);
        prime_field_template::<$c::$f>($b, $n);
        conversions_template::<$c::$f>($b, $n);
    };
}

fn bench(c: &mut Criterion) {
    bench!(ark_bls12_381, c, F = Fr, name = "BLS12-381");
    bench!(ark_bn254, c, F = Fr, name = "BN-254");
    bench!(ark_pallas, c, F = Fr, name = "PALLAS");
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs_f32(3.0))
        .sample_size(100);
    targets = bench
);
criterion_main!(benches);
