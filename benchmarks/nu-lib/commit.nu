use utils log
use utils math *
use utils fs check-file
use utils plot [ into-axis-options, COMMON_OPTIONS, gplt ]
use utils args check-list-arg

use std formats *

# run the "commit" benchmarks
#
# - input: the list of polynomial degrees
# - output: the output path, as NDJSON
export def run [
    --output: path, # the output path (defaults to a random file in $nu.temp-path)
    --curves: list<string>, # the curves to benchmark
    --no-confirm (-y), # does not ask for confirmation if the output file already exists, it will be overwritten
    --nb-measurements: int = 10, # the number of measurements per benchmark run
    --append, # append to the output path instead of overwritting
]: list<int> -> path {
    $curves | check-list-arg --cmd "commit run" --arg "--curves" --span (metadata $curves).span
    $in | check-list-arg --cmd "commit run" --arg "pipeline input"

    let new_file = $output == null
    let output = $output | default (mktemp --tmpdir komodo_commit.XXXXXX)
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

    let options = [
        --release
        --package benchmarks
        --bin commit
        --
        --nb-measurements $nb_measurements
        ...$in
        --curves ...$curves
    ]
    if $append {
        cargo run ...$options out>> $output
    } else {
        cargo run ...$options out> $output
    }

    log info $"results saved to ($pretty_output)"
    $output
}

# plot the "commit" benchmark results
export def plot [
    data: path, # where to load the data from
    --save: path, # an optional path where to save the figure (defaults to showing the figure interactively)
] {
    check-file $data --span (metadata $data).span

    let raw = open --raw $data
        | from ndjson
        | where name !~ '^SEC'
        | ns-to-ms $.times
        | compute-stats $.times
        | insert degree { get label | parse "degree {d}" | into record | get d | into int }
    let graphs = $raw
        | rename --column { degree: "x", mean: "y", stddev: "e" }
        | select name x y e
        | group-by name --to-table
        | reject items.name
        | rename --column { name: "name", items: "points" }
        | insert style.color {|it|
            match $it.name {
                "BLS12-381" => "tab:blue"
                "PALLAS" => "tab:green"
                "BN254" => "tab:orange"
                "CP6-782" => "tab:olive"
                "ED-MNT4-298" => "tab:pink"
                "MNT4-753" => "tab:red"
                _ => "tab:grey"
            }
        }
        | insert style.line.marker.shape {|it|
            match $it.name {
                "BLS12-381" => "s"
                "PALLAS" => "o"
                "BN254" => "^"
                "CP6-782" => "*"
                "ED-MNT4-298" => "X"
                "MNT4-753" => "d"
                _ => null
            }
        }
        | insert style.line.marker.size { 10 }
        | sort-by name

    let options = [
        # --title "time to create trusted setups for certain curves"
        --x-label '$\log_2 d$'
        # --y-label "time"
        ...($graphs.points | flatten | into-axis-options -x "plain" -y "duration")
        ...$COMMON_OPTIONS
        (if $save != null { [ --save $save ] })
        --x-tick-labels ($raw.degree | uniq | math log 2)
        --x-ticks-rotation 0
    ]

    gplt plot ($graphs | to json) ...($options | flatten | compact)
}
