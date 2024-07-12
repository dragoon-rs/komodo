use ../../../nu-utils math *
use ../../../nu-utils plot [ into-axis-options, COMMON_OPTIONS ]
use ../../../nu-utils fs check-file
use ../../../nu-utils plot gplt

use std formats *

# plot the "encoding" benchmark results
export def encoding [
    data: path, # where to load the data from
    --save: path, # an optional path where to save the figure (defaults to showing the figure interactively)
] {
    check-file $data --span (metadata $data).span

    let graphs = open --raw $data
        | from ndjson
        | update label { from json }
        | flatten label
        | ns-to-ms times
        | compute-stats times
        | where name == "BLS12-381" and step == "encode"
        | rename --column { bytes: "x", mean: "y", stddev: "e" }
        | select k x y e
        | sort-by x
        | group-by k --to-table
        | reject items.k
        | rename --column { group: "name", items: "points" }
        | update name { $"$k = ($in)$" }

    let options = [
        # --title "1-encoding"
        ...($graphs.points | flatten | into-axis-options -x "filesize" -y "duration")
        ...$COMMON_OPTIONS
        (if $save != null { [ --save $save ] })
    ]

    gplt plot ($graphs | to json) ...($options | flatten | compact)
}

# plot the "decoding" benchmark results
export def decoding [
    data: path, # where to load the data from
    --save: path, # an optional path where to save the figure (defaults to showing the figure interactively)
] {
    check-file $data --span (metadata $data).span

    let graphs = open --raw $data
        | from ndjson
        | update label { from json }
        | flatten label
        | ns-to-ms times
        | compute-stats times
        | where name == "BLS12-381" and step == "decode"
        | rename --column { bytes: "x", mean: "y", stddev: "e" }
        | select k x y e
        | sort-by x
        | group-by k --to-table
        | reject items.k
        | rename --column { group: "name", items: "points" }
        | update name { $"$k = ($in)$" }

    let options = [
        # --title "k-encoding"
        ...($graphs.points | flatten | into-axis-options -x "filesize" -y "duration")
        --no-legend
        ...$COMMON_OPTIONS
        (if $save != null { [ --save $save ] })
    ]

    gplt plot ($graphs | to json) ...($options | flatten | compact)
}

# plot the "end to end" benchmark results, i.e. a $k$-decoding and a $1$-encoding
export def e2e [
    data: path, # where to load the data from
    --save: path, # an optional path where to save the figure (defaults to showing the figure interactively)
] {
    check-file $data --span (metadata $data).span

    let graphs = open --raw $data
        | from ndjson
        | update label { from json }
        | flatten label
        | insert foo { $"($in.name) / ($in.k) / ($in.bytes)" }
        | group-by foo --to-table
        | update items {|it|
            $it.items
                | update step e2e
                | update times { $it.items.0.times | zip $it.items.1.times | each { $in.0 + $in.1 } }
        }
        | flatten --all
        | reject group foo
        | ns-to-ms times
        | compute-stats times
        | reject times
        | where name == "BLS12-381"
        | rename --column { bytes: "x", mean: "y", stddev: "e" }
        | select k x y e
        | sort-by x
        | group-by k --to-table
        | reject items.k
        | rename --column { group: "name", items: "points" }
        | update name { $"$k = ($in)$" }

    let options = [
        # --title "k-encoding + 1-encoding"
        ...($graphs.points | flatten | into-axis-options -x "filesize" -y "duration")
        --no-legend
        ...$COMMON_OPTIONS
        (if $save != null { [ --save $save ] })
    ]

    gplt plot ($graphs | to json) ...($options | flatten | compact)
}

# plot the "combined" benchmark results, i.e. the "end to end" and "recoding" plots on the same figure
export def combined [
    data: path, # where to load the "fec" data from
    --recoding: path,  # where to load the "recoding" data from
    --save: path, # an optional path where to save the figure (defaults to showing the figure interactively)
] {
    check-file $data --span (metadata $data).span
    check-file $recoding --span (metadata $recoding).span

    let recoding_graphs = open --raw $recoding
        | from ndjson
        | ns-to-ms $.times
        | compute-stats $.times
        | update label { from nuon }
        | flatten --all label
        | where name == "BLS12-381"
        | rename --column { bytes: "x", mean: "y", stddev: "e" }
        | select shards x y e
        | sort-by x
        | group-by shards --to-table
        | insert style.color {|it|
            match $it.items.shards.0 {
                2 => "tab:blue"
                4 => "tab:orange"
                8 => "tab:green"
                16 => "tab:red"
                _ => "tab:grey"
            }
        }
        | reject items.shards
        | insert style.line.type "solid"
        | rename --column { group: "name", items: "points" }
        | update name { $"$k = ($in)$" }

    let re_encoding_graphs = open --raw $data
        | from ndjson
        | update label { from json }
        | flatten label
        | insert key { $"($in.name) / ($in.k) / ($in.bytes)" }
        | group-by key --to-table
        | update items {|it|
            $it.items
                | update step e2e
                | update times { $it.items.0.times | zip $it.items.1.times | each { $in.0 + $in.1 } }
        }
        | flatten --all
        | reject group key
        | ns-to-ms times
        | compute-stats times
        | reject times
        | where name == "BLS12-381"
        | rename --column { bytes: "x", mean: "y", stddev: "e" }
        | select k x y e
        | sort-by x
        | group-by k --to-table
        | insert style.color {|it|
            match $it.items.k.0 {
                2 => "tab:blue"
                4 => "tab:orange"
                8 => "tab:green"
                16 => "tab:red"
                _ => "tab:grey"
            }
        }
        | insert style.line.type "dashed"
        | reject items.k
        | rename --column { group: "name", items: "points" }
        | reject name

    let graphs = $recoding_graphs
        | append $re_encoding_graphs
        | append {
            name: "$(k, 1)$-re-encoding ($k$-decoding + $1$-encoding)",
            legend: "second",
            points: [],
            style: {
                color: "grey",
                line: {
                    type: "dashed",
                    marker: {
                        size: 0,
                    },
                },
            },
        }
        | append {
            name: "$k$-recoding",
            legend: "second",
            points: [],
            style: {
                color: "grey",
                line: {
                    type: "solid",
                    marker: {
                        size: 0,
                    },
                },
            },
        }

    let options = [
        ...($graphs.points | flatten | into-axis-options -x "filesize" -y "duration")
        --legend-loc "upper left" "lower right"
        ...$COMMON_OPTIONS
        (if $save != null { [ --save $save ] })
    ]

    gplt plot ($graphs | to json) ...($options | flatten | compact)
}

# plot the "ratio" benchmark results, i.e. the ratio between "end to end" and "recoding"
export def ratio [
    data: path, # where to load the "fec" data from
    --recoding: path,  # where to load the "recoding" data from
    --save: path, # an optional path where to save the figure (defaults to showing the figure interactively)
] {
    check-file $data --span (metadata $data).span
    check-file $recoding --span (metadata $recoding).span

    let recoding_graphs = open --raw $recoding
        | from ndjson
        | ns-to-ms times
        | compute-stats $.times
        | update label { from nuon }
        | flatten --all label
        | where name == "BLS12-381"
        | select shards bytes mean
        | rename --column { shards: "k" }

    let re_encoding_graphs = open --raw $data
        | from ndjson
        | update label { from json }
        | flatten label
        | insert key { $"($in.name) / ($in.k) / ($in.bytes)" }
        | group-by key --to-table
        | update items {|it|
            $it.items
                | update step e2e
                | update times { $it.items.0.times | zip $it.items.1.times | each { $in.0 + $in.1 } }
        }
        | flatten --all
        | reject group key
        | ns-to-ms times
        | compute-stats times
        | where name == "BLS12-381"
        | select k bytes mean
        | uniq

    let graphs = $recoding_graphs
        | rename --column { mean: "recoding" }
        | insert key { $"($in.k) ($in.bytes)" }
        | join ($re_encoding_graphs | rename --column { mean: "re_encoding" } | insert key { $"($in.k) ($in.bytes)" }) key
        | select k bytes $.recoding re_encoding
        | sort-by k bytes
        | insert cmp { $in.re_encoding / $in.recoding }
        | rename --column { bytes: "x", cmp: "y" }
        | select k x y
        | group-by k --to-table
        | insert style.color {|it|
            match $it.items.k.0 {
                2 => "tab:blue"
                4 => "tab:orange"
                8 => "tab:green"
                16 => "tab:red"
                _ => "tab:grey"
            }
        }
        | reject items.k
        | rename --column { group: "name", items: "points" }
        | update name { $"$k = ($in)$" }

    let options = [
        ...($graphs.points | flatten | into-axis-options -x "filesize" -y "plain")
        --legend-loc "upper right"
        ...$COMMON_OPTIONS
        (if $save != null { [ --save $save ] })
    ]

    gplt plot ($graphs | to json) ...($options | flatten | compact)
}
