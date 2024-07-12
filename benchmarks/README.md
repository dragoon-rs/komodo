## requirements
- install [GPLT](https://gitlab.isae-supaero.fr/a.stevan/gplt)

```nushell
use .nushell/math.nu *
use .nushell/formats.nu *
```

## atomic operations
```nushell
cargo run --release --package benchmarks --bin field_operations -- --nb-measurements 1000 out> field.ndjson
cargo run --release --package benchmarks --bin curve_group_operations -- --nb-measurements 1000 out> curve_group.ndjson
```
```nushell
use .nushell/parse.nu read-atomic-ops

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
cargo run --release --package benchmarks --bin linalg -- --nb-measurements 10 ...$sizes out> linalg.ndjson
```
```nushell
use .nushell/plot.nu [ "into-axis-options", COMMON_OPTIONS ]

let linalg = open linalg.ndjson
    | ns-to-ms $.times
    | compute-stats $.times
    | update label { parse "{op} {n}"}
    | flatten --all label
    | into int n

for graph in [
    [op, title];

    ["inverse", "time to inverse an $n \\times n$ matrix"],
    ["transpose", "time to transpose an $n \\times n$ matrix"],
    ["mul", "time to multiply two $n \\times n$ matrices"]
] {
    let graphs = $linalg
            | where op == $graph.op
            | rename --column { n: "x", mean: "y", stddev: "e" }
            | group-by name --to-table
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
    gplt plot ...[
        --title $graph.title
        --x-label "n"
        --use-tex
        ($graphs | to json)
        ...$COMMON_OPTIONS
        ...($graphs.points | flatten | into-axis-options -x "plain" -y "duration")
        --x-ticks-rotation 0
    ]
}
```

## trusted setup
```nushell
use .nushell/setup/run.nu; seq 0 13 | each { 2 ** $in } | run --output setup.ndjson --curves [ bls12381, pallas, bn254 ]
```
```nushell
use ./.nushell/setup/plot.nu; plot setup.ndjson
```

## commit
```nushell
use .nushell/commit/run.nu; seq 0 13 | each { 2 ** $in } | run --output commit.ndjson --curves [bls12381, pallas, bn254 ]
```
```nushell
use ./.nushell/commit/plot.nu; plot commit.ndjson
```

## end-to-end benchmarks
### recoding
```nushell
use .nushell/recoding/run.nu
seq 0 18 | each { 512 * 2 ** $in } | run --ks [2, 4, 8, 16] --output recoding.ndjson --curves [ bls12381 ]
```
```nushell
use ./.nushell/recoding/plot.nu; plot recoding.ndjson
```

### FEC
```nushell
use .nushell/fec/run.nu
seq 0 18 | each { 512 * 2 ** $in } | run --ks [2, 4, 8, 16] --output fec.ndjson --curves [ bls12381 ]
```
```nushell
use ./.nushell/fec/plot.nu; plot encoding fec.ndjson
use ./.nushell/fec/plot.nu; plot decoding fec.ndjson
use ./.nushell/fec/plot.nu; plot e2e fec.ndjson
```

## combined graph
```nushell
use ./.nushell/fec/plot.nu; plot combined fec.ndjson --recoding recoding.ndjson
```

## ratio graph
```nushell
use ./.nushell/fec/plot.nu; plot ratio fec.ndjson --recoding recoding.ndjson
```
