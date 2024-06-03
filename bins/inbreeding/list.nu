use consts.nu
use parse.nu [ "parse full-experiment" ]
use path.nu [ "remove-cache-prefix" ]

# return experiment names following `$ARG_EXPERIMENT_FORMAT`
export def main []: nothing -> list<string> {
    $consts.CACHE
        | path join '*' '*'
        | into glob
        | ls $in
        | get name
        | each { remove-cache-prefix }
        | where ($it | path split | first | str length) == 64
        | each { parse full-experiment | reject strategy | values | str join '-' }
        | uniq
}
