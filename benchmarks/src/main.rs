use ark_ec::pairing::Pairing;
use ark_poly::univariate::DensePolynomial;

use clap::{Parser, Subcommand, ValueEnum};
use dragoonfri::algorithms::Sha3_256;
use dragoonfri::dynamic_folding_factor;
use rand::{RngCore, SeedableRng};

mod aplonk;
#[allow(clippy::identity_op, clippy::precedence, clippy::erasing_op)]
mod conversions;
mod fri;
mod kzg;
mod macros;
mod random;
mod semi_avid;

use crate::aplonk::bench as aplonk_bench;
use crate::fri::{bench as fri_bench, FRIParams};
use crate::kzg::bench as kzg_bench;
use crate::semi_avid::bench as semi_avid_bench;

pub(crate) struct FECParams {
    pub k: usize,
    pub n: usize,
}

fn dump_kv_pairs<T: Into<Vec<(&'static str, Option<u128>)>>>(t: T) {
    println!(
        "{{{}}}",
        t.into()
            .iter()
            .map(|(k, v)| {
                match v {
                    Some(v) => format!("{}:{}", k, v),
                    None => format!("{}:null", k),
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    );
}

macro_rules! ark_gen {
    ($fn:ident: $c:ident, F=$f:ident, G=$g:ident) => {{
        $fn::<$c::$f, $c::$g, DensePolynomial<$c::$f>>
    }};
    ($fn:ident: $c:ident, E=$e:ident) => {{
        $fn::<$c::$e, DensePolynomial<<$c::$e as Pairing>::ScalarField>>
    }};
    ($fn:ident: $c:ident, N=$n:ident, F=$f:ident, H=$h:ident) => {
        dynamic_folding_factor!(
            let N = $n => $fn::<N, $c::$f, $h, DensePolynomial<$c::$f>>
        )
    }
}

#[derive(clap::ValueEnum, Clone, Hash, PartialEq, Eq, Debug)]
enum Protocol {
    SemiAVID,
    Kzg,
    Aplonk,
    Fri,
}

#[derive(clap::ValueEnum, Clone, Hash, PartialEq, Eq, Debug)]
enum Curve {
    Bn254,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    List,
    Run {
        #[arg(long)]
        nb_bytes: usize,
        #[arg(short)]
        k: usize,
        #[arg(short)]
        n: usize,

        #[arg(long)]
        seed: Option<u64>,

        #[arg(long)]
        protocol: Protocol,

        #[arg(long)]
        curve: Curve,

        #[arg(long)]
        fri_ff: Option<usize>,
        #[arg(long)]
        fri_bf: Option<usize>,
        #[arg(long)]
        fri_rpo: Option<usize>,
        #[arg(long)]
        fri_q: Option<usize>,
    },
}

fn main() {
    let args = Cli::parse();

    if let Some(cmd) = args.command {
        match cmd {
            Commands::List => {
                println!(
                    "protocols:{}",
                    Protocol::value_variants()
                        .iter()
                        .map(|p| p.to_possible_value().unwrap().get_name().to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );
                println!(
                    "curves:{}",
                    Curve::value_variants()
                        .iter()
                        .map(|p| p.to_possible_value().unwrap().get_name().to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );
            }
            Commands::Run {
                nb_bytes,
                k,
                n,
                seed,
                protocol,
                curve,
                fri_ff,
                fri_bf,
                fri_rpo,
                fri_q,
            } => {
                let fec_params = FECParams { k, n };

                let mut rng: Box<dyn RngCore> = match seed {
                    Some(seed) => Box::new(rand::rngs::StdRng::seed_from_u64(seed)),
                    None => Box::new(rand::thread_rng()),
                };

                let kv_pairs: Vec<(&'static str, Option<u128>)> = match protocol {
                    Protocol::SemiAVID => match curve {
                        Curve::Bn254 => ark_gen!(semi_avid_bench: ark_bn254, F=Fr, G=G1Projective)(
                            nb_bytes, fec_params, &mut rng,
                        ),
                    }
                    .into(),
                    Protocol::Kzg => match curve {
                        Curve::Bn254 => {
                            ark_gen!(kzg_bench: ark_bn254, E=Bn254)(nb_bytes, fec_params, &mut rng)
                        }
                    }
                    .into(),
                    Protocol::Aplonk => match curve {
                        Curve::Bn254 => ark_gen!(aplonk_bench: ark_bn254, E=Bn254)(
                            nb_bytes, fec_params, &mut rng,
                        ),
                    }
                    .into(),
                    Protocol::Fri => {
                        let ff = fri_ff.unwrap();
                        let fec_params = FECParams {
                            k,
                            n: k * fri_bf.unwrap(),
                        };
                        let fri_params = FRIParams {
                            bf: fri_bf.unwrap(),
                            rpo: fri_rpo.unwrap(),
                            q: fri_q.unwrap(),
                        };
                        match curve {
                            Curve::Bn254 => ark_gen!(fri_bench: ark_bn254, N=ff, F=Fr, H=Sha3_256)(
                                nb_bytes, fec_params, fri_params, &mut rng,
                            ),
                        }
                        .into()
                    }
                };

                dump_kv_pairs(kv_pairs);
            }
        }
    }
}
