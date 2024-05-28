use std repeat

use consts.nu
use ../../.nushell/plot.nu gplt
use ../../.nushell/color.nu *
use ../../.nushell/error.nu "error throw"

def "parse strategy" []: string -> record<type: string> {
    let s = $in

    if ($s | str starts-with "single") {
        let res = $s
            | parse "single:{n}"
            | into record
            | into int n
        { type: "single", n: $res.n }
    } else {
        let res = $s
            | parse "double:{p}:{n}:{m}"
            | into record
            | into float p
            | into int n
            | into int m
        { type: "double", p: $res.p, n: $res.n, m: $res.m }
    }
}

def get-color []: int -> string {
    match $in {
        10 => "#d62728",
        9 => "#ff7f0e",
        8 => "#bcbd22",
        7 => "#1f77b4",
        6 => "#9467bd",
        5 => "#2ca02c",
        4 => "#17becf",
        3 => "#8c564b",
        2 => "#e377c2",
        _ => "#7f7f7f",
    }
}

export def main [
    --save: path,
    --options: record<k: int>
]: table<strategy: string, diversity: table<x: int, y: float, e: float>> -> nothing {
    let data = $in
    let l = $data.diversity.0 | length

    $data
        | update strategy { parse strategy }
        | insert sort {|it|
            match $it.strategy.type {
                "single" => [$it.strategy.n, 1.0]
                "double" => [$it.strategy.n, $it.strategy.p]
            }
        }
        | sort-by sort
        | reverse
        | reject sort
        | insert name {|it|
            match $it.strategy.type {
                "single" => {
                    let sigma = if $it.strategy.n == $options.k {
                        "k"
                    } else {
                        $"k - ($options.k - $it.strategy.n)"
                    }
                    $"$\\sigma = ($sigma) = ($it.strategy.n)$"
                }
                "double" => $"($it.strategy.p)? ($it.strategy.n) ; ($it.strategy.m)"
            }
        }
        | rename --column { diversity: "points" }
        | insert style {|it|
            let color = match $it.strategy.type {
                "single" => { $it.strategy.n | get-color },
                "double" => {
                    let c1 = $it.strategy.n | get-color | color from-string $in
                    let c2 = $it.strategy.m | get-color | color from-string $in
                    let c = $it.strategy.p

                    color mix $c1 $c2 $c | color to-hex
                },
            }

            let alpha = match $it.strategy.type {
                "single" => 1.0,
                "double" => 0.3,
            }
            let type = match $it.strategy.type {
                "single" => "solid",
                "double" => "dashed",
            }

            { color: $color, line: { alpha: $alpha, type: $type } }
        }
        | reject strategy
        | save --force /tmp/graphs.json

    let x_min = open /tmp/graphs.json | get points.0.x | math min
    let x_max = open /tmp/graphs.json | get points.0.x | math max

    let args = [
        --json-data-file /tmp/graphs.json
        --x-lim ($x_min - 1) ($x_max + 1)
        --y-lim -0.01 1.01
        --fullscreen
        # --title "diversity over time when recoding shards $r$ shards"
        --x-label "time (in nb of steps)"
        --y-label "diversity $\\delta$"
        --dpi 150
        --fig-size ...[16, 5]
        --font ({ size: 15, family: serif, sans-serif: Helvetica } | to json)
        --use-tex
        --legend-loc "upper right"
        (if $save != null { [ --save $save ] })
    ]

    gplt plot ...($args | flatten | compact)

}
