#[derive(clap::ValueEnum, Clone, Hash, PartialEq, Eq, Debug)]
pub enum Curve {
    ARKBLS12381,
    ARKBN254,
    BLS12381,
    BN254,
    CP6782,
    EDOnMnt4298,
    FQ128,
    MNT4753,
    Pallas,
    SECP256K1,
    SECP256R1,
    Vesta,
}
