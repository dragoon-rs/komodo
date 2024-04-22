// see `benches/README.md`
use std::time::Duration;

use ark_ec::CurveGroup;
use ark_ff::PrimeField;

use criterion::{criterion_group, criterion_main, Criterion};

fn random_sampling_template<G: CurveGroup>(c: &mut Criterion, curve: &str) {
    let mut rng = ark_std::test_rng();

    c.bench_function(&format!("random sampling on {}", curve), |b| {
        b.iter(|| {
            let _ = G::rand(&mut rng);
        });
    });
}

fn group_template<F: PrimeField, G: CurveGroup<ScalarField = F>>(c: &mut Criterion, curve: &str) {
    let mut rng = ark_std::test_rng();

    c.bench_function(&format!("addition on {}", curve), |b| {
        b.iter(|| {
            let g1 = G::rand(&mut rng);
            let g2 = G::rand(&mut rng);
            g1 + g2
        });
    });

    c.bench_function(&format!("substraction on {}", curve), |b| {
        b.iter(|| {
            let g1 = G::rand(&mut rng);
            let g2 = G::rand(&mut rng);
            g1 - g2
        });
    });

    c.bench_function(&format!("double on {}", curve), |b| {
        b.iter(|| {
            let g1 = G::rand(&mut rng);
            g1.double()
        });
    });

    c.bench_function(&format!("scalar multiplication on {}", curve), |b| {
        b.iter(|| {
            let g1 = G::rand(&mut rng);
            let f1 = F::rand(&mut rng);
            g1.mul(f1)
        });
    });
}

fn curve_group_template<F: PrimeField, G: CurveGroup<ScalarField = F>>(
    c: &mut Criterion,
    curve: &str,
) {
    let mut rng = ark_std::test_rng();

    c.bench_function(&format!("into affine on {}", curve), |b| {
        b.iter(|| {
            let g1 = G::rand(&mut rng);
            g1.into_affine()
        });
    });

    c.bench_function(&format!("from affine on {}", curve), |b| {
        b.iter(|| {
            let g1_affine = G::rand(&mut rng).into_affine();
            let _: G = g1_affine.into();
        });
    });

    c.bench_function(&format!("affine addition on {}", curve), |b| {
        b.iter(|| {
            let g1_affine = G::rand(&mut rng).into_affine();
            let g2_affine = G::rand(&mut rng).into_affine();
            g1_affine + g2_affine
        });
    });

    c.bench_function(&format!("affine scalar multiplication on {}", curve), |b| {
        b.iter(|| {
            let g1_affine = G::rand(&mut rng).into_affine();
            let f1 = F::rand(&mut rng);
            g1_affine * f1
        });
    });
}

macro_rules! bench {
    ($c:ident, $b:ident, G1=$g:ident, name=$n:expr) => {
        random_sampling_template::<$c::$g>($b, $n);
        group_template::<$c::Fr, $c::$g>($b, $n);
        curve_group_template::<$c::Fr, $c::$g>($b, $n);
    };
}

fn bench(c: &mut Criterion) {
    bench!(ark_bls12_381, c, G1 = G1Projective, name = "BLS12-381");
    bench!(ark_bn254, c, G1 = G1Projective, name = "BN-254");
    bench!(ark_pallas, c, G1 = Projective, name = "PALLAS");
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs_f32(3.0))
        .sample_size(100);
    targets = bench
);
criterion_main!(benches);
