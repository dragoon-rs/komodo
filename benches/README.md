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

## oneshot benchmarks
these are benchmarks that run a single measurement, implemented as _examples_ in
`examples/benches/`.

## commit
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
