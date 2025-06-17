use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use convert_case::{Case, Casing};
use plnk::FnTimed;
use rand::thread_rng;

crate::make_enum_with_all_variants_array!(Operation ALL_OPERATIONS {
    RandomSampling,
    Addition,
    Substraction,
    Double,
    ScalarMultiplication,
    IntoAffine,
    FromAffine,
    AffineAddition,
    AffineScalarMultiplication,
});

pub(crate) fn build<F: PrimeField, G: CurveGroup<ScalarField = F>>(
    ops: &[Operation],
) -> Vec<(String, FnTimed<()>)> {
    ops.iter()
        .map(|op| {
            let bench: FnTimed<()> = match op {
                Operation::RandomSampling => plnk::closure! { crate::timeit_and_discard_output! {
                    G::rand(&mut thread_rng())
                } },
                Operation::Addition => plnk::closure! {
                    let g1 = G::rand(&mut thread_rng());
                    let g2 = G::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { g1 + g2 }
                },
                Operation::Substraction => plnk::closure! {
                    let g1 = G::rand(&mut thread_rng());
                    let g2 = G::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { g1 - g2 }
                },
                Operation::Double => plnk::closure! {
                    let g1 = G::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { g1.double() }
                },
                Operation::ScalarMultiplication => plnk::closure! {
                    let g1 = G::rand(&mut thread_rng());
                    let f1 = F::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { g1.mul(f1) }
                },
                Operation::IntoAffine => plnk::closure! {
                    let g1 = G::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { g1.into_affine() }
                },
                Operation::FromAffine => plnk::closure! {
                    let g1_affine = G::rand(&mut thread_rng()).into_affine();

                    crate::timeit_and_discard_output! { Into::<G>::into(g1_affine) }
                },
                Operation::AffineAddition => plnk::closure! {
                    let g1_affine = G::rand(&mut thread_rng()).into_affine();
                    let g2_affine = G::rand(&mut thread_rng()).into_affine();

                    crate::timeit_and_discard_output! { g1_affine + g2_affine }
                },
                Operation::AffineScalarMultiplication => plnk::closure! {
                    let g1_affine = G::rand(&mut thread_rng()).into_affine();
                    let f1 = F::rand(&mut thread_rng());

                    crate::timeit_and_discard_output! { g1_affine * f1 }
                },
            };
            (format!("{:?}", op).to_case(Case::Lower), bench)
        })
        .collect::<Vec<_>>()
}
