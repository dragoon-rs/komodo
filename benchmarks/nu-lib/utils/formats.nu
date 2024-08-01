export def "from ndnuon" []: [string -> any] {
    lines | each { from nuon }
}

export def "to ndnuon" []: [any -> string] {
    each { to nuon --raw } | to text
}

export def "from nuonl" []: [string -> any] {
    from ndnuon
}

export def "to nuonl" []: [any -> string] {
    to ndnuon
}
