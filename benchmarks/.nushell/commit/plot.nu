use ../../../.nushell math *
use ../../../.nushell fs check-file
use ../../../.nushell plot [ into-axis-options, COMMON_OPTIONS, gplt ]

export def main [data: path, --save: path] {
    check-file $data --span (metadata $data).span

    let raw = open $data
        | where name !~ '^SEC'
        | ns-to-ms $.times
        | compute-stats $.times
        | insert degree { get label | parse "degree {d}" | into record | get d | into int }
    let graphs = $raw
        | rename --column { degree: "x", mean: "y", stddev: "e" }
        | select name x y e
        | group-by name --to-table
        | reject items.name
        | rename --column { group: "name", items: "points" }
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
