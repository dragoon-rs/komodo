use ../src/.nushell/consts.nu CACHE
use ../src/.nushell/path.nu [ "remove-cache-prefix" ]
use ../src/.nushell/parse.nu [ "parse full-experiment" ]

const SEED = "b239e48345ac457b492cc164f58c010d07292c88e4791e607d91796baec7f334"
let k = 10

# # Example
# ``nushell
# # sort files based on their file extensions
# ls | where type == file | sort-by-closure { get name | path parse | get extension }
# ``
# ---
# ``
# ─┬───────────────────
# 0│LICENSE
# 1│Cargo.lock
# 2│CODE_OF_CONDUCT.md
# 3│CONTRIBUTING.md
# 4│README.md
# 5│toolkit.nu
# 6│Cargo.toml
# 7│Cross.toml
# 8│rust-toolchain.toml
# 9│typos.toml
# ─┴───────────────────
# ``
def sort-by-closure [key: closure]: list -> list {
    insert __key__ { do $key $in } | sort-by __key__ | reject __key__
}

def show [x: string] {
    $CACHE
        | path join figures $"($SEED)-($x).png"
        | into glob
        | ls $in
        | sort-by-closure {
            get name
                | remove-cache-prefix
                | path parse --extension 'png'
                | reject extension
                | path join
                | parse full-experiment
                | get n
        }
        | feh ...$in.name
}

show $"fixed:0-($k)-*-*"
show $"random-fixed:0.5:1-($k)-*-*"
show $"fixed:1-($k)-*-*"
show $"*-($k)-20-*"
show $"*-($k)-30-*"
show $"*-($k)-50-*"
show $"*-($k)-100-*"
