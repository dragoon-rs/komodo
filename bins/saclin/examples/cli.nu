#!/usr/bin/env nu
use .. [
    "saclin build",
    "saclin setup",
    "saclin prove",
    "saclin verify",
    "saclin reconstruct",
    "saclin ls",
]
use ../binary.nu [ "bytes from_int" ]

use std assert

const BYTES = "assets/dragoon_32x32.png"
const FEC_PARAMS = { k: 3, n: 5 }

const BLOCKS_TO_VERIFY = [0, 1]
const BLOCKS_TO_RECONSTRUCT = [0, 2, 3]

def main [] {
    saclin build

    saclin setup (open $BYTES | into binary | bytes length)
    saclin prove $BYTES --fec-params $FEC_PARAMS

    let blocks = saclin ls

    saclin verify ...($BLOCKS_TO_VERIFY | each {|i| $blocks | get $i })

    let actual = saclin reconstruct ...($BLOCKS_TO_RECONSTRUCT | each {|i| $blocks | get $i })
    let expected = open $BYTES | bytes from_int
    assert equal $actual $expected

    print "reconstruction was successful"
}
