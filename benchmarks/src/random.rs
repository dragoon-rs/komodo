use rand::Rng;

pub(crate) fn random_bytes(n: usize, rng: &mut impl Rng) -> Vec<u8> {
    (0..n).map(|_| rng.gen::<u8>()).collect()
}
