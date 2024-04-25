```nushell
use scripts/math.nu *
```

## atomic operations
```nushell
cargo run --example bench_field_operations -- --nb-measurements 1000 out> field.ndjson
cargo run --example bench_curve_group_operations -- --nb-measurements 1000 out> curve_group.ndjson
```
```nushell
use scripts/parse.nu read-atomic-ops

python scripts/plot/multi_bar.py --title "simple field operations" -l "time (in ns)" (
    open field.ndjson
        | read-atomic-ops --exclude [ "exponentiation", "legendre", "inverse", "sqrt" ]
        | to json
)
python scripts/plot/multi_bar.py --title "complex field operations" -l "time (in ns)" (
    open field.ndjson
        | read-atomic-ops --include [ "exponentiation", "legendre", "inverse", "sqrt" ]
        | to json
)
python scripts/plot/multi_bar.py --title "simple curve group operations" -l "time (in ns)" (
    open curve_group.ndjson
        | read-atomic-ops --exclude [ "random sampling", "scalar multiplication", "affine scalar multiplication" ]
        | to json
)
python scripts/plot/multi_bar.py --title "complex curve group operations" -l "time (in ns)" (
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
    python scripts/plot/plot.py ...[
        --title $graph.title
        --x-label "size"
        --y-label "time (in ms)"
        --x-scale "log"
        --y-scale "log"
        (
            $linalg
                | where op == $graph.op
                | rename --column { n: "x", name: "curve", mean: "measurement", stddev: "error" }
                | group-by curve --to-table
                | update items { reject curve }
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
python scripts/plot/plot.py ...[
    --title "time to create trusted setups for certain curves"
    --x-label "degree"
    --y-label "time (in ms)"
    (
        open setup.ndjson
            | ns-to-ms $.times
            | compute-stats $.times
            | insert degree { get label | parse "degree {d}" | into record | get d | into int}
            | insert curve {|it| if ($it.name | str starts-with  "ARK") {
                let c = $it.name | parse "ARK setup on {curve}" | into record | get curve
                $"($c)-ark"
            } else {
                $it.name | parse "setup on {curve}" | into record | get curve
            }}
            | rename --column { degree: "x", mean: "measurement", stddev: "error" }
            | group-by curve --to-table
            | update items { reject curve }
            | to json
    )
]
```

## commit
```nushell
let degrees = seq 0 15 | each { 2 ** $in }
cargo run --example bench_commit -- --nb-measurements 10 ...$degrees out> commit.ndjson
```
```nushell
python scripts/plot/plot.py ...[
    --title "time to commit polynomials for certain curves"
    --x-label "degree"
    --y-label "time (in ms)"
    (
        open commit.ndjson
            | ns-to-ms $.times
            | compute-stats $.times
            | update label { parse "degree {d}" | into record | get d | into int }
            | rename --column { label: "x", name: "curve", mean: "measurement", stddev: "error" }
            | group-by curve --to-table
            | update items { reject curve }
            | to json
    )
]
```
