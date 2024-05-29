export const BIN = "./target/release/inbreeding"
export const CACHE = ($nu.home-path | path join .cache komodo inbreeding)

const FMT = {
    env: "(?<env>.*)",
    seed: "(?<seed>[a-zA-Z0-9]*)",
    params: '(?<k>\d+)-(?<n>\d+)-(?<nb_bytes>\d+)',
    timestamp: '(?<timestamp>\d+)',
    strat: "(?<strategy>.*)" ,
}

export const ARG_EXPERIMENT_FORMAT = $FMT.seed + '-' + $FMT.env + '-' + $FMT.params
export const EXPERIMENT_FORMAT = $FMT.timestamp + '-' + $FMT.env + '-' + $FMT.strat + '-' + $FMT.params
export const FULL_EXPERIMENT_FORMAT = $FMT.seed + (char path_sep) + $EXPERIMENT_FORMAT
