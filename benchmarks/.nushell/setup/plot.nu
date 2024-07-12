use ../../../.nushell math *
use ../../../.nushell fs check-file
use ../../../.nushell plot [ into-axis-options, COMMON_OPTIONS, gplt ]

export def main [data: path, --save: path] {
    check-file $data --span (metadata $data).span

    let raw = open $data
        | ns-to-ms times
        | compute-stats times
        | insert degree { get label | parse "degree {d}" | into record | get d | into int }
        | update name {|it| if ($it.name | str starts-with  "ARK") {
            let c = $it.name | parse "ARK setup on {curve}" | into record | get curve
            $"($c)-ark"
        } else {
            $it.name | parse "setup on {curve}" | into record | get curve
        }}
    let graphs = $raw
        | rename --column { degree: "x", mean: "y", stddev: "e" }
        | select name x y e
        | group-by name --to-table
        | reject items.name
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
