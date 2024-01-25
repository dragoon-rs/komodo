#!/usr/bin/env nu
use ../komodo.nu [
    "komodo build",
    "komodo setup",
    "komodo prove",
    "komodo verify",
    "komodo reconstruct",
    "komodo ls",
]
use ../binary.nu [ "bytes from_int" ]

use std assert

const BYTES = "tests/dragoon_32x32.png"
const FEC_PARAMS = { k: 3, n: 5 }

const BLOCKS_TO_VERIFY = [0, 1]
const BLOCKS_TO_RECONSTRUCT = [0, 2, 3]

def main [] {
    komodo build

    komodo setup (open $BYTES | into binary | bytes length)
    komodo prove $BYTES --fec-params $FEC_PARAMS

    let blocks = komodo ls

    komodo verify ...($BLOCKS_TO_VERIFY | each {|i| $blocks | get $i })

    let actual = komodo reconstruct ...($BLOCKS_TO_RECONSTRUCT | each {|i| $blocks | get $i })
    let expected = open $BYTES | bytes from_int
    assert equal $actual $expected

    print "reconstruction was successful"
}
