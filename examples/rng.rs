use clap::Parser;
use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short)]
    n: usize,

    #[arg(long)]
    prng_seed: u8,
}

fn main() {
    let cli = Cli::parse();

    let mut seed: [u8; 32] = [0; 32];
    seed[0] = cli.prng_seed;
    let mut rng = StdRng::from_seed(seed);

    for _ in 0..cli.n {
        println!("{}", rng.gen::<u8>());
    }
}
