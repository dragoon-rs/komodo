use ../../nu-utils log
use ../../nu-utils formats *
use ../../nu-utils math *
use ../../nu-utils plot [ into-axis-options, COMMON_OPTIONS, gplt ]
use ../../nu-utils fs check-file

use std formats *

# run the "recoding" benchmarks
#
# - input: the list of input file sizes
# - output: the output path, as NDJSON
export def run [
    --output: path, # the output path (defaults to a random file in $nu.temp-path)
    --ks: list<int>, # the values of $k$ to benchmark
    --curves: list<string>, # the curves to benchmark
    --force, # does not ask for confirmation if the output file already exists, it will be overwritten
    --nb-measurements: int = 10, # the number of measurements per benchmark run
]: list<int> -> path {
    let input = $in

    if ($ks | is-empty) or ($input | is-empty) or ($curves | is-empty) {
        print "nothing to do"
        return
    }

    let new_file = $output == null
    let output = $output | default (mktemp --tmpdir komodo_recoding.XXXXXX)
    let pretty_output = $"(ansi purple)($output)(ansi reset)"
    if ($output | path exists) and not $new_file {
        log warning $"($pretty_output) already exists"
        if not $force {
            let res = ["no", "yes"] | input list $"Do you want to overwrite ($pretty_output)?"
            if $res == null or $res == "no" {
                log info "aborting"
                return
            }

        }
    }

    "" out> $output

    for k in $ks {
        cargo run --release --package benchmarks --bin recoding -- ...[
            --nb-measurements $nb_measurements
            ...$input
            --shards $k
            --ks $k
            --curves ...$curves
        ] | from ndnuon | to ndjson out>> $output
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
        | rename --column { group: "name", items: "points" }
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
