#!/usr/bin/env run-cargo-script
//! ```cargo
//! [package]
//! name = "komodo-make"
//! version = "1.0.0"
//! edition = "2021"
//!
//! [dependencies]
//! nob = { git = "https://gitlab.isae-supaero.fr/a.stevan/nob.rs", rev = "c2508fdd3fef8f01110a8c2fd0690afaecbc7b16" }
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
    /// Builds the paper
    Paper {
        /// Build as a draft
        #[arg(short, long)]
        draft: bool,
    },
    /// Run all that is needed for the Continuous Integration of the project.
    CI {
        /// Run the "fmt" stage of the Continuous Integration
        #[arg(short, long)]
        fmt: bool,
        /// Run the "test" stage of the Continuous Integration
        #[arg(short, long)]
        test: bool,
        /// Be extra verbose with the output of the Continuous Integration.
        #[arg(short, long)]
        verbose: bool,
    },
    /// Builds the container.
    #[command(subcommand)]
    Container(ContainerCommands),
    /// Run benchmarks.
    #[command(subcommand)]
    Benchmark(BenchmarkCommands),
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

#[derive(Subcommand)]
enum BenchmarkCommands {
    Field {
        /// Arguments to pass to `benchmarks field`
        #[arg(allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    Group {
        /// Arguments to pass to `benchmarks group`
        #[arg(allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    Linalg {
        /// Arguments to pass to `benchmarks linalg`
        #[arg(allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    Setup {
        /// Arguments to pass to `benchmarks setup`
        #[arg(allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    Commit {
        /// Arguments to pass to `benchmarks commit`
        #[arg(allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    Fec {
        /// Arguments to pass to `benchmarks fec`
        #[arg(allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    Recoding {
        /// Arguments to pass to `benchmarks recoding`
        #[arg(allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    SemiAVID {
        /// Arguments to pass to `benchmarks semi_avid`
        #[arg(allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    KZG {
        /// Arguments to pass to `benchmarks kzg`
        #[arg(allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    Aplonk {
        /// Arguments to pass to `benchmarks aplonk`
        #[arg(allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
}

fn fmt(check: bool) {
    if check {
        nob::run_cmd_and_fail!("cargo", "fmt", "--all", "--", "--check");
    } else {
        nob::run_cmd_and_fail!("cargo", "fmt", "--all");
    }
}

fn check() {
    let cmd = vec!["cargo", "check", "--workspace", "--all-targets"];
    extend_and_run(&cmd, &[]);
    extend_and_run(&cmd, &["--features", "kzg"]);
    extend_and_run(&cmd, &["--features", "aplonk"]);
    extend_and_run(&cmd, &["--all-features"]);
}

fn clippy() {
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

fn test(verbose: bool, examples: bool) {
    let mut cmd = vec!["cargo", "test"];

    if verbose {
        cmd.push("--verbose")
    }
    if examples {
        cmd.push("--examples");
    } else {
        cmd.push("--workspace");
        cmd.push("--all-features");
    }

    nob::run_cmd_as_vec_and_fail!(cmd);
}

fn version() {
    nob::run_cmd_and_fail!(@"rustup", "--version", "2>", "/dev/null");
    nob::run_cmd_and_fail!(@"rustup", "show", "active-toolchain");
    nob::run_cmd_and_fail!(@"rustc", "--version");
    nob::run_cmd_and_fail!(@"cargo", "--version");
    nob::run_cmd_and_fail!(@"cargo", "clippy", "--version");
}

fn doc(open: bool, private: bool, features: bool) {
    let mut cmd = vec!["cargo", "doc", "--no-deps"];
    if open {
        cmd.push("--open")
    }
    if private {
        cmd.push("--document-private-items")
    }
    if features {
        cmd.push("--all-features")
    }
    nob::run_cmd_as_vec_and_fail!(cmd);
}

fn paper(draft: bool) {
    let uid = run_and_capture_silent(vec!["id", "-u"]);
    let group = run_and_capture_silent(vec!["id", "-g"]);

    let v = format!("{}:/data", std::env::var("PWD").unwrap());
    let u = format!("{}:{}", uid.trim(), group.trim());

    #[rustfmt::skip]
    let mut cmd = vec![
        "docker", "run", "--rm", "-it",
        "-v", &v,
        "-u", &u,
        "openjournals/inara",
        "-o", "pdf",
        "paper.md",
    ];
    if !draft {
        cmd.push("-p")
    }

    nob::run_cmd_as_vec_and_fail!(cmd);
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Fmt { check }) => fmt(*check),
        Some(Commands::Check) => check(),
        Some(Commands::Clippy) => clippy(),
        Some(Commands::Test { verbose, examples }) => test(*verbose, *examples),
        Some(Commands::Version) => version(),
        Some(Commands::Doc {
            open,
            private,
            features,
        }) => doc(*open, *private, *features),
        Some(Commands::Paper { draft }) => paper(*draft),
        Some(Commands::CI {
            fmt: fmt_stage,
            test: test_stage,
            verbose,
        }) => match (fmt_stage, test_stage) {
            (false, false) | (true, true) => {
                fmt(true);
                version();
                check();
                clippy();
                test(*verbose, false);
                test(*verbose, true);
            }
            (true, false) => fmt(true),
            (false, true) => {
                version();
                check();
                clippy();
                test(*verbose, false);
                test(*verbose, true);
            }
        },
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
        Some(Commands::Benchmark(benchmark_cmd)) => {
            let mut cmd = vec!["cargo", "run", "--quiet", "--package", "benchmarks", "--"];
            let (subcommand, args) = match benchmark_cmd {
                BenchmarkCommands::Field { args } => ("field", args),
                BenchmarkCommands::Group { args } => ("group", args),
                BenchmarkCommands::Linalg { args } => ("linalg", args),
                BenchmarkCommands::Setup { args } => ("setup", args),
                BenchmarkCommands::Commit { args } => ("commit", args),
                BenchmarkCommands::Fec { args } => ("fec", args),
                BenchmarkCommands::Recoding { args } => ("recoding", args),
                BenchmarkCommands::SemiAVID { args } => ("semi-avid", args),
                BenchmarkCommands::KZG { args } => ("kzg", args),
                BenchmarkCommands::Aplonk { args } => ("aplonk", args),
            };
            cmd.push(subcommand);
            extend_and_run(&cmd, &args.iter().map(|x| &**x).collect::<Vec<&str>>());
        }
        None => {}
    }
}

fn docker_images_to_table(lines: String) -> Table {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut headers: Vec<String> = Vec::new();
    for line in lines.lines() {
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
fn run_and_capture_silent(cmd: Vec<&str>) -> String {
    String::from_utf8(nob::run_cmd_as_vec_and_fail!(@+cmd).stdout).expect("Invalid UTF-8 string")
}

// NOTE: this could be migrated to [`nob.rs`](https://gitlab.isae-supaero.fr/a.stevan/nob.rs)
fn extend_and_run_and_capture_silent(cmd: &[&str], args: &[&str]) -> String {
    let mut cmd = cmd.to_vec();
    cmd.extend_from_slice(&args);
    run_and_capture_silent(cmd)
}
