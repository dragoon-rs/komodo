export def "bytes from_int" []: [int -> binary, list<int> -> binary] {
    each { into binary --compact } | bytes collect
}

export def "bytes to_int" []: binary -> list<int> {
    let bytes = $in
    seq 1 ($bytes | bytes length) | each {|i|
        $bytes | bytes at ($i - 1)..($i) | get 0
    }
}
