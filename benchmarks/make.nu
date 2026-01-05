use ../log.nu [ "log warning", "log info", "log debug" ]

const PACKAGE = "benchmarks"

def progress [msg: string] {
    print --no-newline ($msg | fill --width (term size).columns | $in + "\r")
}

const KOMODO_URL = "https://gitlab.isae-supaero.fr/dragoon/komodo"

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
            ^$"./target/($build)/($PACKAGE)" run ...[
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

def send [header: record, body: string, --debug] {
    let lines = [
        ...($header | items { |k, v| $"($k): ($v)" })
        ""
        $body
    ]
    himalaya message send ...[
        ...(if $debug { [--debug] } else { [] }),
        ($lines | str join "\n")
    ]
}

export def main [
    params      : table<p: string, k: int, n: int, b: int>,
    --email     : int, # frequency at which to email
    --dbg-mail,
    --shutdown  : datetime,
    --commit,
    --push,
    --remote    : string,
    --from      : string,
    --to        : string,
    --seed      : int,
    --curve     : string = "bn254",
    --git       : string, # force a Git revision
    --no-confirm (-y),
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
    if $email == 0 { error make {
        msg: $"(ansi red_bold)invalid_args(ansi reset)",
        label: {
            text: $"(ansi default_dimmed)--email(ansi reset) cannot be 0",
            span: (metadata $email).span,
        },
        help: $"set (ansi default_dimmed)--email(ansi reset) to a strictly positive value",
    } }
    if $email < 0 { error make {
        msg: $"(ansi red_bold)invalid_args(ansi reset)",
        label: {
            text: $"(ansi default_dimmed)--email(ansi reset) cannot be negative",
            span: (metadata $email).span,
        },
        help: $"set (ansi default_dimmed)--email(ansi reset) to a strictly positive value",
    } }
    if $email != null and $from == null { error make {
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
    if $email != null and $to == null { error make {
        msg: $"(ansi red_bold)invalid_args(ansi reset)",
        label: {
            text: $"missing (ansi default_dimmed)--to(ansi reset)",
            span: (metadata $email).span,
        },
        help: $"(ansi default_dimmed)--email(ansi reset) is set, try to set (ansi default_dimmed)--to(ansi reset) to a valid receiver",
    } }

    if $push and not $commit { log warning $"(ansi default_dimmed)--push(ansi reset) is set but (ansi default_dimmed)--commit(ansi reset) is not \(won't push\)" }
    # MAIN_ARGS_ERROR_HANDLING */

    if ($params | is-empty) {
        log warning "nothing to do"
        return
    }

    let api = ./target/release/benchmarks list
        | lines
        | parse "{k}:{v}"
        | update v { split row "," }
        | transpose --header-row
        | into record
    if $curve not-in $api.curves { error make {
        msg: $"(ansi red_bold)invalid_args(ansi reset)",
        label: {
            text: $"(ansi default_dimmed)--curve(ansi reset) = ($curve) is invalid",
            span: (metadata $curve).span,
        },
        help: $"set (ansi default_dimmed)--curve(ansi reset) to one of: ($api.curves | str join ', ')",
    } }
    let invalid_protocols = $params | enumerate | where $it.item.p not-in $api.protocols
    if not ($invalid_protocols | is-empty) { error make {
        msg: $"(ansi red_bold)invalid_args(ansi reset)",
        label: {
            text: $"(ansi default_dimmed)$params.p.($invalid_protocols.0.index)(ansi reset) = ($invalid_protocols.0.item.p) is invalid",
            span: (metadata $params).span,
        },
        help: $"$params.($invalid_protocols.0.index) = ($invalid_protocols.0.item) \(found ($invalid_protocols | length) others\)\navailable protocols: ($api.protocols | str join ', ')",
    } }

    let cpu = lscpu | to json | hash sha256
    let git = if $git != null {
        try {
            git rev-parse $git
        } catch { error make {
            msg: $"(ansi red_bold)invalid_args(ansi reset)",
            label: {
                text: $"($git) is not a valid and non-ambiguous Git revision",
                span: (metadata $git).span,
            },
            help: $"set (ansi default_dimmed)--git(ansi reset) to a valid Git revision",
        } }
    } else {
        git fetch $KOMODO_URL main
        git merge-base (git rev-parse FETCH_HEAD) (git rev-parse HEAD)
    }

    let now = date now
    let shutdown = if $shutdown == null {
        null
    } else if $shutdown <= $now {
        error make {
            msg: $"(ansi red_bold)invalid_args(ansi reset)",
            label: {
                text: $"($shutdown) is before now or within a minute",
                span: (metadata $shutdown).span,
            },
            help: ([
                $"now            : ($now | format date "%+")",
                $"scheduled date : ($shutdown | format date "%+")",
            ] | str join "\n"),
        }
    } else {
        $shutdown
    }

    if not $no_confirm {
        print $"(ansi cyan)PARAMETERS(ansi reset):"
        [
            [k          , v];
            ["git"      , $git],
            ["cpu"      , $"($cpu) \((if ($"benchmarks/($cpu).json" | path exists) { '*' } else { ' ' })\)"],
            ["seed"     , $seed],
            ["curve"    , $curve],
            ["shutdown" , (if $shutdown != null { $"scheduled for ($shutdown | format date '%+'), in ($shutdown - $now)" })]
            ["email"    , (if $email    != null { $"($from) -> ($to)" })]
            ["commit"   , (if $commit           { "*"                 })]
            ["push"     , (if $push             { $remote             })]
            ["params"   , ($params | length)],
        ]
            | into string v
            | to md --pretty
            | lines
            | skip 2
            | parse "| {k} | {v} |"
            | str trim v
            | each { $"    ($in.k) : ($in.v)" }
            | str join "\n"
            | print $in

        if (["no", "yes"] | input list $"(ansi cyan)Open the params with pager(ansi reset)") == "yes" {
            $params | to md --pretty | less -R
        }

        if (["no", "yes"] | input list $"(ansi cyan)Proceed with benchmarks(ansi reset)") != "yes" {
            log info "aborting"
            return
        }
    }

    if $shutdown != null {
        const NB_NS_IN_SEC = 1_000_000_000
        let delta_min = $shutdown - $now
            | into int
            | $in / (60 * $NB_NS_IN_SEC)
            | math round --precision 0
            | into int

        if $email != null {
            send --debug=$dbg_mail { From: $from, To: $to, Subject: $"(hostname): Shutdown scheduled" } $"t: (date now | format date "%+")\nshutdown: ($now + 1min * $delta_min)"
        }

        shutdown $"+($delta_min)"
    }

    build

    let start = date now

    for p in ($params | enumerate) {
        let output = $"benchmarks/($p.item.p).csv"
        let start = date now
        (bench
            $p.item.b
            -k $p.item.k
            -n $p.item.n
            --nb-rounds 1
            --protocol $p.item.p
            --fri-ff 2
            --fri-bf ($p.item.n / $p.item.k | into int)
            --fri-rpo 1
            --fri-q 50
            --curve $curve
            --seed $seed)
            | rename --column { k: "__k" }
            | insert k $p.item.k
            | insert n $p.item.n
            | insert bytes $p.item.b
            | insert cpu ($cpu | str substring ..<7)
            | insert git ($git | str substring ..<7)
            | move build --first
            | move cpu --first
            | move git --first
            | insert curve $curve
            | if not ($output | path exists) { to csv } else { to csv --noheaders }
            | save --append $output
        let end = date now

        if $commit {
            git add $output
            git commit --no-gpg-sign --message $"no-ci:auto: s=($seed),p=($p.item.p),k=($p.item.k),b=($p.item.b)"
            if $push { git push $remote HEAD }
        }

        if $email != null and (($p.index + 1) mod $email == 0) {
            {
                from: $from,
                to: $to,
                subject: $"Komodo: (date now | format date "%+")",
                body: [
                    $"done in ($end - $start)",
                    (if $push {
                        let head = git rev-parse HEAD
                        if $email == 1 {
                            $"($KOMODO_URL)/-/commit/($head)"
                        } else {
                            let last = git rev-parse $"HEAD~($email)"
                            $"($KOMODO_URL)/-/compare/($last)...($head)"
                        }
                    }),
                ]
            } | send --debug=$dbg_mail { From: $in.from, To: $in.to, Subject: $in.subject } ($in.body | compact | str join "\n")
        }
    }

    let end = date now

    if $email != null {
        {
            from: $from,
            to: $to,
            subject: $"Komodo: (date now | format date "%+") \(benchmarks done\)",
            body: [
                $"done in ($end - $start)",
                (if $push {
                    let head = git rev-parse HEAD
                    if ($params | length) == 1 {
                        $"($KOMODO_URL)/-/commit/($head)"
                    } else {
                        let start = git rev-parse $"HEAD~($params | length)"
                        $"($KOMODO_URL)/-/compare/($start)...($head)"
                    }
                })
                (if $shutdown != null { $"shutdown scheduled in ($shutdown - (date now))" })
            ]
        } | send --debug=$dbg_mail { From: $in.from, To: $in.to, Subject: $in.subject } ($in.body | compact | str join "\n")
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
    let data = $in | update v { try { math log $log_base } }

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
            log warning $"empty filter"
            log info    $"      p == ($plot.p)"
            log info    $"    __k == ($plot.__k)"
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

export def load-all [] {
    log info "loading data"
    let start = date now
    let res = ["semi-avid", "kzg", "aplonk", "fri" ]
        | each { |it|
            open $"benchmarks/($it).csv" | insert p $it
        }
        | flatten
    let end = date now
    log info $"data loaded in ($end - $start)"
    log debug $"loaded ($res | length) rows"

    $res
}
