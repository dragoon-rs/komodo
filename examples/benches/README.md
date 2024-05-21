## requirements
- install [GPLT](https://gitlab.isae-supaero.fr/a.stevan/gplt)

```nushell
use scripts/math.nu *
use scripts/formats.nu *
```

## atomic operations
```nushell
cargo run --example bench_field_operations -- --nb-measurements 1000 out> field.ndjson
cargo run --example bench_curve_group_operations -- --nb-measurements 1000 out> curve_group.ndjson
```
```nushell
use scripts/parse.nu read-atomic-ops

gplt multi_bar --title "simple field operations" -l "time (in ns)" (
    open field.ndjson
        | read-atomic-ops --exclude [ "exponentiation", "legendre", "inverse", "sqrt" ]
        | to json
)
gplt multi_bar --title "complex field operations" -l "time (in ns)" (
    open field.ndjson
        | read-atomic-ops --include [ "exponentiation", "legendre", "inverse", "sqrt" ]
        | to json
)
gplt multi_bar --title "simple curve group operations" -l "time (in ns)" (
    open curve_group.ndjson
        | read-atomic-ops --exclude [ "random sampling", "scalar multiplication", "affine scalar multiplication" ]
        | to json
)
gplt multi_bar --title "complex curve group operations" -l "time (in ns)" (
    open curve_group.ndjson
        | read-atomic-ops --include [ "random sampling", "scalar multiplication", "affine scalar multiplication" ]
        | to json
)
```

## linear algebra
```nushell
let sizes = seq 0 7 | each { 2 ** $in }
cargo run --example bench_linalg -- --nb-measurements 10 ...$sizes out> linalg.ndjson
    | save --force linalg.ndjson
```
```nushell
let linalg = open linalg.ndjson
    | ns-to-ms $.times
    | compute-stats $.times
    | update label { parse "{op} {n}"}
    | flatten --all label
    | into int n

for graph in [
    [op, title];

    ["inverse", "time to inverse an nxn matrix on certain curves"],
    ["transpose", "time to transpose an nxn matrix on certain curves"],
    ["mul", "time to multiply two nxn matrices on certain curves"]
] {
    gplt plot ...[
        --title $graph.title
        --x-label "size"
        --y-label "time (in ms)"
        --x-scale "log"
        --y-scale "log"
        (
            $linalg
                | where op == $graph.op
                | rename --column { n: "x", mean: "y", stddev: "e" }
                | group-by name --to-table
                | rename --column { group: "name", items: "points" }
                | to json
        )
    ]
}
```

## trusted setup
```nushell
let degrees = seq 0 13 | each { 2 ** $in }
cargo run --example bench_setup -- --nb-measurements 10 ...$degrees out> setup.ndjson
```
```nushell
gplt plot ...[
    # --title "time to create trusted setups for certain curves"
    --x-label "degree"
    --y-label "time (in ms)"
    (
        open setup.ndjson
            | ns-to-ms $.times
            | compute-stats $.times
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
            | to json
    )
    --fullscreen
    --dpi 150
    --fig-size ...[16, 9]
    --font ({ size: 30, family: serif, sans-serif: Helvetica } | to json)
    --use-tex
    # --save setup.pdf
]
```

## commit
```nushell
let degrees = seq 0 15 | each { 2 ** $in }
cargo run --example bench_commit -- --nb-measurements 10 ...$degrees out> commit.ndjson
```
```nushell
gplt plot ...[
    # --title "time to commit polynomials for certain curves"
    --x-label "degree"
    --y-label "time (in ms)"
    (
        open commit.ndjson
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
            | to json
    )
    --fullscreen
    --dpi 150
    --fig-size ...[16, 9]
    --font ({ size: 30, family: serif, sans-serif: Helvetica } | to json)
    --use-tex
    # --save commit.pdf
]
```

## end-to-end benchmarks
```nushell
use scripts/plot.nu [
    into-duration-tick-labels,
    into-filesize-tick-labels,
    into-axis-options,
    COMMON_OPTIONS,
]
```

### recoding
```nushell
"" out> recoding.ndjson

[2, 4, 8, 16] | each { |k|
    cargo run --example bench_recoding -- ...[
        --nb-measurements 10
        ...(seq 0 18 | each { 512 * 2 ** $in })
        --shards $k
        --ks $k
    ] | from ndnuon | to ndjson out>> recoding.ndjson
}
```
```nushell
let graphs = open recoding.ndjson
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

gplt plot ($graphs | to json) ...[
    # --y-label "time (in ms)"
    ...($graphs.points | flatten | into-axis-options)
    --no-legend
    ...$COMMON_OPTIONS
    # --save recoding.pdf
]
```

### FEC
```nushell
"" out> fec.ndjson

[2, 4, 8, 16] | each { |k|
    cargo run --example bench_fec  -- ...[
        ...(seq 0 18 | each { 512 * 2 ** $in })
        --encoding vandermonde
        -k $k
        -n 1
        --nb-measurements 10
    ] | from ndnuon | to ndjson out>> fec.ndjson
}
```
```nushell
let graphs = open fec.ndjson
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

gplt plot ($graphs | to json) ...[
    # --title "1-encoding"
    ...($graphs.points | flatten | into-axis-options -y "duration")
    ...$COMMON_OPTIONS
    # --save encoding.pdf
]

let graphs = open fec.ndjson
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

gplt plot ($graphs | to json) ...[
    # --title "k-decoding"
    --y-label "time (in ms)"
    ...($graphs.points | flatten | into-axis-options -y "duration")
    --no-legend
    ...$COMMON_OPTIONS
    # --save decoding.pdf
]

let graphs = open fec.ndjson
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

gplt plot ($graphs | to json) ...[
    # --title "e2e: k-decoding + 1-encoding"
    --y-label "time (in ms)"
    ...($graphs.points | flatten | into-axis-options -y "duration")
    --no-legend
    ...$COMMON_OPTIONS
    # --save e2e.pdf
]
```

combined graph
```nushell
let true_recoding_graphs = open recoding.ndjson
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

let naive_recoding_graphs = open fec.ndjson
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

let graphs = $true_recoding_graphs
    | append $naive_recoding_graphs
    | append {
        name: "naive recoding ($k$-decoding + $1$-encoding)",
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
        name: "true recoding ($k$-recoding)",
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

gplt plot ($graphs | to json) ...[
    ...($graphs.points | flatten | into-axis-options -y "duration")
    ...$COMMON_OPTIONS
    --legend-loc "upper left" "lower right"
    # --save comparison.png
]
```

ratio graph
```nushell
let true_recoding_graphs = open recoding.ndjson
    | ns-to-ms times
    | compute-stats $.times
    | update label { from nuon }
    | flatten --all label
    | where name == "BLS12-381"
    | select shards bytes mean
    | rename --column { shards: "k" }

let naive_recoding_graphs = open fec.ndjson
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

let graphs = $true_recoding_graphs
    | rename --column { mean: "true" }
    | insert key { $"($in.k) ($in.bytes)" }
    | join ($naive_recoding_graphs | rename --column { mean: "naive" } | insert key { $"($in.k) ($in.bytes)" }) key
    | select k bytes $.true naive
    | sort-by k bytes
    | insert cmp { $in.naive / $in.true }
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

gplt plot ($graphs | to json) ...[
    ...($graphs.points | flatten | into-axis-options)
    ...$COMMON_OPTIONS
    --legend-loc "upper right"
    # --save ratio.png
]
```
