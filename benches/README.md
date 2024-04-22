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
def read-atomic-ops []: list -> record {
    where reason == "benchmark-complete"
        | select id mean.estimate
        | rename --column { mean_estimate: "mean" }
        | sort-by id
        | update id { parse "{op} on {curve}" }
        | flatten --all
        | group-by op --to-table
        | reject items.op
        | update items { transpose -r | into record }
        | transpose -r
        | into record
}
python scripts/plot/multi_bar.py (open field.ndjson | read-atomic-ops | to json) --title "field operations" -l "time (in ns)"
python scripts/plot/multi_bar.py (open curve.ndjson | read-atomic-ops | to json) --title "curve group operations" -l "time (in ns)"
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
