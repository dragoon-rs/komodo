use std iter

def "cartesian product" [
    iters: list  # the iterables you want the cartesian product of
]: nothing -> list {
    def aux [a: list]: nothing -> list {
        if ($a | is-empty) {
            return []
        }

        let head = $a | first
        let tail = aux ($a | skip 1)

        if ($head | is-empty) {
            return $tail
        } else if ($tail | is-empty) {
            return $head
        }

        $head | each {|h| $tail | each {|t| [$h, $t]}} | flatten | each { flatten }
    }

    aux $iters
}

# returns a record with all numeric results, merged with the parameters
def run [
    params: record<
        d: filesize, k: int, bf: int, q: int, h: string, ff: string, n: int, rpo: int
    >
] {
    cargo run --quiet --release --example fri --features fri -- ...[
        --data-size ($params.d | into int)
        -k $params.k
        --blowup-factor $params.bf
        --remainder-degree-plus-one $params.rpo
        --folding-factor $params.n
        --nb-queries $params.q
        --hash $params.h
        --finite-field $params.ff
    ]
        | lines
        | parse "{k}: {v}"
        | into int v
        | transpose --header-row
        | into record
        | merge $params
}

export def main [
    --data-sizes: list<filesize>,
    --ks: list<int>,
    --blowup-factors: list<int>,
    --nb-queries: list<int>,
    --hashes: list<string>,
    --finite-fields: list<string>,
    --folding-factors: list<int>,
    --remainders: list<int>,
] {
    let inputs = [
        $data_sizes, $ks, $blowup_factors, $nb_queries, $hashes, $finite_fields,
        $folding_factors, $remainders
    ]
    if ($inputs | any { is-empty }) {
        error make --unspanned { msg: "one of the inputs is empty" }
    }

    let params = cartesian product $inputs | each { |params|
        [d, k, bf, q, h, ff, n, rpo]
            | iter zip-into-record $params
            | into record
    }

    $params | each { |p|
        print $p
        run $p
    }
}
