#!/usr/bin/env nu
use ../komodo.nu [
    "komodo build",
    "komodo setup",
    "komodo prove",
    "komodo verify",
    "komodo reconstruct",
]
use ../binary.nu [ "bytes decode" ]

use std assert

const BYTES = "tests/dragoon_32x32.png"
const FEC_PARAMS = { k: 3, n: 5 }

const BLOCKS_TO_VERIFY = [0, 1]
const BLOCKS_TO_RECONSTRUCT = [0, 2, 3]

def main [] {
    komodo build

    komodo setup $BYTES
    komodo prove $BYTES --fec-params $FEC_PARAMS

    komodo verify ...($BLOCKS_TO_VERIFY | each { $"blocks/($in).bin" })

    let actual = komodo reconstruct ...($BLOCKS_TO_RECONSTRUCT | each { $"blocks/($in).bin" })
        | bytes decode
    let expected = open $BYTES | into binary | to text | from json | bytes decode
    assert equal $actual $expected

    print "reconstruction was successful"
}
