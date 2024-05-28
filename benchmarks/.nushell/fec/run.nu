use ../../../.nushell/formats.nu *

export def main [
    --output: path = "./fec.ndjson",
    --nb-measurements: int = 10,
    --ks: list<int>,
    --curves: list<string>,
]: list<int> -> nothing {
    let input = $in

    if ($ks | is-empty) or ($input | is-empty) or ($curves | is-empty) {
        print "nothing to do"
        return
    }

    "" out> $output

    for k in $ks {
        cargo run --release --package benchmarks --bin fec -- ...[
            --nb-measurements $nb_measurements
            ...$input
            --encoding vandermonde
            -k $k
            -n 1
            --curves ...$curves
        ] | from ndnuon | to ndjson out>> $output
    }
}
