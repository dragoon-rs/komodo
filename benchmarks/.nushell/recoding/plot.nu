use ../../../.nushell math *
use ../../../.nushell plot [ into-axis-options, COMMON_OPTIONS ]
use ../../../.nushell fs check-file
use ../../../.nushell plot gplt

export def main [data: path, --save: path] {
    check-file $data --span (metadata $data).span

    let graphs = open $data
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
