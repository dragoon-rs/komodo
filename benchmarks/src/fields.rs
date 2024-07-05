use ark_ff::{Fp128, MontBackend, MontConfig};

#[derive(MontConfig)]
#[modulus = "340282366920938463463374557953744961537"]
#[generator = "3"]
pub struct Test;
/// A prime, fft-friendly field isomorph to [`winter_math::fields::f128::BaseElement`].
pub type Fq128 = Fp128<MontBackend<Test, 2>>;
