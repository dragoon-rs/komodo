def "str color" [color: string]: [ string -> string ] {
    $"(ansi $color)($in)(ansi reset)"
}

def __log [level: string, color: string, msg: string] {
    print $"[(ansi $color)($level)(ansi reset)] ($msg)"
}
def "log error" [msg: string] { __log "ERR" "red"   $msg }
def "log info"  [msg: string] { __log "INF" "cyan"  $msg }
def "log ok"    [msg: string] { __log " OK" "green" $msg }

let config = open .nu.cfg
    | lines
    | parse "{key}: {value}"
    | transpose --header-row
    | into record
if (version).commit_hash != $config.REVISION or (version).version != $config.VERSION {
    print --stderr $"(ansi yellow_bold)Warning(ansi reset): unexpected version"
    print --stderr $"    expected (ansi green)($config.VERSION)@($config.REVISION)(ansi reset)"
    print --stderr $"    found    (ansi red)((version).version)@((version).commit_hash)(ansi reset)"
}

def main [base: string, mirror: string, branch: string] {
    let base_remote = random uuid
    let mirror_remote = random uuid

    log info "adding remotes"
    git remote add $base_remote $base
    git remote add $mirror_remote $mirror

    log info "fetching"
    git fetch --quiet $base_remote
    git fetch --quiet $mirror_remote

    let base = git rev-parse $"($base_remote)/($branch)" | str trim
    let mirror = git rev-parse $"($mirror_remote)/($branch)" | str trim

    log info "cleaning"
    git remote remove $base_remote
    git remote remove $mirror_remote

    if $base != $mirror {
        let hist = git rev-list $"($mirror)..($base)" | lines

        log error "mirror is out of date"
        {
            b: ($base | str substring 0..<7),
            m: ($mirror | str substring 0..<7),
            h: ($hist | length),
        }
        | print $"    ($in.b | str color green) | ($in.m | str color red) \(($in.h) commits behind\)"
    } else {
        log ok "mirror is up to date"
    }
}
