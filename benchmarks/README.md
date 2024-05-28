## requirements
- install [GPLT](https://gitlab.isae-supaero.fr/a.stevan/gplt)

```nushell
use .nushell/math.nu *
use .nushell/formats.nu *
```

## atomic operations
```nushell
cargo run --release --example bench_field_operations -- --nb-measurements 1000 out> field.ndjson
cargo run --release --example bench_curve_group_operations -- --nb-measurements 1000 out> curve_group.ndjson
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
cargo run --release --example bench_linalg -- --nb-measurements 10 ...$sizes out> linalg.ndjson
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
