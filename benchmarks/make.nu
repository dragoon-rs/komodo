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
    sizes: list<int>,
    -k: int,
    -n: int,
    --nb-rounds: int = 10,
    --protocol: string,
    --fri-ff  : int,
    --fri-bf  : int,
    --fri-rpo : int,
    --fri-q   : int,
    --debug,
]: [ nothing -> table<step: string, times: list<int>, bytes: int> ] {
    if $protocol == null { error make --unspanned { msg: "missing --protocol" } }

    $sizes
        | each { |nb_bytes|
            1..$nb_rounds
                | each {
                    progress $"protocol=($protocol),b=($nb_bytes),k=($k),n=($n) | ($in)/($nb_rounds)"
                    ^$"./target/(if $debug { 'debug' } else { 'release' })/($PACKAGE)" ...[
                        --nb-bytes $nb_bytes
                        -k $k
                        -n $n
                        --protocol $protocol
                        ...(if $fri_ff != null {[--fri-ff  $fri_ff  ]} else {[]})
                        ...(if $fri_ff != null {[--fri-bf  $fri_bf  ]} else {[]})
                        ...(if $fri_ff != null {[--fri-rpo $fri_rpo ]} else {[]})
                        ...(if $fri_ff != null {[--fri-q   $fri_q   ]} else {[]})
                    ] | lines | each { from nuon } | into record
                }
                | transpose
                | each {{
                    step: $in.column0,
                    times: ($in | reject column0 | values),
                }}
                | insert bytes $nb_bytes
        }
        | flatten
}
