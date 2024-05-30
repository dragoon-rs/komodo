use clap::Parser;
use rand::{rngs::StdRng, Rng, SeedableRng};

fn parse_hex_string(s: &str) -> Result<[u8; 32], String> {
    if s.len() != 64 {
        return Err("Input string must be exactly 64 characters long".to_string());
    }

    match hex::decode(s) {
        // `bytes` will be a `Vec<u8>` of size `32`, so it's safe to `unwrap`
        // the conversion to `[u8: 32]`
        Ok(bytes) => Ok(bytes.try_into().unwrap()),
        Err(e) => Err(format!("Failed to decode hex string: {}", e)),
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short)]
    n: usize,

    #[arg(long, value_parser = parse_hex_string)]
    prng_seed: [u8; 32],
}

fn main() {
    let cli = Cli::parse();

    let mut rng = StdRng::from_seed(cli.prng_seed);

    for _ in 0..cli.n {
        println!("{}", rng.gen::<u8>());
    }
}
