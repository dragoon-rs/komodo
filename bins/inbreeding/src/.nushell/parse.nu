const FMT = {
    env: "(?<env>.*)",
    seed: "(?<seed>[a-zA-Z0-9]*)",
    params: '(?<k>\d+)-(?<n>\d+)-(?<nb_bytes>\d+)',
    strat: "(?<strategy>.*)" ,
}

const ARG_EXPERIMENT_FORMAT = $FMT.seed + '-' + $FMT.env + '-' + $FMT.params
const EXPERIMENT_FORMAT = $FMT.env + '-' + $FMT.strat + '-' + $FMT.params
const FULL_EXPERIMENT_FORMAT = $FMT.seed + (char path_sep) + $EXPERIMENT_FORMAT

export def "parse full-experiment" []: [
    string -> record<
        seed: string, env: string, strategy: string, k: int, n: int, nb_bytes: int
    >
] {
    parse --regex $FULL_EXPERIMENT_FORMAT
        | into record
        | into int k
        | into int n
        | into int nb_bytes
}

export def "parse experiment" []: [
    string -> record<env: string, strategy: string, k: int, n: int, nb_bytes: int>
] {
    parse --regex $EXPERIMENT_FORMAT
        | into record
        | into int k
        | into int n
        | into int nb_bytes
}

export def "parse arg-experiment" [--span: record<start: int, end: int>]: [
    string -> record<seed: string, env: string, k: int, n: int, nb_bytes: int>
] {
    let exp = $in
        | parse --regex $ARG_EXPERIMENT_FORMAT
        | into record
        | into int k
        | into int n
        | into int nb_bytes
    if $exp == {} {
        error throw {
            err: "invalid experiment",
            label: $"should have format '($ARG_EXPERIMENT_FORMAT)', found ($experiment)",
            span: $span,
        }
    }

    $exp
}
