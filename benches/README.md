## run the benchmarks
```shell
nushell> cargo criterion --output-format verbose --message-format json out> results.ndjson
```

## add the _trusted setup_ sizes
```shell
nushell> cargo run --example bench_setup_size out>> results.ndjson
```

## plot the results
```shell
python scripts/plot/benches.py results.ndjson --bench linalg
python scripts/plot/benches.py results.ndjson --bench setup
```

## atomic operations
```nushell
cargo run --example bench_field_operations -- --nb-measurements 1000
    | lines
    | each { from nuon }
    | to ndjson
    | save --force field.ndjson
cargo run --example bench_curve_group_operations -- --nb-measurements 1000
    | lines
    | each { from nuon }
    | to ndjson
    | save --force curve_group.ndjson
```
```nushell
def read-atomic-ops [
    --include: list<string> = [], --exclude: list<string> = []
]: list -> record {
    let raw = $in
        | insert t {|it| $it.times |math avg}
        | reject times
        | rename --column { op: "group", curve: "species", t: "measurement" }

    let included = if $include != [] {
        $raw | where group in $include
    } else {
        $raw
    }

    $included
        | where group not-in $exclude
        | group-by group --to-table
        | reject items.group
        | update items { transpose -r | into record }
        | transpose -r
        | into record
}
```
```nushell
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

## oneshot benchmarks
these are benchmarks that run a single measurement, implemented as _examples_ in
`examples/benches/`.

### commit
```nushell
let degrees = seq 0 15 | each { 2 ** $in }
let res = cargo run --example bench_commit -- --nb-measurements 10 ...$degrees
    | lines
    | each { from nuon }
    | update times { into duration }
    | insert mean {|it| $it.times | math avg}
    | insert stddev {|it| $it.times | into int | into float | math stddev | into int | into duration}

python scripts/plot/bench_commit.py (
    $res | group-by curve --to-table | update items { reject curve } | to json
)
```
