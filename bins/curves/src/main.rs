use ark_ff::PrimeField;

fn show_curve<Fr: PrimeField, Fq: PrimeField>(name: &str) {
    println!(
        "{}: {} -> {}",
        name,
        Fq::MODULUS_BIT_SIZE,
        Fr::MODULUS_BIT_SIZE
    );
}

/// takes a sequence of curve crate names and calls the [`show_curve`] function
/// for you
///
/// this macro accepts a trailling comma in case you have a big list that spans
/// over multiple lines, e.g.
///
/// ## examples
/// ```rust
/// show_curve(ark_bls12_381)
/// ```
/// or
/// ```rust
/// show_curve(
///     ark_bls12_381,
///     ark_bn254,
///     ark_pallas,
///     ark_vesta
/// )
/// ```
/// or
/// ```rust
/// show_curve(
///     ark_bls12_381,
///     ark_bn254,
///     ark_pallas,
///     ark_vesta,
/// )
/// ```
macro_rules! show_curve {
    ($($c:ident),+ $(,)?) => {
        $(show_curve::<$c::Fr, $c::Fq>(stringify!($c));)*
    }
}

fn main() {
    show_curve!(
        ark_bls12_377,
        ark_bls12_381,
        ark_bn254,
        ark_bw6_761,
        ark_cp6_782,
        ark_curve25519,
        ark_ed_on_bls12_377,
        ark_ed_on_bls12_381,
        ark_ed_on_bls12_381_bandersnatch,
        ark_ed_on_bn254,
        ark_ed_on_bw6_761,
        ark_ed_on_cp6_782,
        ark_ed_on_mnt4_298,
        ark_ed_on_mnt4_753,
        ark_ed25519,
        ark_mnt4_298,
        ark_mnt4_753,
        ark_mnt6_298,
        ark_mnt6_753,
        ark_pallas,
        ark_secp256k1,
        ark_secp256r1,
        ark_secp384r1,
        ark_secq256k1,
        ark_vesta,
    );
}
