# Komodo: Cryptographically-proven Erasure Coding

## Usage
Komodo can either be used as a library or as a binary application.

## the library
see `cargo doc`

## the binary application
below is an example of how to use the binary application with Nushell:
```bash
use komodo.nu [
    "komodo build",
    "komodo setup",
    "komodo prove",
    "komodo verify",
    "komodo reconstruct",
]
use binary.nu [ "bytes decode" ]

let bytes = open komodo.nu

komodo build

komodo setup $bytes

komodo prove $bytes --fec-params {k: 3, n: 5}
komodo verify blocks/0.bin blocks/1.bin

(komodo reconstruct blocks/0.bin blocks/2.bin blocks/3.bin | bytes decode) == $bytes
```

`true` should be printed at the end :thumbsup:
