# Komodo: Cryptographically-proven Erasure Coding

## Usage
Komodo can either be used as a library or as a binary application.

## the library
see `cargo doc`

## the binary application
below is an example of how to use the binary application with Nushell:
```bash
./examples/cli.nu
```

> **Note**  
> requires Nushell 0.89.0 or later

tests for the binary application can also be run with
```bash
nu tests/cli.nu
```

## other binaries
Komodo provides a bunch of other binaries that might be interesting of useful to use.

The easiest is to use the `cargo.nu` Nushell module as follows
```bash
use .nushell/cargo.nu "cargo bin"
help cargo bin
```

## the benchmarks
see [the `README`](benchmarks/README.md)
