export module nu-lib/setup.nu
export module nu-lib/commit.nu
export module nu-lib/fec/
export module nu-lib/recoding.nu
export module nu-lib/linalg.nu
export module nu-lib/fri/

use nu-lib/linalg.nu
use nu-lib/setup.nu
use nu-lib/commit.nu
use nu-lib/recoding.nu
use nu-lib/fec/
use nu-lib/fri/

use nu-lib/utils/log.nu
use nu-lib/utils/parse.nu read-atomic-ops

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

export def run [
    benchmarks: record<
        linalg: record<
            enabled: bool,
            sizes: list<int>,
            output: string,
            append: bool,
        >,
        setup: record<
            enabled: bool,
            degrees: list<int>,
            curves: list<string>,
            output: string,
            append: bool,
        >,
        commit: record<
            enabled: bool,
            degrees: list<int>,
            curves: list<string>,
            output: string,
            append: bool,
        >,
        recoding: record<
            enabled: bool,
            sizes: list<int>,
            ks: list<int>,
            curves: list<string>,
            output: string,
            append: bool,
        >,
        fec: record<
            enabled: bool,
            sizes: list<int>,
            ks: list<int>,
            curves: list<string>,
            output: string,
            append: bool,
        >,
        fri: record<
            enabled: bool,
            sizes: list<filesize>,
            ks: list<int>,
            blowup_factors: list<int>,
            ns: list<int>,
            remainder_plus_ones: list<int>,
            nb_queries: list<int>,
            hashes: list<string>,
            ffs: list<string>,
            output: string,
            append: bool,
        >,
        field: record<enabled: bool, nb_measurements: int, output: string, append: bool>,
        curve_group: record<enabled: bool, nb_measurements: int, output: string, append: bool>,
    >,
    --output-dir: path = ".",
] {
    let cpu = lscpu --json
        | from json
        | get lscpu
        | update field { str trim --right --char ":" }
        | transpose --header-row
        | into record
        | select ...$CPU_FIELDS

    let commit = git rev-parse HEAD
    let hash = $cpu | to json | $in + $commit | hash sha256

    let target = $output_dir | path join $hash
    mkdir $target

    $cpu | to json | save --force ($target | path join "cpu.json")
    $commit | save --force ($target | path join "komodo.txt")

    let benchmarks = $benchmarks
        | insert linalg.run {{ |it|
            let output = $target | path join $it.output
            $it.sizes | linalg run --no-confirm --output $output --append=$it.append
        }}
        | insert setup.run {{ |it|
            let output = $target | path join $it.output
            $it.degrees | setup run --curves $it.curves --no-confirm --output $output --append=$it.append
        }}
        | insert commit.run {{ |it|
            let output = $target | path join $it.output
            $it.degrees | commit run --curves $it.curves --no-confirm --output $output --append=$it.append
        }}
        | insert recoding.run {{ |it|
            let output = $target | path join $it.output
            $it.sizes | recoding run --ks $it.ks --curves $it.curves --no-confirm --output $output --append=$it.append
        }}
        | insert fec.run {{ |it|
            let output = $target | path join $it.output
            $it.sizes | fec run --ks $it.ks --curves $it.curves --no-confirm --output $output --append=$it.append
        }}
        | insert fri.run {{ |it|
            # FIXME: refactor this
            if $it.append {
                (
                    fri run
                        --data-sizes $it.sizes
                        --ks $it.ks
                        --blowup-factors $it.blowup_factors
                        --nb-queries $it.nb_queries
                        --hashes $it.hashes
                        --finite-fields $it.ffs
                        --remainders $it.remainder_plus_ones
                        --folding-factors $it.ns
                ) | to ndjson out>> ($target | path join $it.output)
            } else {
                (
                    fri run
                        --data-sizes $it.sizes
                        --ks $it.ks
                        --blowup-factors $it.blowup_factors
                        --nb-queries $it.nb_queries
                        --hashes $it.hashes
                        --finite-fields $it.ffs
                        --remainders $it.remainder_plus_ones
                        --folding-factors $it.ns
                ) | to ndjson out> ($target | path join $it.output)
            }
        }}
    | insert field.run {{ |it|
        let options = [
            --bin field
            --release
            --package benchmarks
            --
            --nb-measurements $it.nb_measurements
        ]
        # FIXME: refactor this
        if $it.append {
            cargo run ...$options out>> ($target | path join $it.output)
        } else {
            cargo run ...$options out> ($target | path join $it.output)
        }
    }}
    | insert curve_group.run {{ |it|
        let options = [
            --bin curve_group
            --release
            --package benchmarks
            --
            --nb-measurements $it.nb_measurements
        ]
        # FIXME: refactor this
        if $it.append {
            cargo run ...$options out>> ($target | path join $it.output)
        } else {
            cargo run ...$options out> ($target | path join $it.output)
        }
    }}

    let _ = $benchmarks | items { |k, b|
        if ($b.enabled? | default true) {
            log info $"running (ansi cyan)($k)(ansi reset)"
            do $b.run $b
        } else {
            log warning $"skipping (ansi cyan)($k)(ansi reset)"
        }
    }
}

export def plot [plots: record, --input-dir: path, --output-dir: path = "./figures/"] {
    mkdir $output_dir

    let linalg_file = $input_dir | path join $plots.linalg.file
    let fec_file = $input_dir | path join $plots.fec.file
    let recoding_file = $input_dir | path join $plots.recoding.file

    for op in [ "mul", "transpose", "inverse" ] {
        linalg plot $linalg_file $op --save ($output_dir | path join $"linalg-($op).png")
    }

    setup plot ($input_dir | path join $plots.setup.file) --save ($output_dir | path join setup.png)
    commit plot ($input_dir | path join $plots.commit.file) --save ($output_dir | path join commit.png)

    recoding plot $recoding_file --save ($output_dir | path join recoding.png)

    fec plot encoding $fec_file --save ($output_dir | path join encoding.png)
    fec plot decoding $fec_file --save ($output_dir | path join decoding.png)
    fec plot e2e $fec_file --save ($output_dir | path join end_to_end.png)
    fec plot combined $fec_file --recoding $recoding_file --save ($output_dir | path join combined.png)
    fec plot ratio $fec_file --recoding $recoding_file --save ($output_dir | path join ratio.png)

    for plot in $plots.fri {(
        fri plot
            --dump-dir $output_dir
            --file ($input_dir | path join fri.ndjson)
            $plot.name
            --y-type $plot.y_type
            --single=$plot.single
            --identity=$plot.identity
            --normalize=$plot.normalize
            --save
    )}

    for plot in ($plots | select field curve_group | values) {
        def output [prefix: string]: [ nothing -> record<path: path, title: string> ] {
            let title_tokens = $plot.title | split row " " | prepend $prefix
            {
                path: ({
                    parent: $output_dir,
                    stem: ($title_tokens | str join "_"),
                    extension: "png",
                } | path join),
                title: ($title_tokens | str join " "),
            }
        }

        let data = open ($input_dir | path join $plot.file)

        output "simple" | gplt multi-bar --title $in.title -l "time (in ns)" (
            $data | read-atomic-ops --include $plot.simple_operations | to json
        ) --save $in.path

        output "complex" | gplt multi-bar --title $in.title -l "time (in ns)" (
            $data | read-atomic-ops --exclude $plot.simple_operations | to json
        ) --save $in.path
    }
}
