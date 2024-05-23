use ../formats.nu *

export def main [
    --output: path = "./recoding.ndjson",
    --nb-measurements: int = 10,
    --ks: list<int>
]: list<int> -> nothing {
    let input = $in

    if ($ks | is-empty) {
        print "nothing to do"
        return
    }

    "" out> $output

    for k in $ks {
        cargo run --release --example bench_recoding -- ...[
            --nb-measurements $nb_measurements
            ...$input
            --shards $k
            --ks $k
        ] | from ndnuon | to ndjson out>> $output
    }
}
