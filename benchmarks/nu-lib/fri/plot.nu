use std formats [ "from ndjson" ]
use ../utils plot [ into-axis-options, COMMON_OPTIONS ]

const NB_MS_IN_NS = 1_000_000

def plot [
    name: string,
    --save,
    --y-type: string,
    --single,
    --identity,
    --normalize,
    --dump-dir: path,
] {
    let ds = $in | get d | uniq

    let graphs = $in
        | select $name d k bf ff
        | group-by { |it| $"($it.k):($it.ff):($it.bf)" }
        | transpose name points
        | update name {
            let res = $in | parse "{k}:{ff}:{bf}" | into record
            $"$k = ($res.k)$, $\\mathbb{F} = $ ($res.ff), $BF = ($res.bf)$"
        }
        | update points {
            rename --column { $name: "y", d: "x" }
                | update y { if $y_type == "duration" { $in / $NB_MS_IN_NS } else { $in } }
                | if $single { update y { |it| $it.y / ($it.k * $it.bf) } } else { $in }
                | if $normalize { update y { |it| $it.y / $it.x } } else { $in }
                | sort-by x
        }
        | insert style { |it|
            let type = match $it.points.ff.0 {
                "fp128" => "solid",
                "bls12-381" => "dashed",
                _ => "",
            }
            let color = match $it.points.k.0 {
                8 => "tab:blue",
                128 => "tab:green",
                1024 => "tab:orange",
                4096 => "tab:red",
                _ => "grey",
            }
            let marker = match $it.points.bf.0 {
                2 => "o",
                4 => "s",
                _ => "*",
            }
            { color: $color, line: { type: $type, marker: { shape: $marker } } }
        }
        | if $identity { append {
            name: "$x \\mapsto x$",
            points: ($ds | wrap x | merge ($ds | wrap y) | if $normalize { update y { |it| $it.y / $it.x } } else { $in }),
            style: { color: "black", line: { type: "dotted" } },
        } } else { $in }

    let title = [
        $name,
        (if $single { "single" }),
        (if $normalize { "normalized" }),
    ] | compact | str join '_'

    let y_type = if $normalize { "plain" } else { $y_type }

    let options = [
        ...($graphs.points | flatten | into-axis-options -x "filesize" -y $y_type)
        --use-tex
        --y-scale log
        --x-scale log
        --x-scale-base 2
        --y-scale-base 2
        --title $title
        ...(if $save { [ --save ($dump_dir | path join $"($title).png") ] } else {[]})
        --fullscreen
    ]

    $graphs | to json | gplt plot $in ...($options | compact)
}

export def main [
    ...x,
    --file: path,
    --y-type: string = "plain",
    --single,
    --identity,
    --normalize,
    --dump-dir: path = "./",
] {
    if ($x | is-empty) {
        error make --unspanned { msg: "nothing to do, x is empty" }
    }
    if $file == null {
        error make --unspanned { msg: "missing --file" }
    }

    if not ($dump_dir | path exists) {
        mkdir $dump_dir
    }

    let data = open $file | where h == "sha3-512" and q == 50

    for i in $x {
        $data | plot --save $i --y-type=$y_type --single=$single --identity=$identity --normalize=$normalize --dump-dir=$dump_dir
    }
}
