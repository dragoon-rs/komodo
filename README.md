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

Other examples that showcase the Komodo API are available in [`examples/`](examples/).

## the benchmarks
see [`benchmarks/`](benchmarks/README.md)

## contributors

Because the code for this project has been originally extracted from
[`dragoon/pcs-fec-id`][pcs-fec-id] by @a.stevan, some contributors might be
missing from the Git tree.

Please consult the original Git tree from [`dragoon/pcs-fec-id`][pcs-fec-id] or
note that the following people have contributed to this code base:
- @a.stevan
- @j.detchart

## citation
If you use the `komodo` library in your research projects, please cite it using the following template:

```bibtex
@software{komodo@1.0.1,
  author  = {Komodo contributors},
  title   = {\texttt{komodo}: Cryptographically-proven Erasure Coding},
  url     = {https://gitlab.isae-supaero.fr/dragoon/komodo/-/tree/1.0.1},
  crate   = {https://crates.io/crates/komodo/1.0.1},
  mirror  = {https://github.com/dragoon-rs/komodo/tree/1.0.1},
  docs    = {https://docs.rs/komodo/1.0.1/komodo/},
  version = {1.0.1},
  year    = {2023},
}
```

> [!tip] TIP
> mentions to `1.0.1` or `v1.0.1` can be removed but are recommended to pin the version of the software.

[pcs-fec-id]: https://gitlab.isae-supaero.fr/dragoon/pcs-fec-id
