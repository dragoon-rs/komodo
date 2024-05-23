export def main [
    --output: path = "./commit.ndjson",
    --nb-measurements: int = 10,
    --curves: list<string>,
]: list<int> -> nothing {
    cargo run --release --example bench_commit -- ...[
        --nb-measurements $nb_measurements
        ...$in
        --curves ...$curves
    ] out> $output

    print $"results saved to `($output)`"
}
