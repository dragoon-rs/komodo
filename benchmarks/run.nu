use ../log.nu [ "log info", "log warning", "log error", "log hint", "str color" ]

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

const BENCHES = [ field, group, setup, commit, linalg, fec, recoding, semi-avid, kzg, aplonk ]

def lscpu []: [ nothing -> record ] {
    ^lscpu --json
        | from json
        | get lscpu
        | update field { str trim --right --char ":" }
        | transpose --header-row
        | into record
}

def --wrapped bench [--rust-build: string, ...args: string] {
    let rust_build = match $rust_build {
            "release" => { cargo_flag: [ "--release" ], build: "release" },
            "debug"   => { cargo_flag: [],              build: "debug"   },
            _         => {
                log warning $"unknown Rust build (ansi yellow)($rust_build)(ansi reset), defaulting to (ansi green)release(ansi reset) for best performance"
                { cargo_flag: [ --release ], build: "release" }
            },
        }
    let options = [
        ...$rust_build.cargo_flag
        --package benchmarks
        --
        --rust-build $rust_build.build
        ...$args
    ]

    log info $"cargo run ($options | str join ' ')"
    cargo run ...$options
}

export def main [
    ...benches        : string,
    --all,
    --output-dir (-o) : path = "./a.out",
    --curves          : list<string> = [],
    --degrees         : list<int>    = [],
    --matrix-sizes    : list<int>    = [],
    --data-sizes      : list<int>    = [],
    --ks              : list<int>    = [],
    --rhos            : list<float>  = [],
    --rust-build      : string       = "release",
    --steps           : list<string>,
    --overwrite,
] {
    let benches = match [ $all, ($benches | length) ] {
        [ false, 0 ] => {
            log error "nothing to do"
            return
        },
        [ false, _ ] => $benches,
        [  true, 0 ] => $BENCHES,
        [  true, _ ] => {
            log warning "--all is raised even though benches have been provided"
            $BENCHES
        },
    }

    let target_dir = $output_dir
    let cpus_dir = $target_dir | path join "cpus"

    if not ($target_dir | path exists) {
        log warning $"creating directory ($target_dir | str color purple)"
        mkdir $target_dir
    }

    let komodo_hash = git rev-parse HEAD
    let src_hash = ls src/*.rs benchmarks/**/*.rs
        | each { open $in.name | hash sha256 }
        | str join
        | hash sha256
    let cpu = lscpu | select ...$CPU_FIELDS
    let cpu_hash = $cpu | to json | hash sha256

    def out [stem: string]: [ nothing -> path] {
        { parent: $target_dir, stem: $stem, extension: "ndjson" } | path join
    }

    for b in $benches {
        let opts = match $b {
            "field"     => { out: $b          , args: { n: 10000, rest: [ --curves ...$curves --all ] } },
            "group"     => { out: $b          , args: { n:  1000, rest: [ --curves ...$curves --all ] } },
            "setup"     => { out: $b          , args: { n:    10, rest: [ --curves ...$curves --degrees ...$degrees ] } },
            "commit"    => { out: $b          , args: { n:    10, rest: [ --curves ...$curves --degrees ...$degrees ] } },
            "linalg"    => { out: $b          , args: { n:    10, rest: [ --curves ...$curves --sizes ...$matrix_sizes ] } },
            "fec"       => { out: "fec"       , args: { n:    10, rest: [ --curves ...$curves --sizes ...$data_sizes --ks ...$ks --rhos ...$rhos --encoding random ] } },
            "recoding"  => { out: "fec"       , args: { n:    10, rest: [ --curves ...$curves --sizes ...$data_sizes --ks ...$ks --shards ...$ks ] } },
            "semi-avid" => { out: "protocols" , args: { n:    10, rest: [ --curves ...$curves --sizes ...$data_sizes --ks ...$ks --rhos ...$rhos ] } },
            "kzg"       => { out: "protocols" , args: { n:     5, rest: [ --curves ...$curves --sizes ...$data_sizes --ks ...$ks --rhos ...$rhos ] } },
            "aplonk"    => { out: "protocols" , args: { n:     1, rest: [ --curves ...$curves --sizes ...$data_sizes --ks ...$ks --rhos ...$rhos ] } },
            _           => {
                log error $"unknown bench `($b)`"
                log hint $"    choose one of these: ($BENCHES | each { str color green })"
                continue
            },
        }
        let opts = [
            (if $overwrite { --overwrite })
            --git $komodo_hash
            --cpu $cpu_hash
            --src $src_hash
            -n $opts.args.n
            -o (out $opts.out)
            $b
            ...(if $steps != null { [--steps, ...$steps] } else { [] })
            ...$opts.args.rest
        ]
        bench --rust-build $rust_build ...($opts | compact)
    }

    mkdir $cpus_dir
    $cpu | to json | save --force ({ parent: $cpus_dir, stem: $cpu_hash, extension: "json"} | path join)
}
