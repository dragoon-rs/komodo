use consts.nu
use path.nu [ "remove-cache-prefix" ]

# return experiment names following `$ARG_EXPERIMENT_FORMAT`
export def main []: nothing -> list<string> {
    $consts.CACHE
        | path join '*' '*'
        | into glob
        | ls $in
        | get name
        | each { remove-cache-prefix }
        | parse --regex $consts.FULL_EXPERIMENT_FORMAT
        | reject strategy
        | each { values | str join '-' }
        | uniq
}
