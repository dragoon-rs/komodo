#!/usr/bin/env run-cargo-script
//! ```cargo
//! [package]
//! name = "komodo-make"
//! version = "1.0.0"
//! edition = "2021"
//!
//! [dependencies]
//! nob = { git = "https://gitlab.isae-supaero.fr/a.stevan/nob.rs", rev = "7ea6be855cf5600558440def6e59a83f78b8b543" }
//! clap = { version = "4.5.17", features = ["derive"] }
//! ```
extern crate clap;

use clap::{Parser, Subcommand};

const REGISTRY: &str = "gitlab-registry.isae-supaero.fr";
const MIRROR_REGISTRY: &str = "ghcr.io/dragoon-rs";
const IMAGE: &str = "dragoon/komodo";
const DOCKERFILE: &str = ".env.dockerfile";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
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
    /// Checks the code.
    Check,
    /// Runs Clippy.
    Clippy,
    /// Runs the tests.
    Test {
        /// Be extra verbose with the output of the tests.
        #[arg(short, long)]
        verbose: bool,
        /// Run the examples instead of regular tests.
        #[arg(short, long)]
        examples: bool,
    },
    /// Shows the version of all the tools used,
    Version,
    /// Builds the documentation
    Doc {
        /// Open the documentation in the browser.
        #[arg(short, long)]
        open: bool,
        /// Document private items.
        #[arg(short, long)]
        private: bool,
        /// Document all features.
        #[arg(short, long)]
        features: bool,
    },
    /// Builds the container.
    Container {
        /// Log into the registry instead of building.
        #[arg(short, long)]
        login: bool,
        /// Push to the registry instead of building.
        #[arg(short, long)]
        push: bool,
    },
}

#[rustfmt::skip]
fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Fmt { check }) => {
            if *check {
                nob::run_cmd_and_fail!("cargo", "fmt", "--all", "--", "--check");
            } else {
                nob::run_cmd_and_fail!("cargo", "fmt", "--all");
            }
        }
        Some(Commands::Check) => {
            nob::run_cmd_and_fail!("cargo", "check", "--workspace", "--all-targets");
            nob::run_cmd_and_fail!("cargo", "check", "--workspace", "--all-targets", "--features", "kzg");
            nob::run_cmd_and_fail!("cargo", "check", "--workspace", "--all-targets", "--features", "aplonk");
            nob::run_cmd_and_fail!("cargo", "check", "--workspace", "--all-targets", "--all-features");
        }
        Some(Commands::Clippy) => {
            nob::run_cmd_and_fail!(
                "cargo",
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings"
            );
        }
        Some(Commands::Test { verbose, examples }) => {
            let mut cmd = vec!["cargo", "test"];

            if *verbose { cmd.push("--verbose") }
            if *examples {
                cmd.push("--examples");
            } else {
                cmd.push("--workspace");
                cmd.push("--all-features");
            }

            nob::run_cmd_as_vec_and_fail!(cmd);
        }
        Some(Commands::Version) => {
            nob::run_cmd_and_fail!(@"rustup", "--version", "2>", "/dev/null");
            nob::run_cmd_and_fail!(@"rustup", "show", "active-toolchain");
            nob::run_cmd_and_fail!(@"rustc", "--version");
            nob::run_cmd_and_fail!(@"cargo", "--version");
            nob::run_cmd_and_fail!(@"cargo", "clippy", "--version");
        }
        Some(Commands::Doc {
            open,
            private,
            features,
        }) => {
            let mut cmd = vec!["cargo", "doc", "--no-deps"];
            if *open { cmd.push("--open") }
            if *private { cmd.push("--document-private-items") }
            if *features { cmd.push("--all-features") }
            nob::run_cmd_as_vec_and_fail!(cmd ; "RUSTDOCFLAGS" => "--html-in-header katex.html");
        }
        Some(Commands::Container { login, push }) => {
            let res = nob::run_cmd_and_fail!(@+"git", "rev-parse", "HEAD");
            let sha = String::from_utf8(res.stdout).expect("Invalid UTF-8 string");
            let image = format!("{}/{}:{}", REGISTRY, IMAGE, sha.trim());
            let mirror_image = format!("{}/{}:{}", MIRROR_REGISTRY, IMAGE, sha.trim());

            if *login {
                nob::run_cmd_and_fail!("docker", "login", REGISTRY);
                nob::run_cmd_and_fail!("docker", "login", MIRROR_REGISTRY);
            } else if *push {
                nob::run_cmd_and_fail!("docker", "push", &image);
                nob::run_cmd_and_fail!("docker", "push", &mirror_image);
            } else {
                nob::run_cmd_and_fail!(
                    "docker",
                    "build",
                    "-t", &image,
                    ".",
                    "--file", DOCKERFILE
                );
                nob::run_cmd_and_fail!(
                    "docker",
                    "build",
                    "-t", &mirror_image,
                    ".",
                    "--file", DOCKERFILE
                );
            }
        }
        None => {}
    }
}
