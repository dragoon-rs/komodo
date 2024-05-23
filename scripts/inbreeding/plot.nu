#!/usr/bin/env nu

use std repeat

use options.nu
use ../plot.nu gplt

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

def main [data: path, --save: path] {
    let data = open $data
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
                    let sigma = if $it.strategy.n == $options.K {
                        "k"
                    } else {
                        $"k - ($options.K - $it.strategy.n)"
                    }
                    $"$\\sigma = ($sigma) = ($it.strategy.n)$"
                }
                "double" => $"($it.strategy.p)? ($it.strategy.n) ; ($it.strategy.m)"
            }
        }
        | update diversity {|it|
            let l = $it.diversity | length
            $it.diversity | wrap y | merge (seq 0 $l | wrap x) | insert e 0
        }
        | rename --column { diversity: "points" }
        | insert style {|it|
            let color = match $it.strategy.n {
                10 => "tab:red",
                9 => "tab:orange",
                8 => "tab:olive",
                7 => "tab:blue",
                6 => "tab:purple",
                5 => "tab:green",
                4 => "tab:cyan",
                3 => "tab:brown",
                2 => "tab:pink",
                _ => "tab:gray",
            }

            { color: $color, line: { alpha: ($it.strategy.p? | default 1.0) } }
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
