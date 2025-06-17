use std::path::PathBuf;

use ark_ec::pairing::Pairing;
use ark_poly::univariate::DensePolynomial;
use benches::fec::Encoding;
use clap::{command, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::iproduct;

mod benches;
mod curves;
mod fields;
mod macros;
mod pretty;
mod random;

use crate::{
    benches::{curve_group, field},
    curves::Curve,
};

fn parse_hex(hex: &str, flag: &str) -> Vec<u8> {
    match hex::decode(hex) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to decode `--{}` hex string: {}", flag, e);
            std::process::exit(1)
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    nb_measurements: usize,

    /// the hash of the Git repo
    #[arg(long)]
    git: String,
    /// the hash of the source code
    #[arg(long)]
    src: String,
    /// the hash of the CPU
    #[arg(long)]
    cpu: String,
    /// the Rust build
    #[arg(long)]
    rust_build: String,

    #[arg(long)]
    overwrite: bool,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    Setup {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        degrees: Vec<usize>,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,
    },
    Commit {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        degrees: Vec<usize>,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,
    },
    Field {
        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        ops: Vec<field::Operation>,

        #[arg(short, long)]
        all: bool,
    },
    Group {
        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        ops: Vec<curve_group::Operation>,

        #[arg(short, long)]
        all: bool,
    },
    Linalg {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        sizes: Vec<usize>,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,
    },
    Fec {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        sizes: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        ks: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        rhos: Vec<f64>,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,

        #[arg(short, long)]
        encoding: Encoding,

        #[arg(long, num_args=1.., value_delimiter = ',', default_values = vec!["encode", "decode"])]
        steps: Vec<String>,
    },
    Recoding {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        sizes: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        ks: Vec<usize>,

        #[arg(long, num_args = 1.., value_delimiter = ' ')]
        shards: Vec<usize>,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,
    },
    SemiAVID {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        sizes: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        ks: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        rhos: Vec<f64>,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,

        #[arg(long, num_args=1.., value_delimiter = ',', default_values = vec!["commit", "proof", "verify"])]
        steps: Vec<String>,
    },
    #[allow(clippy::upper_case_acronyms)]
    KZG {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        sizes: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        ks: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        rhos: Vec<f64>,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,

        #[arg(long, num_args=1.., value_delimiter = ',', default_values = vec!["commit", "proof", "verify"])]
        steps: Vec<String>,
    },
    Aplonk {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        sizes: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        ks: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        rhos: Vec<f64>,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,

        #[arg(long, num_args=1.., value_delimiter = ',', default_values = vec!["commit", "proof", "verify"])]
        steps: Vec<String>,
    },
}

impl std::fmt::Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            Self::Setup { .. } => "setup",
            Self::Commit { .. } => "commit",
            Self::Field { .. } => "field",
            Self::Group { .. } => "group",
            Self::Linalg { .. } => "linalg",
            Self::Fec { .. } => "fec",
            Self::Recoding { .. } => "recoding",
            Self::SemiAVID { .. } => "semi_avid",
            Self::KZG { .. } => "kzg",
            Self::Aplonk { .. } => "aplonk",
        };
        write!(f, "{}", repr)
    }
}

macro_rules! unsupported {
    ($name:expr, $operation:expr) => {{
        eprintln!("{} unsupported for {}", $name, $operation);
        std::process::exit(1);
    }};
}

fn setup(degrees: &[usize], curves: &[Curve]) -> Vec<plnk::LabeledFnTimed<()>> {
    let mut benches = Vec::new();
    for (&deg, curve) in iproduct!(degrees, curves) {
        macro_rules! bench {
            ($c:ident, E=$e:ident) => {
                benches::setup::ark_build::<$c::$e, DensePolynomial<<$c::$e as Pairing>::ScalarField>>(
                    deg,
                )
            };
            ($c:ident, F=$f:ident, G1=$g:ident) => {
                benches::setup::build::<$c::$f, $c::$g, DensePolynomial<$c::$f>>(deg)
            };
        }
        #[rustfmt::skip]
        let func = match curve {
            Curve::ARKBLS12381 => bench!(ark_bls12_381,      E = Bls12_381                 ),
            Curve::ARKBN254    => bench!(ark_bn254,          E = Bn254                     ),
            Curve::BLS12381    => bench!(ark_bls12_381,      F = Fr, G1 = G1Projective     ),
            Curve::BN254       => bench!(ark_bn254,          F = Fr, G1 = G1Projective     ),
            Curve::CP6782      => bench!(ark_cp6_782,        F = Fr, G1 = G1Projective     ),
            Curve::EDOnMnt4298 => bench!(ark_ed_on_mnt4_298, F = Fr, G1 = EdwardsProjective),
            Curve::MNT4753     => bench!(ark_mnt4_753,       F = Fr, G1 = G1Projective     ),
            Curve::Pallas      => bench!(ark_pallas,         F = Fr, G1 = Projective       ),
            Curve::SECP256K1   => bench!(ark_secp256k1,      F = Fr, G1 = Projective       ),
            Curve::SECP256R1   => bench!(ark_secp256r1,      F = Fr, G1 = Projective       ),
            Curve::Vesta       => bench!(ark_vesta,          F = Fr, G1 = Projective       ),

            Curve::FQ128       => unsupported!("FQ128", "setup"),
        };

        benches.push(plnk::LabeledFnTimed {
            label: plnk::label! { curve: curve.to_string(), degree: deg }.to_string(),
            func,
        });
    }

    benches
}

fn commit(degrees: &[usize], curves: &[Curve]) -> Vec<plnk::LabeledFnTimed<()>> {
    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.yellow/orange} {pos:>10}/{len:10} {msg}",
    )
    .unwrap()
    .progress_chars("##-");
    let setup_pb = ProgressBar::new((curves.len() * degrees.len()) as u64)
        .with_style(style.clone())
        .with_message("(setup) building ZK setups 0 b 0");

    let mut benches = Vec::new();
    for (&deg, curve) in iproduct!(degrees, curves) {
        macro_rules! bench {
            ($c:ident, G1=$g:ident) => {
                benches::commit::build::<$c::Fr, $c::$g, DensePolynomial<$c::Fr>>(deg, &setup_pb)
            };
            ($c:ident, E=$e:ident) => {
                benches::commit::ark_build::<
                    $c::$e,
                    DensePolynomial<<$c::$e as Pairing>::ScalarField>,
                >(deg, &setup_pb)
            };
        }
        #[rustfmt::skip]
        let func = match curve {
            Curve::ARKBLS12381 => bench!(ark_bls12_381      ,  E = Bls12_381        ),
            Curve::ARKBN254    => bench!(ark_bn254          ,  E = Bn254            ),
            Curve::BLS12381    => bench!(ark_bls12_381      , G1 = G1Projective     ),
            Curve::BN254       => bench!(ark_bn254          , G1 = G1Projective     ),
            Curve::CP6782      => bench!(ark_cp6_782        , G1 = G1Projective     ),
            Curve::EDOnMnt4298 => bench!(ark_ed_on_mnt4_298 , G1 = EdwardsProjective),
            Curve::MNT4753     => bench!(ark_mnt4_753       , G1 = G1Projective     ),
            Curve::Pallas      => bench!(ark_pallas         , G1 = Projective       ),
            Curve::SECP256K1   => bench!(ark_secp256k1      , G1 = Projective       ),
            Curve::SECP256R1   => bench!(ark_secp256r1      , G1 = Projective       ),
            Curve::Vesta       => bench!(ark_vesta          , G1 = Projective       ),

            Curve::FQ128       => unsupported!("FQ128", "commit"),
        };

        benches.push(plnk::LabeledFnTimed {
            label: plnk::label! { curve: curve.to_string(), degree: deg }.to_string(),
            func,
        });
    }

    setup_pb.finish_with_message(format!("{} done", setup_pb.message()));

    benches
}

fn field(curves: &[Curve], ops: &[field::Operation]) -> Vec<plnk::LabeledFnTimed<()>> {
    macro_rules! bench {
        ($f:path) => {
            benches::field::build::<$f>(ops)
        };
    }

    let mut benches = Vec::new();
    for curve in curves {
        #[rustfmt::skip]
        let funcs = match curve {
            Curve::ARKBLS12381 => bench!(ark_bls12_381::Fr),
            Curve::ARKBN254    => bench!(ark_bn254::Fr),
            Curve::BLS12381    => bench!(ark_bls12_381::Fr),
            Curve::BN254       => bench!(ark_bn254::Fr),
            Curve::CP6782      => bench!(ark_cp6_782::Fr),
            Curve::EDOnMnt4298 => bench!(ark_ed_on_mnt4_298::Fr),
            Curve::FQ128       => bench!(fields::Fq128),
            Curve::MNT4753     => bench!(ark_mnt4_753::Fr),
            Curve::Pallas      => bench!(ark_pallas::Fr),
            Curve::SECP256K1   => bench!(ark_secp256k1::Fr),
            Curve::SECP256R1   => bench!(ark_secp256r1::Fr),
            Curve::Vesta       => bench!(ark_vesta::Fr),
        };

        for (op, func) in funcs {
            benches.push(plnk::LabeledFnTimed {
                label: plnk::label! { curve: curve.to_string(), operation: op }.to_string(),
                func,
            });
        }
    }

    benches
}

fn curve_group(curves: &[Curve], ops: &[curve_group::Operation]) -> Vec<plnk::LabeledFnTimed<()>> {
    macro_rules! bench {
        ($c:ident, G1=$g:ident) => {
            benches::curve_group::build::<$c::Fr, $c::$g>(ops)
        };
    }

    let mut benches = Vec::new();
    for curve in curves {
        #[rustfmt::skip]
        let funcs = match curve {
            Curve::ARKBLS12381 => bench!(ark_bls12_381      , G1 = G1Projective     ),
            Curve::ARKBN254    => bench!(ark_bn254          , G1 = G1Projective     ),
            Curve::BLS12381    => bench!(ark_bls12_381      , G1 = G1Projective     ),
            Curve::BN254       => bench!(ark_bn254          , G1 = G1Projective     ),
            Curve::CP6782      => bench!(ark_cp6_782        , G1 = G1Projective     ),
            Curve::EDOnMnt4298 => bench!(ark_ed_on_mnt4_298 , G1 = EdwardsProjective),
            Curve::MNT4753     => bench!(ark_mnt4_753       , G1 = G1Projective     ),
            Curve::Pallas      => bench!(ark_pallas         , G1 = Projective       ),
            Curve::SECP256K1   => bench!(ark_secp256k1      , G1 = Projective       ),
            Curve::SECP256R1   => bench!(ark_secp256r1      , G1 = Projective       ),
            Curve::Vesta       => bench!(ark_vesta          , G1 = Projective       ),

            Curve::FQ128       => unsupported!("FQ128", "group"),
        };

        for (op, func) in funcs {
            benches.push(plnk::LabeledFnTimed {
                label: plnk::label! { curve: curve.to_string(), operation: op }.to_string(),
                func,
            });
        }
    }

    benches
}

fn linalg(sizes: &[usize], curves: &[Curve]) -> Vec<plnk::LabeledFnTimed<()>> {
    let mut benches = Vec::new();
    for (&size, curve) in iproduct!(sizes, curves) {
        macro_rules! bench {
            ($f:path) => {
                vec![
                    ("inverse", benches::linalg::inverse::<$f>(size)),
                    ("transpose", benches::linalg::transpose::<$f>(size)),
                    ("multiply", benches::linalg::multiply::<$f>(size)),
                ]
            };
        }
        #[rustfmt::skip]
        let funcs = match curve {
            Curve::ARKBLS12381 => bench!(ark_bls12_381::Fr),
            Curve::ARKBN254    => bench!(ark_bn254::Fr),
            Curve::BLS12381    => bench!(ark_bls12_381::Fr),
            Curve::BN254       => bench!(ark_bn254::Fr),
            Curve::CP6782      => bench!(ark_cp6_782::Fr),
            Curve::EDOnMnt4298 => bench!(ark_ed_on_mnt4_298::Fr),
            Curve::FQ128       => bench!(fields::Fq128),
            Curve::MNT4753     => bench!(ark_mnt4_753::Fr),
            Curve::Pallas      => bench!(ark_pallas::Fr),
            Curve::SECP256K1   => bench!(ark_secp256k1::Fr),
            Curve::SECP256R1   => bench!(ark_secp256r1::Fr),
            Curve::Vesta       => bench!(ark_vesta::Fr),
        };
        for (op, func) in funcs {
            benches.push(plnk::LabeledFnTimed {
                label: plnk::label! { curve: curve.to_string(), operation: op, size: size }
                    .to_string(),
                func,
            });
        }
    }

    benches
}

fn fec(
    sizes: &[usize],
    params: &[(usize, usize)],
    curves: &[Curve],
    encoding: benches::fec::Encoding,
    steps: &[String],
) -> Vec<plnk::LabeledFnTimed<()>> {
    let mut benches = Vec::new();
    for (&nb_bytes, (k, n), curve) in iproduct!(sizes, params, curves) {
        macro_rules! bench {
            ($f:path) => {
                benches::fec::build::<$f>(nb_bytes, *k, *n, &encoding)
            };
        }
        #[rustfmt::skip]
        let funcs = match curve {
            Curve::BLS12381    => bench!(ark_bls12_381::Fr),
            Curve::BN254       => bench!(ark_bn254::Fr),
            Curve::CP6782      => bench!(ark_cp6_782::Fr),
            Curve::EDOnMnt4298 => bench!(ark_ed_on_mnt4_298::Fr),
            Curve::FQ128       => bench!(fields::Fq128),
            Curve::MNT4753     => bench!(ark_mnt4_753::Fr),
            Curve::Pallas      => bench!(ark_pallas::Fr),
            Curve::SECP256K1   => bench!(ark_secp256k1::Fr),
            Curve::SECP256R1   => bench!(ark_secp256r1::Fr),
            Curve::Vesta       => bench!(ark_vesta::Fr),

            Curve::ARKBLS12381 => unsupported!("ARKBLS12381", "fec"),
            Curve::ARKBN254    => unsupported!("ARKBN254", "fec"),
        };
        for (step, func) in funcs {
            if steps.contains(&step) {
                benches.push(plnk::LabeledFnTimed {
                    label: plnk::label! {
                        curve: curve.to_string(),
                        nb_bytes: nb_bytes,
                        k: k,
                        n: n,
                        step: step,
                    }
                    .to_string(),
                    func,
                });
            }
        }
    }

    benches
}

fn recoding(
    sizes: &[usize],
    ks: &[usize],
    shards: &[usize],
    curves: &[Curve],
) -> Vec<plnk::LabeledFnTimed<()>> {
    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.yellow/orange} {pos:>10}/{len:10} {msg}",
    )
    .unwrap()
    .progress_chars("##-");
    let setup_pb = ProgressBar::new((sizes.len() * ks.len() * shards.len() * curves.len()) as u64)
        .with_style(style.clone())
        .with_message("(setup) creating shards 0 b 0");

    let mut benches = Vec::new();
    for (&nb_bytes, &nb_shards, &k, curve) in iproduct!(sizes, shards, ks, curves) {
        macro_rules! bench {
            ($f:path) => {
                benches::recoding::build::<$f>(nb_bytes, k, nb_shards, &setup_pb)
            };
        }
        #[rustfmt::skip]
        let func = match curve {
            Curve::ARKBLS12381 => bench!(ark_bls12_381::Fr     ),
            Curve::ARKBN254    => bench!(ark_bn254::Fr         ),
            Curve::BLS12381    => bench!(ark_bls12_381::Fr     ),
            Curve::BN254       => bench!(ark_bn254::Fr         ),
            Curve::CP6782      => bench!(ark_cp6_782::Fr       ),
            Curve::EDOnMnt4298 => bench!(ark_ed_on_mnt4_298::Fr),
            Curve::FQ128       => bench!(fields::Fq128         ),
            Curve::MNT4753     => bench!(ark_mnt4_753::Fr      ),
            Curve::Pallas      => bench!(ark_pallas::Fr        ),
            Curve::SECP256K1   => bench!(ark_secp256k1::Fr     ),
            Curve::SECP256R1   => bench!(ark_secp256r1::Fr     ),
            Curve::Vesta       => bench!(ark_vesta::Fr         ),
        };
        benches.push(plnk::LabeledFnTimed {
            label: plnk::label! {
                curve: curve.to_string(),
                nb_bytes: nb_bytes,
                nb_shards: nb_shards,
                k: k,
                step: "recode",
            }
            .to_string(),
            func,
        });
    }

    benches
}

fn semi_avid(
    sizes: &[usize],
    params: &[(usize, usize)],
    curves: &[Curve],
    steps: &[String],
) -> Vec<plnk::LabeledFnTimed<()>> {
    let mut benches = Vec::new();
    for (&nb_bytes, (k, n), curve) in iproduct!(sizes, params, curves) {
        macro_rules! bench {
            ($c:ident, F=$f:ident, G1=$g:ident) => {{
                benches::semi_avid::build::<$c::$f, $c::$g, DensePolynomial<$c::$f>>(
                    *k, *n, nb_bytes,
                )
            }};
        }
        #[rustfmt::skip]
        let funcs = match curve {
            Curve::BLS12381    => bench!(ark_bls12_381     , F = Fr, G1 = G1Projective),
            Curve::BN254       => bench!(ark_bn254         , F = Fr, G1 = G1Projective),
            Curve::CP6782      => bench!(ark_cp6_782       , F = Fr, G1 = G1Projective),
            Curve::EDOnMnt4298 => bench!(ark_ed_on_mnt4_298, F = Fr, G1 = EdwardsProjective),
            Curve::MNT4753     => bench!(ark_mnt4_753      , F = Fr, G1 = G1Projective),
            Curve::Pallas      => bench!(ark_pallas        , F = Fr, G1 = Projective),
            Curve::SECP256K1   => bench!(ark_secp256k1     , F = Fr, G1 = Projective),
            Curve::SECP256R1   => bench!(ark_secp256r1     , F = Fr, G1 = Projective),
            Curve::Vesta       => bench!(ark_vesta         , F = Fr, G1 = Projective),

            Curve::ARKBLS12381 => unsupported!("ARKBLS12381", "semi_avid"),
            Curve::ARKBN254    => unsupported!("ARKBN254", "semi_avid"),
            Curve::FQ128       => unsupported!("FQ128", "semi_avid"),
        };
        for (step, func) in funcs {
            if steps.contains(&step) {
                benches.push(plnk::LabeledFnTimed {
                    label: plnk::label! {
                        protocol: "semi_avid",
                        curve: curve.to_string(),
                        nb_bytes: nb_bytes,
                        k: k,
                        n: n,
                        step: step,
                    }
                    .to_string(),
                    func,
                });
            }
        }
    }

    benches
}

fn kzg(
    sizes: &[usize],
    params: &[(usize, usize)],
    curves: &[Curve],
    steps: &[String],
) -> Vec<plnk::LabeledFnTimed<()>> {
    let mut benches = Vec::new();
    for (&nb_bytes, (k, n), curve) in iproduct!(sizes, params, curves) {
        macro_rules! bench {
            ($c:ident, F=$f:ident, C=$c_:ident) => {{
                benches::kzg::build::<$c::$c_, DensePolynomial<$c::$f>>(*k, *n, nb_bytes)
            }};
        }
        #[rustfmt::skip]
        let funcs = match curve {
            Curve::BLS12381    => bench!(ark_bls12_381 , F = Fr, C = Bls12_381),
            Curve::BN254       => bench!(ark_bn254     , F = Fr, C = Bn254    ),
            Curve::CP6782      => bench!(ark_cp6_782   , F = Fr, C = CP6_782  ),
            Curve::MNT4753     => bench!(ark_mnt4_753  , F = Fr, C = MNT4_753 ),

            Curve::ARKBLS12381 => unsupported!("ARKBLS12381", "kzg"),
            Curve::ARKBN254    => unsupported!("ARKBN254", "kzg"),
            Curve::EDOnMnt4298 => unsupported!("EDOnMnt4298", "kzg"),
            Curve::FQ128       => unsupported!("FQ128", "kzg"),
            Curve::Pallas      => unsupported!("Pallas", "kzg"),
            Curve::SECP256K1   => unsupported!("SECP256K1", "kzg"),
            Curve::SECP256R1   => unsupported!("SECP256R1", "kzg"),
            Curve::Vesta       => unsupported!("Vesta", "kzg"),
        };
        for (step, func) in funcs {
            if steps.contains(&step) {
                benches.push(plnk::LabeledFnTimed {
                    label: plnk::label! {
                        protocol: "kzg",
                        curve: curve.to_string(),
                        nb_bytes: nb_bytes,
                        k: k,
                        n: n,
                        step: step,
                    }
                    .to_string(),
                    func,
                });
            }
        }
    }

    benches
}

fn aplonk(
    sizes: &[usize],
    params: &[(usize, usize)],
    curves: &[Curve],
    steps: &[String],
) -> Vec<plnk::LabeledFnTimed<()>> {
    let mut benches = Vec::new();
    for (&nb_bytes, (k, n), curve) in iproduct!(sizes, params, curves) {
        macro_rules! bench {
            ($crate_:ident, F=$field:ident, C=$curve:ident) => {{
                benches::aplonk::build::<$crate_::$curve, DensePolynomial<$crate_::$field>>(
                    *k, *n, nb_bytes,
                )
            }};
        }
        #[rustfmt::skip]
        let funcs = match curve {
            Curve::BLS12381    => bench!(ark_bls12_381 , F = Fr , C = Bls12_381),
            Curve::BN254       => bench!(ark_bn254     , F = Fr , C = Bn254    ),
            Curve::CP6782      => bench!(ark_cp6_782   , F = Fr , C = CP6_782  ),
            Curve::MNT4753     => bench!(ark_mnt4_753  , F = Fr , C = MNT4_753 ),

            Curve::ARKBLS12381 => unsupported!("ARKBLS12381", "kzg"),
            Curve::ARKBN254    => unsupported!("ARKBN254", "kzg"),
            Curve::EDOnMnt4298 => unsupported!("EDOnMnt4298", "kzg"),
            Curve::FQ128       => unsupported!("FQ128", "kzg"),
            Curve::Pallas      => unsupported!("Pallas", "kzg"),
            Curve::SECP256K1   => unsupported!("SECP256K1", "kzg"),
            Curve::SECP256R1   => unsupported!("SECP256R1", "kzg"),
            Curve::Vesta       => unsupported!("Vesta", "kzg"),
        };
        for (step, func) in funcs {
            if steps.contains(&step) {
                benches.push(plnk::LabeledFnTimed {
                    label: plnk::label! {
                        protocol: "aplonk",
                        curve: curve.to_string(),
                        nb_bytes: nb_bytes,
                        k: k,
                        n: n,
                        step: step,
                    }
                    .to_string(),
                    func,
                });
            }
        }
    }

    benches
}

fn main() {
    let cli = Cli::parse();

    if cli.command.is_none() {
        eprintln!("WARNING: nothing to do");
        std::process::exit(0);
    }

    let command = cli.command.unwrap();

    let benches = match command.clone() {
        Commands::Setup { degrees, curves } => setup(&degrees, &curves),
        Commands::Commit { degrees, curves } => commit(&degrees, &curves),
        Commands::Field { curves, ops, all } => field(
            &curves,
            &if all {
                field::ALL_OPERATIONS.to_vec()
            } else {
                ops
            },
        ),
        Commands::Group { curves, ops, all } => curve_group(
            &curves,
            &if all {
                curve_group::ALL_OPERATIONS.to_vec()
            } else {
                ops
            },
        ),
        Commands::Linalg { sizes, curves } => linalg(&sizes, &curves),
        Commands::Fec {
            sizes,
            ks,
            rhos,
            curves,
            encoding,
            steps,
        } => {
            let params = iproduct!(ks, rhos)
                .map(|(k, r)| (k, ((k as f64) / r).round() as usize))
                .collect::<Vec<(usize, usize)>>();
            fec(&sizes, &params, &curves, encoding.clone(), &steps)
        }
        Commands::Recoding {
            sizes,
            shards,
            ks,
            curves,
        } => recoding(&sizes, &ks, &shards, &curves),
        Commands::SemiAVID {
            sizes,
            ks,
            rhos,
            curves,
            steps,
        } => {
            let params = iproduct!(ks, rhos)
                .map(|(k, r)| (k, ((k as f64) / r).round() as usize))
                .collect::<Vec<(usize, usize)>>();
            eprintln!("steps: {:?}", steps);
            semi_avid(&sizes, &params, &curves, &steps)
        }
        Commands::KZG {
            sizes,
            ks,
            rhos,
            curves,
            steps,
        } => {
            let params = iproduct!(ks, rhos)
                .map(|(k, r)| (k, ((k as f64) / r).round() as usize))
                .collect::<Vec<(usize, usize)>>();
            kzg(&sizes, &params, &curves, &steps)
        }
        Commands::Aplonk {
            sizes,
            ks,
            rhos,
            curves,
            steps,
        } => {
            let params = iproduct!(ks, rhos)
                .map(|(k, r)| (k, ((k as f64) / r).round() as usize))
                .collect::<Vec<(usize, usize)>>();
            aplonk(&sizes, &params, &curves, &steps)
        }
    };

    let mut bencher = plnk::Bencher::new(cli.nb_measurements).with_name(plnk::label! {
        command: command.to_string(),
        git: hex::encode(parse_hex(&cli.git, "git")),
        cpu: hex::encode(parse_hex(&cli.cpu, "cpu")),
        src: hex::encode(parse_hex(&cli.src, "src")),
        build: cli.rust_build,
    });
    if let Some(file) = cli.output {
        if cli.overwrite {
            bencher = bencher.with_file(file).overwrite();
        } else {
            bencher = bencher.with_file(file).append();
        }
    }
    bencher.bench_multiple(benches)
}
