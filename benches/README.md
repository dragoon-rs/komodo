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
cargo criterion --output-format verbose --message-format json --bench field_operations out> field.ndjson
cargo criterion --output-format verbose --message-format json --bench curve_group_operations out> curve.ndjson
```
```nushell
def read-atomic-ops [
    --clean, --include: list<string> = [], --exclude: list<string> = []
]: list -> record {
    let raw = $in
        | where reason == "benchmark-complete"
        | select id mean.estimate
        | sort-by id
        | update id { parse "{op} on {curve}" }
        | flatten --all
        | rename --column { op: "group", curve: "species", mean_estimate: "measurement" }

    let clean = if $clean {
        $raw | update measurement {|it|
            let species = $it.species
            # FIXME: bug when no parentheses
            let r = (
                $raw
                    | where group == 'random sampling' and species == $species
                    | into record
                    | get measurement
            )
            let l = (
                $raw
                    | where group == 'legendre' and species == $species
                    | into record
                    | get measurement
            )
            match $it.group {
                "addition" | "multiplication" | "substraction" => ($it.measurement - 2 * $r),
                "random sampling" => $it.measurement,
                "sqrt" => ($it.measurement - $r - $l),
                _ => ($it.measurement - $r),
            }
        }
    } else {
        $raw
    }

    let included = if $include != [] {
        $clean | where group in $include
    } else {
        $clean
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
        | read-atomic-ops --clean --exclude [ "exponentiation", "legendre", "inverse", "sqrt" ]
        | to json
)
python scripts/plot/multi_bar.py --title "complex field operations" -l "time (in ns)" (
    open field.ndjson
        | read-atomic-ops --include [ "exponentiation", "legendre", "inverse", "sqrt" ]
        | to json
)
python scripts/plot/multi_bar.py --title "curve group operations" -l "time (in ns)" (
    open curve.ndjson | read-atomic-ops | to json
)
```

## oneshot benchmarks
these are benchmarks that run a single measurement, implemented as _examples_ in
`examples/benches/`.

### commit
```nushell
let res = cargo run --example bench_commit
    | lines
    | parse "{curve}: {degree} -> {t}"
    | into int degree
    | update t { into int | into duration }

python scripts/plot/bench_commit.py (
    $res | group-by curve --to-table | update items { reject curve } | to json
)
```
