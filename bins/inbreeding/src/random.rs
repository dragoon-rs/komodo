use rand::{seq::SliceRandom, Rng, RngCore};
use std::collections::HashSet;

#[allow(dead_code)]
fn draw_unique_indices(n: usize, vec_len: usize, rng: &mut impl RngCore) -> HashSet<usize> {
    let mut indices = HashSet::new();

    while indices.len() < n {
        let idx = rng.gen_range(0..vec_len);
        indices.insert(idx);
    }

    indices
}

pub(super) fn draw_unique_elements<T: Clone>(
    things: &[T],
    n: usize,
    rng: &mut impl RngCore,
) -> Vec<T> {
    let mut things = things.to_vec();
    things.shuffle(rng);

    things.iter().take(n).cloned().collect()
}
