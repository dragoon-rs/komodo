use utils log
use utils formats *
use utils math *
use utils plot [ into-axis-options, COMMON_OPTIONS, gplt ]
use utils fs check-file
use utils args check-list-arg

use std formats *

# run the "recoding" benchmarks
#
# - input: the list of input file sizes
# - output: the output path, as NDJSON
export def run [
    --output: path, # the output path (defaults to a random file in $nu.temp-path)
    --ks: list<int>, # the values of $k$ to benchmark
    --curves: list<string>, # the curves to benchmark
    --no-confirm (-y), # does not ask for confirmation if the output file already exists, it will be overwritten
    --nb-measurements: int = 10, # the number of measurements per benchmark run
    --append, # append to the output path instead of overwritting
]: list<int> -> path {
    $ks | check-list-arg --cmd "recoding run" --arg "--ks" --span (metadata $ks).span
    $curves | check-list-arg --cmd "recoding run" --arg "--curves" --span (metadata $curves).span
    $in | check-list-arg --cmd "recoding run" --arg "pipeline input"

    let new_file = $output == null
    let output = $output | default (mktemp --tmpdir komodo_recoding.XXXXXX)
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
            --bin recoding
            --
            --nb-measurements $nb_measurements
            ...$input
            --shards $k
            --ks $k
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

# plot the "recoding" benchmark results
export def plot [
    data: path, # where to load the data from
    --save: path, # an optional path where to save the figure (defaults to showing the figure interactively)
] {
    check-file $data --span (metadata $data).span

    let graphs = open --raw $data
        | from ndjson
        | ns-to-ms $.times
        | compute-stats $.times
        | update label { from nuon }
        | flatten --all label
        | where name == "BLS12-381"
        | rename --column { bytes: "x", mean: "y", stddev: "e" }
        | select shards x y e
        | group-by shards --to-table
        | reject items.shards
        | rename --column { shards: "name", items: "points" }
        | update name { $"$k = ($in)$"}

    let options = [
        # --y-label "time (in ms)"
        ...($graphs.points | flatten | into-axis-options -x "filesize" -y "duration")
        --no-legend
        ...$COMMON_OPTIONS
        (if $save != null { [ --save $save ] })
    ]

    gplt plot ($graphs | to json) ...($options | flatten | compact)
}
