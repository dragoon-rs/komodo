const BASE_URL = "https://raw.githubusercontent.com/arkworks-rs/algebra/refs/heads/master/curves"

def get-modulus [file: string]: [ nothing -> string ] {
    let resp = http get $"($BASE_URL)/($file)" --full --allow-errors
    if $resp.status != 200 {
        return null
    }
    $resp.body
        | lines
        | parse "#[modulus = \"{modulus}\"]"
        | into record
        | get modulus?
        | default (do {
            let parsed = $resp.body | parse --regex 'pub use ark_(?<mod>.*)::.*;' | into record
            if $parsed == {} {
                null
            } else {
                $parsed.mod
            }
        })
}

let curves = http get $"($BASE_URL)/Cargo.toml"
    | get workspace.members
    | where $it != "curve-constraint-tests"

let res = $curves | each {
    print --stderr --no-newline $"($in) F_r                       \r"
    let r = get-modulus $"($in)/src/fields/fr.rs"

    print --stderr --no-newline $"($in) F_q                       \r"
    let q = get-modulus $"($in)/src/fields/fq.rs"

    { name: $in, r: $r, q: $q }
}

let curves = $res
    | where q != null and r != null
    | update r { |_it| if $_it.r in $res.name { $res | where $it.name == $_it.r | into record | get r } else { $_it.r } }
    | update q { |_it| if $_it.q in $res.name { $res | where $it.name == $_it.q | into record | get q } else { $_it.q } }
    | insert r_size { python3 -c $"print\(bin\(($in.r)\)\)" | str length | $in - 2 }
    | insert q_size { python3 -c $"print\(bin\(($in.q)\)\)" | str length | $in - 2 }

print ($curves | to json)
