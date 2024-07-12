use ../nu-utils binary [ "bytes from_int", "bytes to_int" ]

use std assert

def random-bytes [n: int]: nothing -> list<int> {
    0..$n | each { random int 0..255 }
}

def main [] {
    const hello_world_int = [
        104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33
    ]

    const expected = 0x[68 65 6C 6C 6F 20 77 6F 72 6C 64 21]
    assert equal ($hello_world_int | bytes from_int) $expected

    let bytes = random-bytes 1_000
    assert equal ($bytes | bytes from_int | bytes to_int) $bytes
}
