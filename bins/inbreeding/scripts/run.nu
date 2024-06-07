use ../

const OPTS = {
    nb_bytes: (10 * 1_024),
    k: 10,
    n: null,
    nb_measurements: 100,
    nb_scenarii: 100,
    measurement_schedule: 1,
    measurement_schedule_start: 0,
    max_t: 150,
    strategies: [
        "single:1",
        "single:2",
        "single:3"
        "single:4"
        "single:5",
        "single:6",
        "single:7",
        "single:8",
        "single:9",
        "single:10",
    ],
    environment: null,
}

const RHOS = [ (1 / 2), (1 / 3), (1 / 5), (1 / 10) ]

const ENVS = [
    "fixed:0",
    "random-fixed:0.1:1",
    "random-fixed:0.2:1",
    "random-fixed:0.3:1",
    "random-fixed:0.4:1",
    "random-fixed:0.5:1",
    "random-fixed:0.6:1",
    "random-fixed:0.7:1",
    "random-fixed:0.8:1",
    "random-fixed:0.9:1",
    "fixed:1",
]

def prod [a: list, b: list]: [ nothing -> list<list> ] {
    $a | each { |ia| $b | each { |ib| [$ia, $ib] }} | flatten
}

def main [
    --nb-threads: int = 8,
    --prng-seed: string = "0000000000000000000000000000000000000000000000000000000000000000",
] {
    inbreeding build

    prod $RHOS $ENVS | par-each --threads $nb_threads { |exp|
        let opts = $OPTS | update n ($OPTS.k / $exp.0 | math round --precision 0) | update environment $exp.1
        inbreeding run --options $opts --prng-seed $prng_seed
    }
}
