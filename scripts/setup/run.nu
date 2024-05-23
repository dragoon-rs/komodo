export def main [--output: path = "./setup.ndjson", --nb-measurements: int = 10]: list<int> -> nothing {
    cargo run --release --example bench_setup -- --nb-measurements $nb_measurements ...$in out> $output

    print $"results saved to `($output)`"
}
