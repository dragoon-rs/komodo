use utils log
use utils math *
use utils fs check-file
use utils plot [ into-axis-options, COMMON_OPTIONS, gplt ]

use std formats *

# run the "linear algebra" benchmarks
#
# - input: the list of matrix sizes
# - output: the output path, as NDJSON
export def run [
    --output: path, # the output path (defaults to a random file in $nu.temp-path)
    --force, # does not ask for confirmation if the output file already exists, it will be overwritten
    --nb-measurements: int = 10, # the number of measurements per benchmark run
]: list<int> -> path {
    let input = $in

    if ($input | is-empty) {
        print "nothing to do"
        return
    }

    let new_file = $output == null
    let output = $output | default (mktemp --tmpdir komodo_linalg.XXXXXX)
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

    cargo run --release --package benchmarks --bin linalg -- ...[
        --nb-measurements $nb_measurements
        ...$input
    ] out> $output

    log info $"results saved to ($pretty_output)"
    $output
}

def load-linalg-data [data: path, --span: record<start: int, end: int>]: [ nothing -> table ] {
    check-file $data --span $span

    open --raw $data
        | from ndjson
        | ns-to-ms $.times
        | compute-stats $.times
        | update label { parse "{op} {n}"}
        | flatten --all label
        | into int n
}

def linalg-operations []: [ nothing -> list<string> ] {
    [ "inverse", "transpose", "mul" ]
}

# plot the "linear algebra" benchmark results
export def plot [
    data: path, # where to load the data from
    op: string@linalg-operations,
    --save: path, # an optional path where to save the figure (defaults to showing the figure interactively)
] {
    check-file $data --span (metadata $data).span
    if $op not-in (linalg-operations) {
        error make {
            msg: $"(ansi red_bold)invalid_linalg_op(ansi reset)",
            label: {
                text: $"invalid linear algebra operation '(ansi yellow)($op)(ansi purple)'(ansi reset)",
                span: (metadata $op).span,
            },
            help: $"please choose one of (ansi cyan)(linalg-operations)(ansi reset)"
        }
    }

    let graphs = open --raw $data
        | from ndjson
        | ns-to-ms $.times
        | compute-stats $.times
        | update label { parse "{op} {n}"}
        | flatten --all label
        | into int n
        | where op == $op
        | rename --column { n: "x", mean: "y", stddev: "e" }
        | group-by name --to-table
        | rename --column { group: "name", items: "points" }
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

    let options = [
        --x-label "n"
        --use-tex
        ...$COMMON_OPTIONS
        ...($graphs.points | flatten | into-axis-options -x "plain" -y "duration")
        (if $save != null { [ --save $save ] })
        --x-ticks-rotation 0
    ]

    gplt plot ($graphs | to json) ...($options | flatten | compact)
}
