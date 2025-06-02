const CPU_FIELDS = [
    "Architecture",
    "CPU op-mode(s)",
    "Address sizes",
    "Byte Order",
    "CPU(s)",
    "On-line CPU(s) list",
    "Model name",
    "CPU family",
    "Model",
    "Thread(s) per core",
    "Core(s) per socket",
    "Socket(s)",
    "Stepping",
    "CPU max MHz",
    "CPU min MHz",
    "BogoMIPS",
    "Virtualization",
    "L1d cache",
    "L1i cache",
    "L2 cache",
    "L3 cache",
    "NUMA node(s)",
    "NUMA node0 CPU(s)",
]

def lscpu []: [ nothing -> record ] {
    ^lscpu --json
        | from json
        | get lscpu
        | update field { str trim --right --char ":" }
        | transpose --header-row
        | into record
}

def main [output_dir: path] {
    let cpu = lscpu | select ...$CPU_FIELDS

    let commit = git rev-parse HEAD
    let hash = $cpu | to json | $in + $commit | hash sha256

    let target = $output_dir | path join $hash
    mkdir $target

    $cpu | to json | save --force ($target | path join "cpu.json")
    $commit | save --force ($target | path join "komodo.txt")

    let benchmarks = {
        linalg: {
            sizes: (seq 0 5 | each { 2 ** $in }),
            curves: [ bls12381, pallas, bn254 ],
        },
        setup: {
            degrees: (seq 0 10 | each { 2 ** $in }),
            curves: [ bls12381, pallas, bn254 ],
        },
        commit: {
            degrees: (seq 0 10 | each { 2 ** $in }),
            curves: [ bls12381, pallas, bn254 ],
        },
        recoding: {
            sizes: (seq 0 10 | each { 512 * 2 ** $in }),
            ks: [2, 4, 8, 16],
            curves: [ bls12381 ],
        },
        fec: {
            sizes: (seq 0 10 | each { 512 * 2 ** $in }),
            ks: [2, 4, 8, 16],
            rhos: [0.5, 0.33],
            curves: [ bls12381 ],
            encoding: "random",
        },
        field: {
            nb_measurements: 1000,
            curves: [ bls12381, pallas, bn254 ],
        },
        group: {
            nb_measurements: 1000,
            curves: [ bls12381, pallas, bn254 ],
        },
    }

    for x in ($benchmarks | transpose k v) {
        print $x.k
        let opts = $x.v
                | default 100 nb_measurements
                | items { |k, v|
                    [ $"--($k | str replace --all '_' '-')" ] | append $v
                }
                | flatten

        let output = { parent: $target, stem: $x.k, extension: "ndjson" } | path join

        cargo run --package benchmarks -- $x.k ...$opts out> $output
    }
}
