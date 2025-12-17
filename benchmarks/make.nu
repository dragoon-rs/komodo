use std formats [ "to ndjson", "from ndjson" ]
use ../log.nu [ "log warning", "log info", "log debug" ]

const PACKAGE = "benchmarks"

def progress [msg: string] {
    print --no-newline ($msg | fill --width (term size).columns | $in + "\r")
}

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

export def lscpu []: [ nothing -> record ] {
    ^lscpu --json
        | from json
        | get lscpu
        | update field { str trim --right --char ":" }
        | transpose --header-row
        | into record
        | select ...$CPU_FIELDS
}

export def build [--debug] {
    if $debug {
        cargo build --package $PACKAGE
    } else {
        cargo build --package $PACKAGE --release
    }
}

export def bench [
    size: int,
    -k: int,
    -n: int,
    --nb-rounds: int = 10,
    --protocol: string,
    --fri-ff  : int,
    --fri-bf  : int,
    --fri-rpo : int,
    --fri-q   : int,
    --debug,
    --curve   : string,
    --seed    : int,
]: [ nothing -> table<k: string, v: int, build: string, seed: int> ] {
    if $protocol == null { error make --unspanned { msg: "missing --protocol" } }

    let build = if $debug { "debug" } else { "release" }

    1..$nb_rounds
        | each {
            progress $"started at (date now | format date '%+') | protocol=($protocol),b=($size),k=($k),n=($n) | ($in)/($nb_rounds)"
            ^$"./target/($build)/($PACKAGE)" ...[
                --nb-bytes $size
                -k $k
                -n $n
                --protocol $protocol
                ...(if $fri_ff != null {[--fri-ff  $fri_ff  ]} else {[]})
                ...(if $fri_ff != null {[--fri-bf  $fri_bf  ]} else {[]})
                ...(if $fri_ff != null {[--fri-rpo $fri_rpo ]} else {[]})
                ...(if $fri_ff != null {[--fri-q   $fri_q   ]} else {[]})
                --curve $curve
                ...(if $seed != null {[--seed $seed ]} else {[]})
            ] | lines | each { from nuon } | into record
        }
        | transpose k v
        | insert build $build
        | insert seed $seed
}

def send [header: record, body: string] {
    let lines = [
        ...($header | items { |k, v| $"($k): ($v)" })
        ""
        $body
    ]
    himalaya message send  --debug ($lines | str join "\n")
}

export def main [
    params      : table<p: string, k: int, n: int, b: int>,
    --email,
    --shutdown,
    --commit,
    --push,
    --remote    : string,
    --from      : string,
    --to        : string,
    --seed      : int,
    --curve     : record<name: string, bits: int> = { name: "bn254", bits: 254 },
] {
    # /* MAIN_ARGS_ERROR_HANDLING
    if $seed == null { error make --unspanned {
        msg: $"(ansi red_bold)invalid_args(ansi reset): missing (ansi default_dimmed)--seed(ansi reset)",
    } }
    if $push and $remote == null { error make {
        msg: $"(ansi red_bold)invalid_args(ansi reset)",
        label: {
            text: $"missing (ansi default_dimmed)--remote(ansi reset)",
            span: (metadata $push).span,
        },
        help: $"(ansi default_dimmed)--push(ansi reset) is set, try to set (ansi default_dimmed)--remote(ansi reset) to a valid Git remote name",
    } }
    if $email and $from == null { error make {
        msg: $"(ansi red_bold)invalid_args(ansi reset)",
        label: {
            text: $"missing (ansi default_dimmed)--from(ansi reset)",
            span: (metadata $email).span,
        },
        help: (do {
            const HIMALAYA_CONFIG = "~/.config/himalaya/config.toml" | path expand

            let header = $"(ansi default_dimmed)--email(ansi reset) is set, try to set (ansi default_dimmed)--from(ansi reset) to a valid Himalaya sender"

            let rest = if (which himalaya | is-empty) {
                $"(ansi red_bold)ERROR(ansi reset): cannot list potential Himalaya senders \(Himalaya not found\)"
            } else if not ($HIMALAYA_CONFIG | path exists) {
                $"(ansi red_bold)ERROR(ansi reset): cannot list potential Himalaya senders \(Himalaya config not found\)"
            } else {
                try {
                    let senders = himalaya account list --output json
                        | from json
                        | each { |it|
                            open $HIMALAYA_CONFIG | get accounts | get $it.name | get email
                        }

                        [
                            "list of available Himalaya senders:"
                            ...($senders | each { $"- (ansi cyan)($in)(ansi reset)" })
                        ]
                        | each { $"    ($in)" }
                        | str join "\n"
                } catch {
                    $"(ansi red_bold)ERROR(ansi reset): cannot list potential Himalaya senders \(Himalaya is not setup properly\)"
                }
            }

            $"($header)\n($rest)"
        }),
    } }
    if $email and $to == null { error make {
        msg: $"(ansi red_bold)invalid_args(ansi reset)",
        label: {
            text: $"missing (ansi default_dimmed)--to(ansi reset)",
            span: (metadata $email).span,
        },
        help: $"(ansi default_dimmed)--email(ansi reset) is set, try to set (ansi default_dimmed)--to(ansi reset) to a valid receiver",
    } }

    if $push and not $commit { log warning $"(ansi default_dimmed)--push(ansi reset) is set but (ansi default_dimmed)--commit(ansi reset) is not \(won't push\)" }
    # MAIN_ARGS_ERROR_HANDLING */

    let cpu = lscpu | to json | hash sha256
    let git = git rev-parse HEAD | str trim

    build

    for p in $params {
        let start = date now
        (bench
            $p.b
            -k $p.k
            -n $p.n
            --nb-rounds 1
            --protocol $p.p
            --fri-ff 2
            --fri-bf ($p.n / $p.k | into int)
            --fri-rpo 1
            --fri-q 50
            --curve $curve.name
            --seed $seed)
            | rename --column { k: "__k" }
            | insert k $p.k
            | insert n $p.n
            | insert bytes $p.b
            | insert git $git
            | insert cpu $cpu
            | insert curve $curve.name
            | save --append $"benchmarks/($p.p).ndjson"
        let end = date now

        if $commit {
            git add "benchmarks/*.ndjson"
            git commit --no-gpg-sign --message $"auto: p=($p.p),k=($p.k),b=($p.b)"
            if $push { git push $remote HEAD }
        }

        if $email {
            {
                from: $from,
                to: $to,
                subject: $"Komodo: p=($p.p),k=($p.k)",
                body: [
                    $"t: (date now | format date "%+")"
                    $"done in ($end - $start)",
                    (if $push { $"pushed to https://gitlab.isae-supaero.fr/dragoon/komodo/-/commit/(git rev-parse HEAD)" }),
                ]
            } | send { From: $in.from, To: $in.to, Subject: $in.subject } ($in.body | compact | str join "\n")
        }
    }

    if $shutdown {
        if $email {
            send { From: $from, To: $to, Subject: $"Shutting down (hostname)" } $"t: (date now | format date "%+")"
        }
        shutdown now
    }
}

def sort-by-keys []: [ record -> record ] {
    transpose k v | sort-by k | transpose --header-row | into record
}

# merge table rows that only differ by $.v
export def compact-results []: [
    table # <v: int>
    ->
    table<v: list<int>>
] {
    group-by --to-table { reject v | sort-by-keys | to nuon | hash sha256 }
        | get items
        | each {{
            ...($in | reject v | uniq | into record),
            v: ($in.v | compact),
        }}
}

export def into-matrix [
    --sort-x,
    --sort-y,
    --complete-x: list<int>,
    --complete-y: list<int>,
]: [
    table<
        x: int,
        y: int,
        z: float,
    > -> list<float>
] {
    let by_y = $in | group-by --to-table y | into int y

    let x_completed = if not ($complete_x | is-empty) {
        $by_y | update items { |it|
            $it.items | append (
                $complete_x | each { |x|
                    $it.items.0 | update x $x | update z null
                }
            )
            | uniq-by x
            | where x in $complete_x
        }
    } else {
        $by_y
    }

    let y_completed = if not ($complete_y | is-empty) {
        $x_completed | append (do {
            $complete_y | each { |y| {
                y: $y,
                items: ($x_completed.items.0
                    | update y $y
                    | update z null
                )
            }}
        })
        | uniq-by y
        | where y in $complete_y
    } else {
        $x_completed
    }

    $y_completed
        |                if $sort_y { sort-by y } else { $in } | uniq-by y
        | update items { if $sort_x { sort-by x } else { $in } | uniq-by x }
        | get items
        | each { each { if $in.z != null { $in.z } else { "NaN" } } }
}

export def heat_map [
    values       : list,
    --width  (-W): int,
    --height (-H): int,
    --cbar,
    --cbardir    : string,
    --cbarmin    : float,
    --cbarmax    : float,
    --cbarticks  : table<tick: float, label: string>,
    --xticks     : table<tick: float, label: string>,
    --yticks     : table<tick: float, label: string>,
    --title      : string,
    --xlabel     : string,
    --ylabel     : string,
    --save       : string = "a.png",
    --cmap       : string = "viridis",
    --clut       : int,
    --figsize    : record<w: float, h: float>,
    --overlay,
] {
    uv run benchmarks/heat_map.py ...[
        ...$values
        -W $width
        -H $height
        --save $save
        --cmap $cmap
        ...(if $cbarmin   != null { [--cbarmin       $cbarmin                                        ] } else { [] })
        ...(if $cbarmax   != null { [--cbarmax       $cbarmax                                        ] } else { [] })
        ...(if $cbarticks != null { [--cbarticks ...($cbarticks | each { $"($in.tick):($in.label)" })] } else { [] })
        ...(if $xticks    != null { [--xticks    ...($xticks    | each { $"($in.tick):($in.label)" })] } else { [] })
        ...(if $yticks    != null { [--yticks    ...($yticks    | each { $"($in.tick):($in.label)" })] } else { [] })
        ...(if $title     != null { [--title         $title                                          ] } else { [] })
        ...(if $xlabel    != null { [--xlabel        $xlabel                                         ] } else { [] })
        ...(if $ylabel    != null { [--ylabel        $ylabel                                         ] } else { [] })
        ...(if $overlay           { [--overlay                                                       ] } else { [] })
        ...(if $cbar              { [--cbar                                                          ] } else { [] })
        ...(if $figsize   != null { [--figsize       $figsize.w $figsize.h                           ] } else { [] })
        ...(if $cbardir   != null { [--cbardir       $cbardir                                        ] } else { [] })
        ...(if $clut      != null { [--clut          $clut                                           ] } else { [] })
    ]
}

export def plot [
    plots        : table<p: string, __k: string>,
    --log-base   : int = 10,
    --plot-nb,
    --complete-x : list<int>,
    --complete-y : list<int>,
    --figsize    : record<w: float, h: float>,
    --xticks,
    --yticks,
    --cbar,
    --cmap       : string = "viridis",
]: [ table -> nothing ] {
    let data = $in | update v { if $in == null { null } else { $in | math log $log_base } }

    log debug "find minmax"
    let log_minmax = $data.v
        | flatten
        | compact
        | { max: ($in | math max), min: ($in | math min) }

    let nb_minmax = $data
        | group-by --to-table {
            reject v git cpu curve build seed | transpose k v | sort-by k | get v | str join "-"
        }
        | get items
        | each { get v | compact | length }
        | { min: ($in | math min), max: ($in | math max)}

    log debug "get cbar ticks"
    let cbarticks = seq ($log_minmax.min | math ceil) ($log_minmax.max | math floor)
        | each {{
            tick: $in,
            label: ($log_base ** $in * 1ns | into string),
        }}

    let xticks = if not $xticks { [] } else { $complete_x           | enumerate | rename --column { index: tick, item: label } }
    let yticks = if not $yticks { [] } else { $complete_y | reverse | enumerate | rename --column { index: tick, item: label } }

    def prepare-for-heat-map [] {
        select bytes k v
            | rename x y z
            | (into-matrix
                --complete-x $complete_x
                --complete-y $complete_y
                --sort-x
                --sort-y
            )
            | reverse
            | flatten
    }

    if not $cbar {
        (heat_map []
            --figsize { w: 9.0, h: 0.5 }
            --cbardir horizontal
            --width     0
            --height    0
            --cbar
            --cbarmin   $log_minmax.min
            --cbarmax   $log_minmax.max
            --cbarticks $cbarticks
            --cmap      $cmap
            --save $"cbarh-($cmap).png"
        )
        (heat_map []
            --figsize { h: 9.0, w: 1.5 }
            --cbardir vertical
            --width     0
            --height    0
            --cbar
            --cbarmin   $log_minmax.min
            --cbarmax   $log_minmax.max
            --cbarticks $cbarticks
            --cmap      $cmap
            --save $"cbarv-($cmap).png"
        )
    }

    for plot in $plots {
        log info $"plot p=($plot.p),__k=($plot.__k)"
        let filtered = $data | where p == $plot.p and __k == $plot.__k
        if ($filtered | length) == 0 {
            log warning $"empty filter: p == ($plot.p) and __k == ($plot.__k)"
            continue
        }
        let compacted = $filtered | reject git cpu curve build seed | compact-results

        $compacted
            | update v { try { math avg } }
            | prepare-for-heat-map
            | (heat_map $in
                --figsize   $figsize
                --width    ($complete_x | length)
                --height   ($complete_y | length)
                --cbar=$cbar
                --cbarmin   $log_minmax.min
                --cbarmax   $log_minmax.max
                --xticks    $xticks
                --yticks    $yticks
                --cbarticks $cbarticks
                --cmap      $cmap
                --save $"($plot.p)-($plot.__k).png"
            )

        if $plot_nb {
            $compacted
                | update v { length }
                | prepare-for-heat-map
                | (heat_map $in
                    --figsize   $figsize
                    --width    ($complete_x | length)
                    --height   ($complete_y | length)
                    --cbar=$cbar
                    --clut     ($nb_minmax.max - $nb_minmax.min + 1)
                    --cbarmin   $nb_minmax.min
                    --cbarmax   $nb_minmax.max
                    --xticks    $xticks
                    --yticks    $yticks
                    --cmap "magma"
                    --save $"($plot.p)-($plot.__k).nb.png"
                    --overlay
                )
        }
    }
}

export def "cartesian-product" [...iters: list]: [ nothing -> list ] {
    def aux [its: list<list>]: [ nothing -> list ] {
        match ($its | length) {
            0 => [],
            1 => $its.0,
            _ => {
                $its.0 | each { |a|
                    aux ($its | skip 1) | | each { |b|
                        $a | append $b
                    }
                }
                | flatten
            },
        }
    }

    aux $iters
}
