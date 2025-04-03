const GH_API_OPTIONS = [
    -H "Accept: application/vnd.github+json"
    -H "X-GitHub-Api-Version: 2022-11-28"
]

const GITHUB_MIRROR = "dragoon-rs/komodo"

^$nu.current-exe ./scripts/check-nushell-version.nu

def main [branch: string]: [ nothing -> string ] {
    let res = gh api ...$GH_API_OPTIONS $"/repos/($GITHUB_MIRROR)/actions/runs" | from json

    let runs = $res.workflow_runs
        | where head_branch == $branch
        | select id head_sha status conclusion run_started_at
        | into datetime run_started_at
        | sort-by run_started_at

    $runs
        | update id { $"[`($in)`]\(https://github.com/($GITHUB_MIRROR)/actions/runs/($in)\)" }
        | update run_started_at { format date "%Y-%m-%dT%H:%M:%S" }
        | to md --pretty
}
