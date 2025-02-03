use ../utils log
use ../utils formats *
use ../utils args check-list-arg

use std formats *

# run the "fec" benchmarks
#
# - input: the list of input file sizes
# - output: the output path, as NDJSON
export def main [
    --output: path, # the output path (defaults to a random file in $nu.temp-path)
    --ks: list<int>, # the values of $k$ to benchmark
    --curves: list<string>, # the curves to benchmark
    --no-confirm (-y), # does not ask for confirmation if the output file already exists, it will be overwritten
    --nb-measurements: int = 10, # the number of measurements per benchmark run
    --append, # append to the output path instead of overwritting
]: list<int> -> path {
    $ks | check-list-arg --cmd "fec run" --arg "--ks" --span (metadata $ks).span
    $curves | check-list-arg --cmd "fec run" --arg "--curves" --span (metadata $curves).span
    $in | check-list-arg --cmd "fec run" --arg "pipeline input"

    let new_file = $output == null
    let output = $output | default (mktemp --tmpdir komodo_fec.XXXXXX)
    let pretty_output = $"(ansi purple)($output)(ansi reset)"
    if ($output | path exists) and not $new_file {
        log warning $"($pretty_output) already exists"
        if not $no_confirm {
            let res = ["no", "yes"] | input list $"Do you want to overwrite ($pretty_output)?"
            if $res == null or $res == "no" {
                log info "aborting"
                return
            }

        }
    }

    if not $append {
        "" out> $output
    }

    let input = $in
    for k in $ks {
        let options = [
            --release
            --package benchmarks
            --bin fec
            --
            --nb-measurements $nb_measurements
            ...$input
            --encoding vandermonde
            -k $k
            -n 1
            --curves ...$curves
        ]
        if $append {
            cargo run ...$options | from ndnuon | to ndjson out>> $output
        } else {
            cargo run ...$options | from ndnuon | to ndjson out> $output
        }
    }

    log info $"results saved to ($pretty_output)"
    $output
}
