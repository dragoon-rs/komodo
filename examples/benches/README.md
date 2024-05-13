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
### recoding
```nushell
"" out> recoding.ndjson

[2, 4, 8, 16] | each { |k|
    cargo run --example bench_recoding -- ...[
        --nb-measurements 10
        ...[100, 1_000, 10_000]
        --shards $k
        --ks $k
    ] | from ndnuon | to ndjson out>> recoding.ndjson
}
```
```nushell
gplt plot --title "k-recoding with k = #shards" --x-label '\#bytes' --y-label "time (in ms)" (
    open recoding.ndjson
        | ns-to-ms $.times
        | compute-stats $.times
        | update label { from nuon }
        | flatten --all label
        | insert case { $"($in.name) / ($in.shards)" }
        | rename --column { bytes: "x", mean: "y", stddev: "e" }
        | group-by case --to-table
        | rename --column { group: "name", items: "points" }
        | insert style {|it|
            let g = $it.name | parse "{c} / {s}" | into record | into int s
            let c = match $g.c {
                "BLS12-381" => "blue"
                "BN-254" => "orange"
                "PALLAS" => "green"
                _ => "gray"
            }
            let t = match $g.s {
                2 => "dotted"
                4 => "dashdot"
                8 => "dashed"
                16 => "solid"
                _ => "loosely dotted"
            }
            { color: $c, line: { type: $t } }
        }
        | to json
)
```
an alternate plot
```nushell
gplt plot ...[
    # --title "k-recoding with k = #shards"
    --x-label '\#bytes'
    --y-label "time (in ms)"
    (
        open recoding.ndjson
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
            | to json
    )
    --fullscreen
    --dpi 150
    --fig-size ...[16, 9]
    --font ({ size: 30, family: serif, sans-serif: Helvetica } | to json)
    --use-tex
    # --save recoding.pdf
]
```

### FEC
```nushell
"" out> fec.ndjson

[2, 4, 8, 16] | each { |k|
    cargo run --example bench_fec  -- ...[
        ...[100, 1_000, 10_000]
        --encoding vandermonde
        -k $k
        -n 1
        --nb-measurements 100
    ] | from ndnuon | to ndjson out>> fec.ndjson
}
```
```nushell
gplt plot ...[
    # --title "1-encoding"
    --x-label '\#bytes'
    --y-label "time (in ms)"
    (
        open fec.ndjson
            | update label { from json }
            | flatten label
            | ns-to-ms times
            | compute-stats times
            | where name == "BLS12-381" and step == "encode"
            | rename --column { bytes: "x", mean: "y", stddev: "e" }
            | select k x y e
            | group-by k --to-table
            | reject items.k
            | rename --column { group: "name", items: "points" }
            | update name { $"$k = ($in)$" }
            | to json
    )
    --fullscreen
    --dpi 150
    --fig-size ...[16, 9]
    --font ({ size: 30, family: serif, sans-serif: Helvetica } | to json)
    --use-tex
    # --save encoding.pdf
]

gplt plot ...[
    # --title "k-encoding"
    --x-label '\#bytes'
    --y-label "time (in ms)"
    (
        open fec.ndjson
            | update label { from json }
            | flatten label
            | ns-to-ms times
            | compute-stats times
            | where name == "BLS12-381" and step == "decode"
            | rename --column { bytes: "x", mean: "y", stddev: "e" }
            | select k x y e
            | group-by k --to-table
            | reject items.k
            | rename --column { group: "name", items: "points" }
            | update name { $"$k = ($in)$" }
            | to json
    )
    --fullscreen
    --dpi 150
    --fig-size ...[16, 9]
    --font ({ size: 30, family: serif, sans-serif: Helvetica } | to json)
    --use-tex
    # --save decoding.pdf
]

let x = open fec.ndjson
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

gplt plot ...[
    # --title "e2e: k-decoding + 1-encoding"
    --x-label '\#bytes'
    --y-label "time (in ms)"
    (
        $x
            | ns-to-ms times
            | compute-stats times
            | reject times
            | where name == "BLS12-381"
            | rename --column { bytes: "x", mean: "y", stddev: "e" }
            | select k x y e
            | group-by k --to-table
            | reject items.k
            | rename --column { group: "name", items: "points" }
            | update name { $"$k = ($in)$" }
            | to json
    )
    --fullscreen
    --dpi 150
    --fig-size ...[16, 9]
    --font ({ size: 30, family: serif, sans-serif: Helvetica } | to json)
    --use-tex
    # --save e2e.pdf
]
```
