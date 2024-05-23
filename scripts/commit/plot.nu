use ../math.nu *
use ../fs.nu check-file
use ../plot.nu [ into-axis-options, COMMON_OPTIONS, gplt ]

export def main [data: path, --save: path] {
    check-file $data --span (metadata $data).span

    let graphs = open $data
        | where name !~ '^SEC'
        | ns-to-ms $.times
        | compute-stats $.times
        | insert degree { get label | parse "degree {d}" | into record | get d | into int }
        | rename --column { degree: "x", mean: "y", stddev: "e" }
        | select name x y e
        | group-by name --to-table
        | reject items.name
        | rename --column { group: "name", items: "points" }
        | sort-by name

    let options = [
        # --title "time to create trusted setups for certain curves"
        --x-label "degree"
        --y-label "time (in ms)"
        ...($graphs.points | flatten | into-axis-options -x "plain" -y "duration")
        ...$COMMON_OPTIONS
        (if $save != null { [ --save $save ] })
    ]

    gplt plot ($graphs | to json) ...($options | flatten | compact)
}
