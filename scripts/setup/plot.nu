use ../math.nu *
use ../fs.nu check-file
use ../plot.nu gplt

export def main [data: path, --save: path] {
    let options = [
        # --title "time to create trusted setups for certain curves"
        --x-label "degree"
        --y-label "time (in ms)"
        --fullscreen
        --dpi 150
        --fig-size ...[16, 9]
        --font ({ size: 30, family: serif, sans-serif: Helvetica } | to json)
        --use-tex
        (if $save != null { [ --save $save ] })
    ]

    check-file $data --span (metadata $data).span

    open $data
        | ns-to-ms times
        | compute-stats times
        | insert degree { get label | parse "degree {d}" | into record | get d | into int}
        | update name {|it| if ($it.name | str starts-with  "ARK") {
            let c = $it.name | parse "ARK setup on {curve}" | into record | get curve
            $"($c)-ark"
        } else {
            $it.name | parse "setup on {curve}" | into record | get curve
        }}
        | rename --column { degree: "x", mean: "y", stddev: "e" }
        | select name x y e
        | group-by name --to-table
        | reject items.name
        | rename --column { group: "name", items: "points" }
        | sort-by name
        | gplt plot ($in | to json) ...($options | flatten | compact)
}
