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
