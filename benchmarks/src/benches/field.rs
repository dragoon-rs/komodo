use std::time::Duration;

use ark_ff::PrimeField;

pub(crate) fn run<F: PrimeField>(b: &plnk::Bencher) {
    let rng = &mut ark_std::rand::thread_rng();

    plnk::bench(
        b,
        crate::label! { operation: r#"\"random sampling\""# },
        || plnk::timeit(|| F::rand(rng)),
    );

    plnk::bench(b, crate::label! { operation: r#"\"addition\""# }, || {
        let f1 = F::rand(rng);
        let f2 = F::rand(rng);

        plnk::timeit(|| f1 + f2)
    });

    plnk::bench(
        b,
        crate::label! { operation: r#"\"substraction\""# },
        || {
            let f1 = F::rand(rng);
            let f2 = F::rand(rng);

            plnk::timeit(|| f1 - f2)
        },
    );

    plnk::bench(b, crate::label! { operation: r#"\"double\""# }, || {
        let f1 = F::rand(rng);

        plnk::timeit(|| f1.double())
    });

    plnk::bench(
        b,
        crate::label! { operation: r#"\"multiplication\""# },
        || {
            let f1 = F::rand(rng);
            let f2 = F::rand(rng);

            plnk::timeit(|| f1 * f2)
        },
    );

    plnk::bench(b, crate::label! { operation: r#"\"square\""# }, || {
        let f1 = F::rand(rng);

        plnk::timeit(|| f1.square())
    });

    plnk::bench(b, crate::label! { operation: r#"\"inverse\""# }, || {
        let f1 = F::rand(rng);

        plnk::timeit(|| f1.inverse())
    });

    plnk::bench(b, crate::label! { operation: r#"\"legendre\""# }, || {
        let f1 = F::rand(rng);

        plnk::timeit(|| f1.legendre())
    });

    plnk::bench(b, crate::label! { operation: r#"\"sqrt\""# }, || {
        let f1 = F::rand(rng);
        if f1.legendre().is_qr() {
            plnk::timeit(|| f1.sqrt())
        } else {
            Duration::default()
        }
    });

    plnk::bench(
        b,
        crate::label! { operation: r#"\"exponentiation\""# },
        || {
            let f1 = F::rand(rng);

            plnk::timeit(|| f1.pow(F::MODULUS))
        },
    );

    plnk::bench(b, crate::label! { operation: r#"\"into bigint\""# }, || {
        let f1 = F::rand(rng);

        plnk::timeit(|| f1.into_bigint())
    });
}
