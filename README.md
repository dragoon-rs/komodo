# Komodo: Cryptographically-proven Erasure Coding

## the library
see `cargo doc` or [the library itself](src/)

## the tests
```shell
make
```
or
```shell
make check clippy test-rs
```

### some extra tests
this project defines some tests written in [Nushell](https://www.nushell.sh/) to test an
[implementation of Komodo in a CLI application](bins/saclin/).

If you have [Nushell installed](https://www.nushell.sh/book/installation.html), you can run these
with the following command:
```shell
make test-nu
```

## examples
A [CLI example](bins/saclin/examples/cli.nu) is also provided and can be run with
```shell
make example
```

## the benchmarks
see [`benchmarks/`](benchmarks/README.md)
