use std formats [ "from ndjson" ]

use ../log.nu [ "log error", "log warning", "log info", "log hint", "str color" ]

const PYTHON_VENV = "~/.cache" | path expand | path join "venvs" "C2OuW3BqljMdnmzbpnrjRjgXsfA3MbK1"
const PYTHON_DEPENDENCIES = [ matplotlib, rich ]

const GRAPHS_FILE = "/tmp/graphs.json"

const FIELD = '\mathbb{F}'
const GROUP = '\mathbb{G}_1'
const MATRIX_FIELD_R = $"\\mathcal{M}_{n \\times n}\(($FIELD)_r[X]\)"
const MATRIX_FIELD_Q = $"\\mathcal{M}_{n \\times n}\(($FIELD)_q[X]\)"

const X_BASE = 2
const Y_BASE = 10

const BENCHES = [ field, group, setup, commit, linalg, fec, semi-avid, kzg, aplonk, protocols ]

def curve-to-color [curve: string]: [ nothing -> string] {
    match $curve {
        "BLS12-381" => "tab:blue",
        "BN254"     => "tab:green",
        "PALLAS"    => "tab:orange",
        _           => "tab:gray",
    }
}

def k-to-color [k: int]: [ nothing -> string ] {
    match $in.0.k {
         2 => "tab:blue",
         4 => "tab:green",
         8 => "tab:orange",
        16 => "tab:brown",
        32 => "tab:red",
        64 => "tab:purple",
        _  => "tab:gray",
    }
}

# ```
# nu scripts/get-arkworks-curves.nu
#     | from json
#     | each {
#         $"\"($in.name | str upcase | str replace --all '_' '-')\" => { r: ($in.r_size), q: ($in.q_size) },"
#     }
#     | to text
# ```
def curve-to-field-modulus [curve: string]: [ nothing -> record<q: int, r: int> ] {
    match $curve {
        "BLS12-381" => { r: 255, q: 381 },
        "BN254"     => { r: 254, q: 254 },
        "PALLAS"    => { r: 255, q: 255 },
        _           => { r:  -1, q:  -1 },
    }
}

def update-time-unit [col: cell-path, time_unit: string]: [ table -> table ] {
    let nb_ns_in_time_unit = $"1($time_unit)" | into duration | into int
    $in | update $col { $in / $nb_ns_in_time_unit }
}

def flatten-json-col [col: cell-path]: [ table -> table ] {
    update $col { from json } | flatten $col
}

def error-with-context [msg: string, context: record] {
    if $context == {} {
        log error $msg
    } else {
        log error $"($msg):"
    }

    let width = $context | columns | each { str length } | math max
    $context | items { |k, v|
        let k = $k | fill --alignment right --width $width --character ' '
        log error $"    ($k) : ($v)"
    }

    null
}

def parse-ndjson-benchmark-results [
    --git   : string = '',
    --cpu   : string = '',
    --src   : string = '',
    --build : string = '',
    --bench : string,
]: [
    table<name: string, label: string, times: list<int>> -> table
] {
    let filtered = flatten-json-col name | where (
        $it.git =~ $git and
        $it.cpu =~ $cpu and
        $it.src =~ $src and
        $it.build =~ $build
    )
    if ($filtered | is-empty) {
        error-with-context "filter did not return anything" {
            bench : $bench
            git   : $git
            cpu   : $cpu
            src   : $src
            build : $build
        }
        return []
    }

    # merge times with the same label
    $filtered
        | group-by --to-table { $in.label }
        | get items
        | each { |it|
            $it.0 | update times { $it.times | reduce --fold [] { |it, acc| $acc | append $it } }
        }
        | flatten-json-col label
        # compute stats
        | insert avg { $in.times | math avg }
        | insert stddev { $in.times | into float | math stddev }

}

def plot-bars [
    --label       (-l) : string,
    --title       (-t) : string,
    --log-scale
    --grid
    --grid-behind
    --time-unit   (-u) : string = "ns",
    --curve-field (-c) : string,
    --filter           : closure,
    --grouper          : closure,           # basically the X axis
    --include     (-i) : list<string> = [], # the grouper should be in this
    --exclude     (-E) : list<string> = [], # the grouper should NOT be in this
    --family           : string,
    --styler-color     : closure,
    --git-hash-pattern : string = '',
    --cpu-hash-pattern : string = '',
    --src-hash-pattern : string = '',
    --rust-build       : string = '',
    --bench            : string,
    --output      (-o) : path,
]: [
    table<name: string, label: string, times: list<int>> -> nothing
] {
    if $grouper      == null { log warning "--grouper is null"      ; return }
    if $family       == null { log warning "--family is null"       ; return }
    if $styler_color == null { log warning "--styler-color is null" ; return }

    let raw = $in
        | (parse-ndjson-benchmark-results
            --git $git_hash_pattern
            --cpu $cpu_hash_pattern
            --src $src_hash_pattern
            --build $rust_build
            --bench $bench
        )
        | reject times
        | update-time-unit avg $time_unit
        | update-time-unit stddev $time_unit

    let filtered = $raw
        | if $filter != null {
            where { |it| do $filter $it }
        } else {
            $in
        }
    if ($filtered | is-empty) {
        error-with-context "filter did not return anything" {
            bench : $bench,
            git   : $git_hash_pattern,
            cpu   : $cpu_hash_pattern,
            src   : $src_hash_pattern,
            build : $rust_build,
        }
        log error "```"
        for line in (view source $filter | nu-highlight | lines) {
            log error $line
        }
        log error "```"
        return
    }

    let data = $filtered
        | update curve {
            let modulus = curve-to-field-modulus $in
            $"($in) \($($curve_field): ($modulus | get $curve_field)$ bits\)"
        }
        | insert __grouper $grouper
        | group-by __grouper --to-table
        | reject items.__grouper
        | update items {
            each { { k: ($in | get $family), v: { y: $in.avg, e: $in.stddev} } }
                | transpose --header-row
                | into record
        }

    let to_plot = if ($include | is-empty) {
        $data.__grouper | uniq
    } else {
        $include
    } | where $it not-in $exclude

    let style = $filtered
        | get $family
        | uniq
        | each { |f| { k: $f, v: { color: (do $styler_color $f) } } }
        | transpose --header-row
        | into record

    let options = [
        --style ($style | to json)
        ...(if $log_scale { [ --log-scale ] } else { [] })
        ...(if $grid { [ --grid ] } else { [] })
        ...(if $grid_behind { [ --grid-behind ] } else { [] })
        --fullscreen
        --use-tex
        ...(if $label != null { [ --label ($label | str replace --all '%unit' $time_unit) ] } else { [] })
        ...(if $title != null { [ --title ($title | str replace --all '%unit' $time_unit) ] } else { [] })
        ...(if $output != null { [ --save $output ] } else { [] })
    ]

    let graphs = $data
        | where $it.__grouper in $to_plot
        | transpose --header-row
        | into record

    $graphs | save --force $GRAPHS_FILE
    gplt multi-bar --json-data-file $GRAPHS_FILE ...$options
}

def pretty-duration []: [ list<number> -> list<string> ] {
    each { $in * 1ns } | each {
        if $in < 1us {
            format duration ns
        } else if $in < 1ms {
            format duration us
        } else if $in < 1sec {
            format duration ms
        } else if $in < 1min {
            format duration sec
        } else if $in < 1hr {
            format duration min
        } else {
            format duration hr
        }
    }
}
def pretty-filesize []: [ list<number> -> list<string> ] {
    into filesize | each {
        if $in < 1kib {
            format filesize B
        } else if $in < 1mib {
            format filesize KiB
        } else if $in < 1gib {
            format filesize MiB
        } else if $in < 1tib {
            format filesize GiB
        } else if $in < 1pib {
            format filesize TiB
        } else {
            format filesize PiB
        }
    }
}

def plot [
    --title        (-t): string,
    --x-label      (-x): string,
    --y-label      (-y): string,
    --time-unit    (-u): string = "ns",
    --filter       (-F): closure,           # raw data is passed as first positional argument
    --grouper      (-G): closure,           # filtered data is passed as either first positional argument or input
    --styler-color (-C): closure,           # filtered and grouped data is passed as input
    --styler-line  (-L): closure,           # filtered and grouped data is passed as input
    --uniqer       (-U): closure,           # filtered, grouped and styled data is passed as either first positional argument or input
    --namer        (-N): closure,           # filtered, grouped and styled data is passed as input and field as first positional argument
    --x-col        (-X): cell-path,
    --scale        (-s): string = "linlin",
    --x-unit           : string = "plain",
    --y-unit           : string = "plain",
    --curve-field  (-c): string,
    --git-hash-pattern : string = '',
    --cpu-hash-pattern : string = '',
    --src-hash-pattern : string = '',
    --rust-build       : string = '',
    --bench            : string,
    --output       (-o): path,
]: [
    table<name: string, label: string, times: list<int>> -> nothing
] {
    let raw = $in
        | (parse-ndjson-benchmark-results
            --git $git_hash_pattern
            --cpu $cpu_hash_pattern
            --src $src_hash_pattern
            --build $rust_build
            --bench $bench
        )
        | reject times
    if ($raw | is-empty) { return }

    let x_col = $"($x_col)" | str replace --regex '^\$.' ''

    # give defaults to closure
    let grouper = $grouper | default { || $in.curve }
    let styler_color = $styler_color | default { || curve-to-color $in.0.curve }
    let styler_line = $styler_line | default { || { marker: { shape: "." }, type: "solid"   }}
    let namer = $namer | default { |field|
        let modulus = curve-to-field-modulus $in
        $"($in) \($($field): ($modulus | get $field)$ bits\)"
    }

    let filtered = $raw
        | if $filter != null {
            where { |it| do $filter $it }
        } else {
            $in
        }
    if ($filtered | is-empty) {
        error-with-context "filter did not return anything" {
            bench : $bench,
            git   : $git_hash_pattern,
            cpu   : $cpu_hash_pattern,
            src   : $src_hash_pattern,
            build : $rust_build,
        }
        log error "```"
        for line in (view source $filter | nu-highlight | lines) {
            log error $line
        }
        log error "```"
        return
    }

    let graphs = $filtered
        | insert __grouper $grouper
        | group-by __grouper --to-table
        | reject items.__grouper
        | insert style.color { $in.items | do $styler_color }
        | insert style.line { $in.items | do $styler_line }
        | update items {
            if $uniqer != null {
                insert __uniq_key $uniqer
                    | uniq-by __uniq_key
                    | reject __uniq_key
            } else {
                $in
            }
                | select $x_col avg stddev
                | rename --column { $x_col: x, avg: y, stddev: e }
        }
        | rename --column { __grouper: name, items: points }
        | update name { do $namer $curve_field }

    # NOTE: assume the X ticks are powers of $X_BASE
    let x_values = $graphs.0.points.x
    let min_x_log_value = $x_values | math min | math log $X_BASE | math floor
    let max_x_log_value = $x_values | math max | math log $X_BASE | math ceil
    let x_ticks = $min_x_log_value..$max_x_log_value | each { $X_BASE ** $in }
    let x_tick_labels = $x_ticks | match $x_unit {
        "plain"    => { $in },
        "duration" => { pretty-duration },
        "filesize" => { pretty-filesize },
        _ => {
            log error $"unknown axis unit `($x_unit)` for X"
            log hint $"    choose one of these: (
                [ "plain", "duration", "filesize" ] | each { str color green }
            )"
            return
        }
    }

    # compute
    # - the sub log tick in base $Y_BASE just below the lowest Y value
    # - the sub log tick in base $Y_BASE just above the highest Y value
    let y_values = $graphs.points | flatten | get y | uniq
    let min_y_log_value = $y_values | math min | math log $Y_BASE | math floor # one tick too low
    let max_y_log_value = $y_values | math max | math log $Y_BASE | math ceil  # one tick too high
    let y_ticks = $min_y_log_value..$max_y_log_value | each { $Y_BASE ** $in }
    let y_ticks = $y_ticks
        | append ((($y_values | math min) / ($y_ticks | first) | math floor) * ($y_ticks | first))                  # below the lowest
        | append ((($y_values | math max) / ($y_ticks | last) * $Y_BASE | math ceil) * ($y_ticks | last) / $Y_BASE) # above the highest
        | skip 1     # remove $min_y_log_value and ...
        | reverse    # |
        | drop nth 2 # | ... $max_y_log_value
    let y_tick_labels = $y_ticks | match $y_unit {
        "plain"    => { $in },
        "duration" => { pretty-duration },
        "filesize" => { pretty-filesize },
        _ => {
            log error $"unknown axis unit `($y_unit)` for Y"
            log hint $"    choose one of these: (
                [ "plain", "duration", "filesize" ] | each { str color green }
            )"
            return
        }
    }

    let options = [
        --fullscreen
        --use-tex
        --x-scale-base $X_BASE
        --y-scale-base $Y_BASE
        --x-ticks ...$x_ticks
        --x-tick-labels ...$x_tick_labels
        --y-ticks ...$y_ticks
        --y-tick-labels ...$y_tick_labels
        ...(do {
            match $scale {
                "linlin" => [ --x-scale linear --y-scale linear ]
                "loglin" => [ --x-scale    log --y-scale linear ]
                "linlog" => [ --x-scale linear --y-scale    log ]
                "loglog" => [ --x-scale    log --y-scale    log ]
                _ => {
                    log error $"unknown scale `($scale)`"
                    log hint $"    choose one of these: (
                        [ "linlin", "linlog", "loglin", "loglog" ] | each { str color green }
                    )"
                    return
                }
            }
        }),
        ...(if $x_label != null { [ --x-label ($x_label | str replace --all '%unit' $time_unit) ] } else { [] })
        ...(if $y_label != null { [ --y-label ($y_label | str replace --all '%unit' $time_unit) ] } else { [] })
        ...(if $title != null { [ --title ($title | str replace --all '%unit' $time_unit) ] } else { [] })
        ...(if $output != null { [ --save $output ] } else { [] })
    ]

    $graphs | save --force $GRAPHS_FILE
    gplt plot --json-data-file $GRAPHS_FILE ...$options
}

const BASIC_OPERATIONS = {
    field: {
        complex: [ "inverse", "sqrt", "legendre", "exponentiation", "random sampling" ],
    },
    group: {
        complex: [ "random sampling", "scalar multiplication", "affine scalar multiplication" ],
        reject:  [ "into affine", "from affine" ],
    },
}

export def main [
    ...benches: string,
    --all,
    --input-dir        (-i) : path,
    --output-dir       (-o) : path,
    --git-hash-pattern (-g) : string = '',
    --cpu-hash-pattern (-c) : string = '',
    --src-hash-pattern (-s) : string = '',
    --rust-build       (-r) : string = '',
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

    $env.PATH = $env.PATH | prepend ($PYTHON_VENV | path join "bin")

    if not ($PYTHON_VENV | path exists) {
        log info $"creating virtual environment for plotting in ($PYTHON_VENV | str color purple)"
        virtualenv -q $PYTHON_VENV

        log info $"installing dependencies: ($PYTHON_DEPENDENCIES | each { str color cyan })"
        pip install ...$PYTHON_DEPENDENCIES
    }

    if not ($output_dir | path exists) {
        log warning $"creating directory ($output_dir | str color purple)"
        mkdir $output_dir
    }
    def in  [stem: string]: [ nothing -> path] { { parent: $input_dir,  stem: $stem, extension: "ndjson" } | path join }
    def out [stem: string]: [ nothing -> path] { { parent: $output_dir, stem: $stem, extension: "png" } | path join }

    for b in $benches {
        match $b {
            "field" => {
                let data = open (in "field")
                for it in [
                    [ name                 ,   out         , include                         , exclude                         ];
                    [ "operations"         , field         , []                              , []                              ],
                    [ "simple operations"  , field.simple  , []                              , $BASIC_OPERATIONS.field.complex ],
                    [ "complex operations" , field.complex , $BASIC_OPERATIONS.field.complex , []                              ],
                ] {
                    $data | (plot-bars
                        -o (out $it.out)
                        -t $"($it.name) in $($FIELD)_r$ \(time in %unit\)"
                        -l "time in %unit"
                        --log-scale
                        --grid
                        --grid-behind
                        -u ns
                        -c r
                        --grouper { |it| $it.operation }
                        --family curve
                        --styler-color { |it| curve-to-color $it }
                        -E $it.exclude
                        -i $it.include
                        --git-hash-pattern $git_hash_pattern
                        --cpu-hash-pattern $cpu_hash_pattern
                        --src-hash-pattern $src_hash_pattern
                        --rust-build $rust_build
                        --bench "field"
                    )
                }
            },
            "group" => {
                let data = open (in "group")
                for it in [
                    [ name                 ,   out         , include                         , exclude                                                             ];
                    [ "operations"         , group         , []                              , []                                                                  ],
                    [ "simple operations"  , group.simple  , []                              , ($BASIC_OPERATIONS.group.complex ++ $BASIC_OPERATIONS.group.reject) ],
                    [ "complex operations" , group.complex , $BASIC_OPERATIONS.group.complex , $BASIC_OPERATIONS.group.reject                                      ],
                ] {
                    $data | (plot-bars
                        -o (out $it.out)
                        -t $"($it.name) in $($GROUP)$ \($($FIELD)_q$\) \(time in %unit\)"
                        -l "time in %unit"
                        --log-scale
                        --grid
                        --grid-behind
                        -u ns
                        -c q
                        --grouper { |it| $it.operation }
                        --family curve
                        --styler-color { |it| curve-to-color $it }
                        -E $it.exclude
                        -i $it.include
                        --git-hash-pattern $git_hash_pattern
                        --cpu-hash-pattern $cpu_hash_pattern
                        --src-hash-pattern $src_hash_pattern
                        --rust-build $rust_build
                        --bench "group"
                    )
                }
            },
            "setup" | "commit" => {
                open (in $b) | (plot
                    -o (out $b)
                    -t (if $b == "setup" {
                        $'time to build a cryptographic setup to commit polynomial $P \in ($FIELD)_q[X]$'
                    } else {
                        $'time to commit polynomial $P \in ($FIELD)_q[X]$'
                    })
                    -x '$\deg(P)$'
                    -X degree
                    -s "loglog"
                    --x-unit "plain"
                    --y-unit "duration"
                    -c q
                    -u ms
                    --git-hash-pattern $git_hash_pattern
                    --cpu-hash-pattern $cpu_hash_pattern
                    --src-hash-pattern $src_hash_pattern
                    --rust-build $rust_build
                    --bench $b
                )
            },
            "linalg" => {
                let linalg_data = open (in "linalg")
                for op in [
                    [  name       ,  title                                                    ];
                    [ "inverse"   , $'time to invert matrix $M \in ($MATRIX_FIELD_R)$'        ],
                    [ "multiply"  , $'time to multiply matrices $M, N \in ($MATRIX_FIELD_R)$' ],
                    [ "transpose" , $'time to transpose matrix $M \in ($MATRIX_FIELD_R)$'     ],
                ] {
                    $linalg_data | (plot
                        -o (out $"linalg-($op.name)")
                        -t $op.title
                        -x '$n$'
                        -X size
                        -s "loglog"
                        --x-unit "plain"
                        --y-unit "duration"
                        -F { |it| $it.operation == $op.name }
                        -c r
                        -u ms
                        --git-hash-pattern $git_hash_pattern
                        --cpu-hash-pattern $cpu_hash_pattern
                        --src-hash-pattern $src_hash_pattern
                        --rust-build $rust_build
                        --bench "linalg"
                    )
                }
            },
            "fec" => {
                open (in "fec") | (plot
                    -o (out "fec-encode")
                    -t $"time to \\textit{encode} data of size $|D|$ on \\textbf{BN254} \(r: $(curve-to-field-modulus 'BN254' | get r)$ bits\)"
                    -x '$|D|$'
                    -X nb_bytes
                    -s "loglog"
                    --x-unit "filesize"
                    --y-unit "duration"
                    -F { |it| (
                        $it.step == "encode" and $it.curve == "BN254"
                    ) }
                    -G { || $"($in.k) ($in.n)" }
                    -C { || k-to-color $in.0.k }
                    -L { ||
                        match [$in.0.k, $in.0.n] {
                            [2,  4] => { marker: { shape: "o" }, type: "solid"   },
                            [4,  8] => { marker: { shape: "^" }, type: "dotted"  },
                            [8, 16] => { marker: { shape: "s" }, type: "dashed"  },
                            [2, 10] => { marker: { shape: "2" }, type: "solid"   },
                            [4, 20] => { marker: { shape: "X" }, type: "dotted"  },
                            [8, 40] => { marker: { shape: "h" }, type: "dashed"  },
                            _       => { marker: { shape: "D" }, type: "dashdot" },
                        }
                    }
                    -N { |field|
                        let parsed = $in | parse "{k} {n}" | into record | into int k n
                        $"$k = ($parsed.k)$, $\\rho = ($parsed.k / $parsed.n | math round -p 3)$"
                    }
                    -c r
                    -u ms
                    --git-hash-pattern $git_hash_pattern
                    --cpu-hash-pattern $cpu_hash_pattern
                    --src-hash-pattern $src_hash_pattern
                    --rust-build $rust_build
                    --bench "fec-encode"
                )
                open (in "fec") | (plot
                    -o (out "fec-decode")
                    -t $"time to \\textit{decode} data of size $|D|$ on \\textbf{BN254} \(r: $(curve-to-field-modulus 'BN254' | get r)$ bits, $\\rho$ is irrelevant\)"
                    -x '$|D|$'
                    -X nb_bytes
                    -s "loglog"
                    --x-unit "filesize"
                    --y-unit "duration"
                    -F { |it| $it.step == "decode" and $it.curve == BN254 and $it.k in [ 2, 4, 64 ] }
                    -G { || $in.k }
                    -C { || k-to-color $in.0.k }
                    -L { ||
                        match $in.0.k {
                             2 => { marker: { shape: "o" }, type: "solid"   },
                             4 => { marker: { shape: "^" }, type: "dotted"  },
                             8 => { marker: { shape: "s" }, type: "dashed"  },
                            _  => { marker: { shape: "D" }, type: "dashdot" },
                        }
                    }
                    -U { || $in.nb_bytes }
                    -N { |field|
                        let parsed = $in | parse "{k}" | into record | into int k
                        $"$k = ($parsed.k)$"
                    }
                    -c r
                    -u ms
                    --git-hash-pattern $git_hash_pattern
                    --cpu-hash-pattern $cpu_hash_pattern
                    --src-hash-pattern $src_hash_pattern
                    --rust-build $rust_build
                    --bench "fec-decode"
                )
                open (in "fec") | (plot
                    -o (out "fec-recode")
                    -t $"time to \\textit{recode} data of size $|D|$ on \\textbf{BN254} \(r: $(curve-to-field-modulus 'BN254' | get r)$ bits, $\\rho$ is irrelevant, $x = \\frac{\\#shards}{k}$\)"
                    -x '$|D|$'
                    -X nb_bytes
                    -s "loglog"
                    --x-unit "filesize"
                    --y-unit "duration"
                    -F { |it| (
                        $it.step == "recode" and $it.curve == "BN254" and [$it.k, $it.nb_shards] in [
                            [ 2,  2],
                            [ 4,  2],
                            [ 8,  2],
                            [16,  2],
                            [32,  2],
                            [64,  2],
                            [ 2, 16],
                            [ 4, 16],
                            [ 8, 16],
                            [16, 16],
                            [32, 16],
                            [64, 16],
                        ]
                    ) }
                    -G { || $"($in.k) ($in.nb_shards)" }
                    -C { || k-to-color $in.0.k }
                    -L { ||
                        match [$in.0.k, $in.0.nb_shards] {
                            [ 2,  2] => { marker: { shape: "o" }, type: "solid"   },
                            [ 8,  2] => { marker: { shape: "^" }, type: "dotted"  },
                            [64,  2] => { marker: { shape: "s" }, type: "dashed"  },
                            [ 2, 16] => { marker: { shape: "2" }, type: "solid"   },
                            [ 8, 16] => { marker: { shape: "X" }, type: "dotted"  },
                            [64, 16] => { marker: { shape: "h" }, type: "dashed"  },
                            _        => { marker: { shape: "D" }, type: "dashdot" },
                        }
                    }
                    -N { |_|
                        let parsed = $in | parse "{k} {nb_shards}" | into record | into int k nb_shards
                        $"$k = ($parsed.k)$, $x = ($parsed.nb_shards / $parsed.k | math round -p 3)$"
                    }
                    -c r
                    -u ms
                    --git-hash-pattern $git_hash_pattern
                    --cpu-hash-pattern $cpu_hash_pattern
                    --src-hash-pattern $src_hash_pattern
                    --rust-build $rust_build
                    --bench "fec-recode"
                )
            },
            "semi-avid" | "kzg" | "aplonk" => {
                log warning $"plotting bench `($b)` is not implemented yet"
            },
            "protocols" => {
                for params in [
                    { nb_bytes: 507904    , k: 8 , n: 16 },
                    { nb_bytes: 130023424 , k: 8 , n: 16 },
                ] {
                    open (in "protocols") | (plot-bars
                        -o (out $"protocols-b($params.nb_bytes)-k($params.k)-n($params.n)")
                        -t $"time to run cryptographic \\textit{protocols} data of size $($params.nb_bytes)$, $k = ($params.k)$ on \\textbf{BN254} \(r: $(curve-to-field-modulus 'BN254' | get r)$ bits\), $\\rho = \\frac{1}{2}$"
                        -l "time in %unit"
                        --log-scale
                        --grid
                        --grid-behind
                        -u ms
                        -c r
                        --filter { |it| (
                            $it.curve == "BN254" and
                            $it.nb_bytes == $params.nb_bytes and
                            $it.k == $params.k and
                            $it.n == $params.n
                        )}
                        --grouper { |it| $it.step }
                        --family command
                        --styler-color { |it|
                            match $it {
                                "aplonk"    => "tab:orange",
                                "kzg"       => "tab:blue",
                                "semi_avid" => "tab:green",
                                _           => "tab:gray",
                            }
                        }
                        --git-hash-pattern $git_hash_pattern
                        --cpu-hash-pattern $cpu_hash_pattern
                        --src-hash-pattern $src_hash_pattern
                        --rust-build $rust_build
                        --bench "protocols"
                    )
                }
            },
            _ => {
                log error $"unknown bench `($b)`"
                log hint $"    choose one of these: ($BENCHES | each { str color green })"
            },
        }
    }
}
