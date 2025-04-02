#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! nob = { git = "https://gitlab.isae-supaero.fr/a.stevan/nob.rs" }
//! clap = { version = "4.5.17", features = ["derive"] }
//! ```
extern crate clap;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Formats the code.
    Fmt {
        /// Only checks instead of really formatting.
        #[arg(short, long)]
        check: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    match &cli.command {
        Some(Commands::Fmt { check }) => {
            if *check {
                nob::run_cmd_and_fail!("cargo", "fmt", "--all", "--", "--check");
            } else {
                nob::run_cmd_and_fail!("cargo", "fmt", "--all");
            }
        }
        None => {}
    }
}
