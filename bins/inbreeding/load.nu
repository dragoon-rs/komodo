use consts.nu
use path.nu [ "remove-cache-prefix" ]
use ../../.nushell error "error throw"

use list.nu

export def main [
    experiment: string@list,
]: [
    nothing -> record<
        experiment: record<k: int, n: int, nb_bytes: int, env: string>,
        measurements: table<strategy: string, diversity: table<x: int, y: float, e: float>>,
    >
] {
    let exp = $experiment
        | parse --regex $consts.ARG_EXPERIMENT_FORMAT
        | into record
        | into int k
        | into int n
        | into int nb_bytes
    if $exp == {} {
        error throw {
            err: "invalid experiment",
            label: $"should have format '($consts.ARG_EXPERIMENT_FORMAT)', found ($experiment)",
            span: (metadata $experiment).span,
        }
    }

    let experiment_path = [
        $consts.CACHE,
        $exp.seed,
        ([$exp.env, '*', $exp.k, $exp.n, $exp.nb_bytes] | str join '-')
    ]
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

    let measurements = $experiment_files
        | select name
        | insert . { get name | remove-cache-prefix | parse --regex $consts.EXPERIMENT_FORMAT }
        | flatten --all
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

    {
        experiment: {
            env: $exp.env,
            k: $exp.k,
            n: $exp.n,
            nb_bytes: $exp.nb_bytes,
        },
        measurements: $measurements,
    }
}
