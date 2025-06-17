use ark_ff::PrimeField;
use convert_case::{Case, Casing};
use rand::thread_rng;

crate::make_enum_with_all_variants_array! (Operation ALL_OPERATIONS {
    RandomSampling,
    Addition,
    Substraction,
    Double,
    Multiplication,
    Square,
    Inverse,
    Legendre,
    Sqrt,
    Exponentiation,
    IntoBigint,
});

pub(crate) fn build<F: PrimeField>(ops: &[Operation]) -> Vec<(String, plnk::FnTimed<()>)> {
    ops.iter()
        .map(|op| {
            let bench: plnk::FnTimed<()> = match op {
                Operation::RandomSampling => plnk::closure! { crate::timeit_and_discard_output! {
                    F::rand(&mut thread_rng())
                } },
                Operation::Addition => plnk::closure! {
                    let f1 = F::rand(&mut thread_rng());
                    let f2 = F::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { f1 + f2 }
                },
                Operation::Substraction => plnk::closure! {
                    let f1 = F::rand(&mut thread_rng());
                    let f2 = F::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { f1 - f2 }
                },
                Operation::Double => plnk::closure! {
                    let f1 = F::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { f1.double() }
                },
                Operation::Multiplication => plnk::closure! {
                    let f1 = F::rand(&mut thread_rng());
                    let f2 = F::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { f1 * f2 }
                },
                Operation::Square => plnk::closure! {
                    let f1 = F::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { f1.square() }
                },
                Operation::Inverse => plnk::closure! {
                    let f1 = F::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { f1.inverse().unwrap() }
                },
                Operation::Legendre => plnk::closure! {
                    let f1 = F::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { f1.legendre() }
                },
                Operation::Sqrt => plnk::closure! {
                    let mut f1 = F::rand(&mut thread_rng());
                    while !f1.legendre().is_qr() {
                        f1 = F::rand(&mut thread_rng());
                    };

                    crate::timeit_and_discard_output! { f1.sqrt().unwrap() }
                },
                Operation::Exponentiation => plnk::closure! {
                    let f1 = F::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { f1.pow(F::MODULUS) }
                },
                Operation::IntoBigint => plnk::closure! {
                    let f1 = F::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { f1.into_bigint() }
                },
            };
            (format!("{:?}", op).to_case(Case::Lower), bench)
        })
        .collect::<Vec<_>>()
}
