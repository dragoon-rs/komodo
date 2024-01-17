# decode a list of integer bytes into the underlying encoded string
export def "bytes decode" [encoding: string = "utf-8"]: list<int> -> string {
    each { into binary | bytes at 0..1 } | bytes collect | decode $encoding
}

# encode an encoded string into the underlying list of integer bytes
export def "bytes encode" [encoding: string = "utf-8"]: string -> list<int> {
    let bytes = $in | encode $encoding
    seq 1 ($bytes | bytes length) | each {|i|
        $bytes | bytes at ($i - 1)..($i) | into int
    }
}

export def "bytes from_int" []: [int -> binary, list<int> -> binary] {
    each { into binary --compact } | bytes collect
}

export def "bytes to_int" []: binary -> list<int> {
    let bytes = $in
    seq 1 ($bytes | bytes length) | each {|i|
        $bytes | bytes at ($i - 1)..($i) | get 0
    }
}
