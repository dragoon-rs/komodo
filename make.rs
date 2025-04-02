#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! nob = { git = "https://gitlab.isae-supaero.fr/a.stevan/nob.rs" }
//! ```

fn main() {
    nob::run_cmd_and_fail!("cargo", "test", "linalg");
}
