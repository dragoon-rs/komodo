use std::time::Duration;

use ark_ff::PrimeField;

fn label(op: &str) -> String {
    plnk::label! { operation: format!("{}", op)}.to_string()
}

pub(crate) fn run<F: PrimeField>(b: &plnk::Bencher) {
    let rng = &mut ark_std::rand::thread_rng();

    plnk::bench(b, &label("random sampling"), || {
        plnk::timeit(|| F::rand(rng))
    });

    plnk::bench(b, &label("addition"), || {
        let f1 = F::rand(rng);
        let f2 = F::rand(rng);

        plnk::timeit(|| f1 + f2)
    });

    plnk::bench(b, &label("substraction"), || {
        let f1 = F::rand(rng);
        let f2 = F::rand(rng);

        plnk::timeit(|| f1 - f2)
    });

    plnk::bench(b, &label("double"), || {
        let f1 = F::rand(rng);

        plnk::timeit(|| f1.double())
    });

    plnk::bench(b, &label("multiplication"), || {
        let f1 = F::rand(rng);
        let f2 = F::rand(rng);

        plnk::timeit(|| f1 * f2)
    });

    plnk::bench(b, &label("square"), || {
        let f1 = F::rand(rng);

        plnk::timeit(|| f1.square())
    });

    plnk::bench(b, &label("inverse"), || {
        let f1 = F::rand(rng);

        plnk::timeit(|| f1.inverse())
    });

    plnk::bench(b, &label("legendre"), || {
        let f1 = F::rand(rng);

        plnk::timeit(|| f1.legendre())
    });

    plnk::bench(b, &label("sqrt"), || {
        let f1 = F::rand(rng);
        if f1.legendre().is_qr() {
            plnk::timeit(|| f1.sqrt())
        } else {
            Duration::default()
        }
    });

    plnk::bench(b, &label("exponentiation"), || {
        let f1 = F::rand(rng);

        plnk::timeit(|| f1.pow(F::MODULUS))
    });

    plnk::bench(b, &label("into bigint"), || {
        let f1 = F::rand(rng);

        plnk::timeit(|| f1.into_bigint())
    });
}
