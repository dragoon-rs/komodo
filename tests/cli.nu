use ../komodo.nu [
    "komodo build",
    "komodo setup",
    "komodo prove",
    "komodo verify",
    "komodo reconstruct",
]
use ../binary.nu [ "bytes decode" ]

use std assert

komodo build

let bytes = "tests/dragoon_32x32.png"

komodo setup $bytes

komodo prove $bytes --fec-params {k: 3, n: 5}
komodo verify blocks/0.bin blocks/1.bin

let actual = komodo reconstruct blocks/0.bin blocks/2.bin blocks/3.bin | bytes decode
let expected = open $bytes | into binary | to text | from json | bytes decode
assert equal $actual $expected
print "reconstruction was successful"
