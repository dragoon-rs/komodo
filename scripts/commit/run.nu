export def main [
    --output: path = "./commit.ndjson",
    --nb-measurements: int = 10,
    --curves: list<string>,
]: list<int> -> nothing {
    let input = $in

    if ($input | is-empty) or ($curves | is-empty) {
        print "nothing to do"
        return
    }

    cargo run --release --example bench_commit -- ...[
        --nb-measurements $nb_measurements
        ...$input
        --curves ...$curves
    ] out> $output

    print $"results saved to `($output)`"
}
