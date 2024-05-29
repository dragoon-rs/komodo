use consts.nu
use path.nu [ "remove-cache-prefix" ]

def get-seeds [] [ nothing -> list<string> ] {
    $consts.CACHE | path join '*' | into glob | ls $in | get name | each { path split | last }
}

export def main [seed: int@get-seeds]: [
    nothing -> table<
        seed: string,
        timestamp: string,
        env: string,
        strategy: string,
        k: string,
        n: string,
        nb_bytes: string,
        m: int,
    >
] {
    $consts.CACHE
        | path join ($seed | into string) '*'
        | into glob
        | ls $in
        | insert m { ls $in.name | length }
        | select name m
        | update name {
            remove-cache-prefix
                | parse $consts.FULL_EXPERIMENT_FORMAT
        }
        | flatten --all name
}
