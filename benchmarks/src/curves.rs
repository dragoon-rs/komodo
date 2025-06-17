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

impl std::fmt::Display for Curve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            Curve::ARKBLS12381 => "BLS12-381-ARK",
            Curve::ARKBN254 => "BN254-ARK",
            Curve::BLS12381 => "BLS12-381",
            Curve::BN254 => "BN254",
            Curve::CP6782 => "CP6-782",
            Curve::EDOnMnt4298 => "ED-MNT4-298",
            Curve::FQ128 => "FQ128",
            Curve::MNT4753 => "MNT4-753",
            Curve::Pallas => "PALLAS",
            Curve::SECP256K1 => "SECP256-K1",
            Curve::SECP256R1 => "SECP256-R1",
            Curve::Vesta => "VESTA",
        };
        write!(f, "{}", repr)
    }
}
