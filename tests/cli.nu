use ../komodo.nu [
    "komodo build",
    "komodo setup",
    "komodo prove",
    "komodo verify",
    "komodo reconstruct",
]
use ../binary.nu [ "bytes decode" ]

use std assert

# a simple module to extend the `math` command
module math {
    # `choose k n` is the list of all the possible $k$ indices chosen among $n$ indices
    #
    # see [_test_choose] below for some examples
    export def choose [k: int, n: int]: nothing -> list<list<int>> {
        if $k == 0 {
            return []
        } else if $k == 1 {
            return (seq 0 ($n - 1) | each {[ $in ]})
        }

        choose ($k - 1) $n
            | each { |x|
                let l = $x | last
                if $l != ($n - 1) {
                    seq ($l + 1) ($n - 1) | each {|it| $x | append $it}
                }
            }
            | flatten
    }

    def _test_choose [] {
        use std assert

        assert equal (choose 0 5) []
        assert equal (choose 1 5) [[0], [1], [2], [3], [4]]
        assert equal (choose 2 5) [
            [0, 1], [0, 2], [0, 3], [0, 4], [1, 2], [1, 3], [1, 4], [2, 3], [2, 4], [3, 4]
        ]
        assert equal (choose 3 5) [
            [0, 1, 2],
            [0, 1, 3],
            [0, 1, 4],
            [0, 2, 3],
            [0, 2, 4],
            [0, 3, 4],
            [1, 2, 3],
            [1, 2, 4],
            [1, 3, 4],
            [2, 3, 4],
        ]
        assert equal (choose 4 5) [
            [0, 1, 2, 3],
            [0, 1, 2, 4],
            [0, 1, 3, 4],
            [0, 2, 3, 4],
            [1, 2, 3, 4],
        ]
        assert equal (choose 5 5) [[0, 1, 2, 3, 4]]
    }

    # `perm n` is the list of all the possible permutations on $n$ elements
    #
    # see [_test_perm] below for some examples
    export def perm [n: int]: nothing -> list<list<int>> {
        if $n == 0 {
            return []
        } else if $n == 1 {
            return [[0]]
        }

        perm ($n - 1)
            | each {|x|
                seq 0 ($x | length) | each {|i| $x | insert $i ($n - 1)}
            }
            | flatten
    }

    def _test_perm [] {
        use std assert

        assert equal (perm 0 | sort) []
        assert equal (perm 1 | sort) [[0]]
        assert equal (perm 2 | sort) [[0, 1], [1, 0]]
        assert equal (perm 3 | sort) [
            [0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]
        ]
        assert equal (perm 4 | sort) [
            [0, 1, 2, 3],
            [0, 1, 3, 2],
            [0, 2, 1, 3],
            [0, 2, 3, 1],
            [0, 3, 1, 2],
            [0, 3, 2, 1],
            [1, 0, 2, 3],
            [1, 0, 3, 2],
            [1, 2, 0, 3],
            [1, 2, 3, 0],
            [1, 3, 0, 2],
            [1, 3, 2, 0],
            [2, 0, 1, 3],
            [2, 0, 3, 1],
            [2, 1, 0, 3],
            [2, 1, 3, 0],
            [2, 3, 0, 1],
            [2, 3, 1, 0],
            [3, 0, 1, 2],
            [3, 0, 2, 1],
            [3, 1, 0, 2],
            [3, 1, 2, 0],
            [3, 2, 0, 1],
            [3, 2, 1, 0],
        ]
    }
}

use math

const BYTES = "tests/dragoon_32x32.png"
const FEC_PARAMS = {k: 3, n: 5}

def test [blocks: list<int>] {
    let actual = try {
        komodo reconstruct ...($blocks | each { $"blocks/($in).bin" }) | bytes decode
    } catch {
        error make --unspanned { msg: "woopsie" }
    }

    let expected = open $BYTES | into binary | to text | from json | bytes decode
    assert equal $actual $expected
}

def main [] {
    komodo build

    komodo setup $BYTES
    komodo prove $BYTES --fec-params $FEC_PARAMS

    komodo verify blocks/0.bin blocks/1.bin

    let all_k_choose_n_permutations = seq $FEC_PARAMS.k $FEC_PARAMS.n
        | each {|ki|
            let p = math perm $ki
            math choose $ki $FEC_PARAMS.n
                | each {|it|
                    $p | each { each {|i| $it | get $i} }
                }
                | flatten
        }
        | flatten
    let total = $all_k_choose_n_permutations | length
    $all_k_choose_n_permutations | enumerate | each {|it|
        print $"[($it.index / $total * 100 | into int)%]: ($it.item | str join ', ')"
        test $it.item
    }

    print "reconstruction was successful"
}
