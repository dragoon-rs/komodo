use rand::{rngs::ThreadRng, Rng};

pub(crate) fn random_bytes(n: usize, rng: &mut ThreadRng) -> Vec<u8> {
    (0..n).map(|_| rng.gen::<u8>()).collect()
}
