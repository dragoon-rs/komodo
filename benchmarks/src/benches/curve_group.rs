use ark_ec::CurveGroup;
use ark_ff::PrimeField;

fn label(op: &str) -> String {
    plnk::label! { operation: format!("{}", op)}.to_string()
}

pub(crate) fn run<F: PrimeField, G: CurveGroup<ScalarField = F>>(b: &plnk::Bencher) {
    let rng = &mut ark_std::rand::thread_rng();

    plnk::bench(b, &label("random sampling"), || {
        plnk::timeit(|| G::rand(rng))
    });

    plnk::bench(b, &label("addition"), || {
        let g1 = G::rand(rng);
        let g2 = G::rand(rng);

        plnk::timeit(|| g1 + g2)
    });

    plnk::bench(b, &label("substraction"), || {
        let g1 = G::rand(rng);
        let g2 = G::rand(rng);

        plnk::timeit(|| g1 - g2)
    });

    plnk::bench(b, &label("double"), || {
        let g1 = G::rand(rng);

        plnk::timeit(|| g1.double())
    });

    plnk::bench(b, &label("scalar multiplication"), || {
        let g1 = G::rand(rng);
        let f1 = F::rand(rng);

        plnk::timeit(|| g1.mul(f1))
    });

    plnk::bench(b, &label("into affine"), || {
        let g1 = G::rand(rng);

        plnk::timeit(|| g1.into_affine())
    });

    plnk::bench(b, &label("from affine"), || {
        let g1_affine = G::rand(rng).into_affine();

        plnk::timeit(|| Into::<G>::into(g1_affine))
    });

    plnk::bench(b, &label("affine addition"), || {
        let g1_affine = G::rand(rng).into_affine();
        let g2_affine = G::rand(rng).into_affine();

        plnk::timeit(|| g1_affine + g2_affine)
    });

    plnk::bench(b, &label("affine scalar multiplication"), || {
        let g1_affine = G::rand(rng).into_affine();
        let f1 = F::rand(rng);

        plnk::timeit(|| g1_affine * f1)
    });
}
