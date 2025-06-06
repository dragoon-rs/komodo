const GH_API_OPTIONS = [
    -H "Accept: application/vnd.github+json"
    -H "X-GitHub-Api-Version: 2022-11-28"
]

def "str color" [color: string]: [ string -> string ] {
    $"(ansi $color)($in)(ansi reset)"
}

def __log [level: string, color: string, msg: string] {
    print $"[(ansi $color)($level)(ansi reset)] ($msg)"
}
def "log error" [msg: string] { __log "ERR" "red"   $msg }
def "log info"  [msg: string] { __log "INF" "cyan"  $msg }
def "log ok"    [msg: string] { __log " OK" "green" $msg }

^$nu.current-exe ./scripts/check-nushell-version.nu

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

    log info "pulling mirror runs"
    let res = gh api ...$GH_API_OPTIONS /repos/dragoon-rs/komodo/actions/runs | from json

    let runs = $res.workflow_runs
        | where head_branch == $branch
        | select id head_sha status conclusion run_started_at
        | into datetime run_started_at
        | sort-by run_started_at

    $env.config.table = {
        mode: compact,
        index_mode: always,
        show_empty: true,
        padding: { left: 0, right: 0 },
        header_on_separator: true,
        trim: {
            methodology: wrapping,
            wrapping_try_keep_words: true,
        },
        abbreviated_row_count: null,
        footer_inheritance: false,
    }

    print $runs
}
