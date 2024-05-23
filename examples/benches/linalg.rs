// see `examples/benches/README.md`
use ark_ff::PrimeField;

use clap::{arg, command, Parser};
use komodo::linalg::Matrix;
use plnk::Bencher;

fn inverse_template<F: PrimeField>(b: &Bencher, n: usize) {
    let mut rng = rand::thread_rng();
    let matrix = Matrix::<F>::random(n, n, &mut rng);

    plnk::bench(b, &format!("inverse {}", n), || {
        plnk::timeit(|| matrix.invert().unwrap())
    });
}

fn transpose_template<F: PrimeField>(b: &Bencher, n: usize) {
    let mut rng = rand::thread_rng();
    let matrix = Matrix::<F>::random(n, n, &mut rng);

    plnk::bench(b, &format!("transpose {}", n), || {
        plnk::timeit(|| matrix.transpose())
    });
}

fn mul_template<F: PrimeField>(b: &Bencher, n: usize) {
    let mut rng = rand::thread_rng();
    let mat_a = Matrix::<F>::random(n, n, &mut rng);
    let mat_b = Matrix::<F>::random(n, n, &mut rng);

    plnk::bench(b, &format!("mul {}", n), || {
        plnk::timeit(|| mat_a.mul(&mat_b))
    });
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// the sizes of the matrices to consider
    #[arg(num_args = 1.., value_delimiter = ' ')]
    sizes: Vec<usize>,

    /// the number of measurements to repeat each case, larger values will reduce the variance of
    /// the measurements
    #[arg(short, long)]
    nb_measurements: usize,
}

fn main() {
    let cli = Cli::parse();

    let b = plnk::Bencher::new(cli.nb_measurements);

    for n in cli.sizes {
        inverse_template::<ark_bls12_381::Fr>(&b.with_name("BLS12-381"), n);
        inverse_template::<ark_bn254::Fr>(&b.with_name("BN254"), n);
        inverse_template::<ark_pallas::Fr>(&b.with_name("PALLAS"), n);

        transpose_template::<ark_bls12_381::Fr>(&b.with_name("BLS12-381"), n);
        transpose_template::<ark_bn254::Fr>(&b.with_name("BN254"), n);
        transpose_template::<ark_pallas::Fr>(&b.with_name("PALLAS"), n);

        mul_template::<ark_bls12_381::Fr>(&b.with_name("BLS12-381"), n);
        mul_template::<ark_bn254::Fr>(&b.with_name("BN254"), n);
        mul_template::<ark_pallas::Fr>(&b.with_name("PALLAS"), n);
    }
}
