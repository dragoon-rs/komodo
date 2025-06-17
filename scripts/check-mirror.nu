const GH_API_OPTIONS = [
    -H "Accept: application/vnd.github+json"
    -H "X-GitHub-Api-Version: 2022-11-28"
]

use ../log.nu [ "log debug", "log info", "log ok", "log error", "str color" ]

^$nu.current-exe ./scripts/check-nushell-version.nu

def main [base: string, mirror: string, branch: string] {
    let base_remote = random uuid
    let mirror_remote = random uuid

    log info "adding remotes"
    log debug $base_remote
    git remote add $base_remote $base
    log debug $mirror_remote
    git remote add $mirror_remote $mirror

    log info "fetching"
    log debug $base_remote
    git fetch --quiet $base_remote
    log debug $mirror_remote
    git fetch --quiet $mirror_remote

    let base = git rev-parse $"($base_remote)/($branch)" | str trim
    let mirror = git rev-parse $"($mirror_remote)/($branch)" | str trim

    log info "cleaning"
    log debug $base_remote
    git remote remove $base_remote
    log debug $mirror_remote
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
