use benches::fec::Encoding;
use clap::{command, Parser, Subcommand};

use ark_ec::pairing::Pairing;
use ark_poly::univariate::DensePolynomial;

mod benches;
mod curves;
mod fields;
mod random;

use curves::Curve;
use itertools::iproduct;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Setup {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        degrees: Vec<usize>,

        #[arg(short, long)]
        nb_measurements: usize,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,
    },
    Commit {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        degrees: Vec<usize>,

        #[arg(short, long)]
        nb_measurements: usize,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,
    },
    Field {
        #[arg(short, long)]
        nb_measurements: usize,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,
    },
    Group {
        #[arg(short, long)]
        nb_measurements: usize,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,
    },
    Linalg {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        sizes: Vec<usize>,

        #[arg(short, long)]
        nb_measurements: usize,

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

        #[arg(short, long)]
        nb_measurements: usize,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,

        #[arg(short, long)]
        encoding: Encoding,
    },
    Recoding {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        sizes: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        ks: Vec<usize>,

        #[arg(long, num_args = 1.., value_delimiter = ' ')]
        shards: Vec<usize>,

        #[arg(short, long)]
        nb_measurements: usize,

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

        #[arg(short, long)]
        nb_measurements: usize,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,
    },
    #[allow(clippy::upper_case_acronyms)]
    KZG {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        sizes: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        ks: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        rhos: Vec<f64>,

        #[arg(short, long)]
        nb_measurements: usize,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,
    },
    Aplonk {
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        sizes: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        ks: Vec<usize>,

        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        rhos: Vec<f64>,

        #[arg(short, long)]
        nb_measurements: usize,

        #[arg(short, long, num_args=1.., value_delimiter = ',')]
        curves: Vec<Curve>,
    },
}

macro_rules! unsupported {
    ($name:expr, $operation:expr) => {{
        eprintln!("{} unsupported for {}", $name, $operation);
        std::process::exit(1);
    }};
}

#[rustfmt::skip]
fn setup(degrees: &[usize], nb_measurements: usize, curves: &[Curve]) {
    macro_rules! bench {
        ($c:ident, $e:ident, $d:expr, $label:tt) => {
            benches::setup::ark_run::<$c::$e, DensePolynomial<<$c::$e as Pairing>::ScalarField>>(
                $d,
                nb_measurements,
                $label,
            )
        };
        ($c:ident, $f:ident, $g:ident, $d:expr, $label:tt) => {
            benches::setup::run::<$c::$f, $c::$g, DensePolynomial<$c::$f>>($d, nb_measurements, $label)
        };
    }

    for (&deg, curve) in iproduct!(degrees, curves) {
        match curve {
            Curve::ARKBLS12381 => { bench!(ark_bls12_381,      Bls12_381                   , deg , "BLS12-381"  ) }
            Curve::ARKBN254    => { bench!(ark_bn254,          Bn254                       , deg , "BN254"      ) }
            Curve::BLS12381    => { bench!(ark_bls12_381,      Fr,        G1Projective     , deg , "BLS12-381"  ) }
            Curve::BN254       => { bench!(ark_bn254,          Fr,        G1Projective     , deg , "BN254"      ) }
            Curve::CP6782      => { bench!(ark_cp6_782,        Fr,        G1Projective     , deg , "CP6-782"    ) }
            Curve::EDOnMnt4298 => { bench!(ark_ed_on_mnt4_298, Fr,        EdwardsProjective, deg , "ED-MNT4-298") }
            Curve::FQ128       => unsupported!("FQ128", "setup"),
            Curve::MNT4753     => { bench!(ark_mnt4_753,       Fr,        G1Projective     , deg , "MNT4-753"   ) }
            Curve::Pallas      => { bench!(ark_pallas,         Fr,        Projective       , deg , "PALLAS"     ) }
            Curve::SECP256K1   => { bench!(ark_secp256k1,      Fr,        Projective       , deg , "SECP256-K1" ) }
            Curve::SECP256R1   => { bench!(ark_secp256r1,      Fr,        Projective       , deg , "SECP256-R1" ) }
            Curve::Vesta       => { bench!(ark_vesta,          Fr,        Projective       , deg , "VESTA"      ) }
        }
    }
}

#[rustfmt::skip]
fn commit(degrees: &[usize], nb_measurements: usize, curves: &[Curve]) {
    macro_rules! bench {
        ($c:ident, G1=$g:ident, $d:expr, name=$n:expr) => {
            benches::commit::run::<$c::Fr, $c::$g, DensePolynomial<$c::Fr>>($d, nb_measurements, $n)
        };
        ($c:ident, G1=$g:ident, E=$e:ident, $d:expr, name=$n:expr) => {
            benches::commit::ark_run::<$c::$e, DensePolynomial<<$c::$e as Pairing>::ScalarField>>($d, nb_measurements, concat!($n, "-ark"))
        };
    }

    for (&deg, curve) in iproduct!(degrees, curves) {
        match curve {
            Curve::ARKBLS12381 => { bench!(ark_bls12_381      , G1 = G1Projective      , E = Bls12_381 , deg, name = "BLS12-381"  ) }
            Curve::ARKBN254    => { bench!(ark_bn254          , G1 = G1Projective      , E = Bn254     , deg, name = "BN254"      ) }
            Curve::BLS12381    => { bench!(ark_bls12_381      , G1 = G1Projective                      , deg, name = "BLS12-381"  ) }
            Curve::BN254       => { bench!(ark_bn254          , G1 = G1Projective                      , deg, name = "BN254"      ) }
            Curve::CP6782      => { bench!(ark_cp6_782        , G1 = G1Projective                      , deg, name = "CP6-782"    ) }
            Curve::EDOnMnt4298 => { bench!(ark_ed_on_mnt4_298 , G1 = EdwardsProjective                 , deg, name = "ED-MNT4-298") }
            Curve::FQ128       => unsupported!("FQ128", "commit"),
            Curve::MNT4753     => { bench!(ark_mnt4_753       , G1 = G1Projective                      , deg, name = "MNT4-753"   ) }
            Curve::Pallas      => { bench!(ark_pallas         , G1 = Projective                        , deg, name = "PALLAS"     ) }
            Curve::SECP256K1   => { bench!(ark_secp256k1      , G1 = Projective                        , deg, name = "SECP256-K1" ) }
            Curve::SECP256R1   => { bench!(ark_secp256r1      , G1 = Projective                        , deg, name = "SECP256-R1" ) }
            Curve::Vesta       => { bench!(ark_vesta          , G1 = Projective                        , deg, name = "VESTA"      ) }
        }
    }
}

#[rustfmt::skip]
fn field(nb_measurements: usize, curves: &[Curve]) {
    let bencher = plnk::Bencher::new(nb_measurements);

    macro_rules! bench {
        ($f:path, name=$name:expr) => {
            benches::field::run::<$f>(&bencher.with_name(plnk::label!{ curve: $name }))
        };
    }
    for curve in curves {
        match curve {
            Curve::ARKBLS12381 => unsupported!("ARKBLS12381", "field"),
            Curve::ARKBN254    => unsupported!("ARKBN254", "field"),
            Curve::BLS12381    => { bench!(ark_bls12_381::Fr,         name = "BLS12-381"  ) }
            Curve::BN254       => { bench!(ark_bn254::Fr,             name = "BN254"      ) }
            Curve::CP6782      => { bench!(ark_cp6_782::Fr,           name = "CP6-782"    ) }
            Curve::EDOnMnt4298 => { bench!(ark_ed_on_mnt4_298::Fr,    name = "ED-MNT4-298") }
            Curve::FQ128       => { bench!(fields::Fq128,             name = "Fq128"      ) }
            Curve::MNT4753     => { bench!(ark_mnt4_753::Fr,          name = "MNT4-753"   ) }
            Curve::Pallas      => { bench!(ark_pallas::Fr,            name = "PALLAS"     ) }
            Curve::SECP256K1   => { bench!(ark_secp256k1::Fr,         name = "SECP256-K1" ) }
            Curve::SECP256R1   => { bench!(ark_secp256r1::Fr,         name = "SECP256-R1" ) }
            Curve::Vesta       => { bench!(ark_vesta::Fr,             name = "VESTA"      ) }
        }
    }
}

#[rustfmt::skip]
fn curve_group(nb_measurements: usize, curves: &[Curve]) {
    let bencher = plnk::Bencher::new(nb_measurements);

    macro_rules! bench {
        ($c:ident, G1=$g:ident, name=$n:expr) => {
            benches::curve_group::run::<$c::Fr, $c::$g>(&bencher.with_name(plnk::label! { curve: $n }))
        };
    }

    for curve in curves {
        match curve {
            Curve::ARKBLS12381 => unsupported!("ARKBLS12381", "group"),
            Curve::ARKBN254    => unsupported!("ARKBN254", "group"),
            Curve::BLS12381    => { bench!(ark_bls12_381      , G1 = G1Projective      , name = "BLS12-381"  ) }
            Curve::BN254       => { bench!(ark_bn254          , G1 = G1Projective      , name = "BN254"      ) }
            Curve::CP6782      => { bench!(ark_cp6_782        , G1 = G1Projective      , name = "CP6-782"    ) }
            Curve::EDOnMnt4298 => { bench!(ark_ed_on_mnt4_298 , G1 = EdwardsProjective , name = "ED-MNT4-298") }
            Curve::FQ128       => unsupported!("FQ128", "group"),
            Curve::MNT4753     => { bench!(ark_mnt4_753       , G1 = G1Projective      , name = "MNT4-753"   ) }
            Curve::Pallas      => { bench!(ark_pallas         , G1 = Projective        , name = "PALLAS"     ) }
            Curve::SECP256K1   => { bench!(ark_secp256k1      , G1 = Projective        , name = "SECP256-K1" ) }
            Curve::SECP256R1   => { bench!(ark_secp256r1      , G1 = Projective        , name = "SECP256-R1" ) }
            Curve::Vesta       => { bench!(ark_vesta          , G1 = Projective        , name = "VESTA"      ) }
        }
    }
}

#[rustfmt::skip]
fn linalg(sizes: &[usize], nb_measurements: usize, curves: &[Curve]) {
    let b = plnk::Bencher::new(nb_measurements);

    macro_rules! bench {
        ($f:path : $name:tt, $n: expr) => {{
            let name = plnk::label! { curve: $name, size : $n };
            benches::linalg::run_inverse::<$f>(&b.with_name(name), $n);
            benches::linalg::run_transpose::<$f>(&b.with_name(name), $n);
            benches::linalg::run_multiply::<$f>(&b.with_name(name), $n);
        }};
    }

    for (&size, curve) in iproduct!(sizes, curves) {
        match curve {
            Curve::ARKBLS12381 => { bench!(ark_bls12_381::Fr       : "BLS12-381"   , size) }
            Curve::ARKBN254    => { bench!(ark_bn254::Fr           : "BN254"       , size) }
            Curve::BLS12381    => { bench!(ark_bls12_381::Fr       : "BLS12-381"   , size) }
            Curve::BN254       => { bench!(ark_bn254::Fr           : "BN254"       , size) }
            Curve::CP6782      => { bench!(ark_cp6_782::Fr         : "CP6-782"     , size) }
            Curve::EDOnMnt4298 => { bench!(ark_ed_on_mnt4_298::Fr  : "ED-MNT4-298" , size) }
            Curve::FQ128       => unsupported!("FQ128", "linalg"),
            Curve::MNT4753     => { bench!(ark_mnt4_753::Fr        : "MNT4-753"    , size) }
            Curve::Pallas      => { bench!(ark_pallas::Fr          : "PALLAS"      , size) }
            Curve::SECP256K1   => { bench!(ark_secp256k1::Fr       : "SECP256-K1"  , size) }
            Curve::SECP256R1   => { bench!(ark_secp256r1::Fr       : "SECP256-R1"  , size) }
            Curve::Vesta       => { bench!(ark_vesta::Fr           : "VESTA"       , size) }
        }
    }
}

#[rustfmt::skip]
fn fec(sizes: &[usize], params: &[(usize, usize)], nb_measurements: usize, curves: &[Curve], encoding: benches::fec::Encoding) {
    let b = plnk::Bencher::new(nb_measurements);

    for (&nb_bytes, (k, n), curve) in iproduct!(sizes, params, curves) {
        match curve {
            Curve::ARKBLS12381 => unsupported!("ARKBLS12381", "fec"),
            Curve::ARKBN254    => unsupported!("ARKBN254", "fec"),
            Curve::BLS12381    => { benches::fec::run::<ark_bls12_381::Fr>     (&b.with_name(plnk::label! { curve: "BLS12-381" }),   nb_bytes, *k, *n, &encoding) }
            Curve::BN254       => { benches::fec::run::<ark_bn254::Fr>         (&b.with_name(plnk::label! { curve: "BN254" }),       nb_bytes, *k, *n, &encoding) }
            Curve::CP6782      => { benches::fec::run::<ark_cp6_782::Fr>       (&b.with_name(plnk::label! { curve: "CP6-782" }),     nb_bytes, *k, *n, &encoding) }
            Curve::EDOnMnt4298 => { benches::fec::run::<ark_ed_on_mnt4_298::Fr>(&b.with_name(plnk::label! { curve: "ED-MNT4-298" }), nb_bytes, *k, *n, &encoding) }
            Curve::FQ128       => { benches::fec::run::<fields::Fq128>         (&b.with_name(plnk::label! { curve: "Fq128" }),       nb_bytes, *k, *n, &encoding) }
            Curve::MNT4753     => { benches::fec::run::<ark_mnt4_753::Fr>      (&b.with_name(plnk::label! { curve: "MNT4-753" }),    nb_bytes, *k, *n, &encoding) }
            Curve::Pallas      => { benches::fec::run::<ark_pallas::Fr>        (&b.with_name(plnk::label! { curve: "PALLAS" }),      nb_bytes, *k, *n, &encoding) }
            Curve::SECP256K1   => { benches::fec::run::<ark_secp256k1::Fr>     (&b.with_name(plnk::label! { curve: "SECP256-K1" }),  nb_bytes, *k, *n, &encoding) }
            Curve::SECP256R1   => { benches::fec::run::<ark_secp256r1::Fr>     (&b.with_name(plnk::label! { curve: "SECP256-R1" }),  nb_bytes, *k, *n, &encoding) }
            Curve::Vesta       => { benches::fec::run::<ark_vesta::Fr>         (&b.with_name(plnk::label! { curve: "VESTA" }),       nb_bytes, *k, *n, &encoding) }
        }
    }
}

#[rustfmt::skip]
fn recoding(sizes: &[usize],ks: &[usize], shards: &[usize], nb_measurements: usize, curves: &[Curve]) {
    let bencher = plnk::Bencher::new(nb_measurements);

    for (&nb_bytes, &nb_shards, &k, curve) in iproduct!(sizes, shards, ks, curves) {
        match curve {
            Curve::ARKBLS12381 => { benches::recoding::run::<ark_bls12_381::Fr     >(&bencher.with_name(plnk::label! { curve: "BLS12-381"   }), nb_bytes, k, nb_shards) }
            Curve::ARKBN254    => { benches::recoding::run::<ark_bn254::Fr         >(&bencher.with_name(plnk::label! { curve: "BN254"       }), nb_bytes, k, nb_shards) }
            Curve::BLS12381    => { benches::recoding::run::<ark_bls12_381::Fr     >(&bencher.with_name(plnk::label! { curve: "BLS12-381"   }), nb_bytes, k, nb_shards) }
            Curve::BN254       => { benches::recoding::run::<ark_bn254::Fr         >(&bencher.with_name(plnk::label! { curve: "BN254"       }), nb_bytes, k, nb_shards) }
            Curve::CP6782      => { benches::recoding::run::<ark_cp6_782::Fr       >(&bencher.with_name(plnk::label! { curve: "CP6-782"     }), nb_bytes, k, nb_shards) }
            Curve::EDOnMnt4298 => { benches::recoding::run::<ark_ed_on_mnt4_298::Fr>(&bencher.with_name(plnk::label! { curve: "ED-MNT4-298" }), nb_bytes, k, nb_shards) }
            Curve::FQ128       => { benches::recoding::run::<fields::Fq128         >(&bencher.with_name(plnk::label! { curve: "FQ128"       }), nb_bytes, k, nb_shards) }
            Curve::MNT4753     => { benches::recoding::run::<ark_mnt4_753::Fr      >(&bencher.with_name(plnk::label! { curve: "MNT4-753"    }), nb_bytes, k, nb_shards) }
            Curve::Pallas      => { benches::recoding::run::<ark_pallas::Fr        >(&bencher.with_name(plnk::label! { curve: "PALLAS"      }), nb_bytes, k, nb_shards) }
            Curve::SECP256K1   => { benches::recoding::run::<ark_secp256k1::Fr     >(&bencher.with_name(plnk::label! { curve: "SECP256-K1"  }), nb_bytes, k, nb_shards) }
            Curve::SECP256R1   => { benches::recoding::run::<ark_secp256r1::Fr     >(&bencher.with_name(plnk::label! { curve: "SECP256-R1"  }), nb_bytes, k, nb_shards) }
            Curve::Vesta       => { benches::recoding::run::<ark_vesta::Fr         >(&bencher.with_name(plnk::label! { curve: "VESTA"       }), nb_bytes, k, nb_shards) }
        }
    }
}

#[rustfmt::skip]
fn semi_avid(sizes: &[usize], params: &[(usize, usize)], nb_measurements: usize, curves: &[Curve]) {
    macro_rules! bench {
        ($field:path, $group:path : $name:tt, $k:expr, $n:expr, $nb_bytes:expr) => {{
            let b = plnk::Bencher::new(nb_measurements).with_name(plnk::label! { curve: $name });
            benches::semi_avid::run::<$field, $group, DensePolynomial<$field>>(&b, $k, $n, $nb_bytes)
        }};
    }

    for (&nb_bytes, (k, n), curve) in iproduct!(sizes, params, curves) {
        match curve {
            Curve::ARKBLS12381 => unsupported!("ARKBLS12381", "semi_avid"),
            Curve::ARKBN254    => unsupported!("ARKBN254", "semi_avid"),
            Curve::BLS12381    => { bench!(ark_bls12_381::Fr,      ark_bls12_381::G1Projective           : "BLS12-381"  , *k, *n, nb_bytes) }
            Curve::BN254       => { bench!(ark_bn254::Fr,          ark_bn254::G1Projective               : "BN254"      , *k, *n, nb_bytes) }
            Curve::CP6782      => { bench!(ark_cp6_782::Fr,        ark_cp6_782::G1Projective             : "CP6-782"    , *k, *n, nb_bytes) }
            Curve::EDOnMnt4298 => { bench!(ark_ed_on_mnt4_298::Fr, ark_ed_on_mnt4_298::EdwardsProjective : "ED-MNT4-298", *k, *n, nb_bytes) }
            Curve::FQ128       => unsupported!("FQ128", "semi_avid"),
            Curve::MNT4753     => { bench!(ark_mnt4_753::Fr,       ark_mnt4_753::G1Projective            : "MNT4-753"   , *k, *n, nb_bytes) }
            Curve::Pallas      => { bench!(ark_pallas::Fr,         ark_pallas::Projective                : "PALLAS"     , *k, *n, nb_bytes) }
            Curve::SECP256K1   => { bench!(ark_secp256k1::Fr,      ark_secp256k1::Projective             : "SECP256-K1" , *k, *n, nb_bytes) }
            Curve::SECP256R1   => { bench!(ark_secp256r1::Fr,      ark_secp256r1::Projective             : "SECP256-R1" , *k, *n, nb_bytes) }
            Curve::Vesta       => { bench!(ark_vesta::Fr,          ark_vesta::Projective                 : "VESTA"      , *k, *n, nb_bytes) }
        }
    }
}

#[rustfmt::skip]
fn kzg(sizes: &[usize], params: &[(usize, usize)], nb_measurements: usize, curves: &[Curve]) {
    macro_rules! bench {
        ($field:path, $curve:path : $name:tt, $k:expr, $n:expr, $nb_bytes:expr) => {{
            let b = plnk::Bencher::new(nb_measurements).with_name(plnk::label! { curve: $name });
            benches::kzg::run::<$curve, DensePolynomial<$field>>(&b, $k, $n, $nb_bytes)
        }};
    }

    for (&nb_bytes, (k, n), curve) in iproduct!(sizes, params, curves) {
        match curve {
            Curve::ARKBLS12381 => unsupported!("ARKBLS12381", "kzg"),
            Curve::ARKBN254    => unsupported!("ARKBN254", "kzg"),
            Curve::BLS12381    => { bench!(ark_bls12_381::Fr,      ark_bls12_381::Bls12_381: "BLS12-381"  , *k, *n, nb_bytes) }
            Curve::BN254       => { bench!(ark_bn254::Fr,          ark_bn254::Bn254: "BN254"      , *k, *n, nb_bytes) }
            Curve::CP6782      => { bench!(ark_cp6_782::Fr,        ark_cp6_782::CP6_782: "CP6-782"    , *k, *n, nb_bytes) }
            Curve::EDOnMnt4298 => unsupported!("EDOnMnt4298", "kzg"),
            Curve::FQ128       => unsupported!("FQ128", "kzg"),
            Curve::MNT4753     => { bench!(ark_mnt4_753::Fr,       ark_mnt4_753::MNT4_753: "MNT4-753"   , *k, *n, nb_bytes) }
            Curve::Pallas      => unsupported!("Pallas", "kzg"),
            Curve::SECP256K1   => unsupported!("SECP256K1", "kzg"),
            Curve::SECP256R1   => unsupported!("SECP256R1", "kzg"),
            Curve::Vesta       => unsupported!("Vesta", "kzg"),
        }
    }
}

#[rustfmt::skip]
fn aplonk(sizes: &[usize], params: &[(usize, usize)], nb_measurements: usize, curves: &[Curve]) {
    macro_rules! bench {
        ($field:path, $curve:path : $name:tt, $k:expr, $n:expr, $nb_bytes:expr) => {{
            let b = plnk::Bencher::new(nb_measurements).with_name(plnk::label! { curve: $name });
            benches::aplonk::run::<$curve, DensePolynomial<$field>>(&b, $k, $n, $nb_bytes)
        }};
    }

    for (&nb_bytes, (k, n), curve) in iproduct!(sizes, params, curves) {
        match curve {
            Curve::ARKBLS12381 => unsupported!("ARKBLS12381", "kzg"),
            Curve::ARKBN254    => unsupported!("ARKBN254", "kzg"),
            Curve::BLS12381    => { bench!(ark_bls12_381::Fr,      ark_bls12_381::Bls12_381: "BLS12-381"  , *k, *n, nb_bytes) }
            Curve::BN254       => { bench!(ark_bn254::Fr,          ark_bn254::Bn254: "BN254"      , *k, *n, nb_bytes) }
            Curve::CP6782      => { bench!(ark_cp6_782::Fr,        ark_cp6_782::CP6_782: "CP6-782"    , *k, *n, nb_bytes) }
            Curve::EDOnMnt4298 => unsupported!("EDOnMnt4298", "kzg"),
            Curve::FQ128       => unsupported!("FQ128", "kzg"),
            Curve::MNT4753     => { bench!(ark_mnt4_753::Fr,       ark_mnt4_753::MNT4_753: "MNT4-753"   , *k, *n, nb_bytes) }
            Curve::Pallas      => unsupported!("Pallas", "kzg"),
            Curve::SECP256K1   => unsupported!("SECP256K1", "kzg"),
            Curve::SECP256R1   => unsupported!("SECP256R1", "kzg"),
            Curve::Vesta       => unsupported!("Vesta", "kzg"),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Setup {
            degrees,
            nb_measurements,
            curves,
        }) => setup(degrees, *nb_measurements, curves),
        Some(Commands::Commit {
            degrees,
            nb_measurements,
            curves,
        }) => commit(degrees, *nb_measurements, curves),
        Some(Commands::Field {
            nb_measurements,
            curves,
        }) => field(*nb_measurements, curves),
        Some(Commands::Group {
            nb_measurements,
            curves,
        }) => curve_group(*nb_measurements, curves),
        Some(Commands::Linalg {
            sizes,
            nb_measurements,
            curves,
        }) => linalg(sizes, *nb_measurements, curves),
        Some(Commands::Fec {
            sizes,
            ks,
            rhos,
            nb_measurements,
            curves,
            encoding,
        }) => {
            let params = iproduct!(ks, rhos)
                .map(|(&k, &r)| (k, ((k as f64) / r).round() as usize))
                .collect::<Vec<(usize, usize)>>();
            fec(sizes, &params, *nb_measurements, curves, encoding.clone())
        }
        Some(Commands::Recoding {
            sizes,
            shards,
            ks,
            nb_measurements,
            curves,
        }) => recoding(sizes, ks, shards, *nb_measurements, curves),
        Some(Commands::SemiAVID {
            sizes,
            ks,
            rhos,
            nb_measurements,
            curves,
        }) => {
            let params = iproduct!(ks, rhos)
                .map(|(&k, &r)| (k, ((k as f64) / r).round() as usize))
                .collect::<Vec<(usize, usize)>>();
            semi_avid(sizes, &params, *nb_measurements, curves)
        }
        Some(Commands::KZG {
            sizes,
            ks,
            rhos,
            nb_measurements,
            curves,
        }) => {
            let params = iproduct!(ks, rhos)
                .map(|(&k, &r)| (k, ((k as f64) / r).round() as usize))
                .collect::<Vec<(usize, usize)>>();
            kzg(sizes, &params, *nb_measurements, curves)
        }
        Some(Commands::Aplonk {
            sizes,
            ks,
            rhos,
            nb_measurements,
            curves,
        }) => {
            let params = iproduct!(ks, rhos)
                .map(|(&k, &r)| (k, ((k as f64) / r).round() as usize))
                .collect::<Vec<(usize, usize)>>();
            aplonk(sizes, &params, *nb_measurements, curves)
        }
        None => eprintln!("WARNING: nothing to do"),
    }
}
