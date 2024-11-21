export const COMMON_OPTIONS = [
    --x-scale log
    --x-scale-base 2
    --y-scale log
    --x-ticks-rotation 45
    --fullscreen
    --dpi 150
    --fig-size ...[16, 9]
    --font '{ "size": 30, "family": "serif", "sans-serif": "Helvetica" }'
    --use-tex
]

export def into-duration-tick-labels []: list<int> -> list<string> {
    each { $in * 1ms } | each { to text }
}

export def into-filesize-tick-labels []: list<int> -> list<string> {
    into filesize
        | each {
            if $in < 1000b {
                format filesize iB
            } else if $in < 1000000b {
                format filesize KiB
            } else {
                format filesize MiB
            }
        }
        | each { to text | str replace ".0 " " " }
}

export def into-axis-options [-x: string, -y: string]: table<x: float, y: float> -> list<string> {
    let input = $in

    let xs = $input | flatten | get x | uniq

    let x_tick_labels = match $x {
        "filesize" => ($xs | into-filesize-tick-labels),
        "plain" => $xs,
        _ => {
            print $"warning: ($y) option is unknown for -y"
            $xs
        },
    }
    let options = [
        --x-lim ($xs | first) ($xs | last)
        --x-ticks ...$xs
        --x-tick-labels ...$x_tick_labels
    ]

    let ys = $input | flatten | get y
    let y_ticks = seq ($ys | math min | math log 10 | math ceil | $in - 1) ($ys | math max | math log 10 | math floor)
        | into float
        | each { 10 ** $in }

    let y_tick_labels = match $y {
        "duration" => ($y_ticks | into-duration-tick-labels),
        "filesize" => ($y_ticks | into-filesize-tick-labels),
        "plain" => $y_ticks,
        _ => {
            print $"warning: ($y) option is unknown for -y"
            $y_ticks
        },
    }

    $options | append [
        --y-ticks ...$y_ticks
        --y-tick-labels ...$y_tick_labels
    ]
}

export def --wrapped gplt [...args: string] {
    ^gplt ...$args
}
