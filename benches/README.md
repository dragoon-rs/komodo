## run the benchmarks
```shell
nushell> cargo criterion --output-format verbose --message-format json out> results.ndjson
```

## plot the results
```shell
python benches/plot.py results.ndjson
```
