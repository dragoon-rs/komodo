use ../formats.nu *

export def main [
    --output: path = "./fec.ndjson",
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
        cargo run --release --example bench_fec -- ...[
            --nb-measurements $nb_measurements
            ...$input
            --encoding vandermonde
            -k $k
            -n 1
        ] | from ndnuon | to ndjson out>> $output
    }
}
