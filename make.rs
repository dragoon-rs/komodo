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
//!
//! # for `container --list`
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! prettytable = "0.10.0"
//! ```

use clap::{Parser, Subcommand};
use prettytable::{format, Cell, Row, Table};
use serde_json::Value;

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
    #[command(subcommand)]
    Container(ContainerCommands),
}

#[derive(Subcommand)]
enum ContainerCommands {
    /// Build the current dockerfile.
    Build,
    /// List the local images.
    List {
        /// Print the output table as NDJSON instead of pretty table.
        #[arg(long)]
        json: bool,
    },
    /// Log into the registry instead of building.
    Login,
    /// Push to the registry instead of building.
    Push,
}

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
            let cmd = vec!["cargo", "check", "--workspace", "--all-targets"];
            extend_and_run(&cmd, &[]);
            extend_and_run(&cmd, &["--features", "kzg"]);
            extend_and_run(&cmd, &["--features", "aplonk"]);
            extend_and_run(&cmd, &["--all-features"]);
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

            if *verbose {
                cmd.push("--verbose")
            }
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
            if *open {
                cmd.push("--open")
            }
            if *private {
                cmd.push("--document-private-items")
            }
            if *features {
                cmd.push("--all-features")
            }
            nob::run_cmd_as_vec_and_fail!(cmd);
        }
        Some(Commands::Container(container_cmd)) => {
            let res = nob::run_cmd_and_fail!(@+"git", "rev-parse", "HEAD");
            let sha = String::from_utf8(res.stdout).expect("Invalid UTF-8 string");

            let repo = format!("{}/{}", REGISTRY, IMAGE);
            let image = format!("{}:{}", repo, sha.trim());

            let mirror_repo = format!("{}/{}", MIRROR_REGISTRY, IMAGE);
            let mirror_image = format!("{}:{}", mirror_repo, sha.trim());

            match container_cmd {
                ContainerCommands::Login => {
                    nob::run_cmd_and_fail!("docker", "login", REGISTRY);
                    nob::run_cmd_and_fail!("docker", "login", MIRROR_REGISTRY);
                }
                ContainerCommands::Build => {
                    let cmd = vec!["docker", "build", ".", "--file", DOCKERFILE];
                    extend_and_run(&cmd, &["-t", &image]);
                    extend_and_run(&cmd, &["-t", &mirror_image]);
                }
                ContainerCommands::List { json } => {
                    let cmd = vec!["docker", "image", "list", "--format", "json"];
                    let images = extend_and_run_and_capture_silent(&cmd, &[&repo])
                        + &extend_and_run_and_capture_silent(&cmd, &[&mirror_repo]);

                    if *json {
                        println!("{}", images);
                    } else {
                        docker_images_to_table(images).printstd();
                    }
                }
                ContainerCommands::Push => {
                    nob::run_cmd_and_fail!("docker", "push", &image);
                    nob::run_cmd_and_fail!("docker", "push", &mirror_image);
                }
            }
        }
        None => {}
    }
}

fn docker_images_to_table(lines: String) -> Table {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut headers: Vec<String> = Vec::new();
    for line in lines.lines() {
        let json: Value = serde_json::from_str(&line).unwrap_or_else(|_| Value::Null);
        if let Value::Object(map) = serde_json::from_str(&line).unwrap_or_else(|_| Value::Null) {
            if headers.is_empty() {
                headers = map.keys().cloned().collect();
            }

            let row: Vec<String> = headers
                .iter()
                .map(|key| map.get(key).map_or("".to_string(), |v| v.to_string()))
                .collect();
            rows.push(row);
        }
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(Row::new(headers.iter().map(|h| Cell::new(h)).collect()));
    for row in rows {
        table.add_row(Row::new(row.iter().map(|v| Cell::new(v)).collect()));
    }

    table
}

// NOTE: this could be migrated to [`nob.rs`](https://gitlab.isae-supaero.fr/a.stevan/nob.rs)
fn extend_and_run(cmd: &[&str], args: &[&str]) {
    let mut cmd = cmd.to_vec();
    cmd.extend_from_slice(&args);
    nob::run_cmd_as_vec_and_fail!(cmd);
}

// NOTE: this could be migrated to [`nob.rs`](https://gitlab.isae-supaero.fr/a.stevan/nob.rs)
fn extend_and_run_and_capture_silent(cmd: &[&str], args: &[&str]) -> String {
    let mut cmd = cmd.to_vec();
    cmd.extend_from_slice(&args);
    String::from_utf8(nob::run_cmd_as_vec_and_fail!(@+cmd).stdout).expect("Invalid UTF-8 string")
}
