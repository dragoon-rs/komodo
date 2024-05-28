use consts.nu

def get-experiments []: nothing -> list<string> {
    $consts.CACHE
        | path join '*' '*' '*'
        | into glob
        | ls $in
        | get name
        | path split
        | each { last 3 | reject 1 | str join "-" }
        | uniq
}

export def main [
    experiment: string@get-experiments, # something of the form '<seed>-<env>'
]: nothing -> table<strategy: string, diversity: table<x: int, y: float, e: float>> {
    let exp = $experiment | parse "{seed}-{env}" | into record
    if $exp == {} {
        error throw {
            err: "invalid experiment",
            label: $"should have format '<seed>-<env>', found ($experiment)",
            span: (metadata $experiment).span,
        }
    }

    let experiment_path = [$consts.CACHE, $exp.seed, '*', $exp.env, '*' ]
        | path join
        | into glob
    let experiment_files = try {
        ls $experiment_path
    } catch {
        error throw {
            err: "invalid experiment",
            label: $"experiment '($experiment)' does not appear to have data files",
            span: (metadata $experiment).span,
        }
    }

    $experiment_files
        | insert strategy { get name | path split | last }
        | select name strategy
        | insert diversity {
            ls $in.name
                | each { get name | open | lines }
                | flatten
                | parse "{x}, {y}"
                | into float y
                | group-by x --to-table
                | insert y { get items.y | math avg }
                | insert e { get items.y | math stddev }
                | rename --column { group: "x" }
                | reject items
                | into int x # NOTE: $.x needs to be converted to int here because
                             # `group-by --to-table` converts the grouping key to
                             # string
        }
        | reject name
        | group-by strategy --to-table
        | update items {|it|
            let d = $it.items.diversity
            $d | skip 1 | reduce --fold $d.0 {|it, acc| $acc | append $it}
        }
        | rename --column { group: "strategy", items: "diversity" }
}
